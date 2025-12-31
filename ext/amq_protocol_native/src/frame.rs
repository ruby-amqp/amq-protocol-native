//! AMQP 0-9-1 Frame encoding and decoding

use magnus::{function, prelude::*, Error, Module, RArray, RString, Ruby, TryConvert, Value};

use crate::error::{AmqpError, Result};
use crate::types::{Decoder, Encoder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    Method = 1,
    Headers = 2,
    Body = 3,
    Heartbeat = 8,
}

impl FrameType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(FrameType::Method),
            2 => Some(FrameType::Headers),
            3 => Some(FrameType::Body),
            8 => Some(FrameType::Heartbeat),
            _ => None,
        }
    }

    pub fn from_symbol(sym: Value) -> Option<Self> {
        if let Ok(s) = sym.funcall::<_, _, String>("to_s", ()) {
            match s.as_str() {
                "method" => Some(FrameType::Method),
                "headers" => Some(FrameType::Headers),
                "body" => Some(FrameType::Body),
                "heartbeat" => Some(FrameType::Heartbeat),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn symbol_name(self) -> &'static str {
        match self {
            FrameType::Method => "method",
            FrameType::Headers => "headers",
            FrameType::Body => "body",
            FrameType::Heartbeat => "heartbeat",
        }
    }
}

pub const FRAME_END: u8 = 0xCE;
pub const MAX_CHANNEL: u16 = 65535;
pub const FRAME_HEADER_SIZE: usize = 7;

pub fn encode_frame(frame_type: u8, channel: u16, payload: &[u8]) -> Vec<u8> {
    let mut encoder = Encoder::with_capacity(FRAME_HEADER_SIZE + payload.len() + 1);
    encoder.write_u8(frame_type);
    encoder.write_u16(channel);
    encoder.write_u32(payload.len() as u32);
    encoder.write_bytes(payload);
    encoder.write_u8(FRAME_END);
    encoder.into_bytes().to_vec()
}

pub fn decode_frame_header(data: &[u8]) -> Result<(FrameType, u16, u32)> {
    if data.len() < FRAME_HEADER_SIZE {
        return Err(AmqpError::BufferTooShort {
            needed: FRAME_HEADER_SIZE,
            available: data.len(),
        });
    }

    let mut decoder = Decoder::new(data);
    let type_id = decoder.read_u8()?;
    let channel = decoder.read_u16()?;
    let size = decoder.read_u32()?;

    let frame_type = FrameType::from_u8(type_id).ok_or(AmqpError::InvalidFrameType(type_id))?;

    Ok((frame_type, channel, size))
}

fn rb_frame_encode(
    ruby: &Ruby,
    frame_type: Value,
    payload: RString,
    channel: i64,
) -> std::result::Result<RString, Error> {
    if channel < 0 || channel > MAX_CHANNEL as i64 {
        return Err(Error::new(
            magnus::exception::runtime_error(),
            format!(
                "Channel has to be 0 or an integer in range 1..65535 but was {}",
                channel
            ),
        ));
    }

    let type_id: u8 = if frame_type.is_kind_of(ruby.class_symbol()) {
        let ft = FrameType::from_symbol(frame_type).ok_or_else(|| {
            Error::new(magnus::exception::arg_error(), "Invalid frame type symbol")
        })?;
        ft as u8
    } else {
        let id: i64 = TryConvert::try_convert(frame_type).map_err(|_| {
            Error::new(
                magnus::exception::type_error(),
                "Expected symbol or integer for frame type",
            )
        })?;
        id as u8
    };

    let payload_bytes = unsafe { payload.as_slice() };
    let encoded = encode_frame(type_id, channel as u16, payload_bytes);

    Ok(RString::from_slice(&encoded))
}

fn rb_frame_encode_to_array(
    ruby: &Ruby,
    frame_type: Value,
    payload: RString,
    channel: i64,
) -> std::result::Result<RArray, Error> {
    if channel < 0 || channel > MAX_CHANNEL as i64 {
        return Err(Error::new(
            magnus::exception::runtime_error(),
            format!(
                "Channel has to be 0 or an integer in range 1..65535 but was {}",
                channel
            ),
        ));
    }

    let type_id: u8 = if frame_type.is_kind_of(ruby.class_symbol()) {
        let ft = FrameType::from_symbol(frame_type).ok_or_else(|| {
            Error::new(magnus::exception::arg_error(), "Invalid frame type symbol")
        })?;
        ft as u8
    } else {
        let id: i64 = TryConvert::try_convert(frame_type).map_err(|_| {
            Error::new(
                magnus::exception::type_error(),
                "Expected symbol or integer for frame type",
            )
        })?;
        id as u8
    };

    let payload_bytes = unsafe { payload.as_slice() };
    let mut header = Encoder::with_capacity(FRAME_HEADER_SIZE);
    header.write_u8(type_id);
    header.write_u16(channel as u16);
    header.write_u32(payload_bytes.len() as u32);

    let array = ruby.ary_new();
    array.push(RString::from_slice(header.as_slice()))?;
    array.push(payload)?;
    array.push(RString::from_slice(&[FRAME_END]))?;

    Ok(array)
}

fn rb_frame_decode_header(ruby: &Ruby, header: RString) -> std::result::Result<RArray, Error> {
    let header_bytes = unsafe { header.as_slice() };

    if header_bytes.is_empty() {
        return Err(Error::new(
            magnus::exception::runtime_error(),
            "Empty response",
        ));
    }

    let (frame_type, channel, size) = decode_frame_header(header_bytes).map_err(Error::from)?;

    let array = ruby.ary_new();
    array.push(ruby.sym_new(frame_type.symbol_name()))?;
    array.push(ruby.integer_from_i64(channel as i64))?;
    array.push(ruby.integer_from_i64(size as i64))?;

    Ok(array)
}

pub fn init(ruby: &Ruby, protocol: &impl Module) -> std::result::Result<(), Error> {
    let frame_class = protocol.define_class("Frame", ruby.class_object())?;

    let types_hash = ruby.hash_new();
    types_hash.aset(ruby.sym_new("method"), 1)?;
    types_hash.aset(ruby.sym_new("headers"), 2)?;
    types_hash.aset(ruby.sym_new("body"), 3)?;
    types_hash.aset(ruby.sym_new("heartbeat"), 8)?;
    frame_class.const_set("TYPES", types_hash)?;

    let types_reverse = ruby.hash_new();
    types_reverse.aset(1, ruby.sym_new("method"))?;
    types_reverse.aset(2, ruby.sym_new("headers"))?;
    types_reverse.aset(3, ruby.sym_new("body"))?;
    types_reverse.aset(8, ruby.sym_new("heartbeat"))?;
    frame_class.const_set("TYPES_REVERSE", types_reverse)?;

    let types_options = ruby.ary_new();
    types_options.push(ruby.sym_new("method"))?;
    types_options.push(ruby.sym_new("headers"))?;
    types_options.push(ruby.sym_new("body"))?;
    types_options.push(ruby.sym_new("heartbeat"))?;
    frame_class.const_set("TYPES_OPTIONS", types_options)?;

    let final_octet = RString::from_slice(&[0xCE_u8]);
    frame_class.const_set("FINAL_OCTET", final_octet)?;

    frame_class.define_singleton_method("encode", function!(rb_frame_encode, 3))?;
    frame_class
        .define_singleton_method("encode_to_array", function!(rb_frame_encode_to_array, 3))?;
    frame_class.define_singleton_method("decode_header", function!(rb_frame_decode_header, 1))?;

    protocol.const_set("PACK_CHAR", "C")?;
    protocol.const_set("PACK_UINT16", "n")?;
    protocol.const_set("PACK_UINT16_X2", "n2")?;
    protocol.const_set("PACK_UINT32", "N")?;
    protocol.const_set("PACK_UINT32_X2", "N2")?;
    protocol.const_set("PACK_UINT64_BE", "Q>")?;
    protocol.const_set("PACK_INT64_BE", "q>")?;
    protocol.const_set("PACK_INT8", "c")?;
    protocol.const_set("PACK_INT64", "q")?;
    protocol.const_set("PACK_UCHAR_UINT32", "CN")?;
    protocol.const_set("PACK_CHAR_UINT16_UINT32", "cnN")?;
    protocol.const_set("PACK_32BIT_FLOAT", "f")?;
    protocol.const_set("PACK_64BIT_FLOAT", "G")?;
    protocol.const_set("EMPTY_STRING", "")?;

    Ok(())
}
