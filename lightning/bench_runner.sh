#!/bin/bash

# Benchmark runner script for MonitorUpdatingPersister optimization
# This script runs the benchmarks and provides guidance on analyzing results

set -e

echo "🔬 MonitorUpdatingPersister Benchmarking Suite"
echo "=============================================="
echo "This benchmarks the min_monitor_size_for_updates_bytes optimization"
echo

# Check if criterion is available
if ! cargo bench --help > /dev/null 2>&1; then
    echo "❌ Error: Cargo bench not available. Please install Rust with cargo."
    exit 1
fi

echo "📊 Running benchmarks..."
echo "This will take several minutes and create detailed HTML reports."
echo

# Run with different verbosity levels
if [ "$1" = "--quick" ]; then
    echo "🚀 Running quick benchmark..."
    cargo bench --bench monitor_updating_persister -- --quick
elif [ "$1" = "--detailed" ]; then
    echo "🔍 Running detailed benchmark (this may take 10+ minutes)..."
    cargo bench --bench monitor_updating_persister -- --sample-size 1000
else
    echo "⚡ Running standard benchmark..."
    cargo bench --bench monitor_updating_persister
fi

echo
echo "✅ Benchmark Complete!"
echo

# Check if HTML report was generated
if [ -f "../target/criterion/report/index.html" ]; then
    echo "📈 Results Analysis:"
    echo "==================="
    echo
    echo "🌟 View detailed benchmark results:"
    echo "  📊 HTML Report: ../target/criterion/report/index.html"
    echo "  📁 Raw Data:    ../target/criterion/"
    echo
    echo "🔍 Key areas to analyze:"
    echo "  1. crossover_analysis - Find optimal threshold for different monitor sizes"
    echo "  2. io_performance - Compare write speeds for different scenarios"  
    echo "  3. read_performance - Startup time implications"
    echo "  4. memory_patterns - Bulk operation performance"
    echo
    echo "📊 How to interpret results:"
    echo "  • Lower times (µs) = better performance"
    echo "  • Look for crossover points where strategies switch efficiency"
    echo "  • Consider both write AND read performance for your use case"
    echo "  • Factor in storage space vs performance trade-offs"
    echo
    echo "💡 Quick analysis tips:"
    echo "  1. Open the HTML report in your browser"
    echo "  2. Compare 'full_write' vs 'update_write' times for each size"
    echo "  3. Note where update-based becomes slower than full persistence"
    echo "  4. Consider read performance impact (startup time)"
    echo
    echo "🎯 Expected findings:"
    echo "  • Small monitors: Full persistence likely faster"
    echo "  • Large monitors: Results will show the actual crossover point"
    echo "  • Read operations: Single files typically faster than multiple files"
else
    echo "⚠️  HTML report not found at ../target/criterion/report/index.html"
    echo "   Check for errors above or run from the correct directory."
fi

echo
echo "🏁 Done! Use './bench_runner.sh --help' for options:"
echo "  --quick     : Fast benchmark for testing"
echo "  --detailed  : Thorough benchmark with more samples"
echo "  (no args)   : Standard benchmark" 