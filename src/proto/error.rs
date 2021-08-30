use crate::codec::{RecvError, SendError};
use crate::frame::Reason;

use std::fmt;
use std::io;

/// Either an H2 reason  or an I/O error
#[derive(Debug)]
pub enum Error {
    Ours(Reason),
    Theirs(Reason),
    Io(io::Error),
}

impl Error {
    /// Clone the error for internal purposes.
    ///
    /// `io::Error` is not `Clone`, so we only copy the `ErrorKind`.
    pub(super) fn shallow_clone(&self) -> Error {
        match *self {
            Self::Ours(reason) => Self::Ours(reason),
            Self::Theirs(reason) => Self::Theirs(reason),
            Self::Io(ref io) => Self::Io(io::Error::from(io.kind())),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Ours(reason) => reason.fmt(fmt),
            Self::Theirs(reason) => reason.fmt(fmt),
            Self::Io(ref error) => error.fmt(fmt),
        }
    }
}

impl From<io::Error> for Error {
    fn from(src: io::Error) -> Self {
        Error::Io(src)
    }
}

impl From<Error> for RecvError {
    fn from(src: Error) -> Self {
        Self::Connection(src)
    }
}

impl From<Error> for SendError {
    fn from(src: Error) -> Self {
        Self::Connection(src)
    }
}
