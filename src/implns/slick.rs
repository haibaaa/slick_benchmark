use crate::trait_def::HashTable;
use std::hash::Hash;
use std::ops::Range;

pub struct SlickHashMetaData {
    offset: usize,
    gap: usize,
    threshold: usize,
}

enum Insertion<'t, V> {
    Inserted(&'t mut V),
    Occupied(&'t mut V),
}

struct Backyard<K> {
    entries: Vec<Option<(K, ())>>,
    capacity: usize,
    count: usize,
}

impl<K: Hash + Eq + Clone> Backyard<K> {
    const MAX_LOAD: f64 = 0.75;

    // SLICKBENCH CHANGE: replaced HashMap backyard with open-addressed overflow table
    fn new(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two().max(16);
        Backyard {
            entries: vec![None; capacity],
            capacity,
            count: 0,
        }
    }

    fn grow(&mut self) {
        let new_capacity = self.capacity * 2;
        let mut new_entries = vec![None; new_capacity];
        for entry in self.entries.drain(..).flatten() {
            let mut idx = (crate::hash_utils::hash1(&entry.0) as usize) % new_capacity;
            loop {
                if new_entries[idx].is_none() {
                    new_entries[idx] = Some(entry);
                    break;
                }
                idx = (idx + 1) % new_capacity;
            }
        }
        self.entries = new_entries;
        self.capacity = new_capacity;
    }

    fn insert(&mut self, key: K, value: ()) -> Insertion<'_, ()> {
        if (self.count + 1) as f64 / self.capacity as f64 > Self::MAX_LOAD {
            self.grow();
        }

        let mut idx = (crate::hash_utils::hash1(&key) as usize) % self.capacity;
        loop {
            if self.entries[idx].is_none() {
                self.entries[idx] = Some((key, value));
                self.count += 1;
                let (_, stored) = self.entries[idx].as_mut().unwrap();
                return Insertion::Inserted(stored);
            }

            let matches = self.entries[idx]
                .as_ref()
                .is_some_and(|(existing_key, _)| existing_key == &key);
            if matches {
                let (_, stored) = self.entries[idx].as_mut().unwrap();
                return Insertion::Occupied(stored);
            }

            idx = (idx + 1) % self.capacity;
        }
    }

    fn get(&self, key: &K) -> Option<&()> {
        let mut idx = (crate::hash_utils::hash1(key) as usize) % self.capacity;
        let start = idx;
        loop {
            match &self.entries[idx] {
                None => return None,
                Some((existing_key, stored)) if existing_key == key => return Some(stored),
                _ => {
                    idx = (idx + 1) % self.capacity;
                    if idx == start {
                        return None;
                    }
                }
            }
        }
    }

    fn remove_entry(&mut self, key: &K) -> Option<(K, ())> {
        let mut idx = (crate::hash_utils::hash1(key) as usize) % self.capacity;
        let start = idx;
        loop {
            match &self.entries[idx] {
                None => return None,
                Some((existing_key, _)) if existing_key == key => {
                    let removed = self.entries[idx].take();
                    if removed.is_some() {
                        self.count -= 1;
                    }

                    let mut next = (idx + 1) % self.capacity;
                    while let Some(entry) = self.entries[next].take() {
                        self.count -= 1;
                        let _ = self.insert(entry.0, entry.1);
                        next = (next + 1) % self.capacity;
                    }

                    return removed;
                }
                _ => {
                    idx = (idx + 1) % self.capacity;
                    if idx == start {
                        return None;
                    }
                }
            }
        }
    }

    fn len(&self) -> usize {
        self.count
    }
}

pub struct SlickHash<K> {
    main_table_size: usize,
    block_size: usize,
    number_of_blocks: usize,
    max_slick_size: usize,
    max_offset: usize,
    max_threshold: usize,
    main_table: Vec<(K, ())>,
    meta_data: Vec<SlickHashMetaData>,
    backyard: Backyard<K>,
    no_elements_in_main_table: usize,
}

impl<K> SlickHash<K>
where
    K: Clone + Eq + PartialEq + Hash + Default,
{
    fn with_capacity(capacity: usize) -> Self {
        let block_size: usize = 10;
        let max_slick_size = block_size * 2;
        let max_offset = block_size;
        let max_threshold = block_size;

        let adjusted_capacity = capacity.max(block_size);
        let main_table_size = ((adjusted_capacity + block_size - 1) / block_size) * block_size;
        let number_of_blocks: usize = main_table_size / block_size;
        let main_table: Vec<(K, ())> = vec![Default::default(); main_table_size];
        let mut meta_data: Vec<SlickHashMetaData> = Vec::with_capacity(number_of_blocks);
        for _ in 0..number_of_blocks {
            meta_data.push(SlickHashMetaData {
                offset: 0,
                gap: block_size,
                threshold: 0,
            });
        }

        Self {
            main_table_size,
            block_size,
            number_of_blocks,
            max_slick_size,
            max_offset,
            max_threshold,
            main_table,
            meta_data,
            backyard: Backyard::new(main_table_size),
            no_elements_in_main_table: 0,
        }
    }

    fn block_start(&self, block_index: usize) -> usize {
        assert!(block_index < self.number_of_blocks);
        self.block_size * block_index + self.meta_data[block_index].offset
    }

    fn block_end(&self, block_index: usize) -> usize {
        assert!(block_index < self.number_of_blocks);
        if block_index == self.number_of_blocks - 1 {
            return self.main_table_size - self.meta_data[block_index].gap;
        }
        self.block_size * block_index + self.block_size + self.meta_data[block_index + 1].offset
            - self.meta_data[block_index].gap
    }

    fn block_range(&self, block_index: usize) -> Range<usize> {
        let start = self.block_start(block_index);
        let end = self.block_end(block_index);
        start..end
    }

    fn insert_into_backyard(&mut self, key: K) -> Insertion<'_, ()> {
        self.backyard.insert(key, ())
    }

    fn slide_gap_from_left(&mut self, block_index: usize) -> bool {
        let mut sliding_block_index = block_index;
        while self.meta_data[sliding_block_index].gap == 0 {
            if (sliding_block_index == 0) || (self.meta_data[sliding_block_index].offset == 0) {
                return false;
            }
            sliding_block_index -= 1;
        }

        let empty_block_has_gap_one = (self.meta_data[sliding_block_index].gap == 1)
            && (self.block_start(sliding_block_index) == self.block_end(sliding_block_index));
        if empty_block_has_gap_one {
            return false;
        }

        self.meta_data[sliding_block_index].gap -= 1;
        sliding_block_index += 1;
        while sliding_block_index <= block_index {
            let start_sliding_block = self.block_start(sliding_block_index);
            let end_sliding_block = self.block_end(sliding_block_index);
            self.main_table[start_sliding_block - 1] =
                self.main_table[end_sliding_block - 1].clone();
            self.meta_data[sliding_block_index].offset -= 1;
            sliding_block_index += 1;
        }
        self.meta_data[sliding_block_index - 1].gap += 1;
        true
    }

    fn slide_gap_from_right(&mut self, block_index: usize) -> bool {
        if block_index == self.number_of_blocks - 1 {
            return false;
        }

        let mut sliding_block_index = block_index + 1;
        while self.meta_data[sliding_block_index].gap == 0 {
            if (sliding_block_index == self.number_of_blocks - 1)
                || (self.meta_data[sliding_block_index].offset == self.max_offset)
            {
                return false;
            }
            sliding_block_index += 1;
        }

        if self.meta_data[sliding_block_index].offset == self.max_offset {
            return false;
        }

        let empty_block_has_gap_one = (self.meta_data[sliding_block_index].gap == 1)
            && (self.block_start(sliding_block_index) == self.block_end(sliding_block_index));
        if empty_block_has_gap_one {
            return false;
        }

        let start_sliding_block = self.block_start(sliding_block_index);
        let end_sliding_block = self.block_end(sliding_block_index);
        self.main_table[end_sliding_block] = self.main_table[start_sliding_block].clone();

        self.meta_data[sliding_block_index].offset += 1;
        self.meta_data[sliding_block_index].gap -= 1;
        sliding_block_index -= 1;

        while sliding_block_index > block_index {
            let start_sliding_block = self.block_start(sliding_block_index);
            let end_sliding_block = self.block_end(sliding_block_index);
            self.main_table[end_sliding_block - 1] = self.main_table[start_sliding_block].clone();

            self.meta_data[sliding_block_index].offset += 1;
            sliding_block_index -= 1;
        }
        self.meta_data[sliding_block_index].gap += 1;
        true
    }

    fn hash_block_index(&self, key: &K) -> usize {
        // SLICKBENCH CHANGE: replaced original hash with hash_utils::hash1/hash2
        let hash = crate::hash_utils::hash1(key) as f64;
        ((hash / (u64::MAX as f64)) * self.number_of_blocks as f64) as usize
    }

    fn hash_threshold(&self, key: &K) -> usize {
        // SLICKBENCH CHANGE: replaced original hash with hash_utils::hash1/hash2
        let hash = crate::hash_utils::hash2(key) as f64;
        ((hash / (u64::MAX as f64)) * self.max_threshold as f64) as usize
    }

    fn there_is_no_space(&mut self, block_range: &Range<usize>, block_index: usize) -> bool {
        (block_range.len() >= self.max_slick_size)
            || !(self.meta_data[block_index].gap > 0
                || self.slide_gap_from_left(block_index)
                || self.slide_gap_from_right(block_index))
    }

    fn try_insert(&mut self, key: K) -> Insertion<'_, ()> {
        let block_index = self.hash_block_index(&key);
        let block_start = self.block_start(block_index);
        let block_range = self.block_range(block_index);
        if self.hash_threshold(&key) < self.meta_data[block_index].threshold {
            return self.insert_into_backyard(key);
        }

        if !block_range.is_empty() {
            let block_range_elements_as_mut = &self.main_table[block_range.clone()];
            let mut found_index = None;
            for (index, (iter_key, _)) in block_range_elements_as_mut.iter().enumerate() {
                if *iter_key == key {
                    found_index = Some(index);
                    break;
                }
            }

            if let Some(some_found_index) = found_index {
                return Insertion::Inserted(&mut self.main_table[some_found_index].1);
            }
        }

        let there_is_no_space = self.there_is_no_space(&block_range, block_index);
        if there_is_no_space {
            let mut min_threshold_hash = self.max_threshold + 1;
            for (iter_key, _) in &self.main_table[block_range.clone()] {
                let key_threshold = self.hash_threshold(iter_key);
                if key_threshold < min_threshold_hash {
                    min_threshold_hash = key_threshold;
                }
            }

            assert!(min_threshold_hash < self.max_threshold + 1);

            if self.hash_threshold(&key) < min_threshold_hash {
                min_threshold_hash = self.hash_threshold(&key);
            }
            let t_prime = min_threshold_hash + 1;

            self.meta_data[block_index].threshold = t_prime;
            let mut j = block_start;
            let mut block_end = self.block_end(block_index);
            while j < block_end {
                let (iter_key, _) = &self.main_table[j];
                let key_threshold = self.hash_threshold(iter_key);
                if key_threshold < t_prime {
                    let _ = self.insert_into_backyard(iter_key.clone());
                    self.no_elements_in_main_table -= 1;
                    self.main_table[j] = self.main_table[block_end - 1].clone();
                    self.meta_data[block_index].gap += 1;
                    block_end = self.block_end(block_index);
                } else {
                    j += 1;
                }
            }
            if self.hash_threshold(&key) < t_prime {
                return self.insert_into_backyard(key);
            }
        }

        let current_block_end = self.block_end(block_index);
        self.main_table[current_block_end] = (key, ());
        self.no_elements_in_main_table += 1;
        self.meta_data[block_index].gap -= 1;

        if self.no_elements_in_main_table + self.backyard.len() == 2_000_000 {
            println!(
                "Final number of elements in main table: {}",
                self.no_elements_in_main_table
            );
            println!(
                "Final number of elements in backyard table: {}",
                self.backyard.len()
            );
        }

        Insertion::Inserted(&mut self.main_table[current_block_end].1)
    }

    fn get(&self, key: &K) -> Option<&()> {
        let block_index = self.hash_block_index(key);
        if self.hash_threshold(key) < self.meta_data[block_index].threshold {
            return self.backyard.get(key);
        }
        let block_range = self.block_range(block_index);
        let key_value_in_main_table = self.main_table[block_range]
            .iter()
            .find(|key_value_pair| key_value_pair.0 == *key);
        match key_value_in_main_table {
            Some(kvp) => Some(&kvp.1),
            None => None,
        }
    }

    #[allow(dead_code)]
    fn remove_entry(&mut self, key: &K) -> Option<(K, ())> {
        let block_index = self.hash_block_index(key);
        let mut remove_value = None;
        if self.hash_threshold(key) < self.meta_data[block_index].threshold {
            remove_value = self.backyard.remove_entry(key);
        }
        for i in self.block_range(block_index) {
            if *key == self.main_table[i].0 {
                let key_value_pair = self.main_table[i].clone();
                self.main_table[i] = self.main_table[self.block_end(block_index) - 1].clone();
                self.meta_data[block_index].gap += 1;
                self.no_elements_in_main_table -= 1;
                remove_value = Some(key_value_pair);
                break;
            }
        }
        remove_value
    }
}

// SLICKBENCH CHANGE: added HashTable trait impl
impl<K> HashTable<K> for SlickHash<K>
where
    K: Clone + Eq + PartialEq + Hash + Default,
{
    fn new(capacity: usize) -> Self {
        SlickHash::with_capacity(capacity)
    }

    fn insert(&mut self, key: K) {
        match self.try_insert(key) {
            Insertion::Inserted(value) | Insertion::Occupied(value) => {
                let _ = value;
            }
        }
    }

    fn find(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
}
