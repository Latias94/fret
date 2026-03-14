use fret_render_text::{FontTraceState, GenericFamilyInjectionState, TextFallbackPolicyV1};

pub(crate) struct TextFontRuntimeState {
    pub(crate) font_stack_key: u64,
    pub(crate) font_db_revision: u64,
    pub(crate) fallback_policy: TextFallbackPolicyV1,
    pub(crate) generic_injections: GenericFamilyInjectionState,
    pub(crate) font_trace: FontTraceState,
}

impl TextFontRuntimeState {
    pub(crate) fn new(fallback_policy: TextFallbackPolicyV1) -> Self {
        Self {
            font_stack_key: 1,
            font_db_revision: 1,
            fallback_policy,
            generic_injections: GenericFamilyInjectionState::default(),
            font_trace: FontTraceState::default(),
        }
    }

    pub(crate) fn recompute_font_stack_key(&mut self) {
        use std::hash::{Hash as _, Hasher as _};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "fret.text.font_stack_key.v1".hash(&mut hasher);
        self.font_db_revision.hash(&mut hasher);
        self.fallback_policy.fallback_policy_key.hash(&mut hasher);
        let key = hasher.finish();
        self.font_stack_key = if key == 0 { 1 } else { key };
    }
}
