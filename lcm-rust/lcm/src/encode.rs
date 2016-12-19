use byteorder::{BigEndian, WriteBytesExt};
use generic_array::{ArrayLength, GenericArray};
use std::io::{Result, Write};
use std::mem::size_of;

pub trait Encode {
    fn encode(&self, buffer: &mut Write) -> Result<()>;
    fn size(&self) -> usize;
}

impl Encode for bool {
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

impl Encode for u8 {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_u8(*self)
    }

    fn size(&self) -> usize {
        size_of::<u8>()
    }
}

impl Encode for i8 {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_i8(*self)
    }

    fn size(&self) -> usize {
        size_of::<i8>()
    }
}

impl Encode for i16 {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_i16::<BigEndian>(*self)
    }

    fn size(&self) -> usize {
        size_of::<i16>()
    }
}

impl Encode for i32 {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_i32::<BigEndian>(*self)
    }

    fn size(&self) -> usize {
        size_of::<i32>()
    }
}

impl Encode for i64 {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_i64::<BigEndian>(*self)
    }

    fn size(&self) -> usize {
        size_of::<i64>()
    }
}

impl Encode for f32 {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_f32::<BigEndian>(*self)
    }

    fn size(&self) -> usize {
        size_of::<f32>()
    }
}

impl Encode for f64 {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        buffer.write_f64::<BigEndian>(*self)
    }

    fn size(&self) -> usize {
        size_of::<Self>()
    }
}

impl Encode for str {
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

impl<T> Encode for Vec<T> where
    T : Encode {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        for val in self.iter() {
            val.encode(buffer)?;
        }
        Ok(())
    }

    fn size(&self) -> usize {
        size_of::<T>() * self.len()
    }
}

impl<T,N> Encode for GenericArray<T, N> where
    T : Encode, N : ArrayLength<T> {
    fn encode(&self, buffer: &mut Write) -> Result<()> {
        for val in self.iter() {
            val.encode(buffer)?;
        }
        Ok(())
    }

    fn size(&self) -> usize {
        size_of::<T>() * self.len()
    }
}
