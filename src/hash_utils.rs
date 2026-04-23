//! Centralized hashing helpers shared by all table implementations.
//!
//! Using one module for `hash1` and `hash2` keeps benchmarking fair by ensuring
//! that each custom table observes the same seeded AHasher behavior.

use ahash::{AHasher, RandomState};
use std::hash::{BuildHasher, Hash, Hasher};

/// Fixed seed pair used project-wide.
/// Changing these values invalidates reproducibility across runs.
pub const SEED1: (u64, u64) = (0x243F_6A88_85A3_08D3, 0x1319_8A2E_0370_7344);
pub const SEED2: (u64, u64) = (0xA409_3822_299F_31D0, 0x082E_FA98_EC4E_6C89);

/// Creates a seeded AHasher using a (u64, u64) seed pair.
#[inline]
pub fn make_hasher(seed: (u64, u64)) -> AHasher {
    RandomState::with_seeds(seed.0, seed.1, 0, 0).build_hasher()
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
