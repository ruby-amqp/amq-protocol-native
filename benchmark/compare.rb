#!/usr/bin/env ruby
# frozen_string_literal: true

# Comparative benchmark between amq-protocol-native (Rust) and amq-protocol (Ruby)
#
# Usage: ruby benchmark/compare.rb
#
# Make sure both gems are available:
#   - This gem (amq-protocol-native) must be compiled: bundle exec rake compile
#   - Pure Ruby gem: gem install amq-protocol

require "benchmark/ips"

# Try to load native version
begin
  $LOAD_PATH.unshift(File.expand_path("../lib", __dir__))
  require "amq-protocol-native"
  NATIVE_AVAILABLE = true
rescue LoadError => e
  puts "Warning: Native version not available: #{e.message}"
  NATIVE_AVAILABLE = false
end

# Try to load pure Ruby version
begin
  gem "amq-protocol"
  require "amq/protocol/client"
  PURE_RUBY_AVAILABLE = true

  # Create aliases to differentiate
  module PureRuby
    Table = AMQ::Protocol::Table
    Frame = AMQ::Protocol::Frame
  end
rescue LoadError, Gem::MissingSpecError => e
  puts "Warning: Pure Ruby version not available: #{e.message}"
  PURE_RUBY_AVAILABLE = false
end

unless NATIVE_AVAILABLE || PURE_RUBY_AVAILABLE
  puts "Error: Neither version is available!"
  exit 1
end

puts
puts "=" * 80
puts "AMQ-Protocol Comparison Benchmark"
puts "=" * 80
puts "Ruby: #{RUBY_DESCRIPTION}"
puts "Native (Rust): #{NATIVE_AVAILABLE ? 'Available' : 'NOT AVAILABLE'}"
puts "Pure Ruby: #{PURE_RUBY_AVAILABLE ? 'Available' : 'NOT AVAILABLE'}"
puts "=" * 80
puts

# Test data
SIMPLE_TABLE = {
  "key1" => "value1",
  "key2" => 42,
  "key3" => true
}.freeze

COMPLEX_TABLE = {
  "string" => "hello world",
  "integer" => 123456789,
  "float" => 3.14159,
  "boolean_true" => true,
  "boolean_false" => false,
  "nested" => {
    "inner_key" => "inner_value",
    "inner_number" => 999
  },
  "array" => [1, 2, 3, "four", true]
}.freeze

LARGE_TABLE = (1..50).to_h { |i| ["key_#{i}", "value_#{i}"] }.freeze

if NATIVE_AVAILABLE && PURE_RUBY_AVAILABLE
  puts "=== Table Encoding Comparison ==="
  puts

  # For comparison, we need to use the native module directly
  NativeTable = AMQ::Protocol::Table

  Benchmark.ips do |x|
    x.config(time: 5, warmup: 2)

    x.report("Native: encode simple") do
      NativeTable.encode(SIMPLE_TABLE)
    end

    x.report("Pure Ruby: encode simple") do
      PureRuby::Table.encode(SIMPLE_TABLE)
    end

    x.compare!
  end

  puts

  Benchmark.ips do |x|
    x.config(time: 5, warmup: 2)

    x.report("Native: encode complex") do
      NativeTable.encode(COMPLEX_TABLE)
    end

    x.report("Pure Ruby: encode complex") do
      PureRuby::Table.encode(COMPLEX_TABLE)
    end

    x.compare!
  end

  puts

  Benchmark.ips do |x|
    x.config(time: 5, warmup: 2)

    x.report("Native: encode large (50 keys)") do
      NativeTable.encode(LARGE_TABLE)
    end

    x.report("Pure Ruby: encode large (50 keys)") do
      PureRuby::Table.encode(LARGE_TABLE)
    end

    x.compare!
  end

  # Pre-encode for decode benchmarks
  encoded_simple = NativeTable.encode(SIMPLE_TABLE)
  encoded_complex = NativeTable.encode(COMPLEX_TABLE)
  encoded_large = NativeTable.encode(LARGE_TABLE)

  puts
  puts "=== Table Decoding Comparison ==="
  puts

  Benchmark.ips do |x|
    x.config(time: 5, warmup: 2)

    x.report("Native: decode simple") do
      NativeTable.decode(encoded_simple)
    end

    x.report("Pure Ruby: decode simple") do
      PureRuby::Table.decode(encoded_simple)
    end

    x.compare!
  end

  puts

  Benchmark.ips do |x|
    x.config(time: 5, warmup: 2)

    x.report("Native: decode complex") do
      NativeTable.decode(encoded_complex)
    end

    x.report("Pure Ruby: decode complex") do
      PureRuby::Table.decode(encoded_complex)
    end

    x.compare!
  end

  puts

  Benchmark.ips do |x|
    x.config(time: 5, warmup: 2)

    x.report("Native: decode large (50 keys)") do
      NativeTable.decode(encoded_large)
    end

    x.report("Pure Ruby: decode large (50 keys)") do
      PureRuby::Table.decode(encoded_large)
    end

    x.compare!
  end

  puts
  puts "=== Frame Encoding Comparison ==="
  puts

  MEDIUM_PAYLOAD = "x" * 1024

  Benchmark.ips do |x|
    x.config(time: 5, warmup: 2)

    x.report("Native: Frame.encode(:method)") do
      AMQ::Protocol::Frame.encode(:method, MEDIUM_PAYLOAD, 1)
    end

    x.report("Pure Ruby: Frame.encode(:method)") do
      PureRuby::Frame.encode(:method, MEDIUM_PAYLOAD, 1)
    end

    x.compare!
  end

  header = [1, 0, 1, 0, 0, 4, 0].pack("CnN")

  puts

  Benchmark.ips do |x|
    x.config(time: 5, warmup: 2)

    x.report("Native: Frame.decode_header") do
      AMQ::Protocol::Frame.decode_header(header)
    end

    x.report("Pure Ruby: Frame.decode_header") do
      PureRuby::Frame.decode_header(header)
    end

    x.compare!
  end

else
  puts "Comparison not available - need both versions installed"
  puts
  puts "To run comparison:"
  puts "  1. Compile this gem: bundle exec rake compile"
  puts "  2. Install pure Ruby gem: gem install amq-protocol"
  puts "  3. Run: ruby benchmark/compare.rb"
end
