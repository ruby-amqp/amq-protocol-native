//! AMQP Field Table encoding and decoding

use magnus::{
    function, prelude::*, Error, Module, RArray, RHash, RString, Ruby, Symbol, TryConvert, Value,
};

use crate::error::{AmqpError, Result};
use crate::types::{Decoder, Encoder};

mod type_tags {
    pub const STRING: u8 = b'S';
    pub const INTEGER: u8 = b'I';
    pub const TIME: u8 = b'T';
    pub const DECIMAL: u8 = b'D';
    pub const HASH: u8 = b'F';
    pub const ARRAY: u8 = b'A';
    pub const BYTE: u8 = b'b';
    pub const DOUBLE: u8 = b'd';
    pub const FLOAT: u8 = b'f';
    pub const LONG: u8 = b'l';
    pub const SHORT: u8 = b's';
    pub const BOOLEAN: u8 = b't';
    pub const BYTE_ARRAY: u8 = b'x';
    pub const VOID: u8 = b'V';
}

pub fn encode_table(ruby: &Ruby, hash: RHash) -> Result<Vec<u8>> {
    let mut encoder = Encoder::new();
    encode_table_inner(ruby, hash, &mut encoder)?;
    Ok(encoder.into_bytes().to_vec())
}

fn encode_table_inner(ruby: &Ruby, hash: RHash, encoder: &mut Encoder) -> Result<()> {
    let mut content_encoder = Encoder::new();

    hash.foreach(|key: Value, value: Value| {
        let key_str: String = if key.is_kind_of(ruby.class_symbol()) {
            let sym: Symbol = TryConvert::try_convert(key).map_err(|_| {
                magnus::Error::new(
                    magnus::exception::type_error(),
                    "Expected symbol or string key",
                )
            })?;
            sym.name()
                .map_err(|_| {
                    magnus::Error::new(magnus::exception::encoding_error(), "Invalid symbol name")
                })?
                .to_string()
        } else {
            let s: String = TryConvert::try_convert(key).map_err(|_| {
                magnus::Error::new(
                    magnus::exception::type_error(),
                    "Expected symbol or string key",
                )
            })?;
            s
        };

        if key_str.len() > 255 {
            return Err(magnus::Error::new(
                magnus::exception::arg_error(),
                format!("Table key too long: {} (max 255)", key_str.len()),
            ));
        }
        content_encoder.write_u8(key_str.len() as u8);
        content_encoder.write_bytes(key_str.as_bytes());

        encode_field_value(ruby, value, &mut content_encoder)
            .map_err(|e| magnus::Error::new(magnus::exception::runtime_error(), e.to_string()))?;

        Ok(magnus::r_hash::ForEach::Continue)
    })
    .map_err(|e| AmqpError::EncodingError(e.to_string()))?;

    let content = content_encoder.into_bytes();
    encoder.write_u32(content.len() as u32);
    encoder.write_bytes(&content);

    Ok(())
}

fn encode_field_value(ruby: &Ruby, value: Value, encoder: &mut Encoder) -> Result<()> {
    if value.is_nil() {
        encoder.write_u8(type_tags::VOID);
        return Ok(());
    }

    if value.is_kind_of(ruby.class_string()) {
        let s: String = TryConvert::try_convert(value)
            .map_err(|_| AmqpError::EncodingError("Failed to convert string".into()))?;
        encoder.write_u8(type_tags::STRING);
        encoder.write_long_string(s.as_bytes());
    } else if value.is_kind_of(ruby.class_symbol()) {
        let sym: Symbol = TryConvert::try_convert(value)
            .map_err(|_| AmqpError::EncodingError("Failed to convert symbol".into()))?;
        let s = sym
            .name()
            .map_err(|_| AmqpError::EncodingError("Invalid symbol name".into()))?;
        encoder.write_u8(type_tags::STRING);
        encoder.write_long_string(s.as_bytes());
    } else if value.is_kind_of(ruby.class_integer()) {
        let i: i64 = TryConvert::try_convert(value)
            .map_err(|_| AmqpError::EncodingError("Failed to convert integer".into()))?;
        encoder.write_u8(type_tags::LONG);
        encoder.write_i64(i);
    } else if value.is_kind_of(ruby.class_float()) {
        let f: f64 = TryConvert::try_convert(value)
            .map_err(|_| AmqpError::EncodingError("Failed to convert float".into()))?;
        encoder.write_u8(type_tags::DOUBLE);
        encoder.write_f64(f);
    } else if value.is_kind_of(ruby.class_true_class())
        || value.is_kind_of(ruby.class_false_class())
    {
        let b: bool = TryConvert::try_convert(value)
            .map_err(|_| AmqpError::EncodingError("Failed to convert boolean".into()))?;
        encoder.write_u8(type_tags::BOOLEAN);
        encoder.write_u8(if b { 1 } else { 0 });
    } else if value.is_kind_of(ruby.class_hash()) {
        let hash: RHash = TryConvert::try_convert(value)
            .map_err(|_| AmqpError::EncodingError("Failed to convert hash".into()))?;
        encoder.write_u8(type_tags::HASH);
        encode_table_inner(ruby, hash, encoder)?;
    } else if value.is_kind_of(ruby.class_array()) {
        let array: RArray = TryConvert::try_convert(value)
            .map_err(|_| AmqpError::EncodingError("Failed to convert array".into()))?;
        encoder.write_u8(type_tags::ARRAY);
        encode_array(ruby, array, encoder)?;
    } else if value.is_kind_of(ruby.class_time()) {
        let timestamp: i64 = value
            .funcall("to_i", ())
            .map_err(|_| AmqpError::EncodingError("Failed to get timestamp".into()))?;
        encoder.write_u8(type_tags::TIME);
        encoder.write_i64(timestamp);
    } else {
        let class_name: String = value
            .class()
            .funcall("name", ())
            .unwrap_or_else(|_| "unknown".to_string());
        return Err(AmqpError::InvalidTableValue(
            "unknown".to_string(),
            class_name,
        ));
    }

    Ok(())
}

fn encode_array(ruby: &Ruby, array: RArray, encoder: &mut Encoder) -> Result<()> {
    let mut content_encoder = Encoder::new();

    for i in 0..array.len() {
        let value: Value = array
            .entry(i as isize)
            .map_err(|_| AmqpError::EncodingError("Failed to get array entry".into()))?;
        encode_field_value(ruby, value, &mut content_encoder)?;
    }

    let content = content_encoder.into_bytes();
    encoder.write_u32(content.len() as u32);
    encoder.write_bytes(&content);

    Ok(())
}

pub fn decode_table(ruby: &Ruby, data: &[u8]) -> Result<RHash> {
    let mut decoder = Decoder::new(data);
    decode_table_inner(ruby, &mut decoder)
}

fn decode_table_inner(ruby: &Ruby, decoder: &mut Decoder) -> Result<RHash> {
    let hash = ruby.hash_new();
    let table_length = decoder.read_u32()? as usize;

    if table_length == 0 {
        return Ok(hash);
    }

    let end_pos = decoder.position() + table_length;

    while decoder.position() < end_pos {
        let key = decoder.read_short_string_bytes()?;
        let key_str = std::str::from_utf8(key)
            .map_err(|e| AmqpError::DecodingError(format!("Invalid UTF-8 in key: {}", e)))?;

        let type_tag = decoder.read_u8()?;
        let value = decode_field_value(ruby, type_tag, decoder)?;

        hash.aset(key_str, value)
            .map_err(|e| AmqpError::DecodingError(format!("Failed to set hash key: {}", e)))?;
    }

    Ok(hash)
}

fn decode_field_value(ruby: &Ruby, type_tag: u8, decoder: &mut Decoder) -> Result<Value> {
    match type_tag {
        type_tags::STRING | type_tags::BYTE_ARRAY => {
            let bytes = decoder.read_long_string()?;
            let s = RString::from_slice(bytes);
            Ok(s.as_value())
        }
        type_tags::INTEGER => {
            let v = decoder.read_i32()?;
            Ok(ruby.integer_from_i64(v as i64).as_value())
        }
        type_tags::LONG => {
            let v = decoder.read_i64()?;
            Ok(ruby.integer_from_i64(v).as_value())
        }
        type_tags::SHORT => {
            let v = decoder.read_i16()?;
            Ok(ruby.integer_from_i64(v as i64).as_value())
        }
        type_tags::BYTE => {
            let v = decoder.read_i8()?;
            Ok(ruby.integer_from_i64(v as i64).as_value())
        }
        type_tags::TIME => {
            let timestamp = decoder.read_i64()?;
            let time_class = ruby.class_time();
            let time: Value = time_class
                .funcall("at", (timestamp,))
                .map_err(|e| AmqpError::DecodingError(format!("Failed to create Time: {}", e)))?;
            Ok(time)
        }
        type_tags::DECIMAL => {
            let scale = decoder.read_u8()?;
            let value = decoder.read_u32()?;
            let decimal = (value as f64) / (10_u32.pow(scale as u32) as f64);
            Ok(ruby.float_from_f64(decimal).as_value())
        }
        type_tags::FLOAT => {
            let v = decoder.read_f32()?;
            Ok(ruby.float_from_f64(v as f64).as_value())
        }
        type_tags::DOUBLE => {
            let v = decoder.read_f64()?;
            Ok(ruby.float_from_f64(v).as_value())
        }
        type_tags::BOOLEAN => {
            let v = decoder.read_u8()?;
            Ok(if v != 0 {
                ruby.qtrue().as_value()
            } else {
                ruby.qfalse().as_value()
            })
        }
        type_tags::HASH => {
            let hash = decode_table_inner(ruby, decoder)?;
            Ok(hash.as_value())
        }
        type_tags::ARRAY => {
            let array = decode_array(ruby, decoder)?;
            Ok(array.as_value())
        }
        type_tags::VOID => Ok(ruby.qnil().as_value()),
        _ => Err(AmqpError::InvalidTableType(type_tag as char)),
    }
}

fn decode_array(ruby: &Ruby, decoder: &mut Decoder) -> Result<RArray> {
    let array = ruby.ary_new();
    let array_length = decoder.read_u32()? as usize;

    if array_length == 0 {
        return Ok(array);
    }

    let end_pos = decoder.position() + array_length;

    while decoder.position() < end_pos {
        let type_tag = decoder.read_u8()?;
        let value = decode_field_value(ruby, type_tag, decoder)?;
        array
            .push(value)
            .map_err(|e| AmqpError::DecodingError(format!("Failed to push to array: {}", e)))?;
    }

    Ok(array)
}

fn rb_encode(ruby: &Ruby, hash: RHash) -> std::result::Result<RString, Error> {
    let bytes = encode_table(ruby, hash).map_err(Error::from)?;
    Ok(RString::from_slice(&bytes))
}

fn rb_decode(ruby: &Ruby, data: RString) -> std::result::Result<RHash, Error> {
    let bytes = unsafe { data.as_slice() };
    decode_table(ruby, bytes).map_err(Error::from)
}

fn rb_length(data: RString) -> std::result::Result<u32, Error> {
    let bytes = unsafe { data.as_slice() };
    if bytes.len() < 4 {
        return Err(Error::new(
            magnus::exception::arg_error(),
            "Data too short to contain table length",
        ));
    }
    let length = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    Ok(length)
}

pub fn init(ruby: &Ruby, protocol: &impl Module) -> std::result::Result<(), Error> {
    let table = protocol.define_class("Table", ruby.class_object())?;

    table.define_singleton_method("encode", function!(rb_encode, 1))?;
    table.define_singleton_method("decode", function!(rb_decode, 1))?;
    table.define_singleton_method("length", function!(rb_length, 1))?;

    let type_constants = protocol.define_module("TypeConstants")?;
    type_constants.const_set("TYPE_STRING", "S")?;
    type_constants.const_set("TYPE_INTEGER", "I")?;
    type_constants.const_set("TYPE_TIME", "T")?;
    type_constants.const_set("TYPE_DECIMAL", "D")?;
    type_constants.const_set("TYPE_HASH", "F")?;
    type_constants.const_set("TYPE_ARRAY", "A")?;
    type_constants.const_set("TYPE_BYTE", "b")?;
    type_constants.const_set("TYPE_64BIT_FLOAT", "d")?;
    type_constants.const_set("TYPE_32BIT_FLOAT", "f")?;
    type_constants.const_set("TYPE_SIGNED_64BIT", "l")?;
    type_constants.const_set("TYPE_SIGNED_16BIT", "s")?;
    type_constants.const_set("TYPE_BOOLEAN", "t")?;
    type_constants.const_set("TYPE_BYTE_ARRAY", "x")?;
    type_constants.const_set("TYPE_VOID", "V")?;
    type_constants.const_set("BOOLEAN_TRUE", "\x01")?;
    type_constants.const_set("BOOLEAN_FALSE", "\x00")?;

    Ok(())
}
