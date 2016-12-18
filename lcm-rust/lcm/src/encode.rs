use byteorder::{BigEndian, WriteBytesExt};
use std::io::Write;

pub fn encode_bool(buffer: &mut Write, value: bool) {
    let value = match value {
        true => 1,
        false => 0
    };
    encode_i8(buffer, value);
}

pub fn encode_u8(buffer: &mut Write, value: u8) {
    buffer.write_u8(value);
}

pub fn encode_i8(buffer: &mut Write, value: i8) {
    buffer.write_i8(value);
}

pub fn encode_i16(buffer: &mut Write, value: i16) {
    buffer.write_i16::<BigEndian>(value);
}

pub fn encode_i32(buffer: &mut Write, value: i32) {
    buffer.write_i32::<BigEndian>(value);
}

pub fn encode_i64(buffer: &mut Write, value: i64) {
    buffer.write_i64::<BigEndian>(value);
}

pub fn encode_f32(buffer: &mut Write, value: f32) {
    buffer.write_f32::<BigEndian>(value);
}

pub fn encode_f64(buffer: &mut Write, value: f64) {
    buffer.write_f64::<BigEndian>(value);
}

pub fn encode_str(buffer: &mut Write, value: &str) {
    encode_i32(buffer, value.len() as i32 + 1);
    for &b in value.as_bytes() {
        buffer.write_u8(b);
    }
    buffer.write_u8(0);
}
