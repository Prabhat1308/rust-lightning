# MonitorUpdatingPersister Benchmark Guide

## Overview

This benchmark tests the `min_monitor_size_for_updates_bytes` optimization in `MonitorUpdatingPersister` to determine the optimal threshold for real-world Lightning Network scenarios.

## Quick Start

```bash
# Run the benchmark
./bench_runner.sh

# View results
open ../target/criterion/report/index.html
```

## What This Benchmarks

The benchmark tests the performance trade-off between two persistence strategies:

1. **Full Persistence**: Write the complete monitor data (larger files, fewer writes)
2. **Update-based Persistence**: Write only incremental updates (smaller files, more writes)

The key question: **At what monitor size does it become more efficient to switch strategies?**

## Benchmark Categories

### 1. **Crossover Analysis**

- Tests monitor sizes from 512B to 32KB
- Compares full vs update write performance
- **Goal**: Find the optimal threshold value

### 2. **I/O Performance**

- Tests realistic scenarios (small/medium/large monitors)
- Measures actual filesystem write performance
- **Goal**: Understand real-world write costs

### 3. **Read Performance**

- Compares reading single files vs multiple update files
- **Goal**: Evaluate startup/recovery time impact

### 4. **Memory Patterns**

- Tests rapid update scenarios
- **Goal**: Understand bulk operation performance

## How to Run

### Standard Benchmark

```bash
./bench_runner.sh
```

### Quick Test (faster, less precise)

```bash
./bench_runner.sh --quick
```

### Detailed Analysis (slower, more precise)

```bash
./bench_runner.sh --detailed
```

## Analyzing Results

### 1. Open the HTML Report

```bash
# Open in browser
open ../target/criterion/report/index.html

# Or manually navigate to:
# ../target/criterion/report/index.html
```

### 2. Key Metrics to Look For

**Performance Times:**

- Lower microseconds (µs) = better performance
- Look for crossover points where one strategy becomes faster

**Critical Comparisons:**

- `full_write_XkB` vs `update_write_XkB` for each size
- `full_monitor_read` vs `multiple_updates_read`

### 3. What to Analyze

#### Crossover Analysis

1. **Navigate to**: `crossover_analysis` section
2. **Compare**: Full write times vs Update write times for each size
3. **Find**: The point where update-based becomes slower than full persistence
4. **Example**: If full_write_4kb = 45µs and update_write_4kb = 110µs, then full persistence is 2.4x faster

#### I/O Performance

1. **Navigate to**: `io_performance` section
2. **Compare**: Performance across small/medium/large scenarios
3. **Look for**: Consistent patterns across different monitor sizes

#### Read Performance

1. **Navigate to**: `read_performance` section
2. **Compare**: Single file reads vs multiple file reads
3. **Consider**: Impact on node startup time

## Expected Results

Based on filesystem behavior and I/O patterns:

- **Small monitors (< 2KB)**: Full persistence likely faster
- **Medium monitors (2-8KB)**: May vary based on system
- **Large monitors (> 8KB)**: Traditional assumption is update-based faster, but benchmark will show reality

## Making Decisions

### If Full Persistence is Always Faster:

```rust
// Consider setting threshold to 0 (always use full persistence)
MonitorUpdatingPersister::new_with_monitor_size_threshold(kv_store, 0)

// Or use a very high threshold
MonitorUpdatingPersister::new_with_monitor_size_threshold(kv_store, 1_000_000)
```

### If Update-based is Better for Large Monitors:

```rust
// Use the crossover point from your benchmark results
// For example, if crossover is at 8KB:
MonitorUpdatingPersister::new_with_monitor_size_threshold(kv_store, 8192)
```

### If Current 4KB Default is Optimal:

```rust
// Keep using the default
MonitorUpdatingPersister::new(kv_store)  // Uses 4096 bytes
```

## Interpreting the Data

### Sample Analysis Process:

1. **Run benchmark**: `./bench_runner.sh`
2. **Open results**: View HTML report
3. **Check crossover**: Look at `crossover_analysis` results
4. **Example findings**:
   ```
   1KB:  full=43µs, update=59µs  → full is 37% faster
   4KB:  full=47µs, update=115µs → full is 145% faster
   12KB: full=50µs, update=276µs → full is 452% faster
   ```
5. **Conclusion**: Full persistence is consistently faster
6. **Recommendation**: Set threshold to 0 or very high value

## Key Considerations

- **Write Performance**: How fast can we persist updates?
- **Read Performance**: How fast can we recover on startup?
- **Storage Space**: Update files vs full monitor files
- **Complexity**: Managing multiple files vs single files

## Validation

After choosing a threshold:

1. **Test with real Lightning node** if possible
2. **Monitor filesystem performance** in production
3. **Consider different storage types** (SSD vs HDD)
4. **Measure actual monitor sizes** in your use case

## Files

- `benches/monitor_updating_persister.rs` - Benchmark implementation
- `bench_runner.sh` - Easy execution script
- `../target/criterion/` - Results data and HTML reports

## Why This Works

This benchmark focuses on **I/O performance** - the actual bottleneck in persistence operations. The Lightning Network logic is fast (nanoseconds), but disk operations are slow (microseconds). By testing realistic data sizes with actual filesystem operations, we get accurate performance data without needing real Lightning monitors running.
