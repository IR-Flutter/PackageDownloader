use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Complete Mirror Registry Document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegistryDocument {
    pub schema_version: u32,
    pub registry_version: String,
    pub revision: u64,
    pub updated_at: DateTime<Utc>,
    pub mirrors: Vec<MirrorDefinition>,
}

/// A single mirror definition supporting multiple providers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MirrorDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub website: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub priority: Option<u32>,
    pub providers: MirrorProviders,
}

fn default_true() -> bool {
    true
}

/// Supported provider configurations for a mirror.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MirrorProviders {
    #[serde(default)]
    pub flutter: Option<FlutterProviderConfig>,
    #[serde(default)]
    pub dart: Option<DartProviderConfig>,
    #[serde(default)]
    pub gradle: Option<GradleProviderConfig>,
    #[serde(default)]
    pub android: Option<AndroidProviderConfig>,
    #[serde(default)]
    pub ios: Option<IosProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlutterProviderConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub storage_url: String,
    #[serde(default)]
    pub test_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DartProviderConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub hosted_url: String,
    #[serde(default)]
    pub test_package: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GradleProviderConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub maven_central_url: Option<String>,
    #[serde(default)]
    pub google_maven_url: Option<String>,
    #[serde(default)]
    pub gradle_plugins_url: Option<String>,
    #[serde(default)]
    pub distribution_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AndroidProviderConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub google_maven_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IosProviderConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Configuration Scope: Global vs Project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Global,
    Project,
}

/// Status of an individual ecosystem provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderStatus {
    pub provider_id: String,
    pub is_active: bool,
    pub current_endpoint: Option<String>,
    pub details: HashMap<String, String>,
}

/// A planned change to a file or environment setting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangePlan {
    pub provider_id: String,
    pub description: String,
    pub items: Vec<ChangeItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeItem {
    FileWrite {
        path: PathBuf,
        old_content: Option<String>,
        new_content: String,
        diff: String,
    },
    FileDelete {
        path: PathBuf,
        old_content: Option<String>,
    },
    EnvironmentSet {
        key: String,
        old_value: Option<String>,
        new_value: String,
    },
    EnvironmentUnset {
        key: String,
        old_value: Option<String>,
    },
}

/// Benchmark metric summary for a single endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkProbeResult {
    pub provider_id: String,
    pub endpoint_url: String,
    pub reachable: bool,
    pub http_status: Option<u16>,
    pub dns_ms: Option<u64>,
    pub tcp_ms: Option<u64>,
    pub tls_ms: Option<u64>,
    pub total_latency_ms: u64,
    pub error: Option<String>,
}

/// Overall benchmark outcome for a mirror.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MirrorBenchmarkSummary {
    pub mirror_id: String,
    pub mirror_name: String,
    pub overall_reachable: bool,
    pub average_latency_ms: u64,
    pub probes: Vec<BenchmarkProbeResult>,
}

/// Multi-tenant configuration profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProfileConfig {
    pub name: String,
    pub active_mirror_id: Option<String>,
    pub enabled_providers: Vec<String>,
    pub custom_overrides: HashMap<String, String>,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            active_mirror_id: None,
            enabled_providers: vec![
                "flutter".to_string(),
                "dart".to_string(),
                "gradle".to_string(),
                "android".to_string(),
            ],
            custom_overrides: HashMap::new(),
        }
    }
}
