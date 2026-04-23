# Resources

## Rust
### Ownership and Borrowing
- The Rust Book, chapters 4 and 8: foundational ownership and collection semantics
- Rust By Example: concise examples for references, borrowing, and pattern matching

### Traits and Generics
- The Rust Book, chapter 10: traits, generic bounds, and shared abstractions
- Rust Reference: trait bounds and method resolution details for generic code

### Performance
- Rust Performance Book: profiling, allocation behavior, and data layout guidance
- Criterion documentation: useful background even though this project uses custom timing

## Python
### NumPy
- NumPy User Guide: array semantics and performance-oriented numerical workflows
- NumPy troubleshooting guide: useful for native-extension import issues

### pandas
- pandas Getting Started: DataFrame loading, filtering, grouping, and CSV workflows
- pandas API reference for `read_csv` and group-by style analysis

### Matplotlib
- Matplotlib gallery: practical plotting patterns and styling examples
- Matplotlib pyplot tutorial: direct reference for figure, axes, and bar charts

## Hash Tables
### Probing Strategies
- Knuth, *The Art of Computer Programming*, Volume 3: classic analysis of open addressing
- Research notes on clustering behavior in linear and quadratic probing

### Cuckoo Hashing
- Pagh and Rodler, *Cuckoo Hashing*: original paper describing two-choice placement and eviction
- Follow-up systems papers on insertion failures, rebuild costs, and practical variants

### Cache Behavior
- Engineering literature on cache-aware indexing and memory locality
- Papers comparing predictable probe sequences with pointer-heavy or relocation-heavy schemes

## Benchmarking Context
- Brendan Gregg's systems performance materials for general measurement discipline
- Profiling references for interpreting branch misses, cache misses, and throughput tradeoffs
