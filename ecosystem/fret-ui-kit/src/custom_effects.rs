use fret_core::{
    CustomEffectDescriptorV1, CustomEffectDescriptorV2, CustomEffectRegistrationError,
    CustomEffectService, EffectId,
};

/// Lazily registers a custom effect program (v1) and caches its `EffectId`.
///
/// This is intended for app/ecosystem code that wants to author a small WGSL snippet while keeping
/// the `Scene` contract bounded (ADR 0299).
///
/// Notes:
/// - The cached `EffectId` is tied to the current renderer instance. If the renderer is recreated
///   (e.g. device loss), call `invalidate()` and re-register.
#[derive(Debug, Clone)]
pub struct CustomEffectProgramV1 {
    desc: CustomEffectDescriptorV1,
    id: Option<EffectId>,
}

impl CustomEffectProgramV1 {
    pub fn wgsl_utf8(source: impl Into<String>) -> Self {
        Self {
            desc: CustomEffectDescriptorV1::wgsl_utf8(source),
            id: None,
        }
    }

    pub fn descriptor(&self) -> &CustomEffectDescriptorV1 {
        &self.desc
    }

    pub fn id(&self) -> Option<EffectId> {
        self.id
    }

    pub fn invalidate(&mut self) {
        self.id = None;
    }

    pub fn ensure_registered(
        &mut self,
        effects: &mut dyn CustomEffectService,
    ) -> Result<EffectId, CustomEffectRegistrationError> {
        if let Some(id) = self.id {
            return Ok(id);
        }

        let id = effects.register_custom_effect_v1(self.desc.clone())?;
        self.id = Some(id);
        Ok(id)
    }

    pub fn unregister(&mut self, effects: &mut dyn CustomEffectService) -> bool {
        let Some(id) = self.id else {
            return false;
        };
        self.id = None;
        effects.unregister_custom_effect(id)
    }
}

/// Lazily registers a custom effect program (v2) and caches its `EffectId`.
///
/// V2 programs may reference additional versioned bindings (e.g. a single user image input).
///
/// Notes:
/// - The cached `EffectId` is tied to the current renderer instance. If the renderer is recreated
///   (e.g. device loss), call `invalidate()` and re-register.
#[derive(Debug, Clone)]
pub struct CustomEffectProgramV2 {
    desc: CustomEffectDescriptorV2,
    id: Option<EffectId>,
}

impl CustomEffectProgramV2 {
    pub fn wgsl_utf8(source: impl Into<String>) -> Self {
        Self {
            desc: CustomEffectDescriptorV2::wgsl_utf8(source),
            id: None,
        }
    }

    pub fn descriptor(&self) -> &CustomEffectDescriptorV2 {
        &self.desc
    }

    pub fn id(&self) -> Option<EffectId> {
        self.id
    }

    pub fn invalidate(&mut self) {
        self.id = None;
    }

    pub fn ensure_registered(
        &mut self,
        effects: &mut dyn CustomEffectService,
    ) -> Result<EffectId, CustomEffectRegistrationError> {
        if let Some(id) = self.id {
            return Ok(id);
        }

        let id = effects.register_custom_effect_v2(self.desc.clone())?;
        self.id = Some(id);
        Ok(id)
    }

    pub fn unregister(&mut self, effects: &mut dyn CustomEffectService) -> bool {
        let Some(id) = self.id else {
            return false;
        };
        self.id = None;
        effects.unregister_custom_effect(id)
    }
}
