use std::io::Write;

pub trait Message {
    fn hash(&self) -> i64;
}
