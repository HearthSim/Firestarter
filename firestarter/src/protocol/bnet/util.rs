//! Utility methods for working with client sessions.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Retrieves a collision resistant hash (u64) of the provided item.
pub fn hash_item<X: Hash>(x: X) -> u64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}
