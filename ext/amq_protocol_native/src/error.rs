//! Error types for AMQP protocol handling

use magnus::{exception, Error};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AmqpError {
    #[error("Invalid frame type: {0}")]
    InvalidFrameType(u8),

    #[error("Frame type error: expected one of {0:?}")]
    FrameTypeError(Vec<&'static str>),

    #[error("Empty response")]
    EmptyResponse,

    #[error("Channel out of range: {0} (must be 0-65535)")]
    ChannelOutOfRange(i64),

    #[error("Payload cannot be nil")]
    NilPayload,

    #[error("Invalid table value for key '{0}': {1}")]
    InvalidTableValue(String, String),

    #[error("Invalid table type: {0}")]
    InvalidTableType(char),

    #[error("Buffer too short: need {needed} bytes, have {available}")]
    BufferTooShort { needed: usize, available: usize },

    #[error("Invalid short string length: {0} (max 255)")]
    ShortStringTooLong(usize),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Decoding error: {0}")]
    DecodingError(String),
}

impl From<AmqpError> for Error {
    fn from(err: AmqpError) -> Self {
        match err {
            AmqpError::InvalidFrameType(_)
            | AmqpError::FrameTypeError(_)
            | AmqpError::InvalidTableType(_)
            | AmqpError::InvalidTableValue(_, _) => {
                Error::new(exception::arg_error(), err.to_string())
            }
            AmqpError::EmptyResponse
            | AmqpError::BufferTooShort { .. }
            | AmqpError::DecodingError(_) => {
                Error::new(exception::runtime_error(), err.to_string())
            }
            AmqpError::ChannelOutOfRange(_)
            | AmqpError::NilPayload
            | AmqpError::ShortStringTooLong(_)
            | AmqpError::EncodingError(_) => Error::new(exception::arg_error(), err.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, AmqpError>;
