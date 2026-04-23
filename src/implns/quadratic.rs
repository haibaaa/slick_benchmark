use crate::hash_utils::hash1;
use crate::trait_def::HashTable;
use std::hash::Hash;

pub struct QuadraticTable<K> {
    slots: Vec<Option<K>>,
    capacity: usize,
    count: usize,
}

impl<K: Hash + Eq + Clone> QuadraticTable<K> {
    const MAX_LOAD: f64 = 0.75;

    fn raw_insert(slots: &mut [Option<K>], capacity: usize, key: K) -> bool {
        let start = (hash1(&key) as usize) % capacity;
        let mut idx = start;
        let mut step = 1usize;
        loop {
            match &slots[idx] {
                None => {
                    slots[idx] = Some(key);
                    return true;
                }
                Some(k) if *k == key => return false,
                _ => {
                    idx = (start + step * step) % capacity;
                    step += 1;
                }
            }
        }
    }

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

impl<K: Hash + Eq + Clone> HashTable<K> for QuadraticTable<K> {
    fn new(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two().max(16);
        QuadraticTable {
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
        let mut step = 1usize;
        loop {
            match &self.slots[idx] {
                None => return false,
                Some(k) if k == key => return true,
                _ => {
                    idx = (start + step * step) % self.capacity;
                    step += 1;
                    if step > self.capacity {
                        return false;
                    }
                }
            }
        }
    }
}
