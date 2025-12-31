//! AMQP 0.9.1 Method encoding

use magnus::{function, prelude::*, Error, Module, RHash, RString, Ruby};

use crate::table;
use crate::types::Encoder;

#[allow(dead_code)]
mod indices {
    pub const CONNECTION_START_OK: u32 = 0x000A000B;
    pub const CONNECTION_SECURE_OK: u32 = 0x000A0015;
    pub const CONNECTION_TUNE_OK: u32 = 0x000A001F;
    pub const CONNECTION_OPEN: u32 = 0x000A0028;
    pub const CONNECTION_CLOSE: u32 = 0x000A0032;
    pub const CONNECTION_CLOSE_OK: u32 = 0x000A0033;
    pub const CONNECTION_BLOCKED: u32 = 0x000A003C;
    pub const CONNECTION_UNBLOCKED: u32 = 0x000A003D;
    pub const CONNECTION_UPDATE_SECRET: u32 = 0x000A0046;
    pub const CONNECTION_UPDATE_SECRET_OK: u32 = 0x000A0047;
    pub const CHANNEL_OPEN: u32 = 0x0014000A;
    pub const CHANNEL_FLOW: u32 = 0x00140014;
    pub const CHANNEL_FLOW_OK: u32 = 0x00140015;
    pub const CHANNEL_CLOSE: u32 = 0x00140028;
    pub const CHANNEL_CLOSE_OK: u32 = 0x00140029;
    pub const EXCHANGE_DECLARE: u32 = 0x0028000A;
    pub const EXCHANGE_DELETE: u32 = 0x00280014;
    pub const EXCHANGE_BIND: u32 = 0x0028001E;
    pub const EXCHANGE_UNBIND: u32 = 0x00280028;
    pub const QUEUE_DECLARE: u32 = 0x0032000A;
    pub const QUEUE_BIND: u32 = 0x00320014;
    pub const QUEUE_UNBIND: u32 = 0x00320032;
    pub const QUEUE_PURGE: u32 = 0x0032001E;
    pub const QUEUE_DELETE: u32 = 0x00320028;
    pub const BASIC_QOS: u32 = 0x003C000A;
    pub const BASIC_CONSUME: u32 = 0x003C0014;
    pub const BASIC_CANCEL: u32 = 0x003C001E;
    pub const BASIC_PUBLISH: u32 = 0x003C0028;
    pub const BASIC_GET: u32 = 0x003C0046;
    pub const BASIC_ACK: u32 = 0x003C0050;
    pub const BASIC_REJECT: u32 = 0x003C005A;
    pub const BASIC_RECOVER_ASYNC: u32 = 0x003C0064;
    pub const BASIC_RECOVER: u32 = 0x003C006E;
    pub const BASIC_NACK: u32 = 0x003C0078;
    pub const TX_SELECT: u32 = 0x005A000A;
    pub const TX_COMMIT: u32 = 0x005A0014;
    pub const TX_ROLLBACK: u32 = 0x005A001E;
    pub const CONFIRM_SELECT: u32 = 0x0055000A;
    pub const CONFIRM_SELECT_OK: u32 = 0x0055000B;
}

fn write_method_header(encoder: &mut Encoder, class_id: u16, method_id: u16) {
    encoder.write_u16(class_id);
    encoder.write_u16(method_id);
}

fn pack_u64_be(v: u64) -> [u8; 8] {
    v.to_be_bytes()
}

fn encode_connection_start_ok(
    ruby: &Ruby,
    client_properties: RHash,
    mechanism: RString,
    response: RString,
    locale: RString,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(256);
    write_method_header(&mut encoder, 10, 11);

    let table_bytes = table::encode_table(ruby, client_properties).map_err(Error::from)?;
    encoder.write_bytes(&table_bytes);

    let mech_str = unsafe { mechanism.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(mech_str).unwrap_or("PLAIN"))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let resp_bytes = unsafe { response.as_slice() };
    encoder.write_long_string(resp_bytes);

    let locale_str = unsafe { locale.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(locale_str).unwrap_or("en_US"))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_connection_secure_ok(response: RString) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(64);
    write_method_header(&mut encoder, 10, 21);

    let resp_bytes = unsafe { response.as_slice() };
    encoder.write_long_string(resp_bytes);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_connection_tune_ok(
    channel_max: u16,
    frame_max: u32,
    heartbeat: u16,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(16);
    write_method_header(&mut encoder, 10, 31);
    encoder.write_u16(channel_max);
    encoder.write_u32(frame_max);
    encoder.write_u16(heartbeat);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_connection_open(virtual_host: RString) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(64);
    write_method_header(&mut encoder, 10, 40);

    let vhost_str = unsafe { virtual_host.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(vhost_str).unwrap_or("/"))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    encoder
        .write_short_string("")
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;
    encoder.write_u8(0);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_connection_close(
    reply_code: u16,
    reply_text: RString,
    class_id: u16,
    method_id: u16,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(64);
    write_method_header(&mut encoder, 10, 50);

    encoder.write_u16(reply_code);

    let text_str = unsafe { reply_text.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(text_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    encoder.write_u16(class_id);
    encoder.write_u16(method_id);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_connection_close_ok() -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(8);
    write_method_header(&mut encoder, 10, 51);
    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_connection_blocked(reason: RString) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(64);
    write_method_header(&mut encoder, 10, 60);

    let reason_str = unsafe { reason.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(reason_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_connection_unblocked() -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(8);
    write_method_header(&mut encoder, 10, 61);
    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_connection_update_secret(
    new_secret: RString,
    reason: RString,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(128);
    write_method_header(&mut encoder, 10, 70);

    let secret_bytes = unsafe { new_secret.as_slice() };
    encoder.write_long_string(secret_bytes);

    let reason_str = unsafe { reason.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(reason_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_connection_update_secret_ok() -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(8);
    write_method_header(&mut encoder, 10, 71);
    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_channel_open(out_of_band: RString) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(32);
    write_method_header(&mut encoder, 20, 10);

    let oob_str = unsafe { out_of_band.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(oob_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_channel_flow(active: bool) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(8);
    write_method_header(&mut encoder, 20, 20);
    encoder.write_u8(if active { 1 } else { 0 });
    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_channel_flow_ok(active: bool) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(8);
    write_method_header(&mut encoder, 20, 21);
    encoder.write_u8(if active { 1 } else { 0 });
    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_channel_close(
    reply_code: u16,
    reply_text: RString,
    class_id: u16,
    method_id: u16,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(64);
    write_method_header(&mut encoder, 20, 40);

    encoder.write_u16(reply_code);

    let text_str = unsafe { reply_text.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(text_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    encoder.write_u16(class_id);
    encoder.write_u16(method_id);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_channel_close_ok() -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(8);
    write_method_header(&mut encoder, 20, 41);
    Ok(RString::from_slice(encoder.as_slice()))
}

#[allow(clippy::too_many_arguments)]
fn encode_exchange_declare(
    ruby: &Ruby,
    exchange: RString,
    exchange_type: RString,
    passive: bool,
    durable: bool,
    auto_delete: bool,
    internal: bool,
    nowait: bool,
    arguments: RHash,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(256);
    write_method_header(&mut encoder, 40, 10);
    encoder.write_u16(0);

    let exch_str = unsafe { exchange.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(exch_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let type_str = unsafe { exchange_type.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(type_str).unwrap_or("direct"))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let mut flags: u8 = 0;
    if passive {
        flags |= 1 << 0;
    }
    if durable {
        flags |= 1 << 1;
    }
    if auto_delete {
        flags |= 1 << 2;
    }
    if internal {
        flags |= 1 << 3;
    }
    if nowait {
        flags |= 1 << 4;
    }
    encoder.write_u8(flags);

    let table_bytes = table::encode_table(ruby, arguments).map_err(Error::from)?;
    encoder.write_bytes(&table_bytes);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_exchange_delete(
    exchange: RString,
    if_unused: bool,
    nowait: bool,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(64);
    write_method_header(&mut encoder, 40, 20);
    encoder.write_u16(0);

    let exch_str = unsafe { exchange.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(exch_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let mut flags: u8 = 0;
    if if_unused {
        flags |= 1 << 0;
    }
    if nowait {
        flags |= 1 << 1;
    }
    encoder.write_u8(flags);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_exchange_bind(
    ruby: &Ruby,
    destination: RString,
    source: RString,
    routing_key: RString,
    nowait: bool,
    arguments: RHash,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(256);
    write_method_header(&mut encoder, 40, 30);
    encoder.write_u16(0);

    let dest_str = unsafe { destination.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(dest_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let src_str = unsafe { source.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(src_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let rk_str = unsafe { routing_key.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(rk_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    encoder.write_u8(if nowait { 1 } else { 0 });

    let table_bytes = table::encode_table(ruby, arguments).map_err(Error::from)?;
    encoder.write_bytes(&table_bytes);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_exchange_unbind(
    ruby: &Ruby,
    destination: RString,
    source: RString,
    routing_key: RString,
    nowait: bool,
    arguments: RHash,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(256);
    write_method_header(&mut encoder, 40, 40);
    encoder.write_u16(0);

    let dest_str = unsafe { destination.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(dest_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let src_str = unsafe { source.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(src_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let rk_str = unsafe { routing_key.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(rk_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    encoder.write_u8(if nowait { 1 } else { 0 });

    let table_bytes = table::encode_table(ruby, arguments).map_err(Error::from)?;
    encoder.write_bytes(&table_bytes);

    Ok(RString::from_slice(encoder.as_slice()))
}

#[allow(clippy::too_many_arguments)]
fn encode_queue_declare(
    ruby: &Ruby,
    queue: RString,
    passive: bool,
    durable: bool,
    exclusive: bool,
    auto_delete: bool,
    nowait: bool,
    arguments: RHash,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(256);
    write_method_header(&mut encoder, 50, 10);
    encoder.write_u16(0);

    let queue_str = unsafe { queue.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(queue_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let mut flags: u8 = 0;
    if passive {
        flags |= 1 << 0;
    }
    if durable {
        flags |= 1 << 1;
    }
    if exclusive {
        flags |= 1 << 2;
    }
    if auto_delete {
        flags |= 1 << 3;
    }
    if nowait {
        flags |= 1 << 4;
    }
    encoder.write_u8(flags);

    let table_bytes = table::encode_table(ruby, arguments).map_err(Error::from)?;
    encoder.write_bytes(&table_bytes);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_queue_bind(
    ruby: &Ruby,
    queue: RString,
    exchange: RString,
    routing_key: RString,
    nowait: bool,
    arguments: RHash,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(256);
    write_method_header(&mut encoder, 50, 20);
    encoder.write_u16(0);

    let queue_str = unsafe { queue.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(queue_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let exch_str = unsafe { exchange.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(exch_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let rk_str = unsafe { routing_key.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(rk_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    encoder.write_u8(if nowait { 1 } else { 0 });

    let table_bytes = table::encode_table(ruby, arguments).map_err(Error::from)?;
    encoder.write_bytes(&table_bytes);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_queue_unbind(
    ruby: &Ruby,
    queue: RString,
    exchange: RString,
    routing_key: RString,
    arguments: RHash,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(256);
    write_method_header(&mut encoder, 50, 50);
    encoder.write_u16(0);

    let queue_str = unsafe { queue.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(queue_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let exch_str = unsafe { exchange.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(exch_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let rk_str = unsafe { routing_key.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(rk_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let table_bytes = table::encode_table(ruby, arguments).map_err(Error::from)?;
    encoder.write_bytes(&table_bytes);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_queue_purge(queue: RString, nowait: bool) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(64);
    write_method_header(&mut encoder, 50, 30);

    encoder.write_u16(0);

    let queue_str = unsafe { queue.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(queue_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    encoder.write_u8(if nowait { 1 } else { 0 });

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_queue_delete(
    queue: RString,
    if_unused: bool,
    if_empty: bool,
    nowait: bool,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(64);
    write_method_header(&mut encoder, 50, 40);

    encoder.write_u16(0);

    let queue_str = unsafe { queue.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(queue_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let mut flags: u8 = 0;
    if if_unused {
        flags |= 1 << 0;
    }
    if if_empty {
        flags |= 1 << 1;
    }
    if nowait {
        flags |= 1 << 2;
    }
    encoder.write_u8(flags);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_basic_qos(
    prefetch_size: u32,
    prefetch_count: u16,
    global: bool,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(16);
    write_method_header(&mut encoder, 60, 10);

    encoder.write_u32(prefetch_size);
    encoder.write_u16(prefetch_count);
    encoder.write_u8(if global { 1 } else { 0 });

    Ok(RString::from_slice(encoder.as_slice()))
}

#[allow(clippy::too_many_arguments)]
fn encode_basic_consume(
    ruby: &Ruby,
    queue: RString,
    consumer_tag: RString,
    no_local: bool,
    no_ack: bool,
    exclusive: bool,
    nowait: bool,
    arguments: RHash,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(256);
    write_method_header(&mut encoder, 60, 20);
    encoder.write_u16(0);

    let queue_str = unsafe { queue.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(queue_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let tag_str = unsafe { consumer_tag.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(tag_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let mut flags: u8 = 0;
    if no_local {
        flags |= 1 << 0;
    }
    if no_ack {
        flags |= 1 << 1;
    }
    if exclusive {
        flags |= 1 << 2;
    }
    if nowait {
        flags |= 1 << 3;
    }
    encoder.write_u8(flags);

    let table_bytes = table::encode_table(ruby, arguments).map_err(Error::from)?;
    encoder.write_bytes(&table_bytes);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_basic_cancel(consumer_tag: RString, nowait: bool) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(64);
    write_method_header(&mut encoder, 60, 30);

    let tag_str = unsafe { consumer_tag.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(tag_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    encoder.write_u8(if nowait { 1 } else { 0 });

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_basic_publish(
    exchange: RString,
    routing_key: RString,
    mandatory: bool,
    immediate: bool,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(128);
    write_method_header(&mut encoder, 60, 40);
    encoder.write_u16(0);

    let exch_str = unsafe { exchange.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(exch_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let rk_str = unsafe { routing_key.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(rk_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    let mut flags: u8 = 0;
    if mandatory {
        flags |= 1 << 0;
    }
    if immediate {
        flags |= 1 << 1;
    }
    encoder.write_u8(flags);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_basic_get(queue: RString, no_ack: bool) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(64);
    write_method_header(&mut encoder, 60, 70);
    encoder.write_u16(0);

    let queue_str = unsafe { queue.as_slice() };
    encoder
        .write_short_string(std::str::from_utf8(queue_str).unwrap_or(""))
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.to_string()))?;

    encoder.write_u8(if no_ack { 1 } else { 0 });

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_basic_ack(delivery_tag: u64, multiple: bool) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(16);
    write_method_header(&mut encoder, 60, 80);

    encoder.write_bytes(&pack_u64_be(delivery_tag));
    encoder.write_u8(if multiple { 1 } else { 0 });

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_basic_reject(delivery_tag: u64, requeue: bool) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(16);
    write_method_header(&mut encoder, 60, 90);

    encoder.write_bytes(&pack_u64_be(delivery_tag));
    encoder.write_u8(if requeue { 1 } else { 0 });

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_basic_nack(
    delivery_tag: u64,
    multiple: bool,
    requeue: bool,
) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(16);
    write_method_header(&mut encoder, 60, 120);

    encoder.write_bytes(&pack_u64_be(delivery_tag));

    let mut flags: u8 = 0;
    if multiple {
        flags |= 1 << 0;
    }
    if requeue {
        flags |= 1 << 1;
    }
    encoder.write_u8(flags);

    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_basic_recover(requeue: bool) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(8);
    write_method_header(&mut encoder, 60, 110);
    encoder.write_u8(if requeue { 1 } else { 0 });
    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_basic_recover_async(requeue: bool) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(8);
    write_method_header(&mut encoder, 60, 100);
    encoder.write_u8(if requeue { 1 } else { 0 });
    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_tx_select() -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(8);
    write_method_header(&mut encoder, 90, 10);
    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_tx_commit() -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(8);
    write_method_header(&mut encoder, 90, 20);
    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_tx_rollback() -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(8);
    write_method_header(&mut encoder, 90, 30);
    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_confirm_select(nowait: bool) -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(8);
    write_method_header(&mut encoder, 85, 10);
    encoder.write_u8(if nowait { 1 } else { 0 });
    Ok(RString::from_slice(encoder.as_slice()))
}

fn encode_confirm_select_ok() -> std::result::Result<RString, Error> {
    let mut encoder = Encoder::with_capacity(8);
    write_method_header(&mut encoder, 85, 11);
    Ok(RString::from_slice(encoder.as_slice()))
}

pub fn init(ruby: &Ruby, protocol: &impl Module) -> std::result::Result<(), Error> {
    let class_base = protocol.define_class("Class", ruby.class_object())?;
    let method_base = protocol.define_class("Method", ruby.class_object())?;

    let connection = protocol.define_class("Connection", class_base)?;
    connection.const_set("@name", "connection")?;
    connection.const_set("@method_id", 10)?;

    let start_ok = connection.define_class("StartOk", method_base)?;
    start_ok.const_set("@name", "connection.start-ok")?;
    start_ok.const_set("@method_id", 11)?;
    start_ok.const_set("@index", indices::CONNECTION_START_OK)?;
    start_ok.define_singleton_method("encode", function!(encode_connection_start_ok, 4))?;

    let secure_ok = connection.define_class("SecureOk", method_base)?;
    secure_ok.const_set("@name", "connection.secure-ok")?;
    secure_ok.const_set("@method_id", 21)?;
    secure_ok.const_set("@index", indices::CONNECTION_SECURE_OK)?;
    secure_ok.define_singleton_method("encode", function!(encode_connection_secure_ok, 1))?;

    let tune_ok = connection.define_class("TuneOk", method_base)?;
    tune_ok.const_set("@name", "connection.tune-ok")?;
    tune_ok.const_set("@method_id", 31)?;
    tune_ok.const_set("@index", indices::CONNECTION_TUNE_OK)?;
    tune_ok.define_singleton_method("encode", function!(encode_connection_tune_ok, 3))?;

    let open = connection.define_class("Open", method_base)?;
    open.const_set("@name", "connection.open")?;
    open.const_set("@method_id", 40)?;
    open.const_set("@index", indices::CONNECTION_OPEN)?;
    open.define_singleton_method("encode", function!(encode_connection_open, 1))?;

    let close = connection.define_class("Close", method_base)?;
    close.const_set("@name", "connection.close")?;
    close.const_set("@method_id", 50)?;
    close.const_set("@index", indices::CONNECTION_CLOSE)?;
    close.define_singleton_method("encode", function!(encode_connection_close, 4))?;

    let close_ok = connection.define_class("CloseOk", method_base)?;
    close_ok.const_set("@name", "connection.close-ok")?;
    close_ok.const_set("@method_id", 51)?;
    close_ok.const_set("@index", indices::CONNECTION_CLOSE_OK)?;
    close_ok.define_singleton_method("encode", function!(encode_connection_close_ok, 0))?;

    let blocked = connection.define_class("Blocked", method_base)?;
    blocked.const_set("@name", "connection.blocked")?;
    blocked.const_set("@method_id", 60)?;
    blocked.const_set("@index", indices::CONNECTION_BLOCKED)?;
    blocked.define_singleton_method("encode", function!(encode_connection_blocked, 1))?;

    let unblocked = connection.define_class("Unblocked", method_base)?;
    unblocked.const_set("@name", "connection.unblocked")?;
    unblocked.const_set("@method_id", 61)?;
    unblocked.const_set("@index", indices::CONNECTION_UNBLOCKED)?;
    unblocked.define_singleton_method("encode", function!(encode_connection_unblocked, 0))?;

    let update_secret = connection.define_class("UpdateSecret", method_base)?;
    update_secret.const_set("@name", "connection.update-secret")?;
    update_secret.const_set("@method_id", 70)?;
    update_secret.const_set("@index", indices::CONNECTION_UPDATE_SECRET)?;
    update_secret
        .define_singleton_method("encode", function!(encode_connection_update_secret, 2))?;

    let update_secret_ok = connection.define_class("UpdateSecretOk", method_base)?;
    update_secret_ok.const_set("@name", "connection.update-secret-ok")?;
    update_secret_ok.const_set("@method_id", 71)?;
    update_secret_ok.const_set("@index", indices::CONNECTION_UPDATE_SECRET_OK)?;
    update_secret_ok
        .define_singleton_method("encode", function!(encode_connection_update_secret_ok, 0))?;

    let channel = protocol.define_class("Channel", class_base)?;
    channel.const_set("@name", "channel")?;
    channel.const_set("@method_id", 20)?;

    let ch_open = channel.define_class("Open", method_base)?;
    ch_open.const_set("@name", "channel.open")?;
    ch_open.const_set("@method_id", 10)?;
    ch_open.const_set("@index", indices::CHANNEL_OPEN)?;
    ch_open.define_singleton_method("encode", function!(encode_channel_open, 1))?;

    let ch_flow = channel.define_class("Flow", method_base)?;
    ch_flow.const_set("@name", "channel.flow")?;
    ch_flow.const_set("@method_id", 20)?;
    ch_flow.const_set("@index", indices::CHANNEL_FLOW)?;
    ch_flow.define_singleton_method("encode", function!(encode_channel_flow, 1))?;

    let ch_flow_ok = channel.define_class("FlowOk", method_base)?;
    ch_flow_ok.const_set("@name", "channel.flow-ok")?;
    ch_flow_ok.const_set("@method_id", 21)?;
    ch_flow_ok.const_set("@index", indices::CHANNEL_FLOW_OK)?;
    ch_flow_ok.define_singleton_method("encode", function!(encode_channel_flow_ok, 1))?;

    let ch_close = channel.define_class("Close", method_base)?;
    ch_close.const_set("@name", "channel.close")?;
    ch_close.const_set("@method_id", 40)?;
    ch_close.const_set("@index", indices::CHANNEL_CLOSE)?;
    ch_close.define_singleton_method("encode", function!(encode_channel_close, 4))?;

    let ch_close_ok = channel.define_class("CloseOk", method_base)?;
    ch_close_ok.const_set("@name", "channel.close-ok")?;
    ch_close_ok.const_set("@method_id", 41)?;
    ch_close_ok.const_set("@index", indices::CHANNEL_CLOSE_OK)?;
    ch_close_ok.define_singleton_method("encode", function!(encode_channel_close_ok, 0))?;

    let exchange = protocol.define_class("Exchange", class_base)?;
    exchange.const_set("@name", "exchange")?;
    exchange.const_set("@method_id", 40)?;

    let ex_declare = exchange.define_class("Declare", method_base)?;
    ex_declare.const_set("@name", "exchange.declare")?;
    ex_declare.const_set("@method_id", 10)?;
    ex_declare.const_set("@index", indices::EXCHANGE_DECLARE)?;
    ex_declare.define_singleton_method("encode", function!(encode_exchange_declare, 8))?;

    let ex_delete = exchange.define_class("Delete", method_base)?;
    ex_delete.const_set("@name", "exchange.delete")?;
    ex_delete.const_set("@method_id", 20)?;
    ex_delete.const_set("@index", indices::EXCHANGE_DELETE)?;
    ex_delete.define_singleton_method("encode", function!(encode_exchange_delete, 3))?;

    let ex_bind = exchange.define_class("Bind", method_base)?;
    ex_bind.const_set("@name", "exchange.bind")?;
    ex_bind.const_set("@method_id", 30)?;
    ex_bind.const_set("@index", indices::EXCHANGE_BIND)?;
    ex_bind.define_singleton_method("encode", function!(encode_exchange_bind, 5))?;

    let ex_unbind = exchange.define_class("Unbind", method_base)?;
    ex_unbind.const_set("@name", "exchange.unbind")?;
    ex_unbind.const_set("@method_id", 40)?;
    ex_unbind.const_set("@index", indices::EXCHANGE_UNBIND)?;
    ex_unbind.define_singleton_method("encode", function!(encode_exchange_unbind, 5))?;

    let queue = protocol.define_class("Queue", class_base)?;
    queue.const_set("@name", "queue")?;
    queue.const_set("@method_id", 50)?;

    let q_declare = queue.define_class("Declare", method_base)?;
    q_declare.const_set("@name", "queue.declare")?;
    q_declare.const_set("@method_id", 10)?;
    q_declare.const_set("@index", indices::QUEUE_DECLARE)?;
    q_declare.define_singleton_method("encode", function!(encode_queue_declare, 7))?;

    let q_bind = queue.define_class("Bind", method_base)?;
    q_bind.const_set("@name", "queue.bind")?;
    q_bind.const_set("@method_id", 20)?;
    q_bind.const_set("@index", indices::QUEUE_BIND)?;
    q_bind.define_singleton_method("encode", function!(encode_queue_bind, 5))?;

    let q_unbind = queue.define_class("Unbind", method_base)?;
    q_unbind.const_set("@name", "queue.unbind")?;
    q_unbind.const_set("@method_id", 50)?;
    q_unbind.const_set("@index", indices::QUEUE_UNBIND)?;
    q_unbind.define_singleton_method("encode", function!(encode_queue_unbind, 4))?;

    let q_purge = queue.define_class("Purge", method_base)?;
    q_purge.const_set("@name", "queue.purge")?;
    q_purge.const_set("@method_id", 30)?;
    q_purge.const_set("@index", indices::QUEUE_PURGE)?;
    q_purge.define_singleton_method("encode", function!(encode_queue_purge, 2))?;

    let q_delete = queue.define_class("Delete", method_base)?;
    q_delete.const_set("@name", "queue.delete")?;
    q_delete.const_set("@method_id", 40)?;
    q_delete.const_set("@index", indices::QUEUE_DELETE)?;
    q_delete.define_singleton_method("encode", function!(encode_queue_delete, 4))?;

    let basic = protocol.define_class("Basic", class_base)?;
    basic.const_set("@name", "basic")?;
    basic.const_set("@method_id", 60)?;

    let b_qos = basic.define_class("Qos", method_base)?;
    b_qos.const_set("@name", "basic.qos")?;
    b_qos.const_set("@method_id", 10)?;
    b_qos.const_set("@index", indices::BASIC_QOS)?;
    b_qos.define_singleton_method("encode", function!(encode_basic_qos, 3))?;

    let b_consume = basic.define_class("Consume", method_base)?;
    b_consume.const_set("@name", "basic.consume")?;
    b_consume.const_set("@method_id", 20)?;
    b_consume.const_set("@index", indices::BASIC_CONSUME)?;
    b_consume.define_singleton_method("encode", function!(encode_basic_consume, 7))?;

    let b_cancel = basic.define_class("Cancel", method_base)?;
    b_cancel.const_set("@name", "basic.cancel")?;
    b_cancel.const_set("@method_id", 30)?;
    b_cancel.const_set("@index", indices::BASIC_CANCEL)?;
    b_cancel.define_singleton_method("encode", function!(encode_basic_cancel, 2))?;

    let b_publish = basic.define_class("Publish", method_base)?;
    b_publish.const_set("@name", "basic.publish")?;
    b_publish.const_set("@method_id", 40)?;
    b_publish.const_set("@index", indices::BASIC_PUBLISH)?;
    b_publish.define_singleton_method("encode", function!(encode_basic_publish, 4))?;

    let b_get = basic.define_class("Get", method_base)?;
    b_get.const_set("@name", "basic.get")?;
    b_get.const_set("@method_id", 70)?;
    b_get.const_set("@index", indices::BASIC_GET)?;
    b_get.define_singleton_method("encode", function!(encode_basic_get, 2))?;

    let b_ack = basic.define_class("Ack", method_base)?;
    b_ack.const_set("@name", "basic.ack")?;
    b_ack.const_set("@method_id", 80)?;
    b_ack.const_set("@index", indices::BASIC_ACK)?;
    b_ack.define_singleton_method("encode", function!(encode_basic_ack, 2))?;

    let b_reject = basic.define_class("Reject", method_base)?;
    b_reject.const_set("@name", "basic.reject")?;
    b_reject.const_set("@method_id", 90)?;
    b_reject.const_set("@index", indices::BASIC_REJECT)?;
    b_reject.define_singleton_method("encode", function!(encode_basic_reject, 2))?;

    let b_nack = basic.define_class("Nack", method_base)?;
    b_nack.const_set("@name", "basic.nack")?;
    b_nack.const_set("@method_id", 120)?;
    b_nack.const_set("@index", indices::BASIC_NACK)?;
    b_nack.define_singleton_method("encode", function!(encode_basic_nack, 3))?;

    let b_recover = basic.define_class("Recover", method_base)?;
    b_recover.const_set("@name", "basic.recover")?;
    b_recover.const_set("@method_id", 110)?;
    b_recover.const_set("@index", indices::BASIC_RECOVER)?;
    b_recover.define_singleton_method("encode", function!(encode_basic_recover, 1))?;

    let b_recover_async = basic.define_class("RecoverAsync", method_base)?;
    b_recover_async.const_set("@name", "basic.recover-async")?;
    b_recover_async.const_set("@method_id", 100)?;
    b_recover_async.const_set("@index", indices::BASIC_RECOVER_ASYNC)?;
    b_recover_async.define_singleton_method("encode", function!(encode_basic_recover_async, 1))?;

    let tx = protocol.define_class("Tx", class_base)?;
    tx.const_set("@name", "tx")?;
    tx.const_set("@method_id", 90)?;

    let tx_select = tx.define_class("Select", method_base)?;
    tx_select.const_set("@name", "tx.select")?;
    tx_select.const_set("@method_id", 10)?;
    tx_select.const_set("@index", indices::TX_SELECT)?;
    tx_select.define_singleton_method("encode", function!(encode_tx_select, 0))?;

    let tx_commit = tx.define_class("Commit", method_base)?;
    tx_commit.const_set("@name", "tx.commit")?;
    tx_commit.const_set("@method_id", 20)?;
    tx_commit.const_set("@index", indices::TX_COMMIT)?;
    tx_commit.define_singleton_method("encode", function!(encode_tx_commit, 0))?;

    let tx_rollback = tx.define_class("Rollback", method_base)?;
    tx_rollback.const_set("@name", "tx.rollback")?;
    tx_rollback.const_set("@method_id", 30)?;
    tx_rollback.const_set("@index", indices::TX_ROLLBACK)?;
    tx_rollback.define_singleton_method("encode", function!(encode_tx_rollback, 0))?;

    let confirm = protocol.define_class("Confirm", class_base)?;
    confirm.const_set("@name", "confirm")?;
    confirm.const_set("@method_id", 85)?;

    let confirm_select = confirm.define_class("Select", method_base)?;
    confirm_select.const_set("@name", "confirm.select")?;
    confirm_select.const_set("@method_id", 10)?;
    confirm_select.const_set("@index", indices::CONFIRM_SELECT)?;
    confirm_select.define_singleton_method("encode", function!(encode_confirm_select, 1))?;

    let confirm_select_ok = confirm.define_class("SelectOk", method_base)?;
    confirm_select_ok.const_set("@name", "confirm.select-ok")?;
    confirm_select_ok.const_set("@method_id", 11)?;
    confirm_select_ok.const_set("@index", indices::CONFIRM_SELECT_OK)?;
    confirm_select_ok.define_singleton_method("encode", function!(encode_confirm_select_ok, 0))?;

    Ok(())
}
