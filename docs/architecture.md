# Architecture

## Overview
SlickBench is split into a small Rust benchmarking core and a Python orchestration layer.
The Rust binary is responsible for generating datasets, executing workloads against each hash
table implementation, and appending benchmark rows to `results.csv`. The Python layer automates
matrix execution and produces plots from the accumulated CSV output.

Nix support is optional. The project can be built and executed without Nix. The Nix shell is
provided only to make the Python scientific stack and native runtime dependencies reproducible.

## Directory Tree
```text
.
├── Cargo.toml
├── flake.nix
├── refs/
│   └── slick_core.rs
├── scripts/
│   ├── bench.py
│   └── download_data.py
├── src/
│   ├── datasets/
│   ├── implns/
│   ├── metrics/
│   ├── runner/
│   ├── workloads/
│   ├── hash_utils.rs
│   ├── lib.rs
│   ├── main.rs
│   └── trait_def.rs
├── data/
│   ├── norvig_words.txt
│   └── wiki_titles.txt
└── docs/
    ├── architecture.md
    ├── hash_schemes.md
    ├── report.md
    ├── resources.md
    └── setup.md
```

## Rust Side
### Hash Tables
The `src/implns/` directory contains five evaluated implementations:
- `linear.rs`: open addressing with linear probing
- `quadratic.rs`: open addressing with quadratic probing
- `cuckoo.rs`: two-table cuckoo hashing
- `slick.rs`: adapted Slick hash implementation
- `std_set.rs`: `std::collections::HashSet` baseline with a fixed ahash seed

All custom tables implement the shared `HashTable<K>` trait from `src/trait_def.rs`.

### Hashing
`src/hash_utils.rs` centralizes deterministic hashing. This keeps the benchmark fair by
ensuring that all custom tables use the same seeded AHasher helpers:
- `hash1` for primary indexing
- `hash2` for secondary indexing where required

### Datasets
`src/datasets/` provides numeric and string-key datasets:
- `uniform`, `sequential`, `zipf` for `u64`
- `norvig`, `wikipedia` for `String`

Each dataset function returns a `Dataset<K>` value and preserves duplicates. All datasets are
shuffled with a seeded `SmallRng` before execution.

### Workloads
`src/workloads/` defines the operation schedules:
- `bulk`: all inserts, then all finds
- `mixed`: 80% finds, 20% inserts
- `read_heavy`: 95% finds, 5% inserts

The mixed workloads perform an untimed warm-up insert of the first half of the dataset and then
time per-operation work across the second half.

### Runner and Metrics
`src/runner/bench.rs` owns the repetition logic. It constructs a fresh table for each repetition,
runs the selected workload, and records the best insert and find timings independently.

`src/metrics/record.rs` converts the workload result into CSV rows with:
- dataset name
- workload name
- table name
- load factor
- insert and find cost per operation
- insert and find counts

## Python Side
### `scripts/bench.py`
This script:
1. checks whether external datasets are available
2. builds the release binary once
3. runs a deterministic matrix of datasets and workloads
4. optionally wraps each run with `perf stat`
5. loads `results.csv` with pandas
6. renders workload-specific PNG plots with matplotlib

### `scripts/download_data.py`
This script downloads the Norvig word list and Wikipedia title dump into `data/`.
The Rust code never performs network downloads during benchmarking.

## Nix Environment
`flake.nix` defines an optional development shell with:
- Python
- pandas, numpy, matplotlib
- uv
- gcc
- git

This environment is useful when native Python wheels require runtime libraries such as
`libstdc++`. It is not required for building or running the Rust benchmark binary.

## Data Flow
```text
dataset generator / loader
    -> workload scheduler
    -> table implementation
    -> runner aggregation
    -> BenchRecord rows
    -> results.csv
    -> scripts/bench.py
    -> plots/*.png
```

The important boundary is that Rust produces the benchmark records, while Python consumes those
records for repeated orchestration and reporting.
