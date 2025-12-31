#!/usr/bin/env ruby
# frozen_string_literal: true

# Compare native extension against local pure Ruby 2.4.0

require "benchmark/ips"

# Load native version first
$LOAD_PATH.unshift(File.expand_path("../lib", __dir__))
require "amq-protocol-native"
NativeTable = AMQ::Protocol::Table
NativeFrame = AMQ::Protocol::Frame

# Load pure Ruby version
$LOAD_PATH.unshift(File.expand_path("../../amq-protocol.git/lib", __dir__))
load File.expand_path("../../amq-protocol.git/lib/amq/protocol/table.rb", __dir__)
PureRubyTable = AMQ::Protocol::Table

SIMPLE = { "key1" => "value1", "key2" => 42, "key3" => true }.freeze
COMPLEX = {
  "string" => "hello",
  "int" => 123456789,
  "float" => 3.14159,
  "bool" => true,
  "nested" => { "inner" => "value" },
  "array" => [1, 2, 3]
}.freeze
LARGE = (1..50).to_h { |i| ["key_#{i}", "value_#{i}"] }.freeze

encoded_simple = NativeTable.encode(SIMPLE)
encoded_complex = NativeTable.encode(COMPLEX)
encoded_large = NativeTable.encode(LARGE)

puts "Ruby: #{RUBY_DESCRIPTION}"
puts "Comparing Native (Rust) vs Pure Ruby 2.4.0"
puts "=" * 60
puts

puts "=== Table Encoding ==="
Benchmark.ips do |x|
  x.config(time: 3, warmup: 1)
  x.report("Native: simple") { NativeTable.encode(SIMPLE) }
  x.report("Ruby 2.4.0: simple") { PureRubyTable.encode(SIMPLE) }
  x.compare!
end

puts
Benchmark.ips do |x|
  x.config(time: 3, warmup: 1)
  x.report("Native: complex") { NativeTable.encode(COMPLEX) }
  x.report("Ruby 2.4.0: complex") { PureRubyTable.encode(COMPLEX) }
  x.compare!
end

puts
Benchmark.ips do |x|
  x.config(time: 3, warmup: 1)
  x.report("Native: large (50 keys)") { NativeTable.encode(LARGE) }
  x.report("Ruby 2.4.0: large (50 keys)") { PureRubyTable.encode(LARGE) }
  x.compare!
end

puts
puts "=== Table Decoding ==="
Benchmark.ips do |x|
  x.config(time: 3, warmup: 1)
  x.report("Native: decode simple") { NativeTable.decode(encoded_simple) }
  x.report("Ruby 2.4.0: decode simple") { PureRubyTable.decode(encoded_simple) }
  x.compare!
end

puts
Benchmark.ips do |x|
  x.config(time: 3, warmup: 1)
  x.report("Native: decode complex") { NativeTable.decode(encoded_complex) }
  x.report("Ruby 2.4.0: decode complex") { PureRubyTable.decode(encoded_complex) }
  x.compare!
end

puts
Benchmark.ips do |x|
  x.config(time: 3, warmup: 1)
  x.report("Native: decode large (50 keys)") { NativeTable.decode(encoded_large) }
  x.report("Ruby 2.4.0: decode large (50 keys)") { PureRubyTable.decode(encoded_large) }
  x.compare!
end
