//! Exodus Browser — Performance tests for the plugin system.

#![allow(dead_code)]

use super::manager::ExtensionManager;
use super::storage::ExtensionStorage;
use serde_json::{json, Map};
use std::path::Path;
use std::time::Instant;

/// Benchmark extension loading time.
pub fn benchmark_extension_load(manager: &mut ExtensionManager, extension_path: &Path) -> Result<u128, String> {
    let start = Instant::now();
    
    manager
        .install_from_dir(extension_path)
        .map_err(|e| e.to_string())?;
    
    let duration = start.elapsed();
    Ok(duration.as_millis())
}

/// Benchmark storage read/write operations.
pub fn benchmark_storage_operations(storage: &ExtensionStorage, extension_id: &str, iterations: usize) -> Result<StorageBenchmark, String> {
    let permissions = vec![super::permissions::Permission::Storage];
    
    // Benchmark writes
    let write_start = Instant::now();
    for i in 0..iterations {
        let mut data = Map::new();
        data.insert(format!("key_{}", i), json!(i));
        storage
            .set(extension_id, &permissions, data)
            .map_err(|e| e.to_string())?;
    }
    let write_duration = write_start.elapsed();
    
    // Benchmark reads
    let read_start = Instant::now();
    for i in 0..iterations {
        storage
            .get(extension_id, &permissions, Some(vec![format!("key_{}", i)]))
            .map_err(|e| e.to_string())?;
    }
    let read_duration = read_start.elapsed();
    
    Ok(StorageBenchmark {
        iterations,
        write_time_ms: write_duration.as_millis(),
        read_time_ms: read_duration.as_millis(),
        avg_write_us: write_duration.as_micros() as u64 / iterations as u64,
        avg_read_us: read_duration.as_micros() as u64 / iterations as u64,
    })
}

/// Benchmark content script injection time.
pub fn benchmark_content_script_injection(
    manager: &ExtensionManager,
    page_url: &str,
) -> Result<ContentScriptBenchmark, String> {
    let tabs = super::tabs::TabRegistry::default();
    let start = Instant::now();
    
    let script = manager.document_start_script(page_url, &tabs, "test-label");
    
    let duration = start.elapsed();
    
    Ok(ContentScriptBenchmark {
        script_length: script.len(),
        injection_time_ms: duration.as_millis(),
    })
}

/// Benchmark manifest parsing.
pub fn benchmark_manifest_parsing(manifest_path: &Path) -> Result<ManifestBenchmark, String> {
    let iterations = 100;
    
    let start = Instant::now();
    for _ in 0..iterations {
        super::manifest::load_manifest(manifest_path)
            .map_err(|e| e.to_string())?;
    }
    let duration = start.elapsed();
    
    Ok(ManifestBenchmark {
        iterations,
        total_time_ms: duration.as_millis(),
        avg_parse_us: duration.as_micros() as u64 / iterations as u64,
    })
}

/// Benchmark extension list retrieval.
pub fn benchmark_extension_list(manager: &ExtensionManager) -> Result<ListBenchmark, String> {
    let iterations = 100;
    
    let start = Instant::now();
    for _ in 0..iterations {
        manager.list();
    }
    let duration = start.elapsed();
    
    Ok(ListBenchmark {
        iterations,
        total_time_ms: duration.as_millis(),
        avg_list_us: duration.as_micros() as u64 / iterations as u64,
        extension_count: manager.list().len(),
    })
}

#[derive(Debug, Clone)]
pub struct StorageBenchmark {
    pub iterations: usize,
    pub write_time_ms: u128,
    pub read_time_ms: u128,
    pub avg_write_us: u64,
    pub avg_read_us: u64,
}

#[derive(Debug, Clone)]
pub struct ContentScriptBenchmark {
    pub script_length: usize,
    pub injection_time_ms: u128,
}

#[derive(Debug, Clone)]
pub struct ManifestBenchmark {
    pub iterations: usize,
    pub total_time_ms: u128,
    pub avg_parse_us: u64,
}

#[derive(Debug, Clone)]
pub struct ListBenchmark {
    pub iterations: usize,
    pub total_time_ms: u128,
    pub avg_list_us: u64,
    pub extension_count: usize,
}

/// Run all performance benchmarks and return results.
pub fn run_all_benchmarks(app_data_dir: &Path) -> Result<BenchmarkSuite, String> {
    let mut suite = BenchmarkSuite::default();
    
    // Setup test extension
    let test_ext_dir = app_data_dir.join("test_extension");
    std::fs::create_dir_all(&test_ext_dir).map_err(|e| e.to_string())?;
    std::fs::write(
        test_ext_dir.join("manifest.json"),
        r#"{"manifest_version":3,"name":"Benchmark","version":"1.0","permissions":["storage"]}"#,
    ).map_err(|e| e.to_string())?;
    
    let mut manager = ExtensionManager::new(app_data_dir).map_err(|e| e.to_string())?;
    
    // Benchmark extension load
    let load_time = benchmark_extension_load(&mut manager, &test_ext_dir)?;
    suite.extension_load_ms = Some(load_time);
    
    // Benchmark storage operations
    let storage = ExtensionStorage::new(app_data_dir).map_err(|e| e.to_string())?;
    let storage_bench = benchmark_storage_operations(&storage, "benchmark", 1000)?;
    suite.storage_benchmark = Some(storage_bench);
    
    // Benchmark content script injection
    let script_bench = benchmark_content_script_injection(&manager, "https://example.com")?;
    suite.content_script_benchmark = Some(script_bench);
    
    // Benchmark manifest parsing
    let manifest_bench = benchmark_manifest_parsing(&test_ext_dir.join("manifest.json"))?;
    suite.manifest_benchmark = Some(manifest_bench);
    
    // Benchmark extension list
    let list_bench = benchmark_extension_list(&manager)?;
    suite.list_benchmark = Some(list_bench);
    
    // Cleanup
    let _ = std::fs::remove_dir_all(&test_ext_dir);
    
    Ok(suite)
}

#[derive(Debug, Clone, Default)]
pub struct BenchmarkSuite {
    pub extension_load_ms: Option<u128>,
    pub storage_benchmark: Option<StorageBenchmark>,
    pub content_script_benchmark: Option<ContentScriptBenchmark>,
    pub manifest_benchmark: Option<ManifestBenchmark>,
    pub list_benchmark: Option<ListBenchmark>,
}

impl BenchmarkSuite {
    pub fn print_report(&self) {
        println!("=== Plugin System Performance Benchmark ===\n");
        
        if let Some(load_ms) = self.extension_load_ms {
            println!("Extension Load Time: {} ms", load_ms);
            println!("  Target: < 100ms");
            println!("  Status: {}", if load_ms < 100 { "✅ PASS" } else { "❌ FAIL" });
            println!();
        }
        
        if let Some(storage) = &self.storage_benchmark {
            println!("Storage Operations ({} iterations):", storage.iterations);
            println!("  Total Write Time: {} ms", storage.write_time_ms);
            println!("  Total Read Time: {} ms", storage.read_time_ms);
            println!("  Avg Write: {} μs", storage.avg_write_us);
            println!("  Avg Read: {} μs", storage.avg_read_us);
            println!("  Target: < 100μs per operation");
            println!("  Status: {}", if storage.avg_write_us < 100 && storage.avg_read_us < 100 { "✅ PASS" } else { "❌ FAIL" });
            println!();
        }
        
        if let Some(script) = &self.content_script_benchmark {
            println!("Content Script Injection:");
            println!("  Script Length: {} bytes", script.script_length);
            println!("  Injection Time: {} ms", script.injection_time_ms);
            println!("  Target: < 10ms");
            println!("  Status: {}", if script.injection_time_ms < 10 { "✅ PASS" } else { "❌ FAIL" });
            println!();
        }
        
        if let Some(manifest) = &self.manifest_benchmark {
            println!("Manifest Parsing ({} iterations):", manifest.iterations);
            println!("  Total Time: {} ms", manifest.total_time_ms);
            println!("  Avg Parse: {} μs", manifest.avg_parse_us);
            println!("  Target: < 100μs per parse");
            println!("  Status: {}", if manifest.avg_parse_us < 100 { "✅ PASS" } else { "❌ FAIL" });
            println!();
        }
        
        if let Some(list) = &self.list_benchmark {
            println!("Extension List ({} iterations, {} extensions):", list.iterations, list.extension_count);
            println!("  Total Time: {} ms", list.total_time_ms);
            println!("  Avg List: {} μs", list.avg_list_us);
            println!("  Target: < 10μs per list");
            println!("  Status: {}", if list.avg_list_us < 10 { "✅ PASS" } else { "❌ FAIL" });
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_benchmark_storage_operations() {
        let temp_dir = std::env::temp_dir().join(format!("exodus_perf_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir).ok();
        
        let storage = ExtensionStorage::new(&temp_dir).ok();
        if let Some(s) = storage {
            let result = benchmark_storage_operations(&s, "test", 100);
            assert!(result.is_ok(), "Storage benchmark should complete");
            
            let bench = result.unwrap();
            assert!(bench.iterations == 100, "Should run 100 iterations");
        }
        
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_benchmark_content_script_injection() {
        let app_data_dir = std::env::temp_dir().join(format!("exodus_perf_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&app_data_dir).ok();
        
        let manager = ExtensionManager::new(&app_data_dir).ok();
        if let Some(mgr) = manager {
            let result = benchmark_content_script_injection(&mgr, "https://example.com");
            assert!(result.is_ok(), "Content script benchmark should complete");
        }
        
        let _ = fs::remove_dir_all(&app_data_dir);
    }
}
