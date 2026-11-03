use crate::core::errors::{PdownError, Result};
use crate::core::models::{ChangePlan, MirrorDefinition, ProviderStatus};
use crate::core::provider::Provider;
use std::collections::HashMap;

pub struct IosProvider;

impl Provider for IosProvider {
    fn id(&self) -> &'static str {
        "ios"
    }

    fn display_name(&self) -> &'static str {
        "iOS (CocoaPods & SPM)"
    }

    fn is_supported(&self) -> bool {
        false // Explicitly guarded: no verifiable public mirror exists
    }

    fn detect_current(&self) -> Result<ProviderStatus> {
        let mut details = HashMap::new();
        details.insert(
            "status".to_string(),
            "unsupported / guarded (no verifiable upstream CocoaPods or SPM mirror available in Iran)".to_string(),
        );

        Ok(ProviderStatus {
            provider_id: self.id().to_string(),
            is_active: false,
            current_endpoint: None,
            details,
        })
    }

    fn plan_apply(&self, _mirror: &MirrorDefinition) -> Result<ChangePlan> {
        Err(PdownError::ProviderUnsupported {
            provider: "ios".to_string(),
            reason: "No verifiable public mirror exists for iOS CocoaPods CDN or Swift Package Manager. Fake endpoints are disabled to protect developer environments from build failures.".to_string(),
        })
    }

    fn plan_revert(&self) -> Result<ChangePlan> {
        Ok(ChangePlan {
            provider_id: self.id().to_string(),
            description: "No managed iOS settings exist to revert".to_string(),
            items: Vec::new(),
        })
    }
}
