# amq-protocol-native

This is an **experimental** (see below) native port of [amq-protocol](https://github.com/ruby-amqp/amq-protocol),
an AMQP 0-9-1 serialization library for Ruby, implemented as a Rust extension.

This gem is meant to be a **drop-in replacement** for the pure Ruby [amq-protocol](https://github.com/ruby-amqp/amq-protocol) gem.


## Experiment Results

**TL;DR**: heavily optimized `amq-protocol` [`2.4.0` demonstrates substantial performance improvements](https://github.com/ruby-amqp/amq-protocol/releases/tag/v2.4.0) that
match this Rust implementation.

As a result, the work on this gem **was stopped**.

### Performance Comparison

Benchmarks comparing this native Rust implementation against the **significantly optimized [amq-protocol 2.4.0](https://github.com/ruby-amqp/amq-protocol)** (pure Ruby):

| Operation | Native (Rust) | Pure Ruby 2.4.0 | Difference |
|-----------|---------------|-----------------|------------|
| Encode simple table (3 keys) | 684k i/s | 681k i/s | same |
| Encode complex table (nested) | 222k i/s | 222k i/s | same |
| Encode large table (50 keys) | 47k i/s | 47k i/s | same |
| Decode simple table | 690k i/s | 699k i/s | same |
| Decode complex table | 209k i/s | 207k i/s | same |
| Decode large table (50 keys) | 47k i/s | 48k i/s | same |

*Benchmarks run on Ruby 3.4.8, Apple Silicon (arm64-darwin25)*

### Performance Comparison

The pure Ruby amq-protocol `2.4.0` includes significant optimizations that bring it to performance parity with native code:

 * Built-in `Q>`/`q>` pack/unpack directives (6-7x faster than legacy implementation)
 * `unpack1` instead of `unpack().first`
 * `byteslice` for binary string operations
 * `getbyte` for single-byte access
 * `frozen_string_literal` pragma throughout


## Features

 * Full AMQP 0-9-1 protocol support
 * Drop-in replacement for `amq-protocol` gem
 * Native Rust implementation
 * Pre-built binaries for common platforms (no Rust toolchain required)
 * Cross-platform: Linux (x86_64, aarch64), macOS (Intel, Apple Silicon), Windows

### Supported Platforms

Pre-built native gems are available for:

 * **Linux**: x86_64, aarch64 (both glibc and musl)
 * **macOS**: Intel (x86_64) and Apple Silicon (arm64)
 * **Windows**: x64 (UCRT)


## Usage

Simply require the gem instead of `amq-protocol`:

```ruby
require 'amq-protocol-native'

# Or use the compatibility alias
require 'amq/protocol'
```

The API is fully compatible with the pure Ruby version:

```ruby
# Encode a table
table = AMQ::Protocol::Table.encode({ "x-message-ttl" => 60000 })

# Encode a method frame
frame = AMQ::Protocol::Queue::Declare.encode(
  "my-queue",
  false,  # passive
  true,   # durable
  false,  # exclusive
  false,  # auto_delete
  false,  # nowait
  {}      # arguments
)

# Decode a frame header
type, channel, size = AMQ::Protocol::Frame.decode_header(header_bytes)
```

## Native Extension Differences

1. **Compilation Requirements**: If no pre-built binary is available, you'll need Rust 1.70+ installed
2. **Platform Support**: limited to platforms that are supposed by rb-sys and [Magnus]()
3. **Memory Model**: The Rust extension manages memory differently than Ruby; this is transparent but means GC behavior differs
4. **Error Messages**: Error messages may differ slightly from the pure Ruby version

## Development

### Prerequisites

- Ruby 2.7+
- Rust 1.70+
- Bundler

### Building

```bash
bundle install
bundle exec rake compile
```

### Running Tests

```bash
bundle exec rake spec
```

### Running Benchmarks

```bash
# Native-only benchmarks
bundle exec rake benchmark

# Comparison with pure Ruby (requires amq-protocol gem)
ruby benchmark/compare.rb
```

## License

MIT License. See [LICENSE](LICENSE) for details.

## Copyright

(c) Michael S. Klishin, 2025-2026
