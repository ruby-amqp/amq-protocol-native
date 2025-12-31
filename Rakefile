# frozen_string_literal: true

require "bundler/gem_tasks"
require "rspec/core/rake_task"
require "rake/extensiontask"
require "rb_sys/extensiontask"

GEMSPEC = Gem::Specification.load("amq-protocol-native.gemspec")

RbSys::ExtensionTask.new("amq_protocol_native", GEMSPEC) do |ext|
  ext.lib_dir = "lib/amq/protocol"
  ext.cross_compile = true
  ext.cross_platform = %w[
    aarch64-linux
    aarch64-linux-musl
    arm64-darwin
    x64-mingw-ucrt
    x64-mingw32
    x86_64-darwin
    x86_64-linux
    x86_64-linux-musl
  ]
end

RSpec::Core::RakeTask.new(:spec)

task default: %i[compile spec]

desc "Run benchmarks"
task :benchmark do
  require_relative "benchmark/run"
end

namespace :native do
  desc "Build native gems for all platforms"
  task :build do
    sh "bundle exec rake native gem"
  end
end
