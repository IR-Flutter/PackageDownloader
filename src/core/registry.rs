use crate::core::errors::{PdownError, Result};
use crate::core::models::{MirrorDefinition, RegistryDocument};
use crate::infrastructure::crypto::{verify_signature, EMBEDDED_REGISTRY_PUBLIC_KEY};
use crate::infrastructure::http::build_http_client;
use crate::infrastructure::storage::StorageManager;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;
use tracing::{debug, info, warn};

pub const CANONICAL_REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/IR-Flutter/PackageDownloader/main/registry/mirrors.json";
pub const CANONICAL_SIGNATURE_URL: &str =
    "https://raw.githubusercontent.com/IR-Flutter/PackageDownloader/main/registry/mirrors.json.sig";

pub const CURRENT_SUPPORTED_SCHEMA_VERSION: u32 = 1;

pub const BUNDLED_REGISTRY_JSON: &str = include_str!("../../registry/mirrors.json");
pub const BUNDLED_REGISTRY_SIG: &str = include_str!("../../registry/mirrors.json.sig");

#[derive(Debug, Clone)]
pub struct RegistryManager {
    storage: StorageManager,
    public_key_b64: String,
    remote_registry_url: String,
    remote_signature_url: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RegistryStatus {
    pub active_source: &'static str,
    pub registry_version: String,
    pub revision: u64,
    pub schema_version: u32,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub mirrors_count: usize,
    pub is_signature_valid: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RegistryCheckResult {
    pub update_available: bool,
    pub current_revision: u64,
    pub remote_revision: u64,
    pub current_version: String,
    pub remote_version: String,
}

#[derive(Debug, Clone, serde::Serialize)]
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

    /// Load the bundled registry compiled into the binary.
    pub fn load_bundled(&self) -> Result<RegistryDocument> {
        let doc: RegistryDocument = serde_json::from_str(BUNDLED_REGISTRY_JSON)?;
        Self::validate_schema(&doc)?;
        Ok(doc)
    }

    /// Load and validate cached registry if present.
    pub fn load_cached(&self) -> Result<Option<RegistryDocument>> {
        let cache_path = self.storage.cached_registry_path();
        let sig_path = self.storage.cached_signature_path();

        if !cache_path.exists() || !sig_path.exists() {
            return Ok(None);
        }

        let content = fs::read(&cache_path)?;
        let signature_str = fs::read_to_string(&sig_path)?;

        // Verify cryptographic signature
        if let Err(e) = verify_signature(&content, &signature_str, &self.public_key_b64) {
            warn!("Cached registry signature verification failed: {e}. Discarding cache.");
            return Ok(None);
        }

        let doc: RegistryDocument = match serde_json::from_slice(&content) {
            Ok(d) => d,
            Err(e) => {
                warn!("Cached registry is malformed JSON: {e}. Discarding cache.");
                return Ok(None);
            }
        };

        if let Err(e) = Self::validate_schema(&doc) {
            warn!("Cached registry schema invalid: {e}. Discarding cache.");
            return Ok(None);
        }

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
        let (remote_doc, _remote_sig) = self.fetch_remote().await?;

        let update_available = remote_doc.revision > active.revision;

        Ok(RegistryCheckResult {
            update_available,
            current_revision: active.revision,
            remote_revision: remote_doc.revision,
            current_version: active.registry_version,
            remote_version: remote_doc.registry_version,
        })
    }

    /// Fetch, verify signature, and atomically update local cache.
    pub async fn update_remote(&self, force: bool) -> Result<RegistryUpdateResult> {
        self.storage.ensure_dirs()?;
        let active = self.load_active()?;
        let (remote_doc, remote_sig) = self.fetch_remote().await?;

        if remote_doc.revision <= active.revision && !force {
            return Err(PdownError::RollbackRejected {
                current: active.revision,
                incoming: remote_doc.revision,
            });
        }

        // Atomically write cached registry and signature
        let cache_dir = self.storage.cache_dir();
        let serialized_json = serde_json::to_string_pretty(&remote_doc)?;

        let mut temp_json = NamedTempFile::new_in(cache_dir)?;
        temp_json.write_all(serialized_json.as_bytes())?;
        temp_json.flush()?;

        let mut temp_sig = NamedTempFile::new_in(cache_dir)?;
        temp_sig.write_all(remote_sig.trim().as_bytes())?;
        temp_sig.flush()?;

        // Atomic renames
        temp_json
            .persist(self.storage.cached_registry_path())
            .map_err(|e| {
                PdownError::TransactionFailed(format!("Failed to persist cached registry: {e}"))
            })?;

        temp_sig
            .persist(self.storage.cached_signature_path())
            .map_err(|e| {
                PdownError::TransactionFailed(format!("Failed to persist cached signature: {e}"))
            })?;

        info!(
            "Successfully updated registry from revision {} to {}",
            active.revision, remote_doc.revision
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

    async fn fetch_remote(&self) -> Result<(RegistryDocument, String)> {
        let client = build_http_client(Some(10))?;

        debug!("Fetching remote registry from {}", self.remote_registry_url);
        let resp_json = client.get(&self.remote_registry_url).send().await?;
        if !resp_json.status().is_success() {
            return Err(PdownError::RegistryValidation(format!(
                "Failed to download remote registry: HTTP {}",
                resp_json.status()
            )));
        }
        let json_bytes = resp_json.bytes().await?;

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

        // Cryptographically verify remote payload
        verify_signature(&json_bytes, &sig_text, &self.public_key_b64)?;

        // Validate JSON and schema
        let doc: RegistryDocument = serde_json::from_slice(&json_bytes)?;
        Self::validate_schema(&doc)?;

        Ok((doc, sig_text))
    }

    fn validate_schema(doc: &RegistryDocument) -> Result<()> {
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

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
