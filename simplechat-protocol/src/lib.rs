/// Protocol definitions for a simple chat server
///
/// Defines a simple protocol for sending and receiving chat messages. The
/// format of the messages is:
///
/// ```ignore
/// <verb> [<b64 encoded argument>...]
/// ```
///
/// Where `verb` is a simple ASCII string such as `send` or `receive`.
use thiserror::Error;

mod codec;
mod model;
mod util;

pub use codec::{ClientFrame, ClientFrameCodec, ServerFrame, ServerFrameCodec};
pub use model::{ReceivedMessage, SentMessage};

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("lines parse error: {0}")]
    LinesParseError(#[from] tokio_util::codec::LinesCodecError),

    #[error("invalid frame")]
    InvalidFrame,
}
