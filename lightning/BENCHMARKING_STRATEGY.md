# Benchmarking Strategy for MonitorUpdatingPersister Optimization

## Overview

This document outlines the comprehensive benchmarking strategy to determine the optimal default value for `min_monitor_size_for_updates_bytes` in the `MonitorUpdatingPersister`. This parameter controls when the persister switches between full monitor persistence and update-based persistence.

## Background

The recent changes to `persist.rs` introduced a size-based optimization where:

- **Small monitors** (< threshold): Always persisted in full to avoid update management overhead
- **Large monitors** (≥ threshold): Use update-based persistence for efficiency

The current default is **4096 bytes (4KB)**, but we need empirical data to validate this choice.

## Benchmarking Components

### 1. Test Scenarios (`setup_scenario`)

We simulate four realistic Lightning Network scenarios:

#### Small Monitors (≤ 2KB typically)

- **Use case**: New channels, minimal activity
- **Setup**: 1-2 payments maximum
- **Expected behavior**: Full persistence should be faster due to low overhead

#### Medium Monitors (2-8KB typically)

- **Use case**: Moderate activity channels
- **Setup**: 10 payments with bidirectional flow
- **Expected behavior**: Threshold value critically affects performance

#### Large Monitors (8-20KB typically)

- **Use case**: Active channels with many completed payments
- **Setup**: 50+ payments including multi-hop routing
- **Expected behavior**: Update-based persistence should win

#### Very Large Monitors (>20KB)

- **Use case**: Busy routing nodes with in-flight HTLCs
- **Setup**: 100+ payments with some failures to simulate pending HTLCs
- **Expected behavior**: Strong preference for update-based persistence

### 2. Threshold Values Tested

We test these threshold values (in bytes):

- **0**: Always use update-based persistence
- **1024**: 1KB threshold
- **2048**: 2KB threshold
- **4096**: 4KB threshold (current default)
- **8192**: 8KB threshold
- **16384**: 16KB threshold
- **32768**: 32KB threshold

### 3. Benchmark Categories

#### A. Decision Logic Performance (`bench_thresholds`)

- Measures the computational cost of size-based decisions
- Tests the actual threshold comparison logic
- Helps identify if decision overhead is significant

#### B. I/O Performance (`bench_io_performance`)

- **Full writes**: Large, infrequent monitor writes
- **Update writes**: Small, frequent update writes
- Uses real filesystem operations for accurate measurement
- Critical for understanding real-world performance

#### C. Read Performance (`bench_read_performance`)

- **Full monitor reads**: Single large file read
- **Multiple update reads**: Reading update sequence
- Important for node restart/recovery scenarios

## Performance Metrics

### Primary Metrics

1. **Throughput** (bytes/second): How much data can be processed
2. **Latency** (nanoseconds): Time per operation
3. **I/O efficiency**: Actual disk operations cost

### Secondary Metrics

1. **File count**: Number of files created (affects filesystem performance)
2. **Storage efficiency**: Total disk space used
3. **Recovery time**: Time to read all data (startup performance)

## Expected Results

### Hypothesis

Based on the trade-offs:

1. **Small monitors**: Full persistence wins due to:

   - No update file management overhead
   - Single file I/O is efficient for small data
   - Fewer filesystem operations

2. **Large monitors**: Update-based persistence wins due to:

   - Much smaller write operations
   - Reduced I/O bandwidth usage
   - Better suited for frequent updates

3. **Crossover point**: Expected around 4-8KB where strategies perform similarly

### Decision Matrix

| Monitor Size | Current Strategy | Expected Optimal | Reasoning              |
| ------------ | ---------------- | ---------------- | ---------------------- |
| < 2KB        | Full             | Full             | Low overhead dominates |
| 2-4KB        | Full             | Mixed            | Transition zone        |
| 4-8KB        | Update           | Update           | Balanced benefits      |
| > 8KB        | Update           | Update           | Clear I/O advantages   |

## Running the Benchmarks

### Prerequisites

```bash
# Ensure criterion is installed
cargo install criterion

# Make benchmark runner executable
chmod +x bench_runner.sh
```

### Execution

```bash
# Run all benchmarks with automatic visualization
./bench_runner.sh

# Or run manually
cargo bench --bench monitor_updating_persister
```

### Analysis Tools

The benchmarking suite generates comprehensive HTML reports via Criterion:

**Generated Reports:**

- Interactive HTML dashboard with performance graphs
- Detailed timing statistics with confidence intervals
- Historical performance tracking
- Statistical analysis and trend detection

**Access:**

```bash
# View results in browser
open ../target/criterion/index.html
```

The HTML reports provide:

- Clear performance comparisons across scenarios
- Statistical confidence intervals and variance analysis
- Interactive charts and data tables
- Detailed timing breakdowns for each benchmark

### Results Analysis

1. **View HTML reports**: Open `../target/criterion/index.html`
2. **Compare throughput**: Look for crossover points between strategies
3. **Analyze I/O patterns**: Examine real filesystem performance
4. **Check scalability**: How performance changes with monitor size
5. **Review statistical confidence**: Examine error bars and variance

## Interpreting Results

### Key Questions to Answer

1. **What's the actual crossover point?**

   - At what monitor size does update-based become faster?
   - Is it consistent across different workload patterns?

2. **How sensitive is performance to the threshold?**

   - Large performance cliff or gradual transition?
   - Is the current 4KB default in the right ballpark?

3. **What are the I/O characteristics?**

   - Are write patterns different from expectations?
   - How does read performance (startup) vary?

4. **Real-world implications?**
   - What percentage of channels fall into each size category?
   - What workloads are most sensitive to this optimization?

### Recommendation Framework

Based on benchmark results, adjust the default threshold:

- **If crossover < 2KB**: Decrease default to 2KB
- **If crossover 2-6KB**: Keep default at 4KB
- **If crossover > 6KB**: Increase default to 8KB
- **If no clear crossover**: Consider workload-specific defaults

## Additional Considerations

### Real-World Factors Not Captured

1. **Concurrent access patterns**: Multiple channels updating simultaneously
2. **Filesystem differences**: ext4 vs NTFS vs APFS performance varies
3. **Storage types**: SSD vs HDD vs NVMe characteristics
4. **Memory pressure**: How caching affects I/O patterns
5. **Network effects**: Correlation between channel activity and updates

### Future Optimization Opportunities

1. **Adaptive thresholds**: Dynamic adjustment based on observed patterns
2. **Hybrid strategies**: Time-based persistence switching
3. **Compression**: Reduce monitor size through better serialization
4. **Batching**: Group multiple updates for efficiency

## Production Validation

After benchmarking, consider:

1. **A/B testing**: Deploy different thresholds to subset of nodes
2. **Telemetry**: Monitor actual performance in production
3. **User feedback**: How does it affect real Lightning applications?
4. **Long-term monitoring**: Performance evolution over time

## Conclusion

This benchmarking strategy provides a systematic approach to optimize the `min_monitor_size_for_updates_bytes` parameter. The combination of realistic scenarios, comprehensive metrics, and careful analysis should yield actionable insights for improving Lightning Network persistence performance.

The goal is not just to find the optimal default, but to understand the performance characteristics well enough to provide guidance for different use cases and deployment scenarios.
