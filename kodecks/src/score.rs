pub trait Score {
    type Output;
    fn score(&self) -> Self::Output;
}
