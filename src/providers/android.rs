use crate::core::errors::{PdownError, Result};
use crate::core::models::{ChangePlan, MirrorDefinition, ProviderStatus};
use crate::core::provider::Provider;
use crate::providers::gradle::GradleProvider;
use std::collections::HashMap;

pub struct AndroidProvider;

impl Provider for AndroidProvider {
    fn id(&self) -> &'static str {
        "android"
    }

    fn display_name(&self) -> &'static str {
        "Android (Google Maven & SDK Repositories)"
    }

    fn is_supported(&self) -> bool {
        true
    }

    fn detect_current(&self) -> Result<ProviderStatus> {
        let gradle_status = GradleProvider.detect_current()?;
        let mut details = HashMap::new();
        details.insert(
            "repository_routing".to_string(),
            if gradle_status.is_active {
                "active (routed via ~/.gradle/init.d/pdown-mirror.gradle)".to_string()
            } else {
                "official (https://dl.google.com/dl/android/maven2/)".to_string()
            },
        );

        Ok(ProviderStatus {
            provider_id: self.id().to_string(),
            is_active: gradle_status.is_active,
            current_endpoint: gradle_status.current_endpoint,
            details,
        })
    }

    fn plan_apply(&self, mirror: &MirrorDefinition) -> Result<ChangePlan> {
        let android_config = mirror.providers.android.as_ref().ok_or_else(|| {
            PdownError::InvalidConfig(format!("Mirror '{}' does not support Android", mirror.id))
        })?;

        if !android_config.enabled {
            return Err(PdownError::InvalidConfig(format!(
                "Android provider is disabled for mirror '{}'",
                mirror.id
            )));
        }

        // Android Maven configuration is coordinated through the Gradle provider
        GradleProvider.plan_apply(mirror)
    }

    fn plan_revert(&self) -> Result<ChangePlan> {
        GradleProvider.plan_revert()
    }
}
