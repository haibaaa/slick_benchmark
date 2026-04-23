//! Public crate surface for the benchmarking framework.
//!
//! The library is organized by responsibility: datasets generate or load keys,
//! workloads define operation schedules, implementations provide hash tables,
//! and the runner converts workload results into CSV-ready records.

pub mod datasets;
pub mod hash_utils;
pub mod implns;
pub mod metrics;
pub mod runner;
pub mod trait_def;
pub mod workloads;
