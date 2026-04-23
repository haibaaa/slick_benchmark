# SlickBench: Comparative Analysis of Hash Table Probing and Sliding Gap Layouts

## 1. Abstract
This report presents a comprehensive benchmarking study of diverse hash table implementations, ranging from traditional open-addressing schemes (Linear and Quadratic Probing) to more complex structures like Cuckoo Hashing and Slick Hashing. We evaluate these structures across various synthetic and real-world datasets, measuring both time performance (nanoseconds per operation) and space efficiency (bytes per element). Our findings highlight the significant performance of the standard library implementation while illustrating the trade-offs in memory overhead and probe locality for the specialized Slick Hash structure.

## 2. Introduction

### Problem Definition
Hash tables are fundamental data structures requiring a robust strategy for collision resolution. The choice of probing strategy or overflow management significantly impacts cache locality, average-case latency, and memory utilization.

### Motivation
The Slick Hash implementation introduces a "sliding-gap" layout intended to optimize for specific data distributions and range queries. Understanding how this layout compares to established techniques (like Cuckoo hashing or standard open addressing) in a general benchmarking context is critical for selecting the right tool for high-performance systems.

### Hypothesis
We hypothesize that while Slick Hashing may offer specialized benefits in controlled environments, traditional open-addressing schemes with high load factors will generally exhibit lower overhead per element, whereas the standard library `HashSet` will provide the best overall time performance due to mature optimizations.

### Related Work
- **Linear/Quadratic Probing**: Classical open-addressing techniques explored by Knuth (TAOCP Vol. 3).
- **Cuckoo Hashing**: Introduced by Pagh and Rodler, offering $O(1)$ worst-case lookup time through multi-choice placement.
- **Slick Hashing**: A modern sliding-gap approach that manages blocks and thresholds to handle saturation.

## 3. System Overview
SlickBench is a hybrid benchmarking framework comprised of a high-performance Rust core and a Python-based orchestration and visualization layer. The Rust core ensures monomorphized, low-overhead execution of the benchmark matrix, while the Python layer provides robust statistical analysis and plot generation.

## 4. Methodology

### Benchmark Design
Each benchmark run involves a fresh instantiation of a hash table with a specified initial capacity hint. Workloads are repeated multiple times, with the minimum execution time recorded to eliminate transient noise from the host operating system.

### Datasets
- **Uniform**: Randomly distributed `u64` keys.
- **Sequential**: Monotonically increasing `u64` keys, testing spatial locality.
- **Zipf**: Power-law distribution, simulating uneven access patterns.
- **Norvig**: Real-world string keys from the Norvig word list.
- **Wikipedia**: Real-world string titles from Wikipedia dumps.

### Workloads
- **Bulk**: Massive insertion phase followed by a lookup phase.
- **Mixed**: 20% insertion, 80% search ratio.
- **Read-Heavy**: 5% insertion, 95% search ratio.

### Metrics
- **Time**: Measured as average nanoseconds per operation (`ns/op`).
- **Space**: 
  - `capacity`: Total allocated slots.
  - `bytes_estimate`: Total estimated memory footprint including main and overflow tables.
  - `bytes_per_element`: Normalized memory cost per stored key.

## 5. Implementation Details

### Hash Table Interface
All implementations adhere to a unified `HashTable<K>` trait, ensuring that the benchmark remains generic over the underlying storage strategy.

### Linear and Quadratic Probing
Baseline implementations using open addressing. Linear probing advances one slot at a time, while quadratic probing increases the probe distance $(i^2)$ to mitigate primary clustering.

### Cuckoo Hashing
Utilizes two independent tables and hash functions. If a collision occurs in both primary slots, a "kick" sequence attempts to relocate existing keys, triggering a full rebuild after a fixed threshold of kicks.

### Slick Hashing
Slick Hashing employs a sliding-gap layout within fixed-size blocks. Each block has an associated threshold; keys with a lower hash priority are diverted to a "backyard" (overflow table). This design allows for local adjustments (sliding slots) before resorting to costlier global rehashes or overflow management.

### Engineering Decisions
- **Deterministic Hashing**: All tables use a fixed-seed `AHasher` to ensure fair comparisons regardless of the implementation's internal hashing choices.
- **Backyard Implementation**: In this suite, the Slick backyard uses an open-addressed overflow table to remain consistent with the project's memory tracking rules.

## 6. Experimental Setup
The benchmarks were executed on a Linux system using the Rust 1.75+ toolchain (release profile). Plots were generated using Matplotlib within a Nix-reproducible environment to ensure consistent rendering and dependency management. The dataset size was fixed at 200,000 for numeric keys and 50,000 for string keys to maintain manageable execution times while stressing the tables significantly.

## 7. Results

### Insert Performance
Standard `std_set` exhibits the lowest insertion latency across all workloads, typically ranging from 7ns to 15ns per operation. Linear and Quadratic probing follow, with Slick and Cuckoo hashing showing higher latency (70ns - 110ns) due to the complexity of gap-sliding and kick sequences, respectively.

### Lookup Performance
Lookup performance is highly competitive. Cuckoo hashing demonstrates its $O(1)$ lookup promise with values often below 15ns per op. Slick hashing's find performance is around 25ns-30ns, slightly slower than basic open addressing due to the dual lookup in the main block and the backyard.

### Workload Comparison
Under mixed and read-heavy workloads, the performance gap between implementations narrows as the overhead of complex insertions is amortized over frequent lookups.

### Space Efficiency
- **Linear/Quadratic**: Generally use around 20 bytes per element at high load factors $(0.75)$.
- **Slick**: Occupies a similar footprint (approx. 21 bytes per element) but shows higher block-level metadata overhead.
- **std_set**: Shows high variability in space efficiency due to its internal growth strategy, often achieving as low as 9 bytes per element in bulk scenarios.

## 8. Analysis and Discussion
The results confirm the time-space trade-off inherent in hash table design. While Slick hashing's sliding-gap layout aims to manage block saturation locally, the overhead of managing block metadata and the "backyard" overflow table results in slightly higher latency and memory usage compared to global open addressing in uniform distributions. However, Slick's behavior remains stable even as the main table approaches physical capacity, thanks to its threshold-dumping mechanism.

## 9. Limitations
Memory estimation is based on a static calculation of capacity and entry sizes, which may not capture the full runtime overhead of the allocator or internal bitmask metadata used by the standard library.

## 10. Conclusion
Standard library implementations remain the benchmark for general-purpose use, offering exceptional performance. Specialized structures like Slick Hashing provide interesting architectural alternatives, though their benefits may be more pronounced in specific scenarios (e.g., hardware-assisted search or range-query optimizations) not fully captured by this synthetic benchmark.

## 11. Future Work
- Integration of hardware performance counters (cache misses, branch mispredictions).
- Evaluation under extreme load factors (>90%).
- Testing with non-uniform distributions tailored to Slick's block-sliding strengths.

## 12. References
1. Pagh, R., & Rodler, F. F. (2001). Cuckoo Hashing.
2. Knuth, D. E. (1998). The Art of Computer Programming, Volume 3: Sorting and Searching.
3. Slick Hash reference implementation and architectural specification.
