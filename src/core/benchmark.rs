use crate::core::models::{BenchmarkProbeResult, MirrorBenchmarkSummary, MirrorDefinition};
use crate::infrastructure::http::build_http_client;
use std::time::{Duration, Instant};
use tracing::debug;

pub struct BenchmarkEngine {
    timeout_secs: u64,
}

impl BenchmarkEngine {
    pub fn new(timeout_secs: u64) -> Self {
        Self { timeout_secs }
    }

    /// Benchmark a single mirror across all its enabled ecosystem endpoints.
    pub async fn benchmark_mirror(&self, mirror: &MirrorDefinition) -> MirrorBenchmarkSummary {
        let mut probes = Vec::new();
        let client = match build_http_client(Some(self.timeout_secs)) {
            Ok(c) => c,
            Err(e) => {
                return MirrorBenchmarkSummary {
                    mirror_id: mirror.id.clone(),
                    mirror_name: mirror.name.clone(),
                    overall_reachable: false,
                    average_latency_ms: 0,
                    probes: vec![BenchmarkProbeResult {
                        provider_id: "http_client".into(),
                        endpoint_url: mirror.website.clone(),
                        reachable: false,
                        http_status: None,
                        dns_ms: None,
                        tcp_ms: None,
                        tls_ms: None,
                        total_latency_ms: 0,
                        error: Some(e.to_string()),
                    }],
                };
            }
        };

        // 1. Probe Dart / Pub if enabled
        if let Some(ref dart) = mirror.providers.dart {
            if dart.enabled {
                let pkg = dart.test_package.as_deref().unwrap_or("http");
                let url = format!(
                    "{}/api/packages/{}",
                    dart.hosted_url.trim_end_matches('/'),
                    pkg
                );
                let res = self
                    .probe_endpoint(&client, "dart", &url, Some("name"))
                    .await;
                probes.push(res);
            }
        }

        // 2. Probe Flutter Storage if enabled
        if let Some(ref flutter) = mirror.providers.flutter {
            if flutter.enabled {
                let path = flutter
                    .test_path
                    .as_deref()
                    .unwrap_or("flutter_infra_release/releases/releases_linux.json");
                let url = format!(
                    "{}/{}",
                    flutter.storage_url.trim_end_matches('/'),
                    path.trim_start_matches('/')
                );
                let res = self.probe_endpoint(&client, "flutter", &url, None).await;
                probes.push(res);
            }
        }

        // 3. Probe Gradle Maven Central if enabled
        if let Some(ref gradle) = mirror.providers.gradle {
            if gradle.enabled {
                if let Some(ref mc_url) = gradle.maven_central_url {
                    let url = format!(
                        "{}/org/jetbrains/kotlin/kotlin-stdlib/1.9.0/kotlin-stdlib-1.9.0.pom",
                        mc_url.trim_end_matches('/')
                    );
                    let res = self
                        .probe_endpoint(&client, "gradle (maven central)", &url, None)
                        .await;
                    probes.push(res);
                }
                if let Some(ref gm_url) = gradle.google_maven_url {
                    let url = format!(
                        "{}/androidx/appcompat/appcompat/1.6.1/appcompat-1.6.1.pom",
                        gm_url.trim_end_matches('/')
                    );
                    let res = self
                        .probe_endpoint(&client, "gradle (google maven)", &url, None)
                        .await;
                    probes.push(res);
                }
            }
        }

        // Compute summary metrics
        let total_probes = probes.len();
        let successful_probes: Vec<&BenchmarkProbeResult> =
            probes.iter().filter(|p| p.reachable).collect();
        let overall_reachable = !successful_probes.is_empty();

        let avg_latency = if !successful_probes.is_empty() {
            let sum: u64 = successful_probes.iter().map(|p| p.total_latency_ms).sum();
            sum / successful_probes.len() as u64
        } else {
            0
        };

        debug!(
            "Benchmark for {}: {}/{} probes succeeded, avg latency: {}ms",
            mirror.id,
            successful_probes.len(),
            total_probes,
            avg_latency
        );

        MirrorBenchmarkSummary {
            mirror_id: mirror.id.clone(),
            mirror_name: mirror.name.clone(),
            overall_reachable,
            average_latency_ms: avg_latency,
            probes,
        }
    }

    /// Benchmark a list of mirrors sequentially to avoid saturating local network.
    pub async fn benchmark_all(&self, mirrors: &[MirrorDefinition]) -> Vec<MirrorBenchmarkSummary> {
        let mut summaries = Vec::new();
        for mirror in mirrors {
            summaries.push(self.benchmark_mirror(mirror).await);
        }

        // Sort by quality score (reachable first, then lower average latency)
        summaries.sort_by(|a, b| {
            b.overall_reachable
                .cmp(&a.overall_reachable)
                .then_with(|| a.average_latency_ms.cmp(&b.average_latency_ms))
        });

        summaries
    }

    async fn probe_endpoint(
        &self,
        client: &reqwest::Client,
        provider_id: &str,
        url: &str,
        expected_substring: Option<&str>,
    ) -> BenchmarkProbeResult {
        let start = Instant::now();
        let timeout = Duration::from_secs(self.timeout_secs);

        match tokio::time::timeout(timeout, client.get(url).send()).await {
            Ok(Ok(resp)) => {
                let status = resp.status();
                let is_http_success = status.is_success();

                if is_http_success {
                    if let Some(expected) = expected_substring {
                        match resp.text().await {
                            Ok(body) if body.contains(expected) => BenchmarkProbeResult {
                                provider_id: provider_id.to_string(),
                                endpoint_url: url.to_string(),
                                reachable: true,
                                http_status: Some(status.as_u16()),
                                dns_ms: None,
                                tcp_ms: None,
                                tls_ms: None,
                                total_latency_ms: start.elapsed().as_millis() as u64,
                                error: None,
                            },
                            Ok(_) => BenchmarkProbeResult {
                                provider_id: provider_id.to_string(),
                                endpoint_url: url.to_string(),
                                reachable: false,
                                http_status: Some(status.as_u16()),
                                dns_ms: None,
                                tcp_ms: None,
                                tls_ms: None,
                                total_latency_ms: start.elapsed().as_millis() as u64,
                                error: Some(format!(
                                    "Payload did not contain expected content '{expected}'"
                                )),
                            },
                            Err(e) => BenchmarkProbeResult {
                                provider_id: provider_id.to_string(),
                                endpoint_url: url.to_string(),
                                reachable: false,
                                http_status: Some(status.as_u16()),
                                dns_ms: None,
                                tcp_ms: None,
                                tls_ms: None,
                                total_latency_ms: start.elapsed().as_millis() as u64,
                                error: Some(e.to_string()),
                            },
                        }
                    } else {
                        BenchmarkProbeResult {
                            provider_id: provider_id.to_string(),
                            endpoint_url: url.to_string(),
                            reachable: true,
                            http_status: Some(status.as_u16()),
                            dns_ms: None,
                            tcp_ms: None,
                            tls_ms: None,
                            total_latency_ms: start.elapsed().as_millis() as u64,
                            error: None,
                        }
                    }
                } else {
                    BenchmarkProbeResult {
                        provider_id: provider_id.to_string(),
                        endpoint_url: url.to_string(),
                        reachable: false,
                        http_status: Some(status.as_u16()),
                        dns_ms: None,
                        tcp_ms: None,
                        tls_ms: None,
                        total_latency_ms: start.elapsed().as_millis() as u64,
                        error: Some(format!("HTTP {}", status)),
                    }
                }
            }
            Ok(Err(e)) => BenchmarkProbeResult {
                provider_id: provider_id.to_string(),
                endpoint_url: url.to_string(),
                reachable: false,
                http_status: None,
                dns_ms: None,
                tcp_ms: None,
                tls_ms: None,
                total_latency_ms: start.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
            },
            Err(_) => BenchmarkProbeResult {
                provider_id: provider_id.to_string(),
                endpoint_url: url.to_string(),
                reachable: false,
                http_status: None,
                dns_ms: None,
                tcp_ms: None,
                tls_ms: None,
                total_latency_ms: timeout.as_millis() as u64,
                error: Some(format!("Timed out after {}s", self.timeout_secs)),
            },
        }
    }
}
