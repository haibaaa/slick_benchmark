# AGENT.md — SlickBench

> **READ THIS ENTIRE FILE BEFORE WRITING A SINGLE LINE OF CODE.**
> This document is the authoritative specification. Every section is a constraint, not a suggestion.
> When in doubt, stop and re-read the relevant section. Do not invent solutions not described here.

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Critical Architecture Rules](#2-critical-architecture-rules)
3. [Repository Layout](#3-repository-layout)
4. [Dependencies](#4-dependencies)
5. [Hashing Rules (STRICT)](#5-hashing-rules-strict)
6. [Trait Interface](#6-trait-interface)
7. [Module Specifications](#7-module-specifications)
8. [Slick Hash Integration](#8-slick-hash-integration)
9. [Other Hash Table Implementations](#9-other-hash-table-implementations)
10. [Datasets](#10-datasets)
11. [Workloads](#11-workloads)
12. [Benchmark Runner](#12-benchmark-runner)
13. [Metrics and Output Format](#13-metrics-and-output-format)
14. [Python Orchestration (uv)](#14-python-orchestration-uv)
15. [Phased Development](#15-phased-development)
16. [Forbidden Behaviors](#16-forbidden-behaviors)
17. [Success Criteria](#17-success-criteria)

---

## 1. Project Overview

**SlickBench** is a research-grade benchmarking framework written in Rust. Its purpose is to evaluate
Slick Hash and competing hash table implementations under controlled, real-world-representative
conditions, ensuring:

- **Fair comparison**: all tables hash using the same algorithm (AHasher)
- **No implementation bias**: Slick's algorithm is preserved exactly from `refs/slick_core.rs`
- **Real-world relevance**: datasets include actual word lists and Wikipedia titles
- **Reproducibility**: seeded randomness everywhere, CSV output, Python-driven orchestration
- **Incremental correctness**: phased development — each phase is independently runnable

### What This Project Is

An evaluation of **hash table behavior under controlled real-world conditions**, measuring:
- Insert throughput (ns/op)
- Lookup throughput (ns/op)
- Behavior under different load factors
- Behavior across synthetic and real-world key distributions

### What This Project Is NOT

- A benchmark of hash functions themselves
- An application-specific system
- A replacement or redesign of Slick Hash

---

## 2. Critical Architecture Rules

### 2.1 This Is a Fresh Repository

**DO NOT:**
- Reuse any old benchmark structure from any previous project
- Import any old benchmark files
- Copy any code from outside `refs/slick_core.rs` (except standard library and declared crates)
- Explore external repos unless explicitly instructed in a future phase update

**DO:**
- Start from scratch using only `Cargo.toml`, `src/`, `refs/`, and `scripts/`

### 2.2 Ground Truth for Slick

The file `refs/slick_core.rs` is the **sole source of truth** for Slick Hash.

- Its algorithm must be preserved exactly
- Its control flow must not be altered
- Its data structures must not be simplified
- Only the three modifications listed in §8 are permitted

### 2.3 Single Binary

All tables run from a **single benchmark binary**. There is no separate binary per implementation.

```
cargo run --release -- --dataset uniform --workload bulk
```

---

## 3. Repository Layout

```
slickbench/
├── AGENT.md                  ← This file
├── Cargo.toml
├── Cargo.lock
├── refs/
│   └── slick_core.rs         ← READ-ONLY ground truth. Never import as a module.
│                                Copy-adapt into src/implns/slick.rs per §8.
├── src/
│   ├── main.rs               ← CLI entry point, delegates to runner
│   ├── lib.rs                ← Re-exports all public modules
│   ├── trait_def.rs          ← HashTable trait (§6)
│   ├── hash_utils.rs         ← Hashing helpers (§5)
│   ├── implns/
│   │   ├── mod.rs
│   │   ├── slick.rs          ← Slick Hash (adapted from refs/slick_core.rs)
│   │   ├── linear.rs         ← Linear probing
│   │   ├── quadratic.rs      ← Quadratic probing
│   │   ├── cuckoo.rs         ← Cuckoo hashing
│   │   └── std_set.rs        ← Rust std HashSet baseline
│   ├── datasets/
│   │   ├── mod.rs
│   │   ├── uniform.rs        ← Uniform random u64
│   │   ├── sequential.rs     ← Sequential u64
│   │   ├── zipf.rs           ← Zipf distribution u64
│   │   ├── norvig.rs         ← Norvig word list (String keys)
│   │   └── wikipedia.rs      ← Wikipedia titles (String keys)
│   ├── workloads/
│   │   ├── mod.rs
│   │   ├── bulk.rs           ← Phase 1 workload
│   │   ├── mixed.rs          ← Phase 4 workload
│   │   └── read_heavy.rs     ← Phase 5 workload
│   ├── runner/
│   │   ├── mod.rs
│   │   └── bench.rs          ← Core benchmark loop
│   └── metrics/
│       ├── mod.rs
│       └── record.rs         ← BenchRecord struct, CSV serialization
├── data/
│   ├── norvig_words.txt      ← Downloaded word list (§10.4)
│   └── wiki_titles.txt       ← Downloaded Wikipedia titles (§10.5)
└── scripts/
    ├── bench.py              ← Python orchestrator (§14)
    ├── plot.py               ← Plot generation
    └── download_data.py      ← Fetches data/ files
```

### Module Responsibility Summary

| Module | Responsibility |
|---|---|
| `trait_def.rs` | Defines `HashTable<K>` trait; nothing else |
| `hash_utils.rs` | `make_hasher`, `hash1`, `hash2` functions; no table logic |
| `implns/` | One file per table; each implements `HashTable<K>` |
| `datasets/` | Pure data generation/loading; returns `Vec<K>` |
| `workloads/` | Defines operation sequences over a dataset; returns `WorkloadResult` |
| `runner/bench.rs` | Wires datasets + workloads + tables; calls timing |
| `metrics/record.rs` | `BenchRecord` struct; CSV serialization |
| `main.rs` | CLI parsing; calls `runner` |

---

## 4. Dependencies

Add to `Cargo.toml`:

```toml
[package]
name = "slickbench"
version = "0.1.0"
edition = "2021"

[dependencies]
ahash = "0.8"
rand = { version = "0.8", features = ["small_rng"] }
rand_distr = "0.4"       # for Zipf
csv = "1.3"
serde = { version = "1", features = ["derive"] }
clap = { version = "4", features = ["derive"] }

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

**Do not add** any other hash table crates (hashbrown, indexmap, etc.) unless a future phase
explicitly instructs it. The std `HashSet` baseline uses its built-in implementation.

---

## 5. Hashing Rules (STRICT)

All hash tables in this project must hash keys using **AHasher from the `ahash` crate**.
There are no exceptions.

### 5.1 Why This Matters

If different tables use different hash functions, measured performance differences may reflect
hash function quality rather than table design. This project controls for that variable.

### 5.2 Hash Utility Module (`src/hash_utils.rs`)

This is the only place where hashing logic is defined. All tables call into this module.
**Do not duplicate hashing logic in any `implns/*.rs` file.**

```rust
// src/hash_utils.rs

use ahash::AHasher;
use std::hash::{Hash, Hasher};

/// Fixed seed pair used project-wide.
/// Changing these values invalidates reproducibility across runs.
pub const SEED1: (u64, u64) = (0x243F_6A88_85A3_08D3, 0x1319_8A2E_0370_7344);
pub const SEED2: (u64, u64) = (0xA409_3822_299F_31D0, 0x082E_FA98_EC4E_6C89);

/// Creates a seeded AHasher using a (u64, u64) seed pair.
#[inline]
pub fn make_hasher(seed: (u64, u64)) -> AHasher {
    AHasher::new_with_keys(seed.0, seed.1)
}

/// Computes h1 for any hashable key (used by all tables).
#[inline]
pub fn hash1<K: Hash>(key: &K) -> u64 {
    let mut h = make_hasher(SEED1);
    key.hash(&mut h);
    h.finish()
}

/// Computes h2 for any hashable key (used by Slick only).
#[inline]
pub fn hash2<K: Hash>(key: &K) -> u64 {
    let mut h = make_hasher(SEED2);
    key.hash(&mut h);
    h.finish()
}
```

### 5.3 Per-Table Hashing Rules

| Table | Uses | Seeds |
|---|---|---|
| Slick Hash | `hash1` + `hash2` | SEED1, SEED2 |
| Linear Probing | `hash1` only | SEED1 |
| Quadratic Probing | `hash1` only | SEED1 |
| Cuckoo Hashing | `hash1` + `hash2` | SEED1, SEED2 (for two-table scheme) |
| std HashSet | Internal (cannot override) | N/A — baseline only |

> **Note on std HashSet**: Since we cannot inject AHasher into `std::collections::HashSet` without
> using the `BuildHasher` API, use `std::collections::HashSet<K, ahash::RandomState>` with a fixed
> seed instead. See §9.5 for the exact implementation.

---

## 6. Trait Interface

### 6.1 Definition (`src/trait_def.rs`)

```rust
// src/trait_def.rs

/// Common interface for all hash table implementations.
///
/// RULES:
/// - Do NOT add Slick-specific methods here.
/// - Do NOT add statistics/diagnostics methods here.
/// - Keep this trait minimal: new, insert, find.
/// - Hashing is always internal to the implementation (not passed in).
///
/// K must be Hash + Eq + Clone.
/// Clone is required so datasets can be reused across multiple table instances.
pub trait HashTable<K: std::hash::Hash + Eq + Clone> {
    /// Create a new table with the given initial capacity.
    /// Capacity is a hint; the table may resize.
    fn new(capacity: usize) -> Self;

    /// Insert a key into the table.
    /// If the key already exists, behavior is implementation-defined
    /// (idempotent insert is acceptable; duplicate tracking is not required).
    fn insert(&mut self, key: K);

    /// Return true if key exists in the table, false otherwise.
    fn find(&self, key: &K) -> bool;
}
```

### 6.2 Key Constraints

- Every `implns/*.rs` file must have exactly one `struct` that `impl HashTable<K>` where `K` is
  the concrete key type used (typically `u64` for synthetic datasets, `String` for real datasets).
- The trait must not be modified to accommodate a specific implementation. If an implementation
  requires extra initialization, use associated constants or builder patterns internally.
- Do not make `HashTable` object-safe by adding `where Self: Sized` — this will be needed for
  monomorphized dispatch in the runner.

---

## 7. Module Specifications

### 7.1 `src/hash_utils.rs`

Already fully specified in §5.2. No additional functions. No table logic.

### 7.2 `src/trait_def.rs`

Already fully specified in §6.1. No additional items.

### 7.3 `src/implns/mod.rs`

```rust
// src/implns/mod.rs
pub mod slick;
pub mod linear;
pub mod quadratic;  // Phase 2
pub mod cuckoo;     // Phase 2
pub mod std_set;    // Phase 2
```

Comment out Phase 2+ modules during Phase 1. Do not leave `mod` declarations for unimplemented
modules — this causes compilation failure.

### 7.4 `src/datasets/mod.rs`

```rust
// src/datasets/mod.rs

/// A loaded dataset, ready for benchmarking.
/// Keys are already shuffled and NOT deduplicated.
pub struct Dataset<K> {
    pub name: String,
    pub keys: Vec<K>,
}

pub mod uniform;
pub mod sequential;  // Phase 2
pub mod zipf;        // Phase 3
pub mod norvig;      // Phase 3
pub mod wikipedia;   // Phase 3
```

### 7.5 `src/workloads/mod.rs`

```rust
// src/workloads/mod.rs

/// Result of running one workload on one table instance.
pub struct WorkloadResult {
    /// Total nanoseconds spent on insert operations.
    pub insert_ns: u64,
    /// Total nanoseconds spent on find operations.
    pub find_ns: u64,
    /// Number of insert operations performed.
    pub insert_count: usize,
    /// Number of find operations performed.
    pub find_count: usize,
}

pub mod bulk;
pub mod mixed;       // Phase 4
pub mod read_heavy;  // Phase 5
```

### 7.6 `src/runner/bench.rs`

Specified in §12.

### 7.7 `src/metrics/record.rs`

```rust
// src/metrics/record.rs
use serde::Serialize;

/// One row in the output CSV.
#[derive(Debug, Serialize)]
pub struct BenchRecord {
    pub dataset: String,
    pub workload: String,
    pub table: String,
    /// Load factor at end of workload (inserted_count / capacity).
    pub load_factor: f64,
    /// Average nanoseconds per insert operation.
    pub insert_ns_per_op: f64,
    /// Average nanoseconds per find operation.
    pub find_ns_per_op: f64,
    /// Total number of inserts performed.
    pub insert_count: usize,
    /// Total number of finds performed.
    pub find_count: usize,
}

impl BenchRecord {
    pub fn from_result(
        dataset: &str,
        workload: &str,
        table: &str,
        load_factor: f64,
        result: &crate::workloads::WorkloadResult,
    ) -> Self {
        BenchRecord {
            dataset: dataset.to_string(),
            workload: workload.to_string(),
            table: table.to_string(),
            load_factor,
            insert_ns_per_op: result.insert_ns as f64 / result.insert_count.max(1) as f64,
            find_ns_per_op: result.find_ns as f64 / result.find_count.max(1) as f64,
            insert_count: result.insert_count,
            find_count: result.find_count,
        }
    }
}

/// Write records to a CSV file. Appends if file exists; creates if not.
pub fn write_csv(path: &str, records: &[BenchRecord]) -> Result<(), Box<dyn std::error::Error>> {
    let file_exists = std::path::Path::new(path).exists();
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(!file_exists)
        .from_writer(file);
    for record in records {
        wtr.serialize(record)?;
    }
    wtr.flush()?;
    Ok(())
}
```

---

## 8. Slick Hash Integration

### 8.1 Source File

The Slick Hash implementation lives in `refs/slick_core.rs`. This file is read-only.

**Before starting Phase 1**, read `refs/slick_core.rs` in full. Understand every field,
every method, every control-flow branch. Do not guess.

### 8.2 Creating `src/implns/slick.rs`

Copy the contents of `refs/slick_core.rs` into `src/implns/slick.rs`. Then apply **only** the
three permitted modifications below. Document each change with a `// SLICKBENCH CHANGE:` comment.

#### Permitted Modification 1 — Replace Hashing

Slick's original code uses some internal or ad-hoc hashing. Replace all hashing with calls to
`crate::hash_utils::hash1` and `crate::hash_utils::hash2`.

```rust
// SLICKBENCH CHANGE: replaced original hash with hash_utils::hash1/hash2
let h1 = crate::hash_utils::hash1(&key);
let h2 = crate::hash_utils::hash2(&key);
```

**Do not** change how `h1` and `h2` are used to index into the table. Only change how they are
computed.

#### Permitted Modification 2 — Replace Backyard

If Slick's original code uses `HashMap` as a backyard (overflow store), replace it with a
`Vec<(K, ())>` or a flat open-addressing table that also uses `hash1`. The replacement must:

- Store overflow keys faithfully
- Support insert and lookup
- Use `hash1` for internal hashing (not `HashMap`'s default hasher)
- Be no more complex than a small linear-probing table

Example replacement (if backyard is small):

```rust
// SLICKBENCH CHANGE: replaced HashMap backyard with Vec-based overflow
struct Backyard<K> {
    entries: Vec<Option<K>>,
    capacity: usize,
}

impl<K: std::hash::Hash + Eq + Clone> Backyard<K> {
    fn new(capacity: usize) -> Self {
        Backyard { entries: vec![None; capacity], capacity }
    }

    fn insert(&mut self, key: K) {
        let mut idx = (crate::hash_utils::hash1(&key) as usize) % self.capacity;
        loop {
            match &self.entries[idx] {
                None => { self.entries[idx] = Some(key); return; }
                Some(k) if *k == key => return, // idempotent
                _ => idx = (idx + 1) % self.capacity,
            }
        }
    }

    fn find(&self, key: &K) -> bool {
        let mut idx = (crate::hash_utils::hash1(key) as usize) % self.capacity;
        loop {
            match &self.entries[idx] {
                None => return false,
                Some(k) if k == key => return true,
                _ => idx = (idx + 1) % self.capacity,
            }
        }
    }
}
```

#### Permitted Modification 3 — Adapt to Trait Interface

Add `impl HashTable<K> for SlickTable<K>` (or whatever the original struct is named) that
delegates to the original methods:

```rust
// SLICKBENCH CHANGE: added HashTable trait impl
impl<K: std::hash::Hash + Eq + Clone> crate::trait_def::HashTable<K> for SlickTable<K> {
    fn new(capacity: usize) -> Self {
        SlickTable::with_capacity(capacity) // use original constructor
    }

    fn insert(&mut self, key: K) {
        self.insert_key(key); // use original insert method name
    }

    fn find(&self, key: &K) -> bool {
        self.lookup(key) // use original lookup method name
    }
}
```

Adjust method names to match what actually exists in `refs/slick_core.rs`.

### 8.3 What You Must NOT Change in Slick

- The primary data structure layout (bucket array, slot organization, etc.)
- Probe sequence logic
- Capacity growth / rehashing triggers
- Any bitwise operations or SIMD code
- The overall control flow of insert or lookup

If you are tempted to "simplify" anything in Slick: **stop and re-read this section.**

---

## 9. Other Hash Table Implementations

Each implementation lives in its own file in `src/implns/`. Each must:

1. Define exactly one struct
2. Implement `HashTable<K>` for `K: Hash + Eq + Clone`
3. Use only `hash_utils::hash1` (and `hash2` for Cuckoo)
4. Be self-contained (no cross-imports between `implns/` files)

### 9.1 `src/implns/linear.rs` — Linear Probing

```rust
use crate::hash_utils::hash1;
use crate::trait_def::HashTable;
use std::hash::Hash;

pub struct LinearTable<K> {
    slots: Vec<Option<K>>,
    capacity: usize,
    count: usize,
}

impl<K: Hash + Eq + Clone> LinearTable<K> {
    const MAX_LOAD: f64 = 0.75;

    fn raw_insert(slots: &mut Vec<Option<K>>, capacity: usize, key: K) {
        let start = (hash1(&key) as usize) % capacity;
        let mut idx = start;
        loop {
            match &slots[idx] {
                None => { slots[idx] = Some(key); return; }
                Some(k) if *k == key => return, // idempotent
                _ => { idx = (idx + 1) % capacity; }
            }
        }
    }

    fn grow(&mut self) {
        let new_cap = self.capacity * 2;
        let mut new_slots: Vec<Option<K>> = vec![None; new_cap];
        for slot in self.slots.drain(..) {
            if let Some(k) = slot {
                Self::raw_insert(&mut new_slots, new_cap, k);
            }
        }
        self.slots = new_slots;
        self.capacity = new_cap;
    }
}

impl<K: Hash + Eq + Clone> HashTable<K> for LinearTable<K> {
    fn new(capacity: usize) -> Self {
        // Round up to power of two for efficient modulo (optional optimization)
        let capacity = capacity.next_power_of_two().max(16);
        LinearTable { slots: vec![None; capacity], capacity, count: 0 }
    }

    fn insert(&mut self, key: K) {
        if (self.count + 1) as f64 / self.capacity as f64 > Self::MAX_LOAD {
            self.grow();
        }
        Self::raw_insert(&mut self.slots, self.capacity, key);
        self.count += 1;
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
                    if idx == start { return false; } // full table guard
                }
            }
        }
    }
}
```

### 9.2 `src/implns/quadratic.rs` — Quadratic Probing

Same structure as `linear.rs`. Replace the probe sequence:

```rust
// In raw_insert and find, replace linear step with quadratic:
let mut idx = start;
let mut step = 1usize;
loop {
    match &slots[idx] {
        None => { slots[idx] = Some(key); return; }
        Some(k) if *k == key => return,
        _ => {
            idx = (start + step * step) % capacity;
            step += 1;
        }
    }
}
```

Use the same `MAX_LOAD = 0.75` and same grow logic as `linear.rs`.

### 9.3 `src/implns/cuckoo.rs` — Cuckoo Hashing

Cuckoo hashing uses two separate tables and two hash functions.

```rust
use crate::hash_utils::{hash1, hash2};
use crate::trait_def::HashTable;
use std::hash::Hash;

const MAX_KICKS: usize = 128;

pub struct CuckooTable<K> {
    table1: Vec<Option<K>>,
    table2: Vec<Option<K>>,
    capacity: usize, // per-table capacity
    count: usize,
}

impl<K: Hash + Eq + Clone> CuckooTable<K> {
    fn idx1(key: &K, cap: usize) -> usize { (hash1(key) as usize) % cap }
    fn idx2(key: &K, cap: usize) -> usize { (hash2(key) as usize) % cap }

    fn try_insert(
        t1: &mut Vec<Option<K>>,
        t2: &mut Vec<Option<K>>,
        cap: usize,
        mut key: K,
    ) -> Result<(), K> {
        for _ in 0..MAX_KICKS {
            let i1 = Self::idx1(&key, cap);
            if t1[i1].is_none() { t1[i1] = Some(key); return Ok(()); }
            key = t1[i1].replace(key).unwrap();

            let i2 = Self::idx2(&key, cap);
            if t2[i2].is_none() { t2[i2] = Some(key); return Ok(()); }
            key = t2[i2].replace(key).unwrap();
        }
        Err(key) // eviction cycle detected
    }

    fn rebuild(&mut self) {
        let new_cap = self.capacity * 2;
        loop {
            let mut new_t1 = vec![None; new_cap];
            let mut new_t2 = vec![None; new_cap];
            let mut failed = false;
            let old1 = std::mem::replace(&mut self.table1, vec![]);
            let old2 = std::mem::replace(&mut self.table2, vec![]);
            for k in old1.into_iter().chain(old2.into_iter()).flatten() {
                if Self::try_insert(&mut new_t1, &mut new_t2, new_cap, k).is_err() {
                    failed = true;
                    break;
                }
            }
            if !failed {
                self.table1 = new_t1;
                self.table2 = new_t2;
                self.capacity = new_cap;
                return;
            }
            // Try again with larger capacity
        }
    }
}

impl<K: Hash + Eq + Clone> HashTable<K> for CuckooTable<K> {
    fn new(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two().max(16);
        CuckooTable {
            table1: vec![None; capacity],
            table2: vec![None; capacity],
            capacity,
            count: 0,
        }
    }

    fn insert(&mut self, key: K) {
        // Idempotent check
        if self.find(&key) { return; }
        loop {
            match Self::try_insert(&mut self.table1, &mut self.table2, self.capacity, key.clone()) {
                Ok(()) => { self.count += 1; return; }
                Err(_) => self.rebuild(),
            }
        }
    }

    fn find(&self, key: &K) -> bool {
        let i1 = Self::idx1(key, self.capacity);
        let i2 = Self::idx2(key, self.capacity);
        self.table1[i1].as_ref().map_or(false, |k| k == key)
            || self.table2[i2].as_ref().map_or(false, |k| k == key)
    }
}
```

### 9.4 `src/implns/std_set.rs` — std HashSet Baseline

```rust
use crate::trait_def::HashTable;
use std::hash::Hash;

/// Uses ahash::RandomState with a fixed seed to approximate controlled hashing.
/// This is the closest we can get to AHasher with std::collections::HashSet.
pub struct StdSetTable<K> {
    inner: std::collections::HashSet<K, ahash::RandomState>,
}

impl<K: Hash + Eq + Clone> HashTable<K> for StdSetTable<K> {
    fn new(capacity: usize) -> Self {
        // Use fixed seed for reproducibility
        let state = ahash::RandomState::with_seeds(
            crate::hash_utils::SEED1.0,
            crate::hash_utils::SEED1.1,
            crate::hash_utils::SEED2.0,
            crate::hash_utils::SEED2.1,
        );
        StdSetTable {
            inner: std::collections::HashSet::with_capacity_and_hasher(capacity, state),
        }
    }

    fn insert(&mut self, key: K) {
        self.inner.insert(key);
    }

    fn find(&self, key: &K) -> bool {
        self.inner.contains(key)
    }
}
```

---

## 10. Datasets

### 10.1 Dataset Contract

Every dataset function must:
1. Return `Dataset<K>` (defined in `src/datasets/mod.rs`)
2. Shuffle the output using `rand::seq::SliceRandom::shuffle` with a seeded RNG
3. **NOT** deduplicate keys — preserve distributions as-is
4. Accept a `size: usize` parameter (number of keys to generate)
5. Accept a `seed: u64` parameter for reproducible generation

### 10.2 `src/datasets/uniform.rs`

```rust
use rand::{SeedableRng, Rng};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use crate::datasets::Dataset;

/// Generates `size` uniformly random u64 values.
/// Keys are NOT deduplicated — duplicates are expected and preserved.
pub fn generate(size: usize, seed: u64) -> Dataset<u64> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut keys: Vec<u64> = (0..size).map(|_| rng.gen::<u64>()).collect();
    keys.shuffle(&mut rng);
    Dataset { name: "uniform".to_string(), keys }
}
```

### 10.3 `src/datasets/sequential.rs`

```rust
use rand::{SeedableRng};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use crate::datasets::Dataset;

/// Generates keys [0, 1, 2, ..., size-1] then shuffles them.
/// Represents a cache-friendly, low-entropy distribution.
pub fn generate(size: usize, seed: u64) -> Dataset<u64> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut keys: Vec<u64> = (0u64..size as u64).collect();
    keys.shuffle(&mut rng);
    Dataset { name: "sequential".to_string(), keys }
}
```

### 10.4 `src/datasets/zipf.rs`

```rust
use rand::{SeedableRng};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand_distr::{Zipf, Distribution};
use crate::datasets::Dataset;

/// Generates `size` u64 keys from a Zipf distribution.
/// exponent = 1.1 (configurable, but default to 1.1 for skewed real-world approximation).
/// Keys map directly from Zipf samples (1-indexed u64).
pub fn generate(size: usize, seed: u64) -> Dataset<u64> {
    let mut rng = SmallRng::seed_from_u64(seed);
    // Zipf::new(n, s): n = population size, s = exponent
    let zipf = Zipf::new(size as u64, 1.1).expect("Zipf distribution creation failed");
    let mut keys: Vec<u64> = (0..size).map(|_| zipf.sample(&mut rng) as u64).collect();
    keys.shuffle(&mut rng);
    Dataset { name: "zipf".to_string(), keys }
}
```

### 10.5 `src/datasets/norvig.rs`

Peter Norvig's word frequency list. Download script: `scripts/download_data.py`.

Source URL: `https://norvig.com/ngrams/count_1w.txt`

```rust
use rand::{SeedableRng};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use crate::datasets::Dataset;

/// Loads words from data/norvig_words.txt.
/// Each line is: `<word>\t<count>` — we take only the word.
/// Returns at most `size` words (or all words if the file has fewer).
pub fn load(size: usize, seed: u64) -> Dataset<String> {
    let content = std::fs::read_to_string("data/norvig_words.txt")
        .expect("data/norvig_words.txt not found. Run: uv run scripts/download_data.py");
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut keys: Vec<String> = content
        .lines()
        .filter_map(|line| line.split('\t').next().map(|w| w.to_string()))
        .take(size)
        .collect();
    keys.shuffle(&mut rng);
    Dataset { name: "norvig".to_string(), keys }
}
```

### 10.6 `src/datasets/wikipedia.rs`

Wikipedia article titles. Download script: `scripts/download_data.py`.

Source: Wikimedia dump titles-current.txt (or a preprocessed subset).

```rust
use rand::{SeedableRng};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use crate::datasets::Dataset;

/// Loads titles from data/wiki_titles.txt (one title per line).
/// Returns at most `size` titles.
pub fn load(size: usize, seed: u64) -> Dataset<String> {
    let content = std::fs::read_to_string("data/wiki_titles.txt")
        .expect("data/wiki_titles.txt not found. Run: uv run scripts/download_data.py");
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut keys: Vec<String> = content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .take(size)
        .collect();
    keys.shuffle(&mut rng);
    Dataset { name: "wikipedia".to_string(), keys }
}
```

---

## 11. Workloads

### 11.1 Workload Contract

Every workload function must:
1. Accept a `HashTable<K>` implementor (via generic parameter)
2. Accept a `&Dataset<K>` reference
3. Use `std::time::Instant` for timing
4. Return `WorkloadResult`
5. Not allocate a second copy of the dataset for lookup keys — reuse the same slice

### 11.2 Timing Primitives

Use only `std::time::Instant`. Do not use `std::time::SystemTime`. Do not use external timer crates.

```rust
let t = std::time::Instant::now();
// ... operations ...
let elapsed_ns = t.elapsed().as_nanos() as u64;
```

Wrap each batch of operations (all inserts, then all finds) in a single timer, not per-operation.

### 11.3 `src/workloads/bulk.rs` — Bulk Workload

The Bulk workload inserts all keys, then looks up all keys. It is the simplest workload and
is used in Phase 1 to verify correctness.

```rust
use crate::datasets::Dataset;
use crate::trait_def::HashTable;
use crate::workloads::WorkloadResult;
use std::hash::Hash;
use std::time::Instant;

/// Phase 1 workload: insert all keys, then find all keys.
///
/// Pseudocode:
///   table = HashTable::new(dataset.len() * 2)
///   start_insert = now()
///   for key in dataset.keys:
///       table.insert(key.clone())
///   insert_ns = elapsed(start_insert)
///
///   start_find = now()
///   for key in dataset.keys:
///       table.find(&key)   // result is intentionally discarded (not asserted)
///   find_ns = elapsed(start_find)
///
///   return WorkloadResult { insert_ns, find_ns, insert_count, find_count }
pub fn run<K, T>(table: &mut T, dataset: &Dataset<K>) -> WorkloadResult
where
    K: Hash + Eq + Clone,
    T: HashTable<K>,
{
    let keys = &dataset.keys;
    let n = keys.len();

    // Insert phase
    let t0 = Instant::now();
    for key in keys {
        table.insert(key.clone());
    }
    let insert_ns = t0.elapsed().as_nanos() as u64;

    // Find phase
    let t1 = Instant::now();
    for key in keys {
        let _ = table.find(key);
    }
    let find_ns = t1.elapsed().as_nanos() as u64;

    WorkloadResult {
        insert_ns,
        find_ns,
        insert_count: n,
        find_count: n,
    }
}
```

### 11.4 `src/workloads/mixed.rs` — Mixed Workload (80/20)

The Mixed workload interleaves finds and inserts at an 80% find / 20% insert ratio.

**Key detail**: The first half of keys are inserted up-front (warm-up). The second half
drives the mixed phase, using a seeded RNG to decide insert vs. find for each step.

```rust
use crate::datasets::Dataset;
use crate::trait_def::HashTable;
use crate::workloads::WorkloadResult;
use rand::{SeedableRng, Rng};
use rand::rngs::SmallRng;
use std::hash::Hash;
use std::time::Instant;

const SEED: u64 = 0xDEAD_BEEF_CAFE_1234;
const FIND_RATIO: f64 = 0.80;

/// Mixed workload: 80% find, 20% insert.
///
/// Pseudocode:
///   // Warm-up: insert first half
///   for key in keys[0..n/2]:
///       table.insert(key.clone())
///
///   rng = seeded_rng(SEED)
///   insert_ns = 0, find_ns = 0
///   insert_idx = n/2   // pointer into keys for next insert
///   find_idx = 0       // pointer into keys[0..insert_idx] for finds
///
///   for _ in 0..(n/2):
///       if rng.gen::<f64>() > FIND_RATIO AND insert_idx < n:
///           t = now(); table.insert(keys[insert_idx].clone()); insert_ns += elapsed(t)
///           insert_idx += 1
///       else:
///           find_key = keys[find_idx % insert_idx]
///           t = now(); table.find(&find_key); find_ns += elapsed(t)
///           find_idx += 1
pub fn run<K, T>(table: &mut T, dataset: &Dataset<K>) -> WorkloadResult
where
    K: Hash + Eq + Clone,
    T: HashTable<K>,
{
    let keys = &dataset.keys;
    let n = keys.len();
    let half = n / 2;

    // Warm-up phase (not timed)
    for key in &keys[..half] {
        table.insert(key.clone());
    }

    let mut rng = SmallRng::seed_from_u64(SEED);
    let mut insert_ns = 0u64;
    let mut find_ns = 0u64;
    let mut insert_count = 0usize;
    let mut find_count = 0usize;
    let mut insert_idx = half;
    let mut find_idx = 0usize;

    for _ in 0..half {
        if rng.gen::<f64>() > FIND_RATIO && insert_idx < n {
            let t = Instant::now();
            table.insert(keys[insert_idx].clone());
            insert_ns += t.elapsed().as_nanos() as u64;
            insert_count += 1;
            insert_idx += 1;
        } else {
            let key = &keys[find_idx % insert_idx.max(1)];
            let t = Instant::now();
            let _ = table.find(key);
            find_ns += t.elapsed().as_nanos() as u64;
            find_count += 1;
            find_idx += 1;
        }
    }

    WorkloadResult { insert_ns, find_ns, insert_count, find_count }
}
```

> **WARNING**: Per-operation timing (`Instant::now()` inside the loop) has overhead.
> For mixed workloads this is unavoidable because we interleave. Accept the overhead and
> note it in the output metadata. Do NOT switch to batch timing in mixed workloads, as that
> would require restructuring the operation sequence.

### 11.5 `src/workloads/read_heavy.rs` — Read-Heavy Workload (95/5)

Identical structure to `mixed.rs` with constants changed:

```rust
const FIND_RATIO: f64 = 0.95;
```

Rename the module and update the doc comment. Everything else is identical.

---

## 12. Benchmark Runner

### 12.1 Runner Overview (`src/runner/bench.rs`)

The runner is the core orchestration layer. It:
1. Accepts a dataset (already loaded)
2. Accepts a workload selector
3. Runs each enabled table implementation against the workload
4. Collects `BenchRecord`s
5. Returns them for CSV writing

```rust
// src/runner/bench.rs

use crate::datasets::Dataset;
use crate::metrics::record::BenchRecord;
use crate::workloads::WorkloadResult;
use crate::trait_def::HashTable;
use std::hash::Hash;

/// Configuration for a single benchmark run.
pub struct RunConfig {
    /// Initial capacity hint passed to HashTable::new()
    pub initial_capacity: usize,
    /// Number of times to repeat each workload (take minimum for stability)
    pub repetitions: usize,
}

impl Default for RunConfig {
    fn default() -> Self {
        RunConfig { initial_capacity: 1024, repetitions: 3 }
    }
}

/// Run a workload function on a table and return the BenchRecord.
///
/// `workload_fn` must match signature: `fn(&mut T, &Dataset<K>) -> WorkloadResult`
/// `table_name` is a static string label used in output CSV.
pub fn run_one<K, T, F>(
    config: &RunConfig,
    dataset: &Dataset<K>,
    workload_name: &str,
    table_name: &str,
    workload_fn: F,
) -> BenchRecord
where
    K: Hash + Eq + Clone,
    T: HashTable<K>,
    F: Fn(&mut T, &Dataset<K>) -> WorkloadResult,
{
    let mut best: Option<WorkloadResult> = None;

    for _ in 0..config.repetitions {
        let mut table = T::new(config.initial_capacity);
        let result = workload_fn(&mut table, dataset);
        best = Some(match best {
            None => result,
            Some(prev) => {
                // Take minimum of insert_ns and find_ns independently (best-case measurement)
                WorkloadResult {
                    insert_ns: prev.insert_ns.min(result.insert_ns),
                    find_ns: prev.find_ns.min(result.find_ns),
                    insert_count: result.insert_count,
                    find_count: result.find_count,
                }
            }
        });
    }

    let best = best.unwrap();
    // Load factor approximation: count / initial_capacity
    // A more accurate measure requires knowing the final table size.
    // For now, use dataset size / initial_capacity as a proxy.
    let load_factor = dataset.keys.len() as f64 / config.initial_capacity as f64;

    BenchRecord::from_result(
        &dataset.name,
        workload_name,
        table_name,
        load_factor,
        &best,
    )
}
```

### 12.2 Main Entry Point (`src/main.rs`)

```rust
// src/main.rs
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "slickbench", about = "Hash table benchmarking framework")]
struct Cli {
    /// Dataset to use: uniform | sequential | zipf | norvig | wikipedia
    #[arg(long, default_value = "uniform")]
    dataset: String,

    /// Workload to use: bulk | mixed | read_heavy
    #[arg(long, default_value = "bulk")]
    workload: String,

    /// Number of keys to generate/load
    #[arg(long, default_value_t = 1_000_000)]
    size: usize,

    /// Random seed for dataset generation
    #[arg(long, default_value_t = 42)]
    seed: u64,

    /// Output CSV file path
    #[arg(long, default_value = "results.csv")]
    output: String,

    /// Number of repetitions per (table, workload) pair
    #[arg(long, default_value_t = 3)]
    reps: usize,
}

fn main() {
    let cli = Cli::parse();
    // Dispatch based on cli.dataset and cli.workload
    // See runner module for details
    // This is filled out per phase — see §15
    println!("SlickBench: dataset={}, workload={}, size={}", cli.dataset, cli.workload, cli.size);
}
```

The `main.rs` dispatch logic is extended in each phase. See §15 for per-phase details.

---

## 13. Metrics and Output Format

### 13.1 CSV Schema

Every output row contains these columns (no extras, no missing):

| Column | Type | Description |
|---|---|---|
| `dataset` | String | Name of dataset (uniform, sequential, etc.) |
| `workload` | String | Name of workload (bulk, mixed, read_heavy) |
| `table` | String | Name of table (slick, linear, quadratic, cuckoo, std_set) |
| `load_factor` | f64 | Approximate load factor at end of workload |
| `insert_ns_per_op` | f64 | Average nanoseconds per insert |
| `find_ns_per_op` | f64 | Average nanoseconds per find |
| `insert_count` | usize | Total inserts performed |
| `find_count` | usize | Total finds performed |

### 13.2 Example Output

```csv
dataset,workload,table,load_factor,insert_ns_per_op,find_ns_per_op,insert_count,find_count
uniform,bulk,linear,0.977,45.3,22.1,1000000,1000000
uniform,bulk,slick,0.977,38.1,18.4,1000000,1000000
```

---

## 14. Python Orchestration (uv)

### 14.1 Requirements

- Use `uv` for Python environment management
- Python 3.11+
- Dependencies: `pandas`, `matplotlib`, `seaborn`, `subprocess`

### 14.2 `scripts/bench.py`

Responsibilities:
1. Build the Rust binary in release mode
2. Run it for each combination of (dataset × workload × size)
3. Collect all CSV output into a single combined file
4. Optionally invoke perf if available

```python
#!/usr/bin/env python3
"""
SlickBench orchestrator.
Usage: uv run scripts/bench.py [--datasets uniform sequential] [--sizes 100000 1000000]
"""

import subprocess
import argparse
import os
import sys
from pathlib import Path

DEFAULT_DATASETS = ["uniform"]
DEFAULT_WORKLOADS = ["bulk"]
DEFAULT_SIZES = [100_000, 1_000_000]
OUTPUT_DIR = Path("bench_results")

def build_release():
    print("[bench.py] Building release binary...")
    result = subprocess.run(
        ["cargo", "build", "--release"],
        check=True, capture_output=True, text=True
    )
    print(result.stdout)

def run_bench(dataset: str, workload: str, size: int, seed: int, output: str, reps: int):
    cmd = [
        "./target/release/slickbench",
        "--dataset", dataset,
        "--workload", workload,
        "--size", str(size),
        "--seed", str(seed),
        "--output", output,
        "--reps", str(reps),
    ]
    print(f"[bench.py] Running: {' '.join(cmd)}")
    subprocess.run(cmd, check=True)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--datasets", nargs="+", default=DEFAULT_DATASETS)
    parser.add_argument("--workloads", nargs="+", default=DEFAULT_WORKLOADS)
    parser.add_argument("--sizes", nargs="+", type=int, default=DEFAULT_SIZES)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--reps", type=int, default=3)
    parser.add_argument("--output", default="results.csv")
    args = parser.parse_args()

    OUTPUT_DIR.mkdir(exist_ok=True)
    build_release()

    for dataset in args.datasets:
        for workload in args.workloads:
            for size in args.sizes:
                run_bench(dataset, workload, size, args.seed, args.output, args.reps)

    print(f"[bench.py] Done. Results written to {args.output}")

if __name__ == "__main__":
    main()
```

### 14.3 `scripts/plot.py`

```python
#!/usr/bin/env python3
"""
Usage: uv run scripts/plot.py --input results.csv --output plots/
"""
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import argparse
from pathlib import Path

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", default="results.csv")
    parser.add_argument("--output", default="plots/")
    args = parser.parse_args()

    Path(args.output).mkdir(exist_ok=True)
    df = pd.read_csv(args.input)

    for workload in df["workload"].unique():
        for dataset in df["dataset"].unique():
            subset = df[(df["workload"] == workload) & (df["dataset"] == dataset)]
            if subset.empty:
                continue

            fig, axes = plt.subplots(1, 2, figsize=(14, 5))
            sns.barplot(data=subset, x="table", y="insert_ns_per_op", ax=axes[0])
            axes[0].set_title(f"{dataset}/{workload} — Insert ns/op")
            axes[0].set_ylabel("ns per operation")

            sns.barplot(data=subset, x="table", y="find_ns_per_op", ax=axes[1])
            axes[1].set_title(f"{dataset}/{workload} — Find ns/op")
            axes[1].set_ylabel("ns per operation")

            fname = f"{args.output}/{dataset}_{workload}.png"
            plt.tight_layout()
            plt.savefig(fname, dpi=150)
            plt.close()
            print(f"[plot.py] Saved {fname}")

if __name__ == "__main__":
    main()
```

### 14.4 `scripts/download_data.py`

```python
#!/usr/bin/env python3
"""Download data files needed for Norvig and Wikipedia datasets."""
import urllib.request
from pathlib import Path

Path("data").mkdir(exist_ok=True)

# Norvig word frequencies
url = "https://norvig.com/ngrams/count_1w.txt"
dest = "data/norvig_words.txt"
if not Path(dest).exists():
    print(f"Downloading {url}...")
    urllib.request.urlretrieve(url, dest)
    print(f"Saved to {dest}")
else:
    print(f"{dest} already exists, skipping.")

# Wikipedia titles — use a pre-processed small dump
# Replace this URL with a stable mirror if needed
wiki_url = "https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-all-titles-in-ns0.gz"
wiki_dest = "data/wiki_titles.txt"
if not Path(wiki_dest).exists():
    import gzip, shutil
    gz_dest = wiki_dest + ".gz"
    print(f"Downloading Wikipedia titles (this may be large)...")
    urllib.request.urlretrieve(wiki_url, gz_dest)
    with gzip.open(gz_dest, 'rb') as f_in:
        with open(wiki_dest, 'wb') as f_out:
            shutil.copyfileobj(f_in, f_out)
    Path(gz_dest).unlink()
    print(f"Saved to {wiki_dest}")
else:
    print(f"{wiki_dest} already exists, skipping.")
```

---

## 15. Phased Development

Each phase must:
- Compile and run without errors before starting the next phase
- Produce CSV output (even if only one table and one dataset)
- Not depend on code from future phases

### Phase 1 — Minimal Working System

**Goal**: Single table, single dataset, single workload, correct timing output.

**Files to implement**:
1. `Cargo.toml` — all dependencies
2. `src/hash_utils.rs` — `hash1`, `hash2` (§5.2)
3. `src/trait_def.rs` — `HashTable<K>` trait (§6.1)
4. `src/datasets/mod.rs` — `Dataset<K>` struct
5. `src/datasets/uniform.rs` — `generate(size, seed)` (§10.2)
6. `src/workloads/mod.rs` — `WorkloadResult` struct
7. `src/workloads/bulk.rs` — `run<K, T, F>()` (§11.3)
8. `src/implns/mod.rs` — only `pub mod linear;`
9. `src/implns/linear.rs` — `LinearTable<K>` (§9.1)
10. `src/metrics/record.rs` — `BenchRecord`, `write_csv` (§7.7)
11. `src/runner/bench.rs` — `run_one` (§12.1)
12. `src/main.rs` — hardcoded `dataset=uniform`, `workload=bulk`, `table=linear`

**Phase 1 `main.rs` body**:
```rust
fn main() {
    let config = slickbench::runner::bench::RunConfig::default();
    let dataset = slickbench::datasets::uniform::generate(100_000, 42);
    let record = slickbench::runner::bench::run_one::<u64, slickbench::implns::linear::LinearTable<u64>, _>(
        &config,
        &dataset,
        "bulk",
        "linear",
        slickbench::workloads::bulk::run,
    );
    slickbench::metrics::record::write_csv("results.csv", &[record]).unwrap();
    println!("Phase 1 complete. Check results.csv.");
}
```

**Phase 1 success condition**:
```
cargo run --release
# → results.csv exists with one row, no panics
```

---

### Phase 2 — Add Remaining Tables and Sequential Dataset

**Goal**: All 5 tables run on uniform and sequential datasets with bulk workload.

**New files**:
- `src/implns/quadratic.rs` (§9.2)
- `src/implns/cuckoo.rs` (§9.3)
- `src/implns/std_set.rs` (§9.4)
- `src/implns/slick.rs` (§8 — CRITICAL: read §8 fully first)
- `src/datasets/sequential.rs` (§10.3)

**Update `src/implns/mod.rs`** to uncomment all five modules.

**Update `src/main.rs`** to loop over all tables:
```rust
// Pseudocode for Phase 2 main loop:
for dataset in [uniform::generate(N, SEED), sequential::generate(N, SEED)] {
    for (table_name, run_fn) in [
        ("linear",    run_linear),
        ("quadratic", run_quadratic),
        ("cuckoo",    run_cuckoo),
        ("slick",     run_slick),
        ("std_set",   run_std_set),
    ] {
        let record = run_one(&config, &dataset, "bulk", table_name, run_fn);
        records.push(record);
    }
}
write_csv("results.csv", &records).unwrap();
```

Because each table has a different concrete type, you cannot store `run_fn` in a heterogeneous
Vec directly. Use a macro or write out each call explicitly. Do not use `Box<dyn Fn>` to avoid
dispatch overhead.

**Phase 2 success condition**:
```
cargo run --release
# → results.csv with 10 rows (5 tables × 2 datasets), no panics
```

---

### Phase 3 — Add Zipf, Norvig, Wikipedia Datasets

**Prerequisite**: `uv run scripts/download_data.py` has been run successfully.

**New files**:
- `src/datasets/zipf.rs` (§10.4)
- `src/datasets/norvig.rs` (§10.5)
- `src/datasets/wikipedia.rs` (§10.6)

**Note on key types**: Norvig and Wikipedia datasets return `Dataset<String>`, not `Dataset<u64>`.
This means you need separate `run_one` invocations typed over `String`. The trait `HashTable<K>`
is generic, so `LinearTable<String>`, `SlickTable<String>` etc. must also compile.

Verify that all `impl HashTable<K>` blocks use `K: Hash + Eq + Clone` and not `K = u64`.

**Phase 3 success condition**:
```
cargo run --release -- --dataset zipf --workload bulk --size 500000
cargo run --release -- --dataset norvig --workload bulk --size 50000
# → results.csv rows appear for each
```

---

### Phase 4 — Add Mixed Workload

**New files**:
- `src/workloads/mixed.rs` (§11.4)

**Update `src/main.rs`** to accept `--workload mixed`.

**Phase 4 success condition**:
```
cargo run --release -- --dataset uniform --workload mixed --size 200000
# → results.csv includes mixed workload rows
```

---

### Phase 5 — Add Read-Heavy Workload

**New files**:
- `src/workloads/read_heavy.rs` (§11.5)

**Update `src/main.rs`** to accept `--workload read_heavy`.

**Phase 5 success condition**:
```
cargo run --release -- --dataset uniform --workload read_heavy --size 200000
```

---

### Phase 6 — Python Orchestration and Plotting

**Files**:
- `scripts/bench.py` (§14.2)
- `scripts/plot.py` (§14.3)
- `scripts/download_data.py` (§14.4)

**Verify full pipeline**:
```bash
uv run scripts/bench.py \
    --datasets uniform sequential zipf norvig wikipedia \
    --workloads bulk mixed read_heavy \
    --sizes 100000 1000000

uv run scripts/plot.py --input results.csv --output plots/
```

Output: `plots/` directory with one PNG per (dataset, workload) combination.

---

## 16. Forbidden Behaviors

The following are **hard prohibitions**. Violating any of these invalidates the benchmark results
and constitutes a failure of the agent.

| # | Forbidden Behavior | Reason |
|---|---|---|
| F1 | Using `DefaultHasher` in any table | Not controlled; not reproducible |
| F2 | Using `HashMap` as Slick's backyard | Uses DefaultHasher internally |
| F3 | Using `std::collections::HashSet` without `ahash::RandomState` | Same reason |
| F4 | Rewriting Slick's algorithm | Invalidates research results |
| F5 | Simplifying Slick's control flow | Same reason |
| F6 | Copying code from outside `refs/slick_core.rs` | Introduces uncontrolled changes |
| F7 | Generating Slick from scratch (without reading refs/) | Will diverge from ground truth |
| F8 | Using `step_by` on dataset indices for sampling | Introduces bias (cache effects, patterns) |
| F9 | Separate binary per table | Prevents fair comparison; breaks runner |
| F10 | Deduplicating dataset keys | Alters distribution characteristics |
| F11 | Exposing Slick internal fields through the `HashTable` trait | Breaks interface fairness |
| F12 | Adding per-table hashing logic outside `hash_utils.rs` | Causes hash inconsistency |
| F13 | Importing `lib.rs` or any file from previous repo | Brings in legacy flaws |
| F14 | Using per-operation Instant::now() in Bulk workload | Overhead distorts measurement |
| F15 | Asserting find results in the benchmark loop | Creates branch misprediction bias |

---

## 17. Success Criteria

The benchmark is considered correctly implemented when all of the following are true:

1. **All tables use the same hash function**: `hash1`/`hash2` from `hash_utils.rs` with fixed seeds
2. **Slick's dual-hash is preserved**: `h1` and `h2` are both derived from AHasher with different seeds
3. **Single binary execution**: all tables run from one `cargo run` invocation
4. **Datasets are unbiased**: keys are shuffled with a seeded RNG; no step-by or sorted iteration
5. **Workloads are modular**: each workload can be run independently; failure in one does not crash others
6. **Results are reproducible**: same seed → same CSV output across runs on same hardware
7. **CSV is well-formed**: all 8 columns present, correct types, no NaN or Inf values
8. **Phase 1 passes independently**: `cargo run --release` after implementing only Phase 1 files
9. **No forbidden behaviors**: checklist in §16 passes fully

---
## Version Control Guidelines

After completing each phase successfully:

- Verify the phase builds and runs without errors
- Ensure results.csv is generated correctly
- Then create a Git commit

Example:

```bash
git add .
git commit -m "<short description>"
```

*End of AGENT.md*
