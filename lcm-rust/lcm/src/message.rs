use byteorder::{BigEndian, WriteBytesExt};
use generic_array::{ArrayLength, GenericArray};
use std::io::{Result, Error, ErrorKind, Read, Write};
use std::mem::size_of;

pub trait Message {
    fn hash(&self) -> i64 { 0 }
    fn encode(&self, buffer: &mut Write) -> Result<()>;
    fn decode(&mut self, buffer: &mut Read) -> Result<()> { Err(Error::new(ErrorKind::Other, "Unimplemented")) }
    fn size(&self) -> usize;
}

impl Message for bool {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        let value : i8 = match *self {
            true => 1,
            false => 0
        };
        value.encode(buffer)
    }

    fn size(&self) -> usize {
        size_of::<i8>()
    }
}

impl Message for u8 {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_u8(*self)
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for i8 {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_i8(*self)
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for i16 {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_i16::<BigEndian>(*self)
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for i32 {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_i32::<BigEndian>(*self)
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for i64 {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_i64::<BigEndian>(*self)
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for f32 {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_f32::<BigEndian>(*self)
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for f64 {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_f64::<BigEndian>(*self)
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Message for str {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        let len : i32 = self.len() as i32 + 1;
        len.encode(buffer)?;
        for &b in self.as_bytes() {
            b.encode(buffer)?;
        }
        (1 as u8).encode(buffer)?;
        Ok(())
    }

    fn size(&self) -> usize {
        size_of::<i32>() + self.len() + 1
    }
}

impl<T> Message for Vec<T> where
    T : Message {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        for val in self.iter() {
            val.encode(buffer)?;
        }
        Ok(())
    }

    fn size(&self) -> usize {
        self.iter().map(Message::size).sum()
    }
}

impl<T,N> Message for GenericArray<T, N> where
    T : Message, N : ArrayLength<T> {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        for val in self.iter() {
            val.encode(buffer)?;
        }
        Ok(())
    }

    fn size(&self) -> usize {
        self.iter().map(Message::size).sum()
    }
}
