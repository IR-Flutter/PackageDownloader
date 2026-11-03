//! Provider trait definition and contracts.

use crate::core::errors::Result;
use crate::core::models::{ChangePlan, MirrorDefinition, ProviderStatus};

pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn is_supported(&self) -> bool;
    fn detect_current(&self) -> Result<ProviderStatus>;
    fn plan_apply(&self, mirror: &MirrorDefinition) -> Result<ChangePlan>;
    fn plan_revert(&self) -> Result<ChangePlan>;
}
