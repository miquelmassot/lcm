use std::io::Write;

pub trait Message {
    fn get_hash(&self) -> i64;
}
