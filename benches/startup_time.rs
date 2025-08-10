//! Startup time benchmarks for MCP Helper
//!
//! These benchmarks measure the time it takes for the CLI to start up
//! and display help information, which is a critical performance metric.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::process::Command;
use std::time::Instant;

/// Benchmark the startup time for displaying version information
fn bench_startup_version(c: &mut Criterion) {
    c.bench_function("startup_version", |b| {
        b.iter(|| {
            let output = Command::new("./target/release/mcp")
                .arg("--version")
                .output()
                .expect("Failed to execute command");

            black_box(output);
        });
    });
}

/// Benchmark the startup time for displaying help information
fn bench_startup_help(c: &mut Criterion) {
    c.bench_function("startup_help", |b| {
        b.iter(|| {
            let output = Command::new("./target/release/mcp")
                .arg("--help")
                .output()
                .expect("Failed to execute command");

            black_box(output);
        });
    });
}

/// Benchmark the startup time for subcommand help
fn bench_startup_subcommand_help(c: &mut Criterion) {
    c.bench_function("startup_run_help", |b| {
        b.iter(|| {
            let output = Command::new("./target/release/mcp")
                .args(["help", "run"])
                .output()
                .expect("Failed to execute command");

            black_box(output);
        });
    });
}

/// Benchmark the raw process spawn time
fn bench_raw_spawn_time(c: &mut Criterion) {
    c.bench_function("raw_spawn_time", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();

            for _ in 0..iters {
                let child = Command::new("./target/release/mcp")
                    .arg("--version")
                    .spawn()
                    .expect("Failed to spawn command");

                // Don't wait for completion, just measure spawn time
                std::mem::drop(child);
            }

            start.elapsed()
        });
    });
}

/// Benchmark group for startup performance
fn startup_benches(c: &mut Criterion) {
    bench_startup_version(c);
    bench_startup_help(c);
    bench_startup_subcommand_help(c);
    bench_raw_spawn_time(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(20)  // Reduced sample size for process spawning
        .warm_up_time(std::time::Duration::from_secs(1));
    targets = startup_benches
}

criterion_main!(benches);
