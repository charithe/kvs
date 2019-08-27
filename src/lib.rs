//! In-memory key-value store

#![deny(missing_docs)]
use std::collections::HashMap;

/// Implements a KV store
#[derive(Default)]
pub struct KvStore {
    store: HashMap<String, String>,
}

impl KvStore {
    /// Create new KvStore instance
    pub fn new() -> KvStore {
        KvStore {
            store: HashMap::new(),
        }
    }

    /// Retrieve the value for a key
    pub fn get(&self, key: String) -> Option<String> {
        self.store.get(&key).map(|s| s.to_string())
    }

    /// Set the value for a key
    pub fn set(&mut self, key: String, value: String) {
        self.store.insert(key, value);
    }

    /// Delete a key
    pub fn remove(&mut self, key: String) {
        self.store.remove(&key);
    }
}
