#!/usr/bin/env ruby
# frozen_string_literal: true

BENCHMARK_DIR = File.dirname(__FILE__)

benchmarks = %w[
  table_encoding.rb
  frame_encoding.rb
  method_encoding.rb
]

puts "=" * 80
puts "AMQ-Protocol-Native Benchmark Suite"
puts "=" * 80
puts "Ruby: #{RUBY_DESCRIPTION}"
puts "Time: #{Time.now}"
puts "=" * 80
puts

benchmarks.each do |benchmark|
  benchmark_path = File.join(BENCHMARK_DIR, benchmark)

  if File.exist?(benchmark_path)
    puts "\n>>> Running #{benchmark}..."
    puts

    system("ruby", benchmark_path)
  else
    puts "Warning: #{benchmark_path} not found, skipping..."
  end
end

puts
puts "=" * 80
puts "Benchmark complete!"
puts "=" * 80
