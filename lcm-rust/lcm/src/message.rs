use std::io::Write;

pub trait Message {
    fn get_size(&self) -> usize;
    fn get_hash(&self) -> i64;
    fn encode(&self, buffer: &mut Write);
}
