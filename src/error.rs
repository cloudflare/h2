use crate::codec::{SendError, UserError};
use crate::frame::StreamId;
use crate::proto::{self, Initiator};

use bytes::Bytes;
use std::sync::Arc;
use std::{error, fmt, io};

pub use crate::frame::Reason;

/// Represents HTTP/2.0 operation errors.
///
/// `Error` covers error cases raised by protocol errors caused by the
/// peer, I/O (transport) errors, and errors caused by the user of the library.
///
/// If the error was caused by the remote peer, then it will contain a
/// [`Reason`] which can be obtained with the [`reason`] function.
///
/// [`Reason`]: struct.Reason.html
/// [`reason`]: #method.reason
#[derive(Debug)]
pub struct Error {
    kind: Kind,
}

#[derive(Debug)]
enum Kind {
    /// A RST_STREAM frame was received or sent.
    Reset(StreamId, Reason, Initiator),

    /// A GO_AWAY frame was received or sent.
    GoAway(Bytes, Reason, Initiator),

    /// The user created an error from a bare Reason.
    Reason(Reason),

    /// An error resulting from an invalid action taken by the user of this
    /// library.
    User(UserError),

    /// An `io::Error` occurred while trying to read or write.
    Io(Arc<io::Error>),
}

// ===== impl Error =====

impl Error {
    /// If the error was caused by the remote peer, the error reason.
    ///
    /// This is either an error received by the peer or caused by an invalid
    /// action taken by the peer (i.e. a protocol error).
    pub fn reason(&self) -> Option<Reason> {
        match self.kind {
            Kind::Reset(_, reason, _) | Kind::GoAway(_, reason, _) => Some(reason),
            _ => None,
        }
    }

    /// Returns the true if the error is an io::Error
    pub fn is_io(&self) -> bool {
        match self.kind {
            Kind::Io(_) => true,
            _ => false,
        }
    }

    /// Returns the error if the error is an io::Error
    pub fn get_io(&self) -> Option<&io::Error> {
        match self.kind {
            Kind::Io(ref e) => Some(e),
            _ => None,
        }
    }

    /// Returns the error if the error is an io::Error
    pub fn into_io(self) -> Option<io::Error> {
        match self.kind {
            Kind::Io(e) => Some(io::Error::new(e.kind(), e.to_string())),
            _ => None,
        }
    }

    pub(crate) fn from_io(err: io::Error) -> Self {
        Error {
            kind: Kind::Io(Arc::new(err)),
        }
    }
}

impl From<proto::Error> for Error {
    fn from(src: proto::Error) -> Error {
        use crate::proto::Error::*;

        Error {
            kind: match src {
                Reset(stream_id, reason, initiator) => Kind::Reset(stream_id, reason, initiator),
                GoAway(debug_data, reason, initiator) => {
                    Kind::GoAway(debug_data, reason, initiator)
                }
                Io(e) => Kind::Io(e),
            },
        }
    }
}

impl From<Reason> for Error {
    fn from(src: Reason) -> Error {
        Error {
            kind: Kind::Reason(src),
        }
    }
}

impl From<SendError> for Error {
    fn from(src: SendError) -> Error {
        match src {
            SendError::User(e) => e.into(),
            SendError::Connection(e) => e.into(),
        }
    }
}

impl From<UserError> for Error {
    fn from(src: UserError) -> Error {
        Error {
            kind: Kind::User(src),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            Kind::Reset(_, reason, initiator) => {
                write!(fmt, "stream reset by {}: {}", initiator, reason)
            }
            Kind::GoAway(ref debug_data, reason, initiator) => {
                write!(fmt, "go away from {}: {}", initiator, reason)?;
                if !debug_data.is_empty() {
                    write!(fmt, " ({:?})", debug_data)?;
                }
                Ok(())
            }
            Kind::Reason(reason) => write!(fmt, "protocol error: {}", reason),
            Kind::User(ref e) => write!(fmt, "user error: {}", e),
            Kind::Io(ref e) => e.fmt(fmt),
        }
    }
}

impl error::Error for Error {}
