use crate::EffectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomEffectProgramLanguage {
    WgslUtf8,
}

/// Descriptor used to register a bounded custom effect (v1).
///
/// This is intentionally small and backend-agnostic. Backends may reject registration based on
/// capability gating.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CustomEffectDescriptorV1 {
    pub language: CustomEffectProgramLanguage,
    pub source: String,
}

impl CustomEffectDescriptorV1 {
    pub fn wgsl_utf8(source: impl Into<String>) -> Self {
        Self {
            language: CustomEffectProgramLanguage::WgslUtf8,
            source: source.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomEffectRegistrationError {
    Unsupported,
    InvalidSource,
}

/// Renderer-owned registry for bounded custom effects.
///
/// This mirrors the material registration pattern: callers obtain an `EffectId` handle without
/// receiving backend handles. Backends may deterministically degrade unsupported effects.
pub trait CustomEffectService {
    fn register_custom_effect_v1(
        &mut self,
        desc: CustomEffectDescriptorV1,
    ) -> Result<EffectId, CustomEffectRegistrationError>;

    fn unregister_custom_effect(&mut self, id: EffectId) -> bool;
}
