# Instructions for AI Agents

## Overview

This is a native implementation of the Ruby `amq-protocol` gem (an AMQP 0-9-1 serializer and deserializer)
that strives to be a (much) more efficient drop-in replacement
to be eventually adopted by Bunny.


## Build and Test

```bash
bundle install
bundle exec rake compile
bundle exec rake spec
```

For Rust linting:

```bash
cargo fmt --manifest-path ext/amq_protocol_native/Cargo.toml
cargo clippy --manifest-path ext/amq_protocol_native/Cargo.toml
```

## Target Rust Version

 * This project requires Rust 1.70+

## Rust Code Style

 * Use top-level `use` statements (imports) to fully-qualified names, e.g. `Display` or `fmt::Display` with a `use` statement, to `std::fmt::Display`
 * Never use function-local `use` statements (imports)
 * Add tests to the modules under `tests`, never in the implementation files
 * At the end of each task, run `cargo fmt --all`
 * At the end of each task, run `cargo clippy --all` and fix any warnings it might emit

## Comments

 * Only add very important comments, both in tests and in the implementation

## Git Instructions

 * Never add yourself to the list of commit co-authors

## Style Guide

 * Never add full stops to Markdown list items
