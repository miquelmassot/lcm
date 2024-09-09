use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Result, Error, ErrorKind, Read, Write};
use std::mem::size_of;

/// A message that can be encoded and decoded according to the LCM protocol.
pub trait Message {
    /// Encodes a message into a buffer, with the message hash at the beginning.
    fn encode_with_hash(&self) -> Result<Vec<u8>> where Self: Sized {
        let hash = Self::hash();
        let size = hash.size() + self.size();
        let mut buffer = Vec::with_capacity(size);
        hash.encode(&mut buffer)?;
        self.encode(&mut buffer)?;
        Ok(buffer)
    }

    /// Decodes a message from a buffer,
    /// and also checks that the hash at the beginning is correct.
    fn decode_with_hash(mut buffer: &mut Read) -> Result<Self>
        where Self: Sized
    {
        let hash: u64 = Message::decode(&mut buffer)?;
        if hash != Self::hash() {
            return Err(Error::new(ErrorKind::Other, "Invalid hash"));
        }
        Message::decode(buffer)
    }

    /// Returns the message hash for this type.
    /// Hash values are ignored for the primitive types.
    /// Generated `Lcm` types should implement this function.
    fn hash() -> u64 where Self: Sized;

    /// Encodes a message into a buffer.
    /// `Lcm` uses a `Vec<u8>` with its capacity set to the value returned by [`size()`].
    fn encode(&self, buffer: &mut Write) -> Result<()>;

    /// Decodes a message from a buffer.
    fn decode(buffer: &mut Read) -> Result<Self> where Self: Sized;

    /// Returns the number of bytes this message is expected to take when encoded.
    fn size(&self) -> usize;
}

impl Message for bool {
    fn hash() -> u64 {
        0
    }

    fn encode(&self, buffer: &mut Write) -> Result<()> {
        let value: i8 = match *self {
            true => 1,
            false => 0,
        };
        value.encode(buffer)
    }

    fn decode(buffer: &mut Read) -> Result<Self> {
        let value = buffer.read_i8()?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::new(ErrorKind::InvalidData,
                                "Booleans should be encoded as 0 or 1"))
        }
    }

    fn size(&self) -> usize {
        size_of::<i8>()
    }
}

impl Message for u8 {
    fn hash() -> u64 {
        1 << 0
    }

    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_u8(*self)
    }

    fn decode(buffer: &mut Read) -> Result<Self> {
        buffer.read_u8()
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for i8 {
    fn hash() -> u64 {
        1 << 1
    }

    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_i8(*self)
    }

    fn decode(buffer: &mut Read) -> Result<Self> {
        buffer.read_i8()
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for i16 {
    fn hash() -> u64 {
        1 << 2
    }

    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_i16::<BigEndian>(*self)
    }

    fn decode(buffer: &mut Read) -> Result<Self> {
        buffer.read_i16::<BigEndian>()
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for i32 {
    fn hash() -> u64 {
        1 << 3
    }

    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_i32::<BigEndian>(*self)
    }

    fn decode(buffer: &mut Read) -> Result<Self> {
        buffer.read_i32::<BigEndian>()
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for u64 {
    fn hash() -> u64 {
        1 << 4
    }

    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_u64::<BigEndian>(*self)
    }

    fn decode(buffer: &mut Read) -> Result<Self> {
        buffer.read_u64::<BigEndian>()
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for i64 {
    fn hash() -> u64 {
        1 << 5
    }

    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_i64::<BigEndian>(*self)
    }

    fn decode(buffer: &mut Read) -> Result<Self> {
        buffer.read_i64::<BigEndian>()
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for f32 {
    fn hash() -> u64 {
        1 << 6
    }

    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_f32::<BigEndian>(*self)
    }

    fn decode(buffer: &mut Read) -> Result<Self> {
        buffer.read_f32::<BigEndian>()
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for f64 {
    fn hash() -> u64 {
        1 << 7
    }

    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_f64::<BigEndian>(*self)
    }

    fn decode(buffer: &mut Read) -> Result<Self> {
        buffer.read_f64::<BigEndian>()
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for String {
    fn hash() -> u64 {
        1 << 8
    }

    fn encode(&self, buffer: &mut Write) -> Result<()> {
        let len: i32 = self.len() as i32 + 1;
        len.encode(buffer)?;
        for &b in self.as_bytes() {
            b.encode(buffer)?;
        }
        (0 as u8).encode(buffer)?;
        Ok(())
    }

    fn decode(buffer: &mut Read) -> Result<Self> {
        let len = buffer.read_i32::<BigEndian>()? - 1;
        let mut buf = Vec::with_capacity(len as usize);
        for _ in 0..len {
            buf.push(u8::decode(buffer)?)
        }
        let result = String::from_utf8(buf).map_err(|e| Error::new(ErrorKind::Other, e))?;
        match buffer.read_u8() {
            Ok(0) => Ok(result),
            Ok(_) => Err(Error::new(ErrorKind::InvalidData, "Expected null terminator")),
            Err(e) => Err(e),
        }
    }

    fn size(&self) -> usize {
        size_of::<i32>() + self.len() + 1
    }
}
