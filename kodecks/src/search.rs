pub trait Searchable {
    fn matches_text(&self, name: &str) -> Option<u32>;
    fn matches_tag(&self, key: &str, value: &str) -> Option<u32>;
    fn matches_cmp(&self, lhs: &str, op: &str, rhs: &str) -> Option<u32>;
}
