use pdown::core::benchmark::BenchmarkEngine;
use pdown::core::registry::RegistryManager;
use pdown::infrastructure::storage::StorageManager;
use tempfile::tempdir;

#[tokio::test]
async fn test_benchmark_engine_runs_and_returns_valid_metrics() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::with_custom_root(tmp.path());
    let reg = RegistryManager::new(storage);

    let pub_azs = reg.find_mirror("pub-azs").unwrap().unwrap();
    let engine = BenchmarkEngine::new(5);

    let summary = engine.benchmark_mirror(&pub_azs).await;
    assert_eq!(summary.mirror_id, "pub-azs");
    assert!(!summary.probes.is_empty());
}
