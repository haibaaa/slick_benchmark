//! Linear-probing baseline.

use crate::hash_utils::hash1;
use crate::trait_def::HashTable;
use std::hash::Hash;

/// Open-addressed hash table with one-slot-at-a-time probe advancement.
pub struct LinearTable<K> {
    slots: Vec<Option<K>>,
    capacity: usize,
    count: usize,
}

impl<K: Hash + Eq + Clone> LinearTable<K> {
    const MAX_LOAD: f64 = 0.75;

    /// Inserts into an existing slot array without triggering growth.
    fn raw_insert(slots: &mut Vec<Option<K>>, capacity: usize, key: K) -> bool {
        let start = (hash1(&key) as usize) % capacity;
        let mut idx = start;
        loop {
            match &slots[idx] {
                None => {
                    slots[idx] = Some(key);
                    return true;
                }
                Some(k) if *k == key => return false,
                _ => {
                    idx = (idx + 1) % capacity;
                }
            }
        }
    }

    /// Rehashes all stored keys into a table with doubled capacity.
    fn grow(&mut self) {
        let new_cap = self.capacity * 2;
        let mut new_slots: Vec<Option<K>> = vec![None; new_cap];
        for slot in self.slots.drain(..) {
            if let Some(k) = slot {
                let _ = Self::raw_insert(&mut new_slots, new_cap, k);
            }
        }
        self.slots = new_slots;
        self.capacity = new_cap;
    }
}

impl<K: Hash + Eq + Clone> HashTable<K> for LinearTable<K> {
    fn new(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two().max(16);
        LinearTable {
            slots: vec![None; capacity],
            capacity,
            count: 0,
        }
    }

    fn insert(&mut self, key: K) {
        if (self.count + 1) as f64 / self.capacity as f64 > Self::MAX_LOAD {
            self.grow();
        }
        if Self::raw_insert(&mut self.slots, self.capacity, key) {
            self.count += 1;
        }
    }

    fn find(&self, key: &K) -> bool {
        let start = (hash1(key) as usize) % self.capacity;
        let mut idx = start;
        loop {
            match &self.slots[idx] {
                None => return false,
                Some(k) if k == key => return true,
                _ => {
                    idx = (idx + 1) % self.capacity;
                    if idx == start {
                        return false;
                    }
                }
            }
        }
    }
}
