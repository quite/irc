//! Implementation of line-based codec for Tokio.

use std::io;
use std::io::prelude::*;
use encoding::{DecoderTrap, EncoderTrap, Encoding};
use tokio_core::io::{Codec, EasyBuf};

/// A line-based codec parameterized by an encoding.
pub struct LineCodec<E: Encoding> {
    encoding: E,
}

impl<E> Codec for LineCodec<E> where E: Encoding {
    type In = String;
    type Out = String;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<String>> {
        if let Some(n) = buf.as_ref().iter().position(|b| *b == b'\n') {
            // Remove the next frame from the buffer.
            let line = buf.drain_to(n);

            // Remove the new-line from the buffer.
            buf.drain_to(1);

            // Decode the line using the codec's encoding.
            match self.encoding.decode(line.as_ref(), DecoderTrap::Replace) {
                Ok(data) => Ok(Some(data)),
                Err(data) => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    &format!("Failed to decode {} as {}.", data, self.encoding.name())[..]
                ))
            }
        } else {
            Ok(None)
        }
    }

    fn encode(&mut self, msg: String, buf: &mut Vec<u8>) -> io::Result<()> {
        // Encode the message using the codec's encoding.
        let data = try!(self.encoding.encode(&msg, EncoderTrap::Replace).map_err(|data| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                &format!("Failed to encode {} as {}.", data, self.encoding.name())[..]
            )
        }));

        // Write the encoded message to the output buffer.
        try!(buf.write_all(&data));

        // Flush the output buffer.
        buf.flush()
    }
}