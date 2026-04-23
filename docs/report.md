# Report

## Scope
This summary is based on local benchmark results currently stored in `results.csv`. The observations
below focus on the implemented datasets:
- `uniform`
- `sequential`
- `zipf`
- `norvig`

and the implemented workloads:
- `bulk`
- `mixed`
- `read_heavy`

The compared tables are:
- `linear`
- `quadratic`
- `cuckoo`
- `slick`
- `std_set`

## High-Level Findings
### 1. `std_set` is consistently the fastest baseline
Across the recorded runs, the standard-library hash set with a fixed ahash seed produced the lowest
or near-lowest insert and find timings on every dataset. This is not surprising: it is a mature,
heavily optimized implementation with low constant overhead.

### 2. Cuckoo hashing offers strong lookups but expensive inserts
The cuckoo table usually shows:
- relatively fast finds, especially on `uniform`, `sequential`, and `read_heavy`
- noticeably higher insert cost than the probing-based alternatives

This fits the expected behavior: lookups examine at most two candidate locations, but insertions may
trigger relocation chains or complete rebuilds.

### 3. Slick sits between probing baselines and cuckoo hashing
Slick tends to:
- avoid the worst insert cost seen in cuckoo hashing
- deliver lookup cost that is often better than naive probing on difficult workloads
- remain slower than `std_set`

Its behavior suggests that the block-local layout and overflow strategy do help moderate collision
pressure, but the metadata and overflow path still impose overhead.

## Dataset-Specific Observations
### Uniform
Uniform random keys are the most collision-stressful synthetic baseline. On `bulk`, linear,
quadratic, and Slick all cluster in a similar insert range, while cuckoo pays a higher insertion
cost. On lookup-heavy workloads, cuckoo improves relative to the probing baselines because each
successful lookup checks only a small number of positions.

### Sequential
Sequential keys remain shuffled before use, but their source distribution is still structurally
simpler than a fully random one. Results are broadly similar to `uniform`, though some tables show
slightly better locality or reduced variance. Slick performs somewhat better on `read_heavy`
sequential data than on the corresponding uniform case.

### Zipf
Zipf is the most favorable dataset in the current measurements for all custom tables. Insert and
find costs drop substantially relative to `uniform` and `sequential`, especially for linear and
quadratic probing. The skewed key frequency appears to concentrate accesses into a smaller working
set, which improves effective cache behavior and reduces the practical cost of probing.

### Norvig
Norvig is the heaviest dataset in absolute cost because it uses `String` keys rather than `u64`.
Every table pays extra hashing and equality-check cost. The relative ordering is still familiar:
`std_set` leads, cuckoo inserts are expensive, and Slick sits between cuckoo and the simpler
probing baselines.

## Workload-Specific Observations
### Bulk
Bulk emphasizes pure insertion followed by pure lookup. This makes insertion strategy differences
very visible:
- cuckoo inserts are most expensive
- linear and quadratic probing are competitive
- Slick is often comparable on inserts but usually slower on lookups than the best baselines

### Mixed
Mixed workloads reduce the apparent penalty of expensive insertion paths because only about 20% of
timed operations are inserts. The results tighten accordingly. Cuckoo hashing becomes more
competitive here because its inexpensive finds carry more weight in the aggregate.

### Read-Heavy
Read-heavy workloads amplify lookup efficiency. Cuckoo hashing benefits noticeably on numeric
datasets, while `std_set` remains strongest overall. Slick performs better here than in `bulk`,
which is consistent with a design that pays some insertion overhead to keep later lookups moderate.

## Table Comparisons
### Linear vs Quadratic
The two probing schemes are close across most workloads. Quadratic probing sometimes reduces lookup
cost modestly, but neither scheme dominates universally in the current dataset set.

### Cuckoo
The main tradeoff is clear:
- good to very good lookup cost
- expensive inserts

This is most visible in `bulk`, where insert-heavy behavior dominates the final per-operation
average.

### Slick
Slick appears to trade some implementation complexity for more controlled collision handling:
- better insert cost than cuckoo in many cases
- lookup cost usually worse than `std_set`
- lookup cost sometimes competitive with linear and quadratic probing under skewed data

The likely explanation is that the block metadata and backyard path help contain difficult keys, but
the additional control logic and overflow checks add constant overhead.

### `std_set`
The standard baseline consistently performs best or near-best in the local runs. It should not be
treated as a controlled algorithmic reference in the same way as the custom tables, but it remains
an important practical comparison point.

## Space Efficiency
The addition of memory metrics provides a clearer picture of implementation trade-offs:

- **Probing Tables (Linear/Quadratic)**: Maintain a steady memory footprint relative to their capacity. At a 0.75 load factor, they are among the most space-efficient custom implementations.
- **Cuckoo Hashing**: While lookup-efficient, Cuckoo hashing requires dual tables and can trigger aggressive rehashes when relocation chains fail, occasionally leading to lower physical memory utilization.
- **Slick Hashing**: Shows the highest raw capacity usage because it maintains both a partitioned main table and an overflow "backyard." However, it selectively ejects only high-collision keys, attempting to keep the primary blocks dense and cache-local.
- **std_set**: Often the most space-efficient due to mature internal bucket management, though its exact memory footprint can be harder to predict than fixed-capacity structures.

## Time-Space Tradeoffs
- **Cuckoo Hashing** trades insertion time (due to kicks) for nearly optimal lookup time.
- **Slick Hashing** trades raw memory (main table + backyard) and insertion complexity to maintain probe locality even under high load.
- **Linear/Quadratic Probing** represent the simplest tradeoff: low overhead and fast inserts at the cost of potential find degradation under heavy clustering.

## Key Insights
### Insert vs Find Tradeoffs
- Cuckoo hashing shifts cost toward insertion and away from lookup.
- Linear and quadratic probing keep inserts simple but can pay more under collision pressure.
- Slick attempts to balance the two with block metadata and controlled overflow.

### Effect of Zipf Distribution
Zipf improves results for nearly every implementation. The skewed reuse pattern likely increases the
probability that active keys stay hot in cache and reduces the average practical cost of lookups and
reinserts.

### Why Slick Behaves This Way
Slick maintains a structured main table and moves lower-priority keys into an overflow area when
block pressure rises. That reduces the need for unbounded probe growth inside a block, but it also
adds metadata maintenance and occasional overflow accesses.

### Why Cuckoo Inserts Are Expensive
Cuckoo insertion may evict a chain of existing keys before finding a stable placement. When that
fails, the table rebuilds at a larger size. Those relocation costs are precisely why its bulk insert
times are consistently above the probing alternatives.
