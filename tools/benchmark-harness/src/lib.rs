//! Benchmark harness for comparing document extraction frameworks
//!
//! This crate provides infrastructure for benchmarking Kreuzberg against other
//! document extraction frameworks, measuring performance (throughput, memory, latency)
//! and quality (F1 scores, text accuracy).

pub mod adapter;
pub mod adapters;
pub mod aggregate;
pub mod config;
pub mod consolidate;
pub mod error;
pub mod fixture;
pub mod monitoring;
pub mod output;
pub mod pool_metrics;
pub mod profile_report;
pub mod profiling;
pub mod registry;
pub mod runner;
pub mod types;

pub use adapter::FrameworkAdapter;
pub use adapters::{NativeAdapter, NodeAdapter, PythonAdapter, RubyAdapter};
pub use aggregate::{
    ConsolidationMetadata, DurationPercentiles, FileTypeAggregation, FrameworkModeAggregation, NewConsolidatedResults,
    Percentiles, PerformancePercentiles, aggregate_new_format,
};
pub use config::{BenchmarkConfig, BenchmarkMode, ProfilingConfig, load_framework_sizes};
pub use consolidate::{
    ConsolidatedResults, CrossFrameworkComparison, FrameworkAggregation, FrameworkQuality, QualityAnalysis,
    aggregate_by_framework, analyze_quality, compare_frameworks, consolidate_runs, load_run_results,
    write_consolidated_json,
};
pub use error::{Error, Result};
pub use fixture::{Fixture, FixtureManager};
pub use monitoring::{ResourceMonitor, ResourceSample, ResourceStats};
pub use output::{write_by_extension_analysis, write_json};
pub use pool_metrics::{FilePoolMetrics, PoolMetricsReport};
pub use profile_report::{Hotspot, MemorySnapshot, ProfileReport};
pub use registry::AdapterRegistry;
pub use runner::BenchmarkRunner;
pub use types::{BenchmarkResult, DiskSizeInfo, FrameworkCapabilities, PdfMetadata};
