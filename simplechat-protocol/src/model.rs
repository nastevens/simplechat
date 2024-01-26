/// Model definition for types sent/received by simple chat
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

/// Message as sent by client
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct SentMessage {
    pub author: String,
    pub text: String,
}

impl SentMessage {
    pub fn new(author: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            author: author.into(),
            text: text.into(),
        }
    }
}

impl From<(String, String)> for SentMessage {
    fn from(value: (String, String)) -> Self {
        let (author, text) = value;
        SentMessage::new(author, text)
    }
}

/// Message as relayed from server to other clients (includes timestamp)
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct ReceivedMessage {
    pub author: String,
    pub text: String,
    pub ts: String,
}

impl ReceivedMessage {
    pub fn new(author: impl Into<String>, text: impl Into<String>, ts: impl Into<String>) -> Self {
        Self {
            author: author.into(),
            text: text.into(),
            ts: ts.into(),
        }
    }
}

impl From<SentMessage> for ReceivedMessage {
    fn from(value: SentMessage) -> Self {
        let SentMessage { author, text } = value;
        let ts = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or(String::new());
        Self { author, text, ts }
    }
}
