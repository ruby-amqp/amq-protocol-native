# frozen_string_literal: true

require_relative "lib/amq/protocol/version"

Gem::Specification.new do |spec|
  spec.name = "amq-protocol-native"
  spec.version = AMQ::Protocol::VERSION
  spec.authors = ["Michael Klishin", "RabbitMQ Team"]
  spec.email = ["michael.s.klishin@gmail.com"]

  spec.summary = "Native AMQP 0.9.1 serialization library for Ruby (Rust extension)"
  spec.description = <<~DESC
    amq-protocol-native is a high-performance AMQP 0.9.1 serialization library for Ruby,
    implemented as a native extension in Rust. It is a drop-in replacement for the pure
    Ruby amq-protocol gem, providing significant performance improvements for encoding
    and decoding AMQP frames.
  DESC
  spec.homepage = "https://github.com/ruby-amqp/amq-protocol-native"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 2.7.0"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = spec.homepage
  spec.metadata["changelog_uri"] = "#{spec.homepage}/blob/main/CHANGELOG.md"
  spec.metadata["rubygems_mfa_required"] = "true"

  spec.files = Dir[
    "lib/**/*.rb",
    "ext/**/*.{rs,toml,rb,lock}",
    "Cargo.*",
    "LICENSE*",
    "README*",
    "CHANGELOG*"
  ]

  spec.require_paths = ["lib"]
  spec.extensions = ["ext/amq_protocol_native/extconf.rb"]

  spec.add_dependency "rb_sys", "~> 0.9"
end
