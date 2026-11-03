use crate::core::errors::{PdownError, Result};
use crate::core::models::ProfileConfig;
use crate::infrastructure::storage::StorageManager;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct ProfileManager {
    storage: StorageManager,
}

impl ProfileManager {
    pub fn new(storage: StorageManager) -> Self {
        Self { storage }
    }

    fn active_profile_file(&self) -> PathBuf {
        self.storage.config_dir().join("active_profile.txt")
    }

    fn profile_file(&self, name: &str) -> PathBuf {
        self.storage.profiles_dir().join(format!("{name}.toml"))
    }

    pub fn get_active_profile_name(&self) -> String {
        let path = self.active_profile_file();
        if path.exists() {
            fs::read_to_string(path)
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "default".to_string())
        } else {
            "default".to_string()
        }
    }

    pub fn set_active_profile_name(&self, name: &str) -> Result<()> {
        self.storage.ensure_dirs()?;
        let path = self.active_profile_file();
        crate::system::filesystem::atomic_write(&path, name)?;
        Ok(())
    }

    pub fn list_profiles(&self) -> Result<Vec<String>> {
        self.storage.ensure_dirs()?;
        self.ensure_default_profiles()?;

        let mut list = Vec::new();
        for entry in fs::read_dir(self.storage.profiles_dir())? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "toml") {
                if let Some(stem) = path.file_stem() {
                    list.push(stem.to_string_lossy().to_string());
                }
            }
        }
        list.sort();
        Ok(list)
    }

    pub fn load_profile(&self, name: &str) -> Result<ProfileConfig> {
        self.storage.ensure_dirs()?;
        self.ensure_default_profiles()?;

        let file = self.profile_file(name);
        if !file.exists() {
            return Err(PdownError::InvalidConfig(format!(
                "Profile '{name}' does not exist"
            )));
        }

        let content = fs::read_to_string(file)?;
        let config: ProfileConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_profile(&self, profile: &ProfileConfig) -> Result<()> {
        self.storage.ensure_dirs()?;
        let file = self.profile_file(&profile.name);
        let serialized = toml::to_string_pretty(profile)?;
        crate::system::filesystem::atomic_write(&file, &serialized)?;
        Ok(())
    }

    pub fn create_profile(&self, name: &str) -> Result<ProfileConfig> {
        let clean_name = name.trim().to_lowercase();
        if clean_name.is_empty() {
            return Err(PdownError::InvalidConfig(
                "Profile name cannot be empty".into(),
            ));
        }

        let file = self.profile_file(&clean_name);
        if file.exists() {
            return Err(PdownError::InvalidConfig(format!(
                "Profile '{clean_name}' already exists"
            )));
        }

        let profile = ProfileConfig {
            name: clean_name,
            active_mirror_id: None,
            enabled_providers: vec![
                "flutter".to_string(),
                "dart".to_string(),
                "gradle".to_string(),
                "android".to_string(),
            ],
            custom_overrides: HashMap::new(),
        };

        self.save_profile(&profile)?;
        Ok(profile)
    }

    fn ensure_default_profiles(&self) -> Result<()> {
        let default_file = self.profile_file("default");
        if !default_file.exists() {
            let default_profile = ProfileConfig {
                name: "default".into(),
                active_mirror_id: Some("official".into()),
                enabled_providers: vec![
                    "flutter".to_string(),
                    "dart".to_string(),
                    "gradle".to_string(),
                    "android".to_string(),
                ],
                custom_overrides: HashMap::new(),
            };
            self.save_profile(&default_profile)?;
        }

        let iran_file = self.profile_file("iran");
        if !iran_file.exists() {
            let iran_profile = ProfileConfig {
                name: "iran".into(),
                active_mirror_id: Some("pub-azs".into()),
                enabled_providers: vec![
                    "dart".to_string(),
                    "gradle".to_string(),
                    "android".to_string(),
                ],
                custom_overrides: HashMap::new(),
            };
            self.save_profile(&iran_profile)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_profile_creation_and_listing() {
        let tmp = tempdir().unwrap();
        let storage = StorageManager::with_custom_root(tmp.path());
        let mgr = ProfileManager::new(storage);

        let profiles = mgr.list_profiles().unwrap();
        assert!(profiles.contains(&"default".to_string()));
        assert!(profiles.contains(&"iran".to_string()));

        let custom = mgr.create_profile("work-project").unwrap();
        assert_eq!(custom.name, "work-project");

        let updated_list = mgr.list_profiles().unwrap();
        assert!(updated_list.contains(&"work-project".to_string()));
    }
}
