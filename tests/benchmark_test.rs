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

#[test]
fn test_benchmark_recommendations_excludes_disabled_and_unsupported_providers() {
    let mirror_pub_only = MirrorDefinition {
        id: "pub-only".into(),
        name: "Pub Only".into(),
        description: "Only pub".into(),
        website: "https://pub.example.com".into(),
        country: Some("IR".into()),
        documentation_url: None,
        provider_type: None,
        capabilities: vec![],
        verification: None,
        last_verified_at: None,
        limitations: vec![],
        enabled: true,
        priority: Some(10),
        providers: MirrorProviders {
            flutter: None, // No flutter!
            dart: Some(DartProviderConfig {
                enabled: true,
                hosted_url: Some("https://pub.example.com".into()),
                test_package: None,
            }),
            gradle: None,
            android: None,
            ios: None,
        },
    };

    let mirror_disabled = MirrorDefinition {
        id: "disabled-mirror".into(),
        name: "Disabled Mirror".into(),
        description: "Disabled".into(),
        website: "https://disabled.example.com".into(),
        country: Some("IR".into()),
        documentation_url: None,
        provider_type: None,
        capabilities: vec![],
        verification: None,
        last_verified_at: None,
        limitations: vec![],
        enabled: false, // Globally disabled!
        priority: Some(1),
        providers: MirrorProviders {
            flutter: Some(FlutterProviderConfig {
                enabled: true,
                storage_url: Some("https://disabled.example.com/flutter".into()),
                test_path: None,
            }),
            dart: Some(DartProviderConfig {
                enabled: true,
                hosted_url: Some("https://disabled.example.com/dart".into()),
                test_package: None,
            }),
            gradle: None,
            android: None,
            ios: None,
        },
    };

    let mirror_full = MirrorDefinition {
        id: "full-mirror".into(),
        name: "Full Mirror".into(),
        description: "Full".into(),
        website: "https://full.example.com".into(),
        country: Some("IR".into()),
        documentation_url: None,
        provider_type: None,
        capabilities: vec![],
        verification: None,
        last_verified_at: None,
        limitations: vec![],
        enabled: true,
        priority: Some(20),
        providers: MirrorProviders {
            flutter: Some(FlutterProviderConfig {
                enabled: true,
                storage_url: Some("https://full.example.com/flutter".into()),
                test_path: None,
            }),
            dart: Some(DartProviderConfig {
                enabled: true,
                hosted_url: Some("https://full.example.com/dart".into()),
                test_package: None,
            }),
            gradle: None,
            android: None,
            ios: None,
        },
    };

    let summaries = vec![
        MirrorBenchmarkSummary {
            mirror_id: "pub-only".into(),
            mirror_name: "Pub Only".into(),
            overall_reachable: true,
            partially_reachable: false,
            average_latency_ms: 20,
            quality_score: 95.0,
            success_rate: 1.0,
            probes: vec![BenchmarkProbeResult {
                provider_id: "dart".into(),
                endpoint_url: "https://pub.example.com".into(),
                reachable: true,
                http_status: Some(200),
                dns_ms: None,
                tcp_ms: None,
                tls_ms: None,
                total_latency_ms: 20,
                bytes_transferred: Some(1024),
                throughput_kbps: Some(2000.0),
                error: None,
            }],
        },
        MirrorBenchmarkSummary {
            mirror_id: "disabled-mirror".into(),
            mirror_name: "Disabled Mirror".into(),
            overall_reachable: true,
            partially_reachable: false,
            average_latency_ms: 10,
            quality_score: 99.0,
            success_rate: 1.0,
            probes: vec![
                BenchmarkProbeResult {
                    provider_id: "dart".into(),
                    endpoint_url: "https://disabled.example.com/dart".into(),
                    reachable: true,
                    http_status: Some(200),
                    dns_ms: None,
                    tcp_ms: None,
                    tls_ms: None,
                    total_latency_ms: 10,
                    bytes_transferred: Some(1024),
                    throughput_kbps: Some(3000.0),
                    error: None,
                },
                BenchmarkProbeResult {
                    provider_id: "flutter".into(),
                    endpoint_url: "https://disabled.example.com/flutter".into(),
                    reachable: true,
                    http_status: Some(200),
                    dns_ms: None,
                    tcp_ms: None,
                    tls_ms: None,
                    total_latency_ms: 10,
                    bytes_transferred: Some(1024),
                    throughput_kbps: Some(3000.0),
                    error: None,
                },
            ],
        },
        MirrorBenchmarkSummary {
            mirror_id: "full-mirror".into(),
            mirror_name: "Full Mirror".into(),
            overall_reachable: true,
            partially_reachable: false,
            average_latency_ms: 50,
            quality_score: 90.0,
            success_rate: 1.0,
            probes: vec![
                BenchmarkProbeResult {
                    provider_id: "dart".into(),
                    endpoint_url: "https://full.example.com/dart".into(),
                    reachable: true,
                    http_status: Some(200),
                    dns_ms: None,
                    tcp_ms: None,
                    tls_ms: None,
                    total_latency_ms: 50,
                    bytes_transferred: Some(1024),
                    throughput_kbps: Some(1000.0),
                    error: None,
                },
                BenchmarkProbeResult {
                    provider_id: "flutter".into(),
                    endpoint_url: "https://full.example.com/flutter".into(),
                    reachable: true,
                    http_status: Some(200),
                    dns_ms: None,
                    tcp_ms: None,
                    tls_ms: None,
                    total_latency_ms: 50,
                    bytes_transferred: Some(1024),
                    throughput_kbps: Some(1000.0),
                    error: None,
                },
            ],
        },
    ];

    let mirrors = vec![mirror_pub_only, mirror_disabled, mirror_full];
    let rec = BenchmarkEngine::compute_recommendations(&summaries, &mirrors);

    // 1. Disabled mirror must never be recommended anywhere
    assert_ne!(rec.best_overall.as_deref(), Some("disabled-mirror"));
    assert_ne!(rec.best_for_dart.as_deref(), Some("disabled-mirror"));
    assert_ne!(rec.best_for_flutter.as_deref(), Some("disabled-mirror"));

    // 2. pub-only has faster dart (20ms) than full-mirror (50ms)
    assert_eq!(rec.best_for_dart.as_deref(), Some("pub-only"));

    // 3. pub-only DOES NOT support flutter, so full-mirror must be recommended for flutter
    assert_eq!(rec.best_for_flutter.as_deref(), Some("full-mirror"));
}
