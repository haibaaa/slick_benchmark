//! CSV row definitions and serialization helpers.

use serde::Serialize;

/// One row in the output CSV.
#[derive(Debug, Serialize)]
pub struct BenchRecord {
    pub dataset: String,
    pub workload: String,
    pub table: String,
    /// Load factor at end of workload (inserted_count / capacity).
    pub load_factor: f64,
    /// Average nanoseconds per insert operation.
    pub insert_ns_per_op: f64,
    /// Average nanoseconds per find operation.
    pub find_ns_per_op: f64,
    /// Total number of inserts performed.
    pub insert_count: usize,
    /// Total number of finds performed.
    pub find_count: usize,
    pub capacity: usize,
    pub elements: usize,
    pub bytes_estimate: usize,
    pub bytes_per_element: usize,
}

impl BenchRecord {
    pub fn from_result(
        dataset: &str,
        workload: &str,
        table: &str,
        load_factor: f64,
        result: &crate::workloads::WorkloadResult,
        capacity: usize,
        elements: usize,
        bytes_estimate: usize,
        bytes_per_element: usize,
    ) -> Self {
        BenchRecord {
            dataset: dataset.to_string(),
            workload: workload.to_string(),
            table: table.to_string(),
            load_factor,
            insert_ns_per_op: result.insert_ns as f64 / result.insert_count.max(1) as f64,
            find_ns_per_op: result.find_ns as f64 / result.find_count.max(1) as f64,
            insert_count: result.insert_count,
            find_count: result.find_count,
            capacity,
            elements,
            bytes_estimate,
            bytes_per_element,
        }
    }
}

/// Write records to a CSV file. Appends if file exists; creates if not.
pub fn write_csv(path: &str, records: &[BenchRecord]) -> Result<(), Box<dyn std::error::Error>> {
    let file_exists = std::path::Path::new(path).exists();
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(!file_exists)
        .from_writer(file);
    for record in records {
        wtr.serialize(record)?;
    }
    wtr.flush()?;
    Ok(())
}
