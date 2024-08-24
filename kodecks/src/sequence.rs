use crate::zone::CardZone;

pub trait CardSequence: CardZone {
    fn top(&self) -> Option<&<Self as CardZone>::Item> {
        self.iter().last()
    }

    fn bottom(&self) -> Option<&<Self as CardZone>::Item> {
        self.iter().next()
    }

    fn remove_top(&mut self) -> Option<<Self as CardZone>::Item>;
    fn add_top(&mut self, card: <Self as CardZone>::Item);
}
