//! AMQP 0.9.1 encoding/decoding primitives

use crate::error::{AmqpError, Result};
use bytes::{BufMut, Bytes, BytesMut};

pub struct Encoder {
    buf: BytesMut,
}

impl Encoder {
    pub fn new() -> Self {
        Self {
            buf: BytesMut::with_capacity(256),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: BytesMut::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn write_u8(&mut self, v: u8) {
        self.buf.put_u8(v);
    }

    #[inline]
    pub fn write_u16(&mut self, v: u16) {
        self.buf.put_u16(v);
    }

    #[inline]
    pub fn write_u32(&mut self, v: u32) {
        self.buf.put_u32(v);
    }

    #[inline]
    pub fn write_i64(&mut self, v: i64) {
        self.buf.put_i64(v);
    }

    #[inline]
    pub fn write_f64(&mut self, v: f64) {
        self.buf.put_f64(v);
    }

    pub fn write_short_string(&mut self, s: &str) -> Result<()> {
        let len = s.len();
        if len > 255 {
            return Err(AmqpError::ShortStringTooLong(len));
        }
        self.buf.put_u8(len as u8);
        self.buf.put_slice(s.as_bytes());
        Ok(())
    }

    pub fn write_long_string(&mut self, s: &[u8]) {
        self.buf.put_u32(s.len() as u32);
        self.buf.put_slice(s);
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.buf.put_slice(data);
    }

    pub fn into_bytes(self) -> Bytes {
        self.buf.freeze()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Decoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    fn ensure(&self, n: usize) -> Result<()> {
        if self.remaining() < n {
            Err(AmqpError::BufferTooShort {
                needed: n,
                available: self.remaining(),
            })
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn read_u8(&mut self) -> Result<u8> {
        self.ensure(1)?;
        let v = self.data[self.pos];
        self.pos += 1;
        Ok(v)
    }

    #[inline]
    pub fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    #[inline]
    pub fn read_u16(&mut self) -> Result<u16> {
        self.ensure(2)?;
        let v = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }

    #[inline]
    pub fn read_i16(&mut self) -> Result<i16> {
        Ok(self.read_u16()? as i16)
    }

    #[inline]
    pub fn read_u32(&mut self) -> Result<u32> {
        self.ensure(4)?;
        let v = u32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    #[inline]
    pub fn read_i32(&mut self) -> Result<i32> {
        Ok(self.read_u32()? as i32)
    }

    #[inline]
    pub fn read_u64(&mut self) -> Result<u64> {
        self.ensure(8)?;
        let v = u64::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
            self.data[self.pos + 4],
            self.data[self.pos + 5],
            self.data[self.pos + 6],
            self.data[self.pos + 7],
        ]);
        self.pos += 8;
        Ok(v)
    }

    #[inline]
    pub fn read_i64(&mut self) -> Result<i64> {
        Ok(self.read_u64()? as i64)
    }

    #[inline]
    pub fn read_f32(&mut self) -> Result<f32> {
        self.ensure(4)?;
        let v = f32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    #[inline]
    pub fn read_f64(&mut self) -> Result<f64> {
        self.ensure(8)?;
        let v = f64::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
            self.data[self.pos + 4],
            self.data[self.pos + 5],
            self.data[self.pos + 6],
            self.data[self.pos + 7],
        ]);
        self.pos += 8;
        Ok(v)
    }

    #[allow(dead_code)]
    pub fn read_short_string(&mut self) -> Result<&'a str> {
        let len = self.read_u8()? as usize;
        self.ensure(len)?;
        let s = std::str::from_utf8(&self.data[self.pos..self.pos + len])
            .map_err(|e| AmqpError::DecodingError(format!("Invalid UTF-8: {}", e)))?;
        self.pos += len;
        Ok(s)
    }

    pub fn read_short_string_bytes(&mut self) -> Result<&'a [u8]> {
        let len = self.read_u8()? as usize;
        self.ensure(len)?;
        let bytes = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(bytes)
    }

    pub fn read_long_string(&mut self) -> Result<&'a [u8]> {
        let len = self.read_u32()? as usize;
        self.ensure(len)?;
        let bytes = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(bytes)
    }
}
