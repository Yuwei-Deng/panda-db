//! Provides a type representing a gossip protocol frame as well as utilities for
//! parsing frames from a byte array.

use bytes::{Buf, Bytes};
use std::convert::TryInto;
use std::fmt;
use std::io::Cursor;
use std::num::TryFromIntError;
use std::string::FromUtf8Error;

/// A frame in the Redis protocol.
#[derive(Clone, Debug)]
pub enum Frame {
    FixedLength(Vec<u8>),
    VariantLength(Vec<u8>),
    Sample(Vec<u8>),
    Command(Vec<u8>),
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

#[derive(Debug)]
pub enum Error {
    /// Not enough data is available to parse a message
    Incomplete,

    /// Invalid message encoding
    Other(crate::Error),
}

impl Frame {
    /// Returns an empty array
    pub(crate) fn array() -> Frame {
        Frame::Array(vec![])
    }

    /// Push a "bulk" frame into the array. `self` must be an Array frame.
    ///
    /// # Panics
    ///
    /// panics if `self` is not an array
    pub(crate) fn push_bulk(&mut self, bytes: Bytes) {
        match self {
            Frame::Array(vec) => {
                vec.push(Frame::Bulk(bytes));
            }
            _ => panic!("not an array frame"),
        }
    }

    /// Push an "integer" frame into the array. `self` must be an Array frame.
    ///
    /// # Panics
    ///
    /// panics if `self` is not an array
    pub(crate) fn push_int(&mut self, value: u64) {
        match self {
            Frame::Array(vec) => {
                vec.push(Frame::Integer(value));
            }
            _ => panic!("not an array frame"),
        }
    }

    /// Checks if an entire message can be decoded from `src`
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        log::info!("src={:?}", src);
        match get_u8(src)? {
            super::VARIANT_LENGTH_MARK => {
                log::info!("variant mark");
                let r = get_length_info(src)?;
                log::info!("variant mark, len={}", r);
                src.set_position(r as u64 + 4+ 1);
                Ok(())
            }
            super::SAMPLE_MARK => {
                log::info!("sample mark");
                get_line(src)?;
                Ok(())
            }
            super::COMMAND_MARK=> {
                log::info!("command mark");
                get_line(src)?;
                Ok(())
            }
            b':' => {
                log::info!(": mark");
                let _ = get_decimal(src)?;
                Ok(())
            }
            b'$' => {
                log::info!("$ mark");
                if b'-' == peek_u8(src)? {
                    // Skip '-1\r\n'
                    skip(src, 4)
                } else {
                    // Read the bulk string
                    let len: usize = get_decimal(src)?.try_into()?;

                    // skip that number of bytes + 2 (\r\n).
                    skip(src, len + 2)
                }
            }
            b'*' => {
                log::info!("* mark");
                let len = get_decimal(src)?;

                for _ in 0..len {
                    Frame::check(src)?;
                }
                Ok(())
            }
            actual => {
                log::info!("actual={}", actual);
                Err(format!("protocol error; invalid frame type byte `{}`", actual).into())
            },
        }
    }

    /// The message has already been validated with `check`.
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        match get_u8(src)? {
            super::VARIANT_LENGTH_MARK => {
                let data = get_vec_of_length(src)?.to_vec();
                Ok(Frame::VariantLength(data))
            }
            b'+' => {
                // Read the line and convert it to `Vec<u8>`
                let line = get_line(src)?.to_vec();

                // Convert the line to a String
                let string = String::from_utf8(line)?;

                Ok(Frame::Simple(string))
            }
            b'-' => {
                // Read the line and convert it to `Vec<u8>`
                let line = get_line(src)?.to_vec();

                // Convert the line to a String
                let string = String::from_utf8(line)?;

                Ok(Frame::Error(string))
            }
            b':' => {
                let len = get_decimal(src)?;
                Ok(Frame::Integer(len))
            }
            b'$' => {
                if b'-' == peek_u8(src)? {
                    let line = get_line(src)?;

                    if line != b"-1" {
                        return Err("protocol error; invalid frame format".into());
                    }

                    Ok(Frame::Null)
                } else {
                    // Read the bulk string
                    let len = get_decimal(src)?.try_into()?;
                    let n = len + 2;
                    if src.remaining() < n {
                        return Err(Error::Incomplete);
                    }

                    let data = Bytes::copy_from_slice(&src.chunk()[..len]);

                    // skip that number of bytes + 2 (\r\n).
                    skip(src, n)?;

                    Ok(Frame::Bulk(data))
                }
            }
            b'*' => {
                let len = get_decimal(src)?.try_into()?;
                let mut out = Vec::with_capacity(len);

                for _ in 0..len {
                    out.push(Frame::parse(src)?);
                }

                Ok(Frame::Array(out))
            }
            _ => unimplemented!(),
        }
    }

    /// Converts the frame to an "unexpected frame" error
    pub(crate) fn to_error(&self) -> crate::Error {
        format!("unexpected frame: {}", self).into()
    }
}

impl PartialEq<&str> for Frame {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Frame::Simple(s) => s.eq(other),
            Frame::Bulk(s) => s.eq(other),
            _ => false,
        }
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use std::str;

        match self {
            Frame::Sample(val)=> {
                "sample data".fmt(fmt)
            }
            Frame::Command(command)=> {"command".fmt(fmt)}
            Frame::Simple(response) => response.fmt(fmt),
            Frame::Error(msg) => write!(fmt, "error: {}", msg),
            Frame::Integer(num) => num.fmt(fmt),
            Frame::Bulk(msg) => match str::from_utf8(msg) {
                Ok(string) => string.fmt(fmt),
                Err(_) => write!(fmt, "{:?}", msg),
            },
            Frame::Null => "(nil)".fmt(fmt),
            Frame::Array(parts) => {
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        write!(fmt, " ")?;
                        part.fmt(fmt)?;
                    }
                }
                Ok(())
            }
            Frame::FixedLength(_) => { "fixed length".fmt(fmt) }
            Frame::VariantLength(_) => {"variant length".fmt(fmt)}
        }
    }
}

fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.chunk()[0])
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        log::info!("No remaining data");
        return Err(Error::Incomplete);
    }
    Ok(src.get_u8())
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if src.remaining() < n {
        return Err(Error::Incomplete);
    }

    src.advance(n);
    Ok(())
}

/// Read a new-line terminated decimal
fn get_decimal(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    use atoi::atoi;

    let line = get_line(src)?;

    atoi::<u64>(line).ok_or_else(|| "protocol error; invalid frame format".into())
}

///
/// Get length of Frame
///
fn get_length_info(src: &mut Cursor<&[u8]>) -> Result<u32 , Error> {
    let start  = src.position() as usize;
    if src.get_ref().len() - start < 4 {
        log::error!("Error in complete Length");
        return Err(Error::Incomplete);
    }
    // Get the length info from bytes
    let bytes = &src.get_ref()[start..start+4];
    log::info!("receive len:{:?}", bytes);
    // decode the info
    let len_expect = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    log::info!("len of buf={}, expected={}", src.get_ref().len()- start -4, len_expect );
    if src.get_ref().len() - start  <  len_expect as usize + 4 {
        log::error!("length is not right={}", len_expect);
        return Err(Error::Incomplete);
    }
    Ok(len_expect)
}

fn get_vec_of_length<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start  = (src.position() + 4) as usize;
    log::info!("Get Vec of length");
    let len = get_length_info(src)?;
    log::info!("len={}", len);
    let end = len as usize + start ;
    Ok(&src.get_ref()[start..end])
}

/// Find a line
fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    // Scan the bytes directly
    let start = src.position() as usize;
    // Scan to the second to last byte
    let end = src.get_ref().len() - 1;
    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            // We found a line, update the position to be *after* the \n
            src.set_position((i + 2) as u64);

            // Return the line
            return Ok(&src.get_ref()[start..i]);
        }
    }

    Err(Error::Incomplete)
}


impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src.into())
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Error {
        src.to_string().into()
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_src: FromUtf8Error) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for Error {
    fn from(_src: TryFromIntError) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Incomplete => "stream ended early".fmt(fmt),
            Error::Other(err) => err.fmt(fmt),
        }
    }
}


#[test]
fn test_range() {
    let a = vec![1, 2,3,4, 5 ,6];
    assert_eq!(a[0..3] , vec![1,2,3]);
}