pub trait Message {
    fn hash(&self) -> i64;
}
