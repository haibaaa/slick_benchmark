# Hash Schemes

This document summarizes the implemented table designs in pseudocode. The goal is conceptual
clarity rather than line-by-line translation from Rust.

## Linear Probing
### Idea
Store every key in one array. On collision, advance one slot at a time until an empty slot or the
target key is found.

**Space Characteristics:** Extremely high memory locality and minimal overhead (1 metadata bit/slot). Efficiency is high until the table exceeds 70-80% capacity.

### Pseudocode
```python
def insert(key):
    if load_factor_exceeds_limit():
        grow_and_rehash()

    start = hash1(key) % capacity
    i = start
    while True:
        if slots[i] is empty:
            slots[i] = key
            return
        if slots[i] == key:
            return
        i = (i + 1) % capacity
```

```python
def find(key):
    start = hash1(key) % capacity
    i = start
    while True:
        if slots[i] is empty:
            return False
        if slots[i] == key:
            return True
        i = (i + 1) % capacity
        if i == start:
            return False
```

## Quadratic Probing
### Idea
Use one array as in linear probing, but widen probe distance quadratically to reduce primary
clustering.

**Space Characteristics:** Identical to linear probing in raw slot usage, but improves search efficiency by spreading keys more evenly across the allocated memory.

### Pseudocode
```python
def insert(key):
    if load_factor_exceeds_limit():
        grow_and_rehash()

    start = hash1(key) % capacity
    step = 1
    i = start
    while True:
        if slots[i] is empty:
            slots[i] = key
            return
        if slots[i] == key:
            return
        i = (start + step * step) % capacity
        step += 1
```

```python
def find(key):
    start = hash1(key) % capacity
    step = 1
    i = start
    while True:
        if slots[i] is empty:
            return False
        if slots[i] == key:
            return True
        i = (start + step * step) % capacity
        step += 1
        if step > capacity:
            return False
```

## Cuckoo Hashing
### Idea
Maintain two tables and two hash functions. A key may live in one location in table 1 or one
location in table 2. Insertions may evict existing keys and trigger rebuilds.

**Space Characteristics:** Requires two distinct tables (effectively $2 \times$ minimum size). Memory efficiency is lower than probing during growth phases, but lookups are strictly $O(1)$.

### Pseudocode
```python
def try_insert(key):
    for _ in range(MAX_KICKS):
        i1 = hash1(key) % capacity
        if table1[i1] is empty:
            table1[i1] = key
            return True
        key, table1[i1] = table1[i1], key

        i2 = hash2(key) % capacity
        if table2[i2] is empty:
            table2[i2] = key
            return True
        key, table2[i2] = table2[i2], key

    return False
```

```python
def insert(key):
    if find(key):
        return

    while not try_insert(key):
        grow_tables()
        reinsert_every_existing_key()
```

```python
def find(key):
    return (
        table1[hash1(key) % capacity] == key
        or table2[hash2(key) % capacity] == key
    )
```

## Slick Hashing
### Idea
Slick partitions the main table into blocks. Each block tracks metadata for offset, gap size, and
threshold. Inserts attempt to stay within the block, slide nearby gaps when possible, and move
lower-priority keys to an overflow structure when a block becomes too constrained.

**Space Characteristics:** Significant metadata overhead per block. Further requires a "backyard" overflow table, leading to higher overall `bytes_per_element` compared to simple probing.

### Pseudocode
```python
def insert(key):
    block = hash1(key) mapped into block_index

    if threshold_hash(key) < block.threshold:
        backyard_insert(key)
        return

    if key already exists in block_range(block):
        return

    if block_has_no_immediately_usable_space(block):
        t_prime = 1 + minimum_threshold_hash_among_block_keys_and_new_key()
        block.threshold = t_prime

        for each key_in_block:
            if threshold_hash(key_in_block) < t_prime:
                backyard_insert(key_in_block)
                remove_key_from_block(key_in_block)

        if threshold_hash(key) < t_prime:
            backyard_insert(key)
            return

    append_key_to_end_of_block(key)
    shrink_block_gap(block)
```

```python
def find(key):
    block = hash1(key) mapped into block_index

    if threshold_hash(key) < block.threshold:
        return backyard_contains(key)

    return key in block_range(block)
```

### Key Differences
- Linear probing uses the simplest contiguous probe sequence.
- Quadratic probing trades simpler locality for reduced clustering.
- Cuckoo hashing makes lookups very cheap but can make inserts expensive due to relocations and rebuilds.
- Slick hashing introduces block metadata and an overflow path, aiming to preserve locality in the main table while selectively ejecting difficult keys.
