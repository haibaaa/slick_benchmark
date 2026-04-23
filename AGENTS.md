You are working on a stable benchmarking project.

Before implementing any feature, you MUST first study the project.

--------------------------------
PHASE 0 — PROJECT STUDY (MANDATORY)
--------------------------------

Read and understand:

1. docs/
   - architecture.md
   - setup.md

2. Source code:
   - src/implns/ (all hash tables)
   - src/datasets/
   - src/workloads/
   - src/main.rs
   - src/runner/
   - src/metrics/

3. Code comments:
   - Understand design decisions
   - Identify where metrics are collected

4. results.csv format:
   - Identify existing columns
   - Do NOT break compatibility

You MUST:
- understand how data flows through the system
- identify where to integrate new metrics safely

--------------------------------
GOAL
--------------------------------

Add space usage metrics WITHOUT affecting existing behavior.

--------------------------------
STEP 0 — CREATE BRANCH
--------------------------------

git checkout -b space-analysis

--------------------------------
STEP 1 — EXTEND TRAIT
--------------------------------

Locate HashTable trait.

Add:

fn capacity(&self) -> usize;
fn len(&self) -> usize;
fn extra_space(&self) -> usize;

Rules:
- Do NOT modify existing methods
- Do NOT break compilation

--------------------------------
STEP 2 — IMPLEMENT FOR ALL TABLES
--------------------------------

Using actual implementation details:

Linear / Quadratic:
- capacity = table size
- len = number of elements
- extra_space = 0

Cuckoo:
- capacity = combined size of both tables
- len = number of elements
- extra_space = 0

Slick:
- capacity = main table size
- len = number of elements
- extra_space = backyard size (based on implementation)

--------------------------------
STEP 3 — MEMORY ESTIMATION
--------------------------------

Use:

std::mem::size_of::<T>()

Compute:

bytes_estimate =
  capacity * size_of::<Entry>()
  + extra_space * size_of::<Key>()

bytes_per_element =
  bytes_estimate / len

--------------------------------
STEP 4 — INTEGRATE INTO METRICS
--------------------------------

Locate where results are written.

Add new columns:

capacity
elements
bytes_estimate
bytes_per_element

Rules:
- Preserve existing CSV schema
- Append new columns only

--------------------------------
STEP 5 — VERIFY
--------------------------------

Run:

cargo run --release -- --dataset uniform --workload bulk --size 200000

Check:
- program runs
- CSV updated
- values are reasonable

--------------------------------
CONSTRAINTS
--------------------------------

- DO NOT modify:
  - hashing logic
  - Slick algorithm
  - workloads
  - datasets

- DO NOT introduce new dependencies

--------------------------------
STEP 6 — COMMIT
--------------------------------

git add .
git commit -m "add space usage metrics to benchmark framework"

--------------------------------
STOP
--------------------------------

Stop after successful implementation.