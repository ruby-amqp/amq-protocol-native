//! Native AMQP 0.9.1 serialization library for Ruby

mod error;
mod frame;
mod methods;
mod table;
mod types;

use magnus::{prelude::*, Error, Ruby};

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let amq = ruby.define_module("AMQ")?;
    let protocol = amq.define_module("Protocol")?;

    protocol.const_set("PROTOCOL_VERSION", "0.9.1")?;
    protocol.const_set("PREAMBLE", "AMQP\x00\x00\x09\x01")?;
    protocol.const_set("DEFAULT_PORT", 5672)?;
    protocol.const_set("TLS_PORT", 5671)?;
    protocol.const_set("SSL_PORT", 5671)?;

    table::init(ruby, &protocol)?;
    frame::init(ruby, &protocol)?;
    methods::init(ruby, &protocol)?;

    Ok(())
}
