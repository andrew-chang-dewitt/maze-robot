use super::List;

#[derive(Debug)]
pub struct Hashmap<K: Hash, V, const Size: usize> {
    buckets: [Option<List<K>>; Size],
    // values:
}
