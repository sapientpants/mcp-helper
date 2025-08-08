# Performance Benchmarks for MCP Helper

## Overview

This document describes the performance benchmarking infrastructure for MCP Helper, including startup time measurements, memory usage analysis, and performance optimization guidelines.

## Benchmark Suite

### 1. Startup Time Benchmarks (`benches/startup_time.rs`)

Measures the critical startup performance metrics:

- **Version Display**: Time to show version information (`mcp --version`)
- **Help Display**: Time to show help text (`mcp --help`)
- **Subcommand Help**: Time to show subcommand help (`mcp help run`)
- **Raw Spawn Time**: Process spawn overhead without execution

### 2. Performance Benchmarks (`benches/performance.rs`)

Comprehensive performance measurements:

- **Server Type Detection**: Performance of detecting different server types
- **Dependency Checking**: Speed of checking for Node.js, Python, Docker
- **Client Operations**: Client detection and registry operations
- **Configuration Loading**: Config manager initialization
- **Validation Operations**: Server name and package validation

## Running Benchmarks

### Local Development

```bash
# Run all benchmarks
make bench

# Run startup time benchmarks only
make bench-startup

# Run performance benchmarks only
make bench-performance

# Run with detailed output
cargo bench --bench startup_time -- --verbose
```

### CI Integration

Benchmarks run automatically on:
- Push to main branch
- Pull requests
- Manual workflow dispatch

Results are:
- Tracked over time
- Compared against baselines
- Alert on >20% regression

## Performance Goals

### Startup Time
- **Target**: < 100ms for `--version` flag
- **Acceptable**: < 150ms for `--help` flag
- **Maximum**: < 200ms for any basic command

### Memory Usage
- **Idle**: < 10MB resident memory
- **Active**: < 50MB during operations
- **Peak**: < 100MB for large operations

### Operation Performance
- Server type detection: < 1μs
- Dependency check: < 10ms per dependency
- Config loading: < 5ms
- Validation: < 100ns per validation

## Optimization Guidelines

### 1. Lazy Loading
- Load dependencies only when needed
- Defer client detection until required
- Cache expensive operations

### 2. Efficient Parsing
- Use fast path for common cases
- Avoid regex when simple string ops suffice
- Pre-compile regular expressions

### 3. Memory Management
- Use references instead of clones
- Stream large data instead of loading
- Clear caches when appropriate

### 4. Concurrency
- Parallelize independent operations
- Use async for I/O operations
- Avoid blocking in hot paths

## Benchmark Results Interpretation

### Criterion Output
```
startup_version         time:   [45.2 ms 46.1 ms 47.0 ms]
                        change: [-2.1% +0.5% +3.2%] (p = 0.72 > 0.05)
                        No change in performance detected.
```

- **time**: [lower bound, estimate, upper bound]
- **change**: Performance change from baseline
- **p-value**: Statistical significance (< 0.05 = significant)

### Direct Measurements
```
Run 1: 0.045 seconds
Run 2: 0.043 seconds
...
Average: 0.044 seconds
```

Used for:
- Real-world validation
- Platform-specific testing
- Distribution analysis

## Continuous Monitoring

### GitHub Actions Integration
- Automated benchmark runs on every commit
- Performance tracking dashboard
- Regression alerts via comments
- Historical trend analysis

### Local Development
```bash
# Compare against main branch
git checkout main
cargo bench --bench startup_time -- --save-baseline main
git checkout feature-branch
cargo bench --bench startup_time -- --baseline main
```

## Common Performance Issues

### 1. Slow Startup
**Symptoms**: High startup time measurements
**Common Causes**:
- Large dependency tree
- Synchronous initialization
- Excessive file I/O

**Solutions**:
- Lazy load dependencies
- Parallelize initialization
- Cache configuration

### 2. Memory Bloat
**Symptoms**: High memory usage
**Common Causes**:
- Memory leaks
- Large static data
- Inefficient caching

**Solutions**:
- Profile with valgrind
- Use weak references
- Implement cache eviction

### 3. CPU Spikes
**Symptoms**: High CPU usage
**Common Causes**:
- Inefficient algorithms
- Busy waiting
- Excessive polling

**Solutions**:
- Profile hot paths
- Use efficient data structures
- Implement proper async/await

## Future Improvements

1. **Additional Benchmarks**
   - Network operation latency
   - Concurrent operation scaling
   - Large file handling

2. **Platform-Specific Testing**
   - Windows process creation overhead
   - macOS security prompt impact
   - Linux distribution variations

3. **Real-World Scenarios**
   - Full installation workflow
   - Multiple server management
   - Configuration migration

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [cargo-bench Documentation](https://doc.rust-lang.org/cargo/commands/cargo-bench.html)