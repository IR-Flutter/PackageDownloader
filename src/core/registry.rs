use crate::core::errors::{PdownError, Result};
use crate::core::models::{MirrorDefinition, RegistryDocument};
use crate::infrastructure::crypto::{verify_signature, EMBEDDED_REGISTRY_PUBLIC_KEY};
use crate::infrastructure::http::build_http_client;
use crate::infrastructure::storage::StorageManager;
use crate::system::filesystem::atomic_write_bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};
use uuid::Uuid;

pub const CANONICAL_REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/IR-Flutter/PackageDownloader/main/registry/mirrors.json";
pub const CANONICAL_SIGNATURE_URL: &str =
    "https://raw.githubusercontent.com/IR-Flutter/PackageDownloader/main/registry/mirrors.json.sig";

pub const CURRENT_SUPPORTED_SCHEMA_VERSION: u32 = 1;

pub const BUNDLED_REGISTRY_JSON: &str = include_str!("../../registry/mirrors.json");
pub const BUNDLED_REGISTRY_SIG: &str = include_str!("../../registry/mirrors.json.sig");
pub const REGISTRY_SCHEMA_JSON: &str = include_str!("../../registry/schema.json");

/// Persistent monotonic state tracking accepted registry revisions across restarts and cache clears.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentRegistryState {
    pub highest_accepted_revision: u64,
    pub active_generation: Option<String>,
    pub last_known_good_generation: Option<String>,
    pub last_checked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct RegistryManager {
    storage: StorageManager,
    public_key_b64: String,
    remote_registry_url: String,
    remote_signature_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistryStatus {
    pub active_source: &'static str,
    pub registry_version: String,
    pub revision: u64,
    pub schema_version: u32,
    pub updated_at: DateTime<Utc>,
    pub mirrors_count: usize,
    pub is_signature_valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistryCheckResult {
    pub update_available: bool,
    pub current_revision: u64,
    pub remote_revision: u64,
    pub current_version: String,
    pub remote_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistryUpdateResult {
    pub previous_revision: u64,
    pub new_revision: u64,
    pub new_version: String,
}

impl RegistryManager {
    pub fn new(storage: StorageManager) -> Self {
        Self {
            storage,
            public_key_b64: EMBEDDED_REGISTRY_PUBLIC_KEY.to_string(),
            remote_registry_url: CANONICAL_REGISTRY_URL.to_string(),
            remote_signature_url: CANONICAL_SIGNATURE_URL.to_string(),
        }
    }

    pub fn with_custom_endpoints(
        storage: StorageManager,
        registry_url: String,
        signature_url: String,
        public_key_b64: String,
    ) -> Self {
        Self {
            storage,
            public_key_b64,
            remote_registry_url: registry_url,
            remote_signature_url: signature_url,
        }
    }

    /// Load or initialize persistent anti-rollback and generation tracking state.
    pub fn load_persistent_state(&self) -> Result<PersistentRegistryState> {
        let state_path = self.storage.registry_state_path();
        if state_path.exists() {
            if let Ok(bytes) = fs::read(&state_path) {
                if let Ok(state) = serde_json::from_slice::<PersistentRegistryState>(&bytes) {
                    return Ok(state);
                }
            }
        }

        // Initialize state tracking using bundled registry revision as baseline
        let bundled: RegistryDocument = serde_json::from_str(BUNDLED_REGISTRY_JSON)?;
        Ok(PersistentRegistryState {
            highest_accepted_revision: bundled.revision,
            active_generation: None,
            last_known_good_generation: None,
            last_checked_at: None,
        })
    }

    /// Persist the anti-rollback state atomically.
    pub fn save_persistent_state(&self, state: &PersistentRegistryState) -> Result<()> {
        self.storage.ensure_dirs()?;
        let json_bytes = serde_json::to_vec_pretty(state)?;
        atomic_write_bytes(&self.storage.registry_state_path(), &json_bytes, None)?;
        Ok(())
    }

    /// Explicit administrative recovery to reset anti-rollback state to the bundled baseline.
    pub fn reset_anti_rollback_state(&self) -> Result<()> {
        let bundled: RegistryDocument = serde_json::from_str(BUNDLED_REGISTRY_JSON)?;
        let state = PersistentRegistryState {
            highest_accepted_revision: bundled.revision,
            active_generation: None,
            last_known_good_generation: None,
            last_checked_at: None,
        };
        self.save_persistent_state(&state)?;
        info!("Administrative reset: Anti-rollback state reset to bundled revision baseline.");
        Ok(())
    }

    /// Load the bundled registry compiled into the binary.
    pub fn load_bundled(&self) -> Result<RegistryDocument> {
        Self::validate_json_schema(BUNDLED_REGISTRY_JSON.as_bytes())?;
        let doc: RegistryDocument = serde_json::from_str(BUNDLED_REGISTRY_JSON)?;
        Self::validate_semantics(&doc)?;
        Ok(doc)
    }

    /// Load and validate cached registry generation if present.
    /// Supports generation directories, automatic recovery from last-known-good, and exact byte verification.
    pub fn load_cached(&self) -> Result<Option<RegistryDocument>> {
        let state = self.load_persistent_state()?;

        // 1. Try loading from active generation directory
        if let Some(ref gen_id) = state.active_generation {
            let gen_dir = self.storage.registry_generations_dir().join(gen_id);
            if let Ok(Some(doc)) = self.load_generation_from_dir(&gen_dir) {
                return Ok(Some(doc));
            }
            warn!("Active generation '{gen_id}' is missing or corrupted. Attempting recovery to last known good...");

            // 2. Try loading from last-known-good generation directory
            if let Some(ref lkg_id) = state.last_known_good_generation {
                let lkg_dir = self.storage.registry_generations_dir().join(lkg_id);
                if let Ok(Some(doc)) = self.load_generation_from_dir(&lkg_dir) {
                    info!("Successfully recovered registry from last-known-good generation '{lkg_id}'");
                    let mut updated_state = state.clone();
                    updated_state.active_generation = Some(lkg_id.clone());
                    let _ = self.save_persistent_state(&updated_state);
                    return Ok(Some(doc));
                }
            }
        }

        // 3. Fallback to direct cache files if present (backwards compatibility)
        let cache_path = self.storage.cached_registry_path();
        let sig_path = self.storage.cached_signature_path();

        if !cache_path.exists() || !sig_path.exists() {
            return Ok(None);
        }

        let content = fs::read(&cache_path)?;
        let signature_str = fs::read_to_string(&sig_path)?;

        // Verify cryptographic signature against exact persisted bytes
        if let Err(e) = verify_signature(&content, &signature_str, &self.public_key_b64) {
            warn!("Cached registry signature verification failed: {e}. Discarding cache.");
            return Ok(None);
        }

        // Validate JSON Schema
        if let Err(e) = Self::validate_json_schema(&content) {
            warn!("Cached registry JSON Schema validation failed: {e}. Discarding cache.");
            return Ok(None);
        }

        let doc: RegistryDocument = match serde_json::from_slice(&content) {
            Ok(d) => d,
            Err(e) => {
                warn!("Cached registry is malformed JSON: {e}. Discarding cache.");
                return Ok(None);
            }
        };

        if let Err(e) = Self::validate_semantics(&doc) {
            warn!("Cached registry semantic validation failed: {e}. Discarding cache.");
            return Ok(None);
        }

        Ok(Some(doc))
    }

    fn load_generation_from_dir(&self, dir: &Path) -> Result<Option<RegistryDocument>> {
        let json_file = dir.join("mirrors.json");
        let sig_file = dir.join("mirrors.json.sig");

        if !json_file.exists() || !sig_file.exists() {
            return Ok(None);
        }

        let json_bytes = fs::read(&json_file)?;
        let sig_text = fs::read_to_string(&sig_file)?;

        if let Err(e) = verify_signature(&json_bytes, &sig_text, &self.public_key_b64) {
            warn!("Generation at '{}' failed signature verification: {e}", dir.display());
            return Ok(None);
        }

        if let Err(e) = Self::validate_json_schema(&json_bytes) {
            warn!("Generation at '{}' failed schema validation: {e}", dir.display());
            return Ok(None);
        }

        let doc: RegistryDocument = serde_json::from_slice(&json_bytes)?;
        Self::validate_semantics(&doc)?;

        Ok(Some(doc))
    }

    /// Load the active registry (cached if valid and not older than bundled, otherwise bundled).
    pub fn load_active(&self) -> Result<RegistryDocument> {
        let bundled = self.load_bundled()?;

        if let Ok(Some(cached)) = self.load_cached() {
            if cached.revision >= bundled.revision {
                debug!("Using cached registry (revision {})", cached.revision);
                return Ok(cached);
            }
            info!(
                "Bundled registry revision ({}) is newer than cached ({}). Using bundled.",
                bundled.revision, cached.revision
            );
        }

        debug!("Using bundled registry (revision {})", bundled.revision);
        Ok(bundled)
    }

    /// Inspect current registry status.
    pub fn status(&self) -> Result<RegistryStatus> {
        let bundled = self.load_bundled()?;
        let (source, active) = match self.load_cached()? {
            Some(cached) if cached.revision >= bundled.revision => ("cached", cached),
            _ => ("bundled", bundled),
        };

        Ok(RegistryStatus {
            active_source: source,
            registry_version: active.registry_version,
            revision: active.revision,
            schema_version: active.schema_version,
            updated_at: active.updated_at,
            mirrors_count: active.mirrors.len(),
            is_signature_valid: true,
        })
    }

    /// Check if a newer registry version exists online.
    pub async fn check_remote(&self) -> Result<RegistryCheckResult> {
        let active = self.load_active()?;
        let (remote_doc, _bytes, _remote_sig) = self.fetch_remote().await?;

        let update_available = remote_doc.revision > active.revision;

        Ok(RegistryCheckResult {
            update_available,
            current_revision: active.revision,
            remote_revision: remote_doc.revision,
            current_version: active.registry_version,
            remote_version: remote_doc.registry_version,
        })
    }

    /// Fetch, verify signature against exact bytes, stage generation, and atomically update local cache.
    /// Enforces monotonic anti-rollback protection which cannot be bypassed even with force.
    pub async fn update_remote(&self, force: bool) -> Result<RegistryUpdateResult> {
        self.storage.ensure_dirs()?;
        let persistent_state = self.load_persistent_state()?;
        let active = self.load_active()?;
        let (remote_doc, raw_bytes, remote_sig) = self.fetch_remote().await?;

        // Anti-rollback check: Reject older revisions unconditionally.
        // Even with `force`, an older signed revision can NEVER be accepted.
        if remote_doc.revision < persistent_state.highest_accepted_revision {
            return Err(PdownError::RollbackRejected {
                current: persistent_state.highest_accepted_revision,
                incoming: remote_doc.revision,
            });
        }

        // If revisions match and not forcing re-cache, reject
        if remote_doc.revision == active.revision && !force {
            return Err(PdownError::RollbackRejected {
                current: active.revision,
                incoming: remote_doc.revision,
            });
        }

        // Create staging generation directory
        let gen_id = format!("{}_{}", remote_doc.revision, Uuid::new_v4());
        let generations_root = self.storage.registry_generations_dir();
        fs::create_dir_all(&generations_root)?;

        let staging_dir = generations_root.join(format!("{gen_id}.staging"));
        if staging_dir.exists() {
            let _ = fs::remove_dir_all(&staging_dir);
        }
        fs::create_dir_all(&staging_dir)?;

        // Write EXACT raw downloaded bytes and signature (zero re-serialization)
        let staged_json = staging_dir.join("mirrors.json");
        let staged_sig = staging_dir.join("mirrors.json.sig");
        let staged_meta = staging_dir.join("meta.json");

        fs::write(&staged_json, &raw_bytes)?;
        fs::write(&staged_sig, remote_sig.trim().as_bytes())?;

        // Calculate and persist generation metadata
        let mut hasher = Sha256::new();
        hasher.update(&raw_bytes);
        let sha256_hash = format!("{:x}", hasher.finalize());

        let meta = serde_json::json!({
            "generation_id": gen_id,
            "revision": remote_doc.revision,
            "registry_version": remote_doc.registry_version,
            "sha256": sha256_hash,
            "created_at": Utc::now().to_rfc3339(),
        });
        fs::write(&staged_meta, serde_json::to_vec_pretty(&meta)?)?;

        // Pre-activation verification of staged generation in place
        if let Err(e) = verify_signature(&fs::read(&staged_json)?, &fs::read_to_string(&staged_sig)?, &self.public_key_b64) {
            let _ = fs::remove_dir_all(&staging_dir);
            return Err(PdownError::SignatureVerification(format!(
                "Staged generation failed signature verification: {e}"
            )));
        }

        // Atomic activation of generation directory
        let final_gen_dir = generations_root.join(&gen_id);
        fs::rename(&staging_dir, &final_gen_dir)?;

        // Also atomically update standard cached registry and signature paths
        atomic_write_bytes(&self.storage.cached_registry_path(), &raw_bytes, None)?;
        atomic_write_bytes(&self.storage.cached_signature_path(), remote_sig.trim().as_bytes(), None)?;

        // Update persistent state tracking
        let new_state = PersistentRegistryState {
            highest_accepted_revision: std::cmp::max(
                persistent_state.highest_accepted_revision,
                remote_doc.revision,
            ),
            last_known_good_generation: persistent_state.active_generation,
            active_generation: Some(gen_id),
            last_checked_at: Some(Utc::now()),
        };
        self.save_persistent_state(&new_state)?;

        info!(
            "Successfully activated registry generation for revision {}",
            remote_doc.revision
        );

        Ok(RegistryUpdateResult {
            previous_revision: active.revision,
            new_revision: remote_doc.revision,
            new_version: remote_doc.registry_version,
        })
    }

    /// Find a mirror definition by its identifier.
    pub fn find_mirror(&self, id: &str) -> Result<Option<MirrorDefinition>> {
        let doc = self.load_active()?;
        let target = id.trim().to_lowercase();
        Ok(doc
            .mirrors
            .into_iter()
            .find(|m| m.id.to_lowercase() == target))
    }

    /// List all mirrors, optionally filtering by ecosystem capability.
    pub fn list_mirrors(&self, provider_filter: Option<&str>) -> Result<Vec<MirrorDefinition>> {
        let doc = self.load_active()?;
        let mut mirrors: Vec<MirrorDefinition> = doc
            .mirrors
            .into_iter()
            .filter(|m| {
                if let Some(filter) = provider_filter {
                    match filter.to_lowercase().as_str() {
                        "flutter" => m.providers.flutter.as_ref().is_some_and(|p| p.enabled),
                        "dart" | "pub" => m.providers.dart.as_ref().is_some_and(|p| p.enabled),
                        "gradle" => m.providers.gradle.as_ref().is_some_and(|p| p.enabled),
                        "android" => m.providers.android.as_ref().is_some_and(|p| p.enabled),
                        "ios" => m.providers.ios.as_ref().is_some_and(|p| p.enabled),
                        _ => true,
                    }
                } else {
                    true
                }
            })
            .collect();

        // Sort by priority (ascending, None treated as 100)
        mirrors.sort_by_key(|m| m.priority.unwrap_or(100));
        Ok(mirrors)
    }

    async fn fetch_remote(&self) -> Result<(RegistryDocument, Vec<u8>, String)> {
        let client = build_http_client(Some(10))?;

        debug!("Fetching remote registry from {}", self.remote_registry_url);
        let resp_json = client.get(&self.remote_registry_url).send().await?;
        if !resp_json.status().is_success() {
            return Err(PdownError::RegistryValidation(format!(
                "Failed to download remote registry: HTTP {}",
                resp_json.status()
            )));
        }
        let json_bytes = resp_json.bytes().await?.to_vec();

        debug!(
            "Fetching remote signature from {}",
            self.remote_signature_url
        );
        let resp_sig = client.get(&self.remote_signature_url).send().await?;
        if !resp_sig.status().is_success() {
            return Err(PdownError::RegistryValidation(format!(
                "Failed to download remote registry signature: HTTP {}",
                resp_sig.status()
            )));
        }
        let sig_text = resp_sig.text().await?;

        // Cryptographically verify remote payload against EXACT bytes
        verify_signature(&json_bytes, &sig_text, &self.public_key_b64)?;

        // Validate JSON Schema
        Self::validate_json_schema(&json_bytes)?;

        // Deserialize document and validate semantics
        let doc: RegistryDocument = serde_json::from_slice(&json_bytes)?;
        Self::validate_semantics(&doc)?;

        Ok((doc, json_bytes, sig_text))
    }

    /// Real JSON Schema validation against embedded registry/schema.json.
    pub fn validate_json_schema(raw_json: &[u8]) -> Result<()> {
        let json_val: serde_json::Value = serde_json::from_slice(raw_json)?;
        let schema_val: serde_json::Value = serde_json::from_str(REGISTRY_SCHEMA_JSON)?;
        let validator = jsonschema::validator_for(&schema_val)
            .map_err(|e| PdownError::RegistryValidation(format!("Invalid embedded JSON schema: {e}")))?;

        if let Err(err) = validator.validate(&json_val) {
            return Err(PdownError::RegistryValidation(format!(
                "JSON Schema validation failed: {} at {}",
                err, err.instance_path
            )));
        }
        Ok(())
    }

    /// Deep semantic validation for mirror definitions, URLs, duplicate IDs, and capabilities.
    pub fn validate_semantics(doc: &RegistryDocument) -> Result<()> {
        if doc.schema_version > CURRENT_SUPPORTED_SCHEMA_VERSION {
            return Err(PdownError::SchemaMismatch {
                expected: CURRENT_SUPPORTED_SCHEMA_VERSION,
                got: doc.schema_version,
            });
        }
        if doc.mirrors.is_empty() {
            return Err(PdownError::RegistryValidation(
                "Registry must contain at least one mirror definition".into(),
            ));
        }

        let mut seen_ids = HashSet::new();

        for mirror in &doc.mirrors {
            // 1. Duplicate mirror IDs
            let norm_id = mirror.id.trim().to_lowercase();
            if !seen_ids.insert(norm_id) {
                return Err(PdownError::RegistryValidation(format!(
                    "Duplicate mirror ID found: '{}'",
                    mirror.id
                )));
            }

            // 2. Validate website URL
            Self::validate_url(&mirror.website, "website", &mirror.id)?;

            // 3. Priority bounds validation
            if let Some(p) = mirror.priority {
                if !(1..=1000).contains(&p) {
                    return Err(PdownError::RegistryValidation(format!(
                        "Mirror '{}' has priority {} outside valid range 1..=1000",
                        mirror.id, p
                    )));
                }
            }

            // 4. Provider endpoints completeness
            if let Some(ref dart) = mirror.providers.dart {
                if dart.enabled {
                    Self::validate_url(&dart.hosted_url, "dart.hosted_url", &mirror.id)?;
                }
            }

            if let Some(ref flutter) = mirror.providers.flutter {
                if flutter.enabled {
                    Self::validate_url(&flutter.storage_url, "flutter.storage_url", &mirror.id)?;
                }
            }

            if let Some(ref gradle) = mirror.providers.gradle {
                if gradle.enabled {
                    let mut has_endpoint = false;
                    if let Some(ref u) = gradle.maven_central_url {
                        Self::validate_url(u, "gradle.maven_central_url", &mirror.id)?;
                        has_endpoint = true;
                    }
                    if let Some(ref u) = gradle.google_maven_url {
                        Self::validate_url(u, "gradle.google_maven_url", &mirror.id)?;
                        has_endpoint = true;
                    }
                    if let Some(ref u) = gradle.gradle_plugins_url {
                        Self::validate_url(u, "gradle.gradle_plugins_url", &mirror.id)?;
                        has_endpoint = true;
                    }
                    if let Some(ref u) = gradle.distribution_url {
                        Self::validate_url(u, "gradle.distribution_url", &mirror.id)?;
                        has_endpoint = true;
                    }
                    if !has_endpoint {
                        return Err(PdownError::RegistryValidation(format!(
                            "Enabled Gradle provider in mirror '{}' must specify at least one endpoint",
                            mirror.id
                        )));
                    }
                }
            }

            if let Some(ref android) = mirror.providers.android {
                if android.enabled {
                    if let Some(ref u) = android.google_maven_url {
                        Self::validate_url(u, "android.google_maven_url", &mirror.id)?;
                    } else {
                        return Err(PdownError::RegistryValidation(format!(
                            "Enabled Android provider in mirror '{}' is missing google_maven_url",
                            mirror.id
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_url(url_str: &str, field: &str, mirror_id: &str) -> Result<()> {
        if !url_str.starts_with("https://") && !url_str.starts_with("http://") {
            return Err(PdownError::RegistryValidation(format!(
                "Mirror '{}' has invalid URL scheme for {}: '{}'. Only https:// and http:// are supported.",
                mirror_id, field, url_str
            )));
        }
        if url_str.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(PdownError::RegistryValidation(format!(
                "Mirror '{}' contains invalid whitespace or control characters in {}: '{}'",
                mirror_id, field, url_str
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_schema_compilation() {
        let schema_val: serde_json::Value = serde_json::from_str(REGISTRY_SCHEMA_JSON).unwrap();
        let validator = jsonschema::validator_for(&schema_val).unwrap();
        let bundled_val: serde_json::Value = serde_json::from_str(BUNDLED_REGISTRY_JSON).unwrap();
        if let Err(err) = validator.validate(&bundled_val) {
            println!("SCHEMA ERROR: {} at {}", err, err.instance_path);
            panic!("Schema validation failed: {err}");
        }
    }

    #[test]
    fn test_load_bundled_registry() {
        let tmp = tempdir().unwrap();
        let storage = StorageManager::with_custom_root(tmp.path());
        let manager = RegistryManager::new(storage);

        let bundled = manager
            .load_bundled()
            .expect("Bundled registry must be valid");
        assert_eq!(bundled.schema_version, 1);
        assert!(!bundled.mirrors.is_empty());
        assert!(bundled.mirrors.iter().any(|m| m.id == "pub-azs"));
        assert!(bundled.mirrors.iter().any(|m| m.id == "runflare"));
    }

    #[test]
    fn test_active_fallback_without_cache() {
        let tmp = tempdir().unwrap();
        let storage = StorageManager::with_custom_root(tmp.path());
        let manager = RegistryManager::new(storage);

        let active = manager.load_active().expect("Active registry must load");
        assert_eq!(active.revision, 1);
    }

    #[test]
    fn test_find_mirror() {
        let tmp = tempdir().unwrap();
        let storage = StorageManager::with_custom_root(tmp.path());
        let manager = RegistryManager::new(storage);

        let pub_azs = manager
            .find_mirror("pub-azs")
            .unwrap()
            .expect("pub-azs must exist");
        assert_eq!(pub_azs.id, "pub-azs");
        assert!(pub_azs.providers.dart.unwrap().enabled);

        let unknown = manager.find_mirror("non-existent-mirror").unwrap();
        assert!(unknown.is_none());
    }

    #[test]
    fn test_list_mirrors_filtering() {
        let tmp = tempdir().unwrap();
        let storage = StorageManager::with_custom_root(tmp.path());
        let manager = RegistryManager::new(storage);

        let flutter_mirrors = manager.list_mirrors(Some("flutter")).unwrap();
        assert!(!flutter_mirrors.is_empty());
        assert!(flutter_mirrors
            .iter()
            .all(|m| m.providers.flutter.as_ref().unwrap().enabled));

        let gradle_mirrors = manager.list_mirrors(Some("gradle")).unwrap();
        assert!(!gradle_mirrors.is_empty());
    }
}
