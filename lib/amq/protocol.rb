# frozen_string_literal: true

require_relative "protocol/version"

# Load the native extension
begin
  RUBY_VERSION =~ /(\d+\.\d+)/
  require_relative "protocol/#{$1}/amq_protocol_native"
rescue LoadError
  begin
    require_relative "protocol/amq_protocol_native"
  rescue LoadError
    require "amq_protocol_native/amq_protocol_native"
  end
end

module AMQ
  module Protocol
    # Re-export the METHODS hash for compatibility
    # This will be populated by the native extension at load time
    METHODS ||= begin
      Method.methods.each_with_object({}) do |klass, hash|
        hash[klass.index] = klass if klass.respond_to?(:index)
      end
    rescue
      {}
    end

    class Method
      class << self
        def methods
          @methods ||= []
        end

        def inherited(base)
          methods << base if self == Method
        end

        # Split user headers into properties and custom headers
        def split_headers(user_headers)
          properties = {}
          headers = {}

          user_headers.each do |key, value|
            if Basic::PROPERTIES.include?(key)
              properties[key] = value
            else
              headers[key] = value
            end
          end

          [properties, headers]
        end

        # Encode message body into body frames
        def encode_body(body, channel, frame_size)
          return [] if body.empty?

          # 8 = 1 byte frame type + 2 bytes channel + 4 bytes size + 1 byte frame end
          limit = frame_size - 8
          return [BodyFrame.new(body, channel)] if body.bytesize < limit

          body = body.dup.force_encoding(Encoding::BINARY) if body.encoding != Encoding::BINARY

          frames = []
          while body && !body.empty?
            payload = body[0, limit]
            body = body[limit, body.length - limit]
            frames << BodyFrame.new(payload, channel)
          end

          frames
        end

        def has_content?
          false
        end
      end
    end

    class Class
      class << self
        def classes
          @classes ||= []
        end

        def inherited(base)
          classes << base if self == Protocol::Class
        end
      end
    end

    # FrameSubclass provides instance-based frame handling
    class FrameSubclass < Frame
      class << self
        attr_accessor :id

        def encode(payload, channel)
          Frame.encode(@id, payload, channel)
        end
      end

      attr_accessor :channel
      attr_reader :payload

      def initialize(payload, channel)
        @payload = payload
        @channel = channel
      end

      def size
        @payload.bytesize
      end

      def encode
        self.class.encode(@payload, @channel)
      end

      def final?
        true
      end
    end

    # Redefine frame classes to inherit from FrameSubclass
    class MethodFrame < FrameSubclass
      self.id = 1

      def method_class
        @method_class ||= begin
          klass_id, method_id = @payload.unpack("nn")
          index = (klass_id << 16) | method_id
          METHODS[index]
        end
      end

      def final?
        !method_class&.has_content?
      end

      def decode_payload
        method_class&.decode(@payload[4..-1])
      end
    end

    class HeaderFrame < FrameSubclass
      self.id = 2

      def final?
        false
      end

      def body_size
        decode_payload
        @body_size
      end

      def weight
        decode_payload
        @weight
      end

      def klass_id
        decode_payload
        @klass_id
      end

      def properties
        decode_payload
        @properties
      end

      def decode_payload
        @decoded_payload ||= begin
          @klass_id, @weight = @payload.unpack("nn")
          @body_size = @payload[4, 8].unpack("Q>").first
          @data = @payload[12..-1]
          @properties = Basic.decode_properties(@data) if defined?(Basic)
        end
      end
    end

    class BodyFrame < FrameSubclass
      self.id = 3

      def decode_payload
        @payload
      end

      def final?
        false
      end
    end

    class HeartbeatFrame < FrameSubclass
      self.id = 8

      def self.encode
        super("", 0)
      end

      def final?
        true
      end
    end

    # Frame class lookup
    Frame::CLASSES = {
      Frame::TYPES[:method] => MethodFrame,
      Frame::TYPES[:headers] => HeaderFrame,
      Frame::TYPES[:body] => BodyFrame,
      Frame::TYPES[:heartbeat] => HeartbeatFrame
    }.freeze

    # Exception classes
    class Error < StandardError; end

    class SoftError < Error
    end

    class HardError < Error
    end

    class FrameTypeError < Error
      def initialize(types)
        super("Invalid frame type. Expected one of: #{types.inspect}")
      end
    end

    class EmptyResponseError < Error
      def initialize
        super("Empty response received")
      end
    end

    # Standard AMQP errors
    class ContentTooLarge < SoftError; VALUE = 311; end
    class NoRoute < SoftError; VALUE = 312; end
    class NoConsumers < SoftError; VALUE = 313; end
    class AccessRefused < SoftError; VALUE = 403; end
    class NotFound < SoftError; VALUE = 404; end
    class ResourceLocked < SoftError; VALUE = 405; end
    class PreconditionFailed < SoftError; VALUE = 406; end

    class ConnectionForced < HardError; VALUE = 320; end
    class InvalidPath < HardError; VALUE = 402; end
    class FrameError < HardError; VALUE = 501; end
    class SyntaxError < HardError; VALUE = 502; end
    class CommandInvalid < HardError; VALUE = 503; end
    class ChannelError < HardError; VALUE = 504; end
    class UnexpectedFrame < HardError; VALUE = 505; end
    class ResourceError < HardError; VALUE = 506; end
    class NotAllowed < HardError; VALUE = 530; end
    class NotImplemented < HardError; VALUE = 540; end
    class InternalError < HardError; VALUE = 541; end

    # Basic properties
    if defined?(Basic)
      Basic::PROPERTIES = [
        :content_type,
        :content_encoding,
        :headers,
        :delivery_mode,
        :priority,
        :correlation_id,
        :reply_to,
        :expiration,
        :message_id,
        :timestamp,
        :type,
        :user_id,
        :app_id,
        :cluster_id
      ].freeze
    end
  end
end

# Backwards compatibility
module AMQ
  Pack = Protocol
  Hacks = Protocol
end
