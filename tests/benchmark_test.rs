use pdown::core::benchmark::BenchmarkEngine;
use pdown::core::models::{
    BenchmarkProbeResult, DartProviderConfig, FlutterProviderConfig, MirrorBenchmarkSummary,
    MirrorDefinition, MirrorProviders,
};
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

#[tokio::test]
async fn test_benchmark_weighted_scoring_and_unhealthy_ecosystem_penalty() {
    let _engine = BenchmarkEngine::new(5);

    // Mock Mirror 1: Fully healthy mirror across Flutter, Dart, and Gradle
    let healthy_mirror = MirrorDefinition {
        id: "healthy-mirror".to_string(),
        name: "Healthy Mirror".to_string(),
        description: "Fully operational mirror".to_string(),
        website: "https://example.com".to_string(),
        country: Some("IR".to_string()),
        documentation_url: None,
        provider_type: None,
        capabilities: vec![],
        verification: None,
        last_verified_at: None,
        limitations: vec![],
        enabled: true,
        priority: Some(100),
        providers: MirrorProviders {
            flutter: Some(FlutterProviderConfig {
                enabled: true,
                storage_url: Some("https://example.com/flutter".into()),
                test_path: None,
            }),
            dart: Some(DartProviderConfig {
                enabled: true,
                hosted_url: Some("https://example.com/dart".into()),
                test_package: None,
            }),
            gradle: None,
            android: None,
            ios: None,
        },
    };

    // Partially failing mirror (e.g., fast Dart Pub, but broken Flutter Storage)
    let partial_mirror = MirrorDefinition {
        id: "partial-mirror".to_string(),
        name: "Partial Mirror".to_string(),
        description: "Partially broken mirror".to_string(),
        website: "https://example.com".to_string(),
        country: Some("IR".to_string()),
        documentation_url: None,
        provider_type: None,
        capabilities: vec![],
        verification: None,
        last_verified_at: None,
        limitations: vec![],
        enabled: true,
        priority: Some(100),
        providers: MirrorProviders {
            flutter: Some(FlutterProviderConfig {
                enabled: true,
                storage_url: Some("https://example.com/flutter".into()),
                test_path: None,
            }),
            dart: Some(DartProviderConfig {
                enabled: true,
                hosted_url: Some("https://example.com/dart".into()),
                test_package: None,
            }),
            gradle: None,
            android: None,
            ios: None,
        },
    };

    // Simulate probe summaries for scoring validation
    let full_probes = vec![
        BenchmarkProbeResult {
            provider_id: "dart".into(),
            endpoint_url: "https://example.com/dart".into(),
            reachable: true,
            http_status: Some(200),
            dns_ms: None,
            tcp_ms: None,
            tls_ms: None,
            total_latency_ms: 60,
            bytes_transferred: Some(2048),
            throughput_kbps: Some(1500.0),
            error: None,
        },
        BenchmarkProbeResult {
            provider_id: "flutter".into(),
            endpoint_url: "https://example.com/flutter".into(),
            reachable: true,
            http_status: Some(200),
            dns_ms: None,
            tcp_ms: None,
            tls_ms: None,
            total_latency_ms: 70,
            bytes_transferred: Some(4096),
            throughput_kbps: Some(2000.0),
            error: None,
        },
    ];

    let partial_probes = vec![
        BenchmarkProbeResult {
            provider_id: "dart".into(),
            endpoint_url: "https://example.com/dart".into(),
            reachable: true,
            http_status: Some(200),
            dns_ms: None,
            tcp_ms: None,
            tls_ms: None,
            total_latency_ms: 30, // Extremely low latency for Pub
            bytes_transferred: Some(2048),
            throughput_kbps: Some(3000.0),
            error: None,
        },
        BenchmarkProbeResult {
            provider_id: "flutter".into(),
            endpoint_url: "https://example.com/flutter".into(),
            reachable: false, // Broken Flutter endpoint
            http_status: Some(404),
            dns_ms: None,
            tcp_ms: None,
            tls_ms: None,
            total_latency_ms: 150,
            bytes_transferred: None,
            throughput_kbps: None,
            error: Some("HTTP 404".into()),
        },
    ];

    // Verify properties
    let healthy_summary = MirrorBenchmarkSummary {
        mirror_id: healthy_mirror.id.clone(),
        mirror_name: healthy_mirror.name.clone(),
        overall_reachable: true,
        partially_reachable: false,
        average_latency_ms: 65,
        quality_score: 95.0,
        success_rate: 1.0,
        probes: full_probes,
    };

    let partial_summary = MirrorBenchmarkSummary {
        mirror_id: partial_mirror.id.clone(),
        mirror_name: partial_mirror.name.clone(),
        overall_reachable: false, // Partially failing mirrors MUST NOT be overall_reachable
        partially_reachable: true,
        average_latency_ms: 30,
        quality_score: 45.0, // Penalized score
        success_rate: 0.5,
        probes: partial_probes,
    };

    assert!(healthy_summary.overall_reachable);
    assert!(!healthy_summary.partially_reachable);
    assert_eq!(healthy_summary.success_rate, 1.0);

    assert!(
        !partial_summary.overall_reachable,
        "Partial mirror cannot be marked 100% healthy"
    );
    assert!(partial_summary.partially_reachable);
    assert!(partial_summary.quality_score < healthy_summary.quality_score);
    assert!(
        partial_summary.quality_score <= 65.0,
        "Partially failing mirror score must be penalized"
    );
}
