#!/usr/bin/env ruby
# frozen_string_literal: true

$LOAD_PATH.unshift(File.expand_path("../lib", __dir__))

require "amq-protocol-native"
require "benchmark/ips"

puts
puts "-" * 80
puts "AMQP Method Encoding Benchmarks on #{RUBY_DESCRIPTION}"
puts "-" * 80

puts "=== Connection Methods ==="

Benchmark.ips do |x|
  x.config(time: 5, warmup: 2)

  x.report("Connection::StartOk.encode") do
    AMQ::Protocol::Connection::StartOk.encode(
      { product: "benchmark" },
      "PLAIN",
      "\x00guest\x00guest",
      "en_US"
    )
  end

  x.report("Connection::TuneOk.encode") do
    AMQ::Protocol::Connection::TuneOk.encode(2047, 131072, 60)
  end

  x.report("Connection::Open.encode") do
    AMQ::Protocol::Connection::Open.encode("/")
  end

  x.report("Connection::Close.encode") do
    AMQ::Protocol::Connection::Close.encode(200, "Normal shutdown", 0, 0)
  end

  x.compare!
end

puts
puts "=== Queue/Exchange Methods ==="

Benchmark.ips do |x|
  x.config(time: 5, warmup: 2)

  x.report("Queue::Declare.encode") do
    AMQ::Protocol::Queue::Declare.encode(
      "test.queue",
      false,
      true,
      false,
      false,
      false,
      {}
    )
  end

  x.report("Queue::Bind.encode") do
    AMQ::Protocol::Queue::Bind.encode(
      "test.queue",
      "test.exchange",
      "routing.key",
      false,
      {}
    )
  end

  x.report("Exchange::Declare.encode") do
    AMQ::Protocol::Exchange::Declare.encode(
      "test.exchange",
      "topic",
      false,
      true,
      false,
      false,
      false,
      {}
    )
  end

  x.compare!
end

puts
puts "=== Basic Methods (Hot Path) ==="

Benchmark.ips do |x|
  x.config(time: 5, warmup: 2)

  x.report("Basic::Publish.encode") do
    AMQ::Protocol::Basic::Publish.encode(
      "test.exchange",
      "routing.key",
      false,
      false
    )
  end

  x.report("Basic::Consume.encode") do
    AMQ::Protocol::Basic::Consume.encode(
      "test.queue",
      "consumer-tag",
      false,
      true,
      false,
      false,
      {}
    )
  end

  x.report("Basic::Ack.encode") do
    AMQ::Protocol::Basic::Ack.encode(12345, false)
  end

  x.report("Basic::Nack.encode") do
    AMQ::Protocol::Basic::Nack.encode(12345, false, true)
  end

  x.report("Basic::Reject.encode") do
    AMQ::Protocol::Basic::Reject.encode(12345, true)
  end

  x.compare!
end
