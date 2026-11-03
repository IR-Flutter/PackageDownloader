use crate::core::errors::{PdownError, Result};
use crate::core::models::{ChangePlan, MirrorDefinition, ProviderStatus};
use crate::core::provider::Provider;
use crate::system::environment::{
    EnvironmentManager, MARKER_END_FLUTTER_DART, MARKER_START_FLUTTER_DART,
};
use std::collections::HashMap;

pub const KEY_PUB_HOSTED_URL: &str = "PUB_HOSTED_URL";

pub struct DartProvider;

impl Provider for DartProvider {
    fn id(&self) -> &'static str {
        "dart"
    }

    fn display_name(&self) -> &'static str {
        "Dart / Pub"
    }

    fn is_supported(&self) -> bool {
        true
    }

    fn detect_current(&self) -> Result<ProviderStatus> {
        let current_val = EnvironmentManager::get_var(KEY_PUB_HOSTED_URL);
        let mut details = HashMap::new();
        details.insert(
            KEY_PUB_HOSTED_URL.to_string(),
            current_val
                .clone()
                .unwrap_or_else(|| "official (https://pub.dev)".to_string()),
        );

        Ok(ProviderStatus {
            provider_id: self.id().to_string(),
            is_active: current_val.is_some(),
            current_endpoint: current_val,
            details,
        })
    }

    fn plan_apply(&self, mirror: &MirrorDefinition) -> Result<ChangePlan> {
        let dart_config = mirror.providers.dart.as_ref().ok_or_else(|| {
            PdownError::InvalidConfig(format!("Mirror '{}' does not support Dart/Pub", mirror.id))
        })?;

        if !dart_config.enabled {
            return Err(PdownError::InvalidConfig(format!(
                "Dart/Pub provider is disabled for mirror '{}'",
                mirror.id
            )));
        }

        let mut items = Vec::new();
        let shell_files = EnvironmentManager::get_unix_shell_files();

        // Include current flutter storage url if set
        let flutter_val = EnvironmentManager::get_var("FLUTTER_STORAGE_BASE_URL");
        let mut vars = vec![(KEY_PUB_HOSTED_URL, dart_config.hosted_url.as_str())];
        if let Some(ref f) = flutter_val {
            vars.push(("FLUTTER_STORAGE_BASE_URL", f.as_str()));
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

        Ok(ChangePlan {
            provider_id: self.id().to_string(),
            description: format!("Configure PUB_HOSTED_URL='{}'", dart_config.hosted_url),
            items,
        })
    }

    fn plan_revert(&self) -> Result<ChangePlan> {
        let mut items = Vec::new();
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

        Ok(ChangePlan {
            provider_id: self.id().to_string(),
            description: "Remove managed PUB_HOSTED_URL settings".to_string(),
            items,
        })
    }
}
