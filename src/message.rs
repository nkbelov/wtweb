use tokio_util::codec::{Encoder, Decoder};
use bytes::{BytesMut, Buf};
use serde::{Serialize, Deserialize};

pub const PORT: u16 = 6379;

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Text(String)
}

pub struct JsonCodec;

impl JsonCodec {
    pub fn new() -> Self {
        Self { }
    }
}

const MAX: usize = 8 * 1024 * 1024;

impl Encoder<Message> for JsonCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {

        let ser = serde_json::to_vec(&item)?;
        
        if ser.len() > MAX {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", ser.len())
            ));
        }

        // Convert the length into a byte array.
        // The cast to u32 cannot overflow due to the length check above.
        let len_slice = u32::to_le_bytes(ser.len() as u32);

        // Reserve space in the buffer.
        dst.reserve(4 + ser.len());

        // Write the length and string to the buffer.
        dst.extend_from_slice(&len_slice);
        dst.extend_from_slice(&ser);
        Ok(())
    }
}

impl Decoder for JsonCodec {
    type Item = Message;
    type Error = std::io::Error;

    fn decode(
        &mut self,
        src: &mut BytesMut
    ) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            // Not enough data to read length marker.
            return Ok(None);
        }

        // Read length marker.
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_le_bytes(length_bytes) as usize;

        // Check that the length is not too large to avoid a denial of
        // service attack where the server runs out of memory.
        if length > MAX {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", length)
            ));
        }

        if src.len() < 4 + length {
            // The full byte string has not yet arrived.
            //
            // We reserve more space in the buffer. This is not strictly
            // necessary, but is a good idea performance-wise.
            src.reserve(4 + length - src.len());

            // We inform the Framed that we need more bytes to form the next
            // frame.
            return Ok(None);
        }

        // Use advance to modify src such that it no longer contains
        // this frame.
        let data = src[4..4 + length].to_vec();
        src.advance(4 + length);

        let de: Message = serde_json::from_slice(&data)?;
        Ok(Some(de))
    }
}