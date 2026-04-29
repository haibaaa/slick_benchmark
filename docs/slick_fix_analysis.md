# Slick Hash Performance Fix - Analysis

## Overview
Documentation of changes made to fix Slick hash table performance in the benchmark framework.

## The Problem

### Symptoms
Slick performed poorly at ALL load factors:
- 41-59% slower inserts than linear probing
- 41-50% slower finds than linear probing (at low scales)

### Root Cause
The benchmark hardcoded `initial_capacity=1024` in `src/main.rs`:
```rust
// BEFORE (bug)
let config = slickbench::runner::bench::RunConfig {
    initial_capacity: 1024,  // ← HARDCODED!
    repetitions: reps,
};
```

Slick's `with_capacity()` calculated main table size from this value:
```rust
// src/implns/slick.rs:158-166
fn with_capacity(capacity: usize) -> Self {
    let block_size: usize = 10;
    let main_table_size = ((capacity + 10 - 1) / 10) * 10;
    // With capacity=1024: main_table_size = 1030 (FIXED!)
}
```

**Result:** Slick's main table was ALWAYS 1030 slots, even with 100M elements!

### Impact
With 10M elements and 1030 main table slots:
- Elements in main table: ~1030 (max)
- **Elements in backyard: ~9,998,970 (99.99%!)**
- Backyard = simple linear probing table
- Slick became "slow linear probing with extra overhead"

**Evidence from `results_var.csv` (before fix):**
```
uniform,bulk,slick,9765.625,136.28,60.59,10000000,10000000,1030,...
```
- `capacity=1030` (main table)
- `elements=10,000,000`
- **99.99% of elements in backyard (linear probing)**

---

## The Fix

### Changes Made

#### 1. Fix Initial Capacity (src/main.rs)
```rust
// AFTER (fixed)
fn run_workload<K>(dataset: &Dataset<K>, reps: usize, workload: &str) -> Vec<BenchRecord>
where
    K: Hash + Eq + Clone + Default,
{
    let config = slickbench::runner::bench::RunConfig {
        initial_capacity: dataset.keys.len(),  // ← Use dataset size
        repetitions: reps,
    };
    // ...
}
```

#### 2. Fix Load Factor Calculation (src/runner/bench.rs)
```rust
// AFTER (fixed)
let load_factor = if metrics.0 > 0 {
    metrics.1 as f64 / metrics.0 as f64  // elements / actual_capacity
} else {
    0.0
};
```

#### 3. Fix Plotting (scripts/var.py)
- Added `--fresh` flag to delete old CSV
- Sort data by `total_ops` before plotting
- Set explicit x-ticks for all step values
- Remove duplicate rows from CSV

---

## Performance Results

### Slick is NOW Better For: Find Operations ✅

**At 10M total operations (load factor 1.0):**

| Table | Insert ns/op | Find ns/op | Improvement |
|-------|---------------|------------|-------------|
| **Slick (fixed)** | 133.5 ns | **23.1 ns** | **48% faster than Linear** |
| Linear | **119.5 ns** | 44.6 ns | - |
| Quadratic | 120.3 ns | 45.0 ns | - |
| Cuckoo | 181.4 ns | 56.5 ns | - |
| std_set | 46.4 ns | 30.2 ns | - |

### Why Slick Finds Are Faster

1. **Block-local caching:** Elements stored in 10-element blocks
2. **Better cache behavior:** Probe sequence stays within one block
3. **Threshold-based filtering:** Elements with low hash2 go to backyard early

**Code: `src/implns/slick.rs:396-409`**
```rust
fn get(&self, key: &K) -> Option<&()> {
    let block_index = self.hash_block_index(key);
    if self.hash_threshold(key) < self.meta_data[block_index].threshold {
        return self.backyard.get(key);  // Filtered early
    }
    // Scan only within block range (10-20 elements)
    let block_range = self.block_range(block_index);
    // ...
}
```

### Slick is Worse For: Insert Operations ❌

**Same 10M operations benchmark:**

| Table | Insert ns/op | vs Linear |
|-------|---------------|----------|
| **Slick** | 133.5 ns | **12% slower** |
| Linear | **119.5 ns** | Baseline |

### Why Slick Inserts Are Slower

1. **Double hashing overhead:**
   - `hash1()` for block index
   - `hash2()` for threshold check
   - Linear only computes `hash1()`

2. **Block metadata overhead:**
   ```rust
   // Every insert checks block metadata
   if self.hash_threshold(&key) < self.meta_data[block_index].threshold {
       return self.insert_into_backyard(key);
   }
   ```

3. **Sliding gap complexity:** When blocks are full, Slick slides adjacent blocks

---

## Workload Recommendations

### Use Slick When:
- ✅ **Read-heavy workloads** (95%+ finds)
- ✅ **Cache efficiency matters** (block-local access)
- ✅ **Load factor ~1.0** (balanced between space and speed)

### Avoid Slick When:
- ❌ **Write-heavy workloads** (many inserts)
- ❌ **Latency-sensitive inserts** (double hashing overhead)
- ❌ **Simple use cases** (linear probing is simpler and faster for inserts)

---

## Conclusion

The fix correctly sizes Slick's main table, allowing it to achieve the paper's claimed performance:
> "Slick Hash aims to provide an efficient balance between space consumption and speed" (paper-source/README.md)

**Key takeaway:** Slick trades insert speed for find speed through block-based layout and double hashing. This is now visible in the benchmarks after the fix.

### Performance Summary

| Metric | Better than Linear? | Better than Cuckoo? |
|--------|---------------------|---------------------|
| **Find latency** | ✅ Yes (~48% faster at 10M ops) | ✅ Yes (~59% faster) |
| **Insert latency** | ❌ No (~12% slower) | ✅ Yes (~26% faster) |
| **Space efficiency** | Similar (~16 bytes/element) | ✅ Yes (2x better than 26) |

**Plot location:** `plots/var_uniform_bulk.png`
