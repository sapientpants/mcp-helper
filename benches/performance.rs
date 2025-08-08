//! Comprehensive performance benchmarks for MCP Helper
//!
//! This suite measures various performance aspects including:
//! - Startup time
//! - Memory usage
//! - Command parsing speed
//! - Configuration loading

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mcp_helper::client::ClientRegistry;
use mcp_helper::config::ConfigManager;
use mcp_helper::deps::DependencyChecker;
use mcp_helper::server::{detect_server_type, parse_npm_package};
use std::hint::black_box as hint_black_box;

/// Benchmark server type detection for different input types
fn bench_server_type_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("server_type_detection");

    let test_cases = vec![
        ("npm_simple", "cowsay"),
        ("npm_scoped", "@modelcontextprotocol/server-filesystem"),
        ("npm_version", "@anthropic/server@1.2.3"),
        ("docker", "docker:nginx:alpine"),
        (
            "binary_url",
            "https://github.com/owner/repo/releases/binary",
        ),
        ("python", "server.py"),
        ("local_path", "./local/server"),
    ];

    for (name, input) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &input, |b, &input| {
            b.iter(|| {
                let server_type = detect_server_type(black_box(input));
                hint_black_box(server_type);
            });
        });
    }

    group.finish();
}

/// Benchmark dependency checking performance
fn bench_dependency_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("dependency_checking");

    group.bench_function("check_nodejs", |b| {
        use mcp_helper::deps::NodeChecker;
        let checker = NodeChecker::new();
        b.iter(|| {
            let status = checker.check();
            let _ = hint_black_box(status);
        });
    });

    group.bench_function("check_python", |b| {
        use mcp_helper::deps::PythonChecker;
        let checker = PythonChecker::new();
        b.iter(|| {
            let status = checker.check();
            let _ = hint_black_box(status);
        });
    });

    group.bench_function("check_docker", |b| {
        use mcp_helper::deps::DockerChecker;
        let checker = DockerChecker::new();
        b.iter(|| {
            let status = checker.check();
            let _ = hint_black_box(status);
        });
    });

    group.finish();
}

/// Benchmark client detection and registry operations
fn bench_client_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("client_operations");

    group.bench_function("detect_all_clients", |b| {
        b.iter(|| {
            let clients = mcp_helper::client::detect_clients();
            hint_black_box(clients);
        });
    });

    group.bench_function("registry_creation", |b| {
        b.iter(|| {
            let registry = ClientRegistry::new();
            hint_black_box(registry);
        });
    });

    group.finish();
}

/// Benchmark configuration manager operations
fn bench_config_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_operations");

    group.bench_function("config_manager_new", |b| {
        b.iter(|| {
            let manager = ConfigManager::new();
            let _ = hint_black_box(manager);
        });
    });

    group.finish();
}

/// Benchmark validation operations
fn bench_validation(c: &mut Criterion) {
    use mcp_helper::core::validation::*;

    let mut group = c.benchmark_group("validation");

    group.bench_function("validate_server_name", |b| {
        let names = vec![
            "@modelcontextprotocol/server-filesystem",
            "simple-server",
            "docker:image:tag",
            "../../../etc/passwd",
        ];

        b.iter(|| {
            for name in &names {
                let result = validate_server_name(black_box(name));
                let _ = hint_black_box(result);
            }
        });
    });

    group.bench_function("parse_npm_package", |b| {
        let packages = vec!["@scope/package@1.2.3", "simple-package", "@org/pkg@latest"];

        b.iter(|| {
            for pkg in &packages {
                let result = parse_npm_package(black_box(pkg));
                let _ = hint_black_box(result);
            }
        });
    });

    group.finish();
}

/// Main benchmark group
fn performance_benches(c: &mut Criterion) {
    bench_server_type_detection(c);
    bench_dependency_checking(c);
    bench_client_operations(c);
    bench_config_operations(c);
    bench_validation(c);
}

criterion_group!(benches, performance_benches);
criterion_main!(benches);
