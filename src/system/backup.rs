use crate::core::errors::{PdownError, Result};
use crate::infrastructure::storage::StorageManager;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: Uuid,
    pub original_path: PathBuf,
    pub backup_path: PathBuf,
    pub sha256_hash: String,
    pub permissions_mode: Option<u32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    pub fn new(storage: &StorageManager) -> Self {
        Self {
            backup_dir: storage.backup_dir(),
        }
    }

    pub fn with_custom_dir(backup_dir: PathBuf) -> Self {
        Self { backup_dir }
    }

    pub fn backup_dir(&self) -> &Path {
        &self.backup_dir
    }

    /// Backup a file before modification. Returns BackupRecord if file existed.
    pub fn backup_file(&self, file_path: &Path) -> Result<Option<BackupRecord>> {
        if !file_path.exists() {
            return Ok(None);
        }

        fs::create_dir_all(&self.backup_dir)?;

        let metadata = fs::metadata(file_path)?;
        let content = fs::read(file_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());

        let id = Uuid::new_v4();
        let filename = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".to_string());

        let backup_filename = format!("{}_{}_{}", id, Utc::now().format("%Y%m%d%H%M%S"), filename);
        let backup_path = self.backup_dir.join(backup_filename);

        fs::copy(file_path, &backup_path)?;

        #[cfg(unix)]
        let permissions_mode = {
            use std::os::unix::fs::PermissionsExt;
            Some(metadata.permissions().mode())
        };
        #[cfg(not(unix))]
        let permissions_mode = None;

        Ok(Some(BackupRecord {
            id,
            original_path: file_path.to_path_buf(),
            backup_path,
            sha256_hash: hash,
            permissions_mode,
            created_at: Utc::now(),
        }))
    }

    /// Restore a file from its backup record. Byte-preserving and permission-retaining.
    pub fn restore(&self, record: &BackupRecord) -> Result<()> {
        if !record.backup_path.exists() {
            return Err(PdownError::BackupFailed {
                path: record.original_path.clone(),
                message: format!(
                    "Backup file '{}' no longer exists",
                    record.backup_path.display()
                ),
            });
        }

        // Verify hash integrity before restore
        let content = fs::read(&record.backup_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let current_hash = format!("{:x}", hasher.finalize());

        if current_hash != record.sha256_hash {
            return Err(PdownError::BackupFailed {
                path: record.original_path.clone(),
                message: "Backup file checksum mismatch. Refusing to restore corrupted snapshot."
                    .into(),
            });
        }

        if let Some(parent) = record.original_path.parent() {
            fs::create_dir_all(parent)?;
        }

        #[cfg(unix)]
        let permissions = record.permissions_mode.map(|mode| {
            use std::os::unix::fs::PermissionsExt;
            fs::Permissions::from_mode(mode)
        });
        #[cfg(not(unix))]
        let permissions = None;

        crate::system::filesystem::atomic_write_bytes(
            &record.original_path,
            &content,
            permissions,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_backup_and_restore_cycle() {
        let tmp = tempdir().unwrap();
        let backup_dir = tmp.path().join("backups");
        let target_file = tmp.path().join("config.txt");

        fs::write(&target_file, "original content").unwrap();

        let mgr = BackupManager::with_custom_dir(backup_dir);
        let record = mgr
            .backup_file(&target_file)
            .unwrap()
            .expect("Backup must succeed");

        // Mutate target file
        fs::write(&target_file, "corrupted content").unwrap();

        // Restore
        mgr.restore(&record).unwrap();
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "original content"
        );
    }
}
