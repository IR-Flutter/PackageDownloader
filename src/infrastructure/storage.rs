use crate::core::errors::{PdownError, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct StorageManager {
    config_dir: PathBuf,
    data_dir: PathBuf,
    cache_dir: PathBuf,
}

impl StorageManager {
    pub fn new() -> Result<Self> {
        if let Some(proj_dirs) = ProjectDirs::from("ir", "IR-Flutter", "pdown") {
            Ok(Self {
                config_dir: proj_dirs.config_dir().to_path_buf(),
                data_dir: proj_dirs.data_dir().to_path_buf(),
                cache_dir: proj_dirs.cache_dir().to_path_buf(),
            })
        } else {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map(PathBuf::from)
                .map_err(|_| {
                    PdownError::InvalidConfig("Could not determine user home directory".into())
                })?;
            let base = home.join(".pdown");
            Ok(Self {
                config_dir: base.join("config"),
                data_dir: base.join("data"),
                cache_dir: base.join("cache"),
            })
        }
    }

    /// Construct a storage manager with an explicit base directory (useful for isolated tests).
    pub fn with_custom_root(root: &Path) -> Self {
        Self {
            config_dir: root.join("config"),
            data_dir: root.join("data"),
            cache_dir: root.join("cache"),
        }
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn backup_dir(&self) -> PathBuf {
        self.data_dir.join("backups")
    }

    pub fn profiles_dir(&self) -> PathBuf {
        self.config_dir.join("profiles")
    }

    pub fn cached_registry_path(&self) -> PathBuf {
        self.cache_dir.join("mirrors.json")
    }

    pub fn cached_signature_path(&self) -> PathBuf {
        self.cache_dir.join("mirrors.json.sig")
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.config_dir)?;
        fs::create_dir_all(&self.data_dir)?;
        fs::create_dir_all(&self.cache_dir)?;
        fs::create_dir_all(self.backup_dir())?;
        fs::create_dir_all(self.profiles_dir())?;
        Ok(())
    }
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize standard storage manager")
    }
}
