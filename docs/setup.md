# Setup

This project can be run without Nix. Nix is optional and is recommended only when a reproducible
Python environment is important, especially for plotting dependencies.

## A. Standard Setup
This is the recommended path for most users.

### Prerequisites
- Rust toolchain with `cargo`
- Python 3.10 or newer
- Python packages:
  - `numpy`
  - `pandas`
  - `matplotlib`

### Steps
```bash
cargo build --release
python scripts/bench.py
```

If you prefer `uv`, the same project can also be run with:
```bash
uv run scripts/bench.py
```

## B. uv-Based Setup
Use this path if you prefer `uv` for Python dependency management.

### Prerequisites
- Rust toolchain with `cargo`
- `uv`

### Steps
```bash
cargo build --release
uv run scripts/bench.py
```

The inline script metadata in `scripts/bench.py` will request the plotting dependencies when
needed.

## C. Nix-Based Setup
Use this path when you want a reproducible shell for Python plotting dependencies and native
runtime compatibility.

### Prerequisites
- Nix with flakes enabled

### Steps
```bash
nix develop
uv run scripts/bench.py
```

The Nix shell is optional. The project does not require Nix for Rust compilation or for the core
benchmark logic.

## External Datasets
The string-key benchmarks require the following local files:
- `data/norvig_words.txt`
- `data/wiki_titles.txt`

If they are missing, run:
```bash
uv run scripts/download_data.py
```

The benchmark binary does not download data at runtime.

## Understanding `results.csv`
Each row records one `(dataset, workload, table)` benchmark outcome:
- `dataset`: input key distribution
- `workload`: operation schedule
- `table`: evaluated implementation
- `load_factor`: approximate occupancy relative to the initial capacity hint
- `insert_ns_per_op`: average insert cost in nanoseconds
- `find_ns_per_op`: average find cost in nanoseconds
- `find_count`: number of finds executed by the workload

### Understanding Space Metrics
Newer columns in `results.csv` provide insight into memory efficiency:
- `capacity`: The raw slot count allocated.
- `elements`: Number of unique keys stored.
- `bytes_estimate`: The calculated total bytes occupied by the structure (main tables + overflow areas).
- `bytes_per_element`: The average byte cost per key, useful for comparing space-time tradeoffs across implementations.

Lower timing values and lower `bytes_per_element` values indicate better performance and efficiency.

## Reading the Plots
The plotting pipeline generates one PNG per workload in `plots/`.
Each plot contains:
- insert cost by dataset
- find cost by dataset
- one bar series per table

Workload-specific space efficiency plots are also generated as `plots/space_{workload}.png`, visualizing the `bytes_per_element` metric across datasets.

Use these plots to compare tradeoffs between tables under the same workload rather than comparing
across unrelated workloads.

## Common Issues
### Missing Datasets
Symptom:
- the benchmark panics when loading Norvig or Wikipedia data

Resolution:
```bash
uv run scripts/download_data.py
```

### Python Dependency Errors
Symptom:
- `ModuleNotFoundError` for pandas or matplotlib

Resolution:
- install the missing packages into your Python environment
- or run through `uv`
- or use `nix develop` for a fully provisioned shell

### Native Runtime or Path Issues
Symptom:
- Python scientific packages fail to import due to native library problems

Resolution:
- prefer the Nix shell if your host environment does not expose the required runtime libraries
- otherwise ensure your Python installation and package manager are consistent

### Command Not Found
Symptom:
- `cargo`, `python`, `uv`, or `nix` is not available

Resolution:
- install the missing tool and restart the shell
- verify `PATH` contains the selected toolchain
