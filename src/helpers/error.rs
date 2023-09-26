use std::fmt::Display;
use std::io::{Error, ErrorKind};

pub fn to_io_error<T: Display>(err: T) -> Error {
    Error::new(ErrorKind::InvalidData, err.to_string())
}
