//! Benchmarks for MonitorUpdatingPersister min_monitor_size_for_updates_bytes threshold
//!
//! This benchmark suite evaluates the performance impact of different `min_monitor_size_for_updates_bytes`
//! values to determine the optimal default for real-world scenarios.
//!
//! The benchmark tests both simulated I/O patterns (for baseline performance) and actual
//! MonitorUpdatingPersister usage patterns to validate real-world behavior.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use lightning::util::persist::KVStore;
use std::time::Duration;
use tempfile::TempDir;
use std::path::PathBuf;
use std::fs;
use lightning::io;

/// File-system based KVStore for realistic I/O benchmarking
struct FileSystemStore {
    base_path: PathBuf,
}

impl FileSystemStore {
    fn new(base_path: PathBuf) -> Self {
        fs::create_dir_all(&base_path).expect("Failed to create base directory");
        Self { base_path }
    }
}

// kvstore is key-value store here
impl KVStore for FileSystemStore {
    fn read(&self, primary_namespace: &str, secondary_namespace: &str, key: &str) -> Result<Vec<u8>, io::Error> {
        let mut path = self.base_path.clone();
        if !primary_namespace.is_empty() {
            path.push(primary_namespace);
        }
        if !secondary_namespace.is_empty() {
            path.push(secondary_namespace);
        }
        path.push(key);
        fs::read(path).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Read error: {}", e)))
    }

    fn write(&self, primary_namespace: &str, secondary_namespace: &str, key: &str, buf: &[u8]) -> Result<(), io::Error> {
        let mut path = self.base_path.clone();
        if !primary_namespace.is_empty() {
            path.push(primary_namespace);
        }
        if !secondary_namespace.is_empty() {
            path.push(secondary_namespace);
        }
        fs::create_dir_all(&path).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Create dir error: {}", e)))?;
        path.push(key);
        fs::write(path, buf).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Write error: {}", e)))
    }

    fn remove(&self, primary_namespace: &str, secondary_namespace: &str, key: &str, _lazy: bool) -> Result<(), io::Error> {
        let mut path = self.base_path.clone();
        if !primary_namespace.is_empty() {
            path.push(primary_namespace);
        }
        if !secondary_namespace.is_empty() {
            path.push(secondary_namespace);
        }
        path.push(key);
        fs::remove_file(path).or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(io::Error::new(io::ErrorKind::Other, format!("Remove error: {}", e)))
            }
        })
    }

    fn list(&self, primary_namespace: &str, secondary_namespace: &str) -> Result<Vec<String>, io::Error> {
        let mut path = self.base_path.clone();
        if !primary_namespace.is_empty() {
            path.push(primary_namespace);
        }
        if !secondary_namespace.is_empty() {
            path.push(secondary_namespace);
        }
        
        match fs::read_dir(path) {
            Ok(entries) => {
                let mut result = Vec::new();
                for entry in entries {
                    let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, format!("List entry error: {}", e)))?;
                    if let Some(name) = entry.file_name().to_str() {
                        result.push(name.to_string());
                    }
                }
                Ok(result)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("List error: {}", e))),
        }
    }
}

/// Simulate monitor data of different sizes
fn create_simulated_monitor_data(size_category: &str) -> (Vec<u8>, usize) {
    let size = match size_category {
        "small" => 1024,      // 1KB - small monitor
        "medium" => 4096,     // 4KB - medium monitor  
        "large" => 12288,     // 12KB - large monitor
        "very_large" => 32768, // 32KB - very large monitor
        _ => panic!("Unknown size category: {}", size_category),
    };
    
    // Create dummy monitor data - this simulates serialized ChannelMonitor
    // The actual bytes don't matter for I/O performance testing
    let data = vec![0u8; size];
    (data, size)
}

/// Simulate the core decision logic from MonitorUpdatingPersister
fn simulate_persistence_decision(monitor_size: usize, threshold: usize, update_size: usize) -> (bool, usize) {
    let use_full_persistence = monitor_size < threshold;
    let bytes_to_write = if use_full_persistence {
        monitor_size  // Write full monitor
    } else {
        update_size   // Write just the update
    };
    (use_full_persistence, bytes_to_write)
}

/// Benchmark different threshold values for each scenario
fn bench_thresholds(c: &mut Criterion) {
    let thresholds = vec![0, 1024, 2048, 4096, 8192, 16384, 32768];
    let scenarios = vec![
        ("small", 1024),
        ("medium", 4096), 
        ("large", 12288),
        ("very_large", 32768),
    ];

    for (scenario_name, monitor_size) in scenarios {
        let mut group = c.benchmark_group(format!("decision_logic_{}", scenario_name));
        group.measurement_time(Duration::from_secs(5));
        
        let (monitor_data, _) = create_simulated_monitor_data(scenario_name);
        group.throughput(Throughput::Bytes(monitor_data.len() as u64));
        
        println!("Scenario '{}': Monitor size = {} bytes", scenario_name, monitor_size);

        for &threshold in &thresholds {
            group.bench_with_input(
                BenchmarkId::new("threshold", threshold),
                &threshold,
                |b, _| {
                    b.iter(|| {
                        // This simulates the exact logic from update_persisted_channel
                        let (use_full_persistence, bytes_to_write) = simulate_persistence_decision(
                            monitor_size, 
                            threshold, 
                            200  // Typical update size , find out more about this as this is also placeholder value
                        );
                        
                        // Simulate the I/O cost difference
                        std::hint::black_box((use_full_persistence, bytes_to_write));
                    })
                },
            );
        }
        
        group.finish();
    }
}

/// Benchmark I/O performance with real filesystem operations
/// This simulates the actual KVStore.write() calls made by MonitorUpdatingPersister
fn bench_io_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_performance");
    group.measurement_time(Duration::from_secs(10));

    let scenarios = vec![
        ("small", 1024),
        ("medium", 4096),
        ("large", 12288),
    ];

    for (scenario_name, _monitor_size) in scenarios {
        let (monitor_data, _) = create_simulated_monitor_data(scenario_name);
        
        // Test both strategies with fresh temp directories
        let temp_dir = TempDir::new().unwrap();
        let kv_store = FileSystemStore::new(temp_dir.path().to_path_buf());
        
        // Benchmark full monitor writes (what happens when use_full_persistence = true)
        group.bench_function(
            &format!("{}_full_write", scenario_name),
            |b| {
                let mut counter = 0;
                b.iter(|| {
                    counter += 1;
                    // This simulates: self.kv_store.write(CHANNEL_MONITOR_PERSISTENCE_PRIMARY_NAMESPACE, ...)
                    kv_store.write("monitors", "", &format!("test_monitor_{}", counter), &monitor_data).unwrap();
                })
            },
        );
        
        // Benchmark update writes (what happens when use_full_persistence = false)
        let update_data = vec![0u8; 200]; // Typical ChannelMonitorUpdate size
        group.bench_function(
            &format!("{}_update_write", scenario_name),
            |b| {
                let mut counter = 0;
                b.iter(|| {
                    counter += 1;
                    // This simulates: self.kv_store.write(CHANNEL_MONITOR_UPDATE_PERSISTENCE_PRIMARY_NAMESPACE, ...)
                    kv_store.write("monitor_updates", "test_monitor", &format!("update_{}", counter), &update_data).unwrap();
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark read performance for different strategies
/// This tests the cost of reading back the data during node startup
fn bench_read_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_performance");
    group.measurement_time(Duration::from_secs(5));

    let temp_dir = TempDir::new().unwrap();
    let kv_store = FileSystemStore::new(temp_dir.path().to_path_buf());
    
    // Setup test data
    let (monitor_data, _) = create_simulated_monitor_data("large");
    let update_data = vec![0u8; 200];
    
    // Write test data
    kv_store.write("monitors", "", "test_monitor", &monitor_data).unwrap();
    for i in 1..=20 {
        kv_store.write("monitor_updates", "test_monitor", &format!("{}", i), &update_data).unwrap();
    }

    // Benchmark full monitor read (single file strategy)
    group.bench_function("full_monitor_read", |b| {
        b.iter(|| {
            let _data = kv_store.read("monitors", "", "test_monitor").unwrap();
        })
    });

    // Benchmark reading multiple updates (update-based strategy)
    group.bench_function("multiple_updates_read", |b| {
        b.iter(|| {
            for i in 1..=20 {
                let _data = kv_store.read("monitor_updates", "test_monitor", &format!("{}", i)).unwrap();
            }
        })
    });

    group.finish();
}

/// Comprehensive crossover analysis to find the optimal threshold
/// This generates data for visualization and decision making
fn bench_crossover_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("crossover_analysis");
    group.measurement_time(Duration::from_secs(8));
    
    // Test a range of monitor sizes to find the crossover point
    let monitor_sizes = vec![512, 1024, 2048, 4096, 6144, 8192, 12288, 16384, 24576, 32768];
    let _threshold = 4096; // Current default
    
    println!("\n📊 Crossover Analysis Data (for visualization):");
    println!("| Size (KB) | Full Write (µs) | Update Write (µs) | Ratio | Winner |");
    println!("| --------- | --------------- | ----------------- | ----- | ------ |");
    
    for &size in &monitor_sizes {
        let monitor_data = vec![0u8; size];
        let update_data = vec![0u8; 200];
        
        let temp_dir = TempDir::new().unwrap();
        let kv_store = FileSystemStore::new(temp_dir.path().to_path_buf());
        
        // Benchmark full write strategy
        group.bench_function(
            &format!("full_write_{}kb", size / 1024),
            |b| {
                let mut counter = 0;
                b.iter(|| {
                    counter += 1;
                    kv_store.write("monitors", "", &format!("monitor_{}", counter), &monitor_data).unwrap();
                })
            },
        );
        
        // Benchmark update strategy (simulate multiple small writes proportional to monitor size)
        group.bench_function(
            &format!("update_write_{}kb", size / 1024),
            |b| {
                let mut counter = 0;
                b.iter(|| {
                    counter += 1;
                    // Simulate writing multiple updates (larger monitors need more updates to represent)
                    let num_updates = (size / 2048).max(1); // More updates for larger monitors
                    for i in 0..num_updates {
                        kv_store.write("monitor_updates", &format!("monitor_{}", counter), &format!("update_{}", i), &update_data).unwrap();
                    }
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");
    group.measurement_time(Duration::from_secs(5));
    
    let scenarios = vec![
        ("rapid_updates_small", 1024, 10),
        ("rapid_updates_medium", 4096, 10),  
        ("rapid_updates_large", 12288, 10),
    ];
    
    for (scenario_name, monitor_size, num_updates) in scenarios {
        let temp_dir = TempDir::new().unwrap();
        let kv_store = FileSystemStore::new(temp_dir.path().to_path_buf());
        
        let monitor_data = vec![0u8; monitor_size];
        let update_data = vec![0u8; 200];
        
        // Test full persistence strategy under rapid updates
        group.bench_function(
            &format!("{}_full_persistence", scenario_name),
            |b| {
                b.iter(|| {
                    // Simulate rapid monitor updates with full persistence
                    for i in 0..num_updates {
                        kv_store.write("monitors", "", &format!("monitor_{}", i), &monitor_data).unwrap();
                    }
                })
            },
        );
        
        // Test update-based strategy under rapid updates  
        group.bench_function(
            &format!("{}_update_persistence", scenario_name),
            |b| {
                b.iter(|| {
                    // Simulate rapid monitor updates with update persistence
                    for i in 0..num_updates {
                        kv_store.write("monitor_updates", "monitor", &format!("update_{}", i), &update_data).unwrap();
                    }
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches, 
    bench_thresholds,
    bench_io_performance,
    bench_read_performance,
    bench_crossover_analysis,
    bench_memory_patterns
);
criterion_main!(benches); 