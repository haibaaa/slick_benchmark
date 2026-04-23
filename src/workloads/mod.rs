/// Result of running one workload on one table instance.
pub struct WorkloadResult {
    /// Total nanoseconds spent on insert operations.
    pub insert_ns: u64,
    /// Total nanoseconds spent on find operations.
    pub find_ns: u64,
    /// Number of insert operations performed.
    pub insert_count: usize,
    /// Number of find operations performed.
    pub find_count: usize,
}

pub mod bulk;
pub mod mixed;
pub mod read_heavy;
