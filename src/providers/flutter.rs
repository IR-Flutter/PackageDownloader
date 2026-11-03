use crate::core::errors::{PdownError, Result};
use crate::core::models::{ChangePlan, MirrorDefinition, ProviderStatus};
use crate::core::provider::Provider;
use crate::system::environment::{
    EnvironmentManager, MARKER_END_FLUTTER_DART, MARKER_START_FLUTTER_DART,
};
use std::collections::HashMap;

pub const KEY_FLUTTER_STORAGE_URL: &str = "FLUTTER_STORAGE_BASE_URL";

pub struct FlutterProvider;

impl Provider for FlutterProvider {
    fn id(&self) -> &'static str {
        "flutter"
    }

    fn display_name(&self) -> &'static str {
        "Flutter Storage"
    }

    fn is_supported(&self) -> bool {
        true
    }

    fn detect_current(&self) -> Result<ProviderStatus> {
        let current_val = EnvironmentManager::get_var(KEY_FLUTTER_STORAGE_URL);
        let mut details = HashMap::new();
        details.insert(
            KEY_FLUTTER_STORAGE_URL.to_string(),
            current_val
                .clone()
                .unwrap_or_else(|| "official (https://storage.googleapis.com)".to_string()),
        );

        Ok(ProviderStatus {
            provider_id: self.id().to_string(),
            is_active: current_val.is_some(),
            current_endpoint: current_val,
            details,
        })
    }

    fn plan_apply(&self, mirror: &MirrorDefinition) -> Result<ChangePlan> {
        let flutter_config = mirror.providers.flutter.as_ref().ok_or_else(|| {
            PdownError::InvalidConfig(format!("Mirror '{}' does not support Flutter", mirror.id))
        })?;

        if !flutter_config.enabled {
            return Err(PdownError::ProviderUnsupported {
                provider: "flutter".to_string(),
                reason: format!(
                    "Mirror '{}' has not verified or does not support Flutter storage artifacts.",
                    mirror.id
                ),
            });
        }

        let storage_url = flutter_config.storage_url.as_deref().ok_or_else(|| {
            PdownError::InvalidConfig(format!(
                "Mirror '{}' has enabled Flutter provider but storage_url is missing",
                mirror.id
            ))
        })?;

        let mut items = Vec::new();
        if cfg!(target_os = "windows") {
            let old_value = EnvironmentManager::get_var(KEY_FLUTTER_STORAGE_URL);
            items.push(crate::core::models::ChangeItem::EnvironmentSet {
                key: KEY_FLUTTER_STORAGE_URL.to_string(),
                old_value,
                new_value: storage_url.to_string(),
            });
        } else {
            let shell_files = EnvironmentManager::get_unix_shell_files();

            // Preserve current pub hosted url if set
            let pub_val = EnvironmentManager::get_var("PUB_HOSTED_URL");
            let mut vars = vec![(KEY_FLUTTER_STORAGE_URL, storage_url)];
            if let Some(ref p) = pub_val {
                vars.push(("PUB_HOSTED_URL", p.as_str()));
            }

            for file in shell_files {
                let item = EnvironmentManager::plan_unix_file_update(
                    &file,
                    MARKER_START_FLUTTER_DART,
                    MARKER_END_FLUTTER_DART,
                    &vars,
                )?;
                items.push(item);
            }
        }

        Ok(ChangePlan {
            provider_id: self.id().to_string(),
            description: format!("Configure FLUTTER_STORAGE_BASE_URL='{}'", storage_url),
            items,
        })
    }

    fn plan_revert(&self) -> Result<ChangePlan> {
        let mut items = Vec::new();
        if cfg!(target_os = "windows") {
            let old_value = EnvironmentManager::get_var(KEY_FLUTTER_STORAGE_URL);
            items.push(crate::core::models::ChangeItem::EnvironmentUnset {
                key: KEY_FLUTTER_STORAGE_URL.to_string(),
                old_value,
            });
        } else {
            let shell_files = EnvironmentManager::get_unix_shell_files();

            for file in shell_files {
                if let Some(item) = EnvironmentManager::plan_unix_file_revert(
                    &file,
                    MARKER_START_FLUTTER_DART,
                    MARKER_END_FLUTTER_DART,
                )? {
                    items.push(item);
                }
            }
        }

        Ok(ChangePlan {
            provider_id: self.id().to_string(),
            description: "Remove managed FLUTTER_STORAGE_BASE_URL settings".to_string(),
            items,
        })
    }
}
