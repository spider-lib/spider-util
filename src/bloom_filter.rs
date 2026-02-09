use seahash::hash;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A proper Bloom Filter implementation using a bit vector for memory efficiency.
/// This is used for efficiently checking if a URL has potentially been visited before,
/// reducing the need for expensive lookups in the main visited URLs cache.
pub struct BloomFilter {
    bit_set: Vec<u64>,
    num_bits: u64,
    hash_functions: usize,
}

impl BloomFilter {
    /// Creates a new BloomFilter with the specified capacity and number of hash functions.
    pub fn new(num_bits: u64, hash_functions: usize) -> Self {
        let size = ((num_bits as f64 / 64.0).ceil() as usize).max(1);
        Self {
            bit_set: vec![0; size],
            num_bits,
            hash_functions,
        }
    }

    /// Adds an item to the BloomFilter.
    pub fn add(&mut self, item: &str) {
        for i in 0..self.hash_functions {
            let index = self.get_bit_index(item, i);
            let bucket_idx = (index / 64) as usize;
            let bit_idx = (index % 64) as usize;

            if bucket_idx < self.bit_set.len() {
                self.bit_set[bucket_idx] |= 1u64 << bit_idx;
            }
        }
    }

    /// Checks if an item might be in the BloomFilter.
    /// Returns true if the item might be in the set, false if it definitely isn't.
    pub fn might_contain(&self, item: &str) -> bool {
        for i in 0..self.hash_functions {
            let index = self.get_bit_index(item, i);
            let bucket_idx = (index / 64) as usize;
            let bit_idx = (index % 64) as usize;

            if bucket_idx >= self.bit_set.len() {
                return false;
            }

            if (self.bit_set[bucket_idx] & (1u64 << bit_idx)) == 0 {
                return false;
            }
        }
        true
    }

    /// Calculates the bit index for an item using double hashing technique.
    fn get_bit_index(&self, item: &str, i: usize) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        let hash1 = hasher.finish();

        let combined = format!("{}{}", item, i);
        let hash2 = hash(combined.as_bytes());

        let combined_hash = hash1.wrapping_add((i as u64).wrapping_mul(hash2));
        combined_hash % self.num_bits
    }
}
