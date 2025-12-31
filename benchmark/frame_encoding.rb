#!/usr/bin/env ruby
# frozen_string_literal: true

$LOAD_PATH.unshift(File.expand_path("../lib", __dir__))

require "amq-protocol-native"
require "benchmark/ips"

puts
puts "-" * 80
puts "Frame Encoding Benchmarks on #{RUBY_DESCRIPTION}"
puts "-" * 80

SMALL_PAYLOAD = "x" * 100
MEDIUM_PAYLOAD = "x" * 1024
LARGE_PAYLOAD = "x" * 16384

CHANNEL = 1

Benchmark.ips do |x|
  x.config(time: 5, warmup: 2)

  x.report("Frame.encode(:method, small)") do
    AMQ::Protocol::Frame.encode(:method, SMALL_PAYLOAD, CHANNEL)
  end

  x.report("Frame.encode(:method, medium)") do
    AMQ::Protocol::Frame.encode(:method, MEDIUM_PAYLOAD, CHANNEL)
  end

  x.report("Frame.encode(:body, large)") do
    AMQ::Protocol::Frame.encode(:body, LARGE_PAYLOAD, CHANNEL)
  end

  x.report("MethodFrame.new + encode") do
    frame = AMQ::Protocol::MethodFrame.new(SMALL_PAYLOAD, CHANNEL)
    frame.encode
  end

  x.report("BodyFrame.new + encode") do
    frame = AMQ::Protocol::BodyFrame.new(MEDIUM_PAYLOAD, CHANNEL)
    frame.encode
  end

  x.report("HeartbeatFrame.encode") do
    AMQ::Protocol::HeartbeatFrame.encode
  end

  x.compare!
end

puts
puts "-" * 80
puts "Frame Header Decoding"
puts "-" * 80

METHOD_HEADER = [1, 0, 1, 0, 0, 0, 100].pack("CnN")
BODY_HEADER = [3, 0, 5, 0, 0, 16, 0].pack("CnN")

Benchmark.ips do |x|
  x.config(time: 5, warmup: 2)

  x.report("Frame.decode_header (method)") do
    AMQ::Protocol::Frame.decode_header(METHOD_HEADER)
  end

  x.report("Frame.decode_header (body)") do
    AMQ::Protocol::Frame.decode_header(BODY_HEADER)
  end

  x.compare!
end
