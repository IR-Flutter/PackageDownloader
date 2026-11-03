use crate::core::models::{BenchmarkProbeResult, MirrorBenchmarkSummary, MirrorDefinition};
use crate::infrastructure::http::build_http_client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BenchmarkRecommendations {
    pub best_overall: Option<String>,
    pub best_for_flutter: Option<String>,
    pub best_for_dart: Option<String>,
    pub best_for_gradle: Option<String>,
}

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
                    partially_reachable: false,
                    average_latency_ms: 0,
                    quality_score: 0.0,
                    success_rate: 0.0,
                    probes: vec![BenchmarkProbeResult {
                        provider_id: "http_client".into(),
                        endpoint_url: mirror.website.clone(),
                        reachable: false,
                        http_status: None,
                        dns_ms: None,
                        tcp_ms: None,
                        tls_ms: None,
                        total_latency_ms: 0,
                        bytes_transferred: None,
                        throughput_kbps: None,
                        error: Some(e.to_string()),
                    }],
                };
            }
        };

        // 1. Probe Dart / Pub if enabled
        if let Some(ref dart) = mirror.providers.dart {
            if dart.enabled {
                if let Some(ref hosted_url) = dart.hosted_url {
                    let pkg = dart.test_package.as_deref().unwrap_or("http");
                    let url = format!("{}/api/packages/{}", hosted_url.trim_end_matches('/'), pkg);
                    let res = self
                        .probe_endpoint(&client, "dart", &url, Some("name"))
                        .await;
                    probes.push(res);
                }
            }
        }

        // 2. Probe Flutter Storage if enabled
        if let Some(ref flutter) = mirror.providers.flutter {
            if flutter.enabled {
                if let Some(ref storage_url) = flutter.storage_url {
                    let path = flutter
                        .test_path
                        .as_deref()
                        .unwrap_or("flutter_infra_release/releases/releases_linux.json");
                    let url = format!(
                        "{}/{}",
                        storage_url.trim_end_matches('/'),
                        path.trim_start_matches('/')
                    );
                    let res = self.probe_endpoint(&client, "flutter", &url, None).await;
                    probes.push(res);
                }
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

        // Compute multi-factor quality scoring metrics
        let total_probes = probes.len();
        let successful_probes: Vec<&BenchmarkProbeResult> =
            probes.iter().filter(|p| p.reachable).collect();
        let overall_reachable = total_probes > 0 && successful_probes.len() == total_probes;
        let partially_reachable =
            !successful_probes.is_empty() && successful_probes.len() < total_probes;
        let success_rate = if total_probes > 0 {
            successful_probes.len() as f64 / total_probes as f64
        } else {
            0.0
        };

        let avg_latency = if !successful_probes.is_empty() {
            let sum: u64 = successful_probes.iter().map(|p| p.total_latency_ms).sum();
            sum / successful_probes.len() as u64
        } else {
            0
        };

        // 1. Availability factor (40%): proportion of enabled endpoints that responded
        let availability_score = success_rate * 100.0;

        // 2. Reliability factor (25%): HTTP 200 without network or content integrity errors
        let reliable_probes = probes
            .iter()
            .filter(|p| p.reachable && p.http_status == Some(200) && p.error.is_none())
            .count();
        let reliability_score = if total_probes > 0 {
            (reliable_probes as f64 / total_probes as f64) * 100.0
        } else {
            0.0
        };

        // 3. Latency factor (20%): graduated score based on average response time
        let latency_score = if successful_probes.is_empty() || avg_latency >= 3000 {
            0.0
        } else if avg_latency <= 100 {
            100.0
        } else {
            (1.0 - ((avg_latency - 100) as f64 / 2900.0)) * 100.0
        };

        // 4. Throughput factor (15%): data transfer rate in KB/s
        let throughput_sum: f64 = successful_probes
            .iter()
            .filter_map(|p| p.throughput_kbps)
            .sum();
        let throughput_count = successful_probes
            .iter()
            .filter(|p| p.throughput_kbps.is_some())
            .count();
        let avg_throughput = if throughput_count > 0 {
            throughput_sum / throughput_count as f64
        } else {
            0.0
        };
        let throughput_score = if avg_throughput >= 1000.0 {
            100.0
        } else if avg_throughput > 0.0 {
            (avg_throughput / 1000.0) * 100.0
        } else {
            0.0
        };

        // Multi-factor composite quality score
        let quality_score = if total_probes == 0 {
            0.0
        } else {
            let raw = (0.40 * availability_score)
                + (0.25 * reliability_score)
                + (0.20 * latency_score)
                + (0.15 * throughput_score);
            // Penalize partially failing mirrors so they are never treated as 100% healthy
            if partially_reachable {
                (raw * success_rate).min(65.0)
            } else {
                raw
            }
        };

        debug!(
            "Benchmark for {}: {}/{} probes succeeded, avg latency: {}ms, quality score: {:.1}",
            mirror.id,
            successful_probes.len(),
            total_probes,
            avg_latency,
            quality_score
        );

        MirrorBenchmarkSummary {
            mirror_id: mirror.id.clone(),
            mirror_name: mirror.name.clone(),
            overall_reachable,
            partially_reachable,
            average_latency_ms: avg_latency,
            quality_score,
            success_rate,
            probes,
        }
    }

    /// Benchmark a list of mirrors sequentially to avoid saturating local network.
    pub async fn benchmark_all(&self, mirrors: &[MirrorDefinition]) -> Vec<MirrorBenchmarkSummary> {
        let mut summaries = Vec::new();
        for mirror in mirrors {
            summaries.push(self.benchmark_mirror(mirror).await);
        }

        // Sort by quality score (highest composite quality first, then lower average latency)
        summaries.sort_by(|a, b| {
            b.quality_score
                .partial_cmp(&a.quality_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.overall_reachable.cmp(&a.overall_reachable))
                .then_with(|| a.average_latency_ms.cmp(&b.average_latency_ms))
        });

        summaries
    }

    /// Computes per-ecosystem and overall recommendations based on benchmark summaries
    /// and mirror definitions.
    /// Strictly excludes mirrors that are globally disabled or have disabled providers.
    pub fn compute_recommendations(
        summaries: &[MirrorBenchmarkSummary],
        mirrors: &[MirrorDefinition],
    ) -> BenchmarkRecommendations {
        let mut rec = BenchmarkRecommendations::default();

        // 1. Overall best: highest quality score among mirrors that are enabled and overall_reachable
        let best_overall = summaries.iter().find(|s| {
            if !s.overall_reachable {
                return false;
            }
            mirrors.iter().any(|m| m.id == s.mirror_id && m.enabled)
        });
        rec.best_overall = best_overall.map(|s| s.mirror_id.clone());

        // 2. Best for Flutter: mirror with enabled Flutter provider, with reachable flutter probe
        let mut best_flutter: Option<(&MirrorDefinition, u64)> = None;
        for s in summaries {
            if let Some(m) = mirrors.iter().find(|m| m.id == s.mirror_id && m.enabled) {
                if m.providers.flutter.as_ref().is_some_and(|f| f.enabled) {
                    if let Some(probe) = s
                        .probes
                        .iter()
                        .find(|p| p.provider_id == "flutter" && p.reachable)
                    {
                        match best_flutter {
                            None => best_flutter = Some((m, probe.total_latency_ms)),
                            Some((_, lowest_lat)) if probe.total_latency_ms < lowest_lat => {
                                best_flutter = Some((m, probe.total_latency_ms));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        rec.best_for_flutter = best_flutter.map(|(m, _)| m.id.clone());

        // 3. Best for Dart: mirror with enabled Dart provider, with reachable dart probe
        let mut best_dart: Option<(&MirrorDefinition, u64)> = None;
        for s in summaries {
            if let Some(m) = mirrors.iter().find(|m| m.id == s.mirror_id && m.enabled) {
                if m.providers.dart.as_ref().is_some_and(|d| d.enabled) {
                    if let Some(probe) = s
                        .probes
                        .iter()
                        .find(|p| p.provider_id == "dart" && p.reachable)
                    {
                        match best_dart {
                            None => best_dart = Some((m, probe.total_latency_ms)),
                            Some((_, lowest_lat)) if probe.total_latency_ms < lowest_lat => {
                                best_dart = Some((m, probe.total_latency_ms));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        rec.best_for_dart = best_dart.map(|(m, _)| m.id.clone());

        // 4. Best for Gradle: mirror with enabled Gradle provider, with reachable gradle probe
        let mut best_gradle: Option<(&MirrorDefinition, u64)> = None;
        for s in summaries {
            if let Some(m) = mirrors.iter().find(|m| m.id == s.mirror_id && m.enabled) {
                if m.providers.gradle.as_ref().is_some_and(|g| g.enabled) {
                    let reachable_gradle_probes: Vec<&BenchmarkProbeResult> = s
                        .probes
                        .iter()
                        .filter(|p| p.provider_id.starts_with("gradle") && p.reachable)
                        .collect();
                    if !reachable_gradle_probes.is_empty() {
                        let avg_gradle_lat: u64 = reachable_gradle_probes
                            .iter()
                            .map(|p| p.total_latency_ms)
                            .sum::<u64>()
                            / reachable_gradle_probes.len() as u64;
                        match best_gradle {
                            None => best_gradle = Some((m, avg_gradle_lat)),
                            Some((_, lowest_lat)) if avg_gradle_lat < lowest_lat => {
                                best_gradle = Some((m, avg_gradle_lat));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        rec.best_for_gradle = best_gradle.map(|(m, _)| m.id.clone());

        rec
    }

    /// Execute a multi-sample probe: runs two consecutive samples to smooth network variance
    /// and separates latency from throughput metrics.
    async fn probe_endpoint(
        &self,
        client: &reqwest::Client,
        provider_id: &str,
        url: &str,
        expected_substring: Option<&str>,
    ) -> BenchmarkProbeResult {
        let sample1 = self
            .execute_single_probe(client, provider_id, url, expected_substring)
            .await;
        if !sample1.reachable {
            return sample1;
        }

        let sample2 = self
            .execute_single_probe(client, provider_id, url, expected_substring)
            .await;
        if !sample2.reachable {
            return sample1;
        }

        let avg_latency = (sample1.total_latency_ms + sample2.total_latency_ms) / 2;
        let avg_throughput = match (sample1.throughput_kbps, sample2.throughput_kbps) {
            (Some(t1), Some(t2)) => Some((t1 + t2) / 2.0),
            (Some(t1), None) => Some(t1),
            (None, Some(t2)) => Some(t2),
            (None, None) => None,
        };

        BenchmarkProbeResult {
            provider_id: provider_id.to_string(),
            endpoint_url: url.to_string(),
            reachable: true,
            http_status: sample2.http_status.or(sample1.http_status),
            dns_ms: None,
            tcp_ms: None,
            tls_ms: None,
            total_latency_ms: avg_latency,
            bytes_transferred: sample2.bytes_transferred.or(sample1.bytes_transferred),
            throughput_kbps: avg_throughput,
            error: None,
        }
    }

    async fn execute_single_probe(
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
                    let transfer_start = Instant::now();
                    match resp.bytes().await {
                        Ok(bytes) => {
                            let total_latency_ms = start.elapsed().as_millis() as u64;
                            let transfer_duration = transfer_start.elapsed().as_secs_f64();
                            let bytes_len = bytes.len();
                            let throughput_kbps = if transfer_duration > 0.0001 {
                                Some((bytes_len as f64 / 1024.0) / transfer_duration)
                            } else if total_latency_ms > 0 {
                                Some(
                                    (bytes_len as f64 / 1024.0)
                                        / (total_latency_ms as f64 / 1000.0),
                                )
                            } else {
                                None
                            };

                            if let Some(expected) = expected_substring {
                                let body_text = String::from_utf8_lossy(&bytes);
                                if body_text.contains(expected) {
                                    BenchmarkProbeResult {
                                        provider_id: provider_id.to_string(),
                                        endpoint_url: url.to_string(),
                                        reachable: true,
                                        http_status: Some(status.as_u16()),
                                        dns_ms: None,
                                        tcp_ms: None,
                                        tls_ms: None,
                                        total_latency_ms,
                                        bytes_transferred: Some(bytes_len),
                                        throughput_kbps,
                                        error: None,
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
                                        total_latency_ms,
                                        bytes_transferred: Some(bytes_len),
                                        throughput_kbps,
                                        error: Some(format!(
                                            "Payload did not contain expected content '{expected}'"
                                        )),
                                    }
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
                                    total_latency_ms,
                                    bytes_transferred: Some(bytes_len),
                                    throughput_kbps,
                                    error: None,
                                }
                            }
                        }
                        Err(e) => {
                            let total_latency_ms = start.elapsed().as_millis() as u64;
                            BenchmarkProbeResult {
                                provider_id: provider_id.to_string(),
                                endpoint_url: url.to_string(),
                                reachable: false,
                                http_status: Some(status.as_u16()),
                                dns_ms: None,
                                tcp_ms: None,
                                tls_ms: None,
                                total_latency_ms,
                                bytes_transferred: None,
                                throughput_kbps: None,
                                error: Some(e.to_string()),
                            }
                        }
                    }
                } else {
                    let total_latency_ms = start.elapsed().as_millis() as u64;
                    BenchmarkProbeResult {
                        provider_id: provider_id.to_string(),
                        endpoint_url: url.to_string(),
                        reachable: false,
                        http_status: Some(status.as_u16()),
                        dns_ms: None,
                        tcp_ms: None,
                        tls_ms: None,
                        total_latency_ms,
                        bytes_transferred: None,
                        throughput_kbps: None,
                        error: Some(format!("HTTP {}", status)),
                    }
                }
            }
            Ok(Err(e)) => {
                let total_latency_ms = start.elapsed().as_millis() as u64;
                BenchmarkProbeResult {
                    provider_id: provider_id.to_string(),
                    endpoint_url: url.to_string(),
                    reachable: false,
                    http_status: None,
                    dns_ms: None,
                    tcp_ms: None,
                    tls_ms: None,
                    total_latency_ms,
                    bytes_transferred: None,
                    throughput_kbps: None,
                    error: Some(e.to_string()),
                }
            }
            Err(_) => {
                let total_latency_ms = timeout.as_millis() as u64;
                BenchmarkProbeResult {
                    provider_id: provider_id.to_string(),
                    endpoint_url: url.to_string(),
                    reachable: false,
                    http_status: None,
                    dns_ms: None,
                    tcp_ms: None,
                    tls_ms: None,
                    total_latency_ms,
                    bytes_transferred: None,
                    throughput_kbps: None,
                    error: Some(format!("Probe timed out after {}s", self.timeout_secs)),
                }
            }
        }
    }
}
