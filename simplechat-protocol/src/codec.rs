/// Codecs for simple chat protocol
use crate::{
    model::{ReceivedMessage, SentMessage},
    util::ResultExt,
    Error,
};
use base64::{
    engine::general_purpose::STANDARD as B64_STANDARD, read::DecoderReader, write::EncoderWriter,
};
use std::io::{Cursor, Write};
use tokio_util::{
    bytes::{BufMut, BytesMut},
    codec::{Decoder, Encoder, LinesCodec},
};

// 640k ought to be enough for anyone
const MAX_LENGTH: usize = 1024 * 640;

/// Messages sent from client to server
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum ClientFrame {
    Send(SentMessage),
    Leave,
}

impl ClientFrame {
    pub fn send(msg: impl Into<SentMessage>) -> Self {
        Self::Send(msg.into())
    }

    pub fn leave() -> Self {
        Self::Leave
    }
}

/// Codec for client frames
#[derive(Debug)]
pub struct ClientFrameCodec {
    inner: LinesCodec,
}

impl Default for ClientFrameCodec {
    fn default() -> Self {
        Self {
            inner: LinesCodec::new_with_max_length(MAX_LENGTH),
        }
    }
}

impl Decoder for ClientFrameCodec {
    type Item = ClientFrame;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some((verb, args)) = decode_frame(src, &mut self.inner)? {
            match verb.as_str() {
                "send" => {
                    let [author, text] = destructure_args(args)?;
                    Ok(Some(ClientFrame::Send(SentMessage { author, text })))
                }
                "leave" => Ok(Some(ClientFrame::Leave)),
                _ => Err(Error::InvalidFrame),
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder<ClientFrame> for ClientFrameCodec {
    type Error = Error;

    fn encode(&mut self, frame: ClientFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        use ClientFrame::*;
        match frame {
            Send(msg) => encode_frame(b"send", [&msg.author, &msg.text], dst),
            Leave => encode_frame(b"leave", [], dst),
        }
    }
}

/// Messages sent from server to client
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum ServerFrame {
    Receive(ReceivedMessage),
}

impl ServerFrame {
    pub fn receive(msg: impl Into<ReceivedMessage>) -> Self {
        Self::Receive(msg.into())
    }
}

/// Codec for server frames
#[derive(Debug)]
pub struct ServerFrameCodec {
    inner: LinesCodec,
}

impl Default for ServerFrameCodec {
    fn default() -> Self {
        Self {
            inner: LinesCodec::new_with_max_length(MAX_LENGTH),
        }
    }
}

impl Decoder for ServerFrameCodec {
    type Item = ServerFrame;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some((verb, args)) = decode_frame(src, &mut self.inner)? {
            match verb.as_str() {
                "receive" => {
                    let [author, text, ts] = destructure_args(args)?;
                    Ok(Some(ServerFrame::Receive(ReceivedMessage {
                        author,
                        text,
                        ts,
                    })))
                }
                _ => Err(Error::InvalidFrame),
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder<ServerFrame> for ServerFrameCodec {
    type Error = Error;

    fn encode(&mut self, frame: ServerFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        use ServerFrame::*;
        match frame {
            Receive(msg) => encode_frame(b"receive", [&msg.author, &msg.text, &msg.ts], dst),
        }
    }
}

// Common logic for decoding frames
fn decode_frame(
    src: &mut BytesMut,
    frame_decoder: &mut LinesCodec,
) -> Result<Option<(String, Vec<String>)>, Error> {
    if let Some(frame) = frame_decoder.decode(src)? {
        let mut split = frame.trim().split(' ');
        let verb = split.next().map(String::from).ok_or(Error::InvalidFrame)?;
        let args = split
            .map(|value| {
                let reader = DecoderReader::new(Cursor::new(value), &B64_STANDARD);
                std::io::read_to_string(reader).or_invalid_frame()
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Some((verb, args)))
    } else {
        Ok(None)
    }
}

// Common logic for encoding frames
fn encode_frame<const N: usize>(
    verb: &[u8],
    args: [&str; N],
    dst: &mut BytesMut,
) -> Result<(), Error> {
    // Reserve enough space for full encoding to avoid reallocating
    dst.reserve(
        args.iter()
            .map(|s| Ok(base64::encoded_len(s.len(), true).or_invalid_frame()? + 1))
            .sum::<Result<usize, Error>>()?
            + verb.len(),
    );

    // Write out verb and arguments
    dst.put_slice(verb);
    for s in args {
        dst.put_u8(b' ');
        EncoderWriter::new(dst.writer(), &B64_STANDARD)
            .write_all(s.as_bytes())
            .expect("infallible");
    }
    dst.put_u8(b'\n');
    Ok(())
}

// Rust can destructure into an array, and a Vec can be turned into an array
// with `try_into`. This lets us write ergonomic code like
// `let [a, b] = destructure_args(some_vec)?` that will return an error if there
// aren't the right number of arguments.
fn destructure_args<const N: usize>(args: Vec<String>) -> Result<[String; N], Error> {
    args.try_into().or_invalid_frame()
}

#[cfg(test)]
mod test {
    use super::{ClientFrame, ClientFrameCodec, ServerFrame, ServerFrameCodec};
    use crate::{Error, ReceivedMessage, SentMessage};
    use tokio_util::{
        bytes::BytesMut,
        codec::{Decoder, Encoder},
    };

    fn do_encode<T, E>(item: T, mut encoder: E) -> String
    where
        E: Encoder<T>,
        <E as Encoder<T>>::Error: std::fmt::Debug,
    {
        let mut output = BytesMut::new();
        encoder.encode(item, &mut output).unwrap();
        String::from_utf8(output.to_vec()).unwrap()
    }

    fn do_decode<T, D>(bytes: &str, mut decoder: D) -> T
    where
        D: Decoder<Item = T, Error = Error>,
        <D as Decoder>::Error: std::fmt::Debug,
    {
        let mut buffer = BytesMut::from(bytes);
        decoder.decode(&mut buffer).unwrap().unwrap()
    }

    #[test]
    fn test_client_codec() {
        #[rustfmt::skip]
        let tests = vec![
            (
                ClientFrame::send(SentMessage::new("The Thing", "It's Clobbering Time")),
                "send VGhlIFRoaW5n SXQncyBDbG9iYmVyaW5nIFRpbWU=\n"
            ),
            (
                ClientFrame::leave(),
                "leave\n"
            ),
        ];
        for test in tests {
            let (item, bytes) = test;
            let encoded = do_encode(item.clone(), ClientFrameCodec::default());
            assert_eq!(encoded, bytes);
            let decoded = do_decode(bytes, ClientFrameCodec::default());
            assert_eq!(decoded, item);
        }
    }

    #[test]
    fn test_server_codec() {
        const TS: &str = "2000-01-01T00:00:00Z";
        #[rustfmt::skip]
        let tests = vec![
            (
                ServerFrame::receive(ReceivedMessage::new("Reed Richards", "I'm really smart", TS)),
                "receive UmVlZCBSaWNoYXJkcw== SSdtIHJlYWxseSBzbWFydA== MjAwMC0wMS0wMVQwMDowMDowMFo=\n"
            ),
        ];
        for test in tests {
            let (item, bytes) = test;
            let encoded = do_encode(item.clone(), ServerFrameCodec::default());
            assert_eq!(encoded, bytes);
            let decoded = do_decode(bytes, ServerFrameCodec::default());
            assert_eq!(decoded, item);
        }
    }
}
