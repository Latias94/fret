use crate::{fallback_policy, parley_shaper::ParleyShaper};
use parley::fontique::FamilyId as ParleyFamilyId;
use parley::fontique::GenericFamily as ParleyGenericFamily;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct GenericFamilyInjectionState {
    injected_by_generic: HashMap<ParleyGenericFamily, Vec<ParleyFamilyId>>,
}

impl GenericFamilyInjectionState {
    pub fn clear(&mut self) {
        self.injected_by_generic.clear();
    }

    pub fn apply_generic_stack(
        &mut self,
        shaper: &mut ParleyShaper,
        generic: ParleyGenericFamily,
        primary: Option<ParleyFamilyId>,
        fallbacks: &[ParleyFamilyId],
    ) -> bool {
        let mut injected: Vec<ParleyFamilyId> = Vec::new();
        if let Some(id) = primary {
            injected.push(id);
        }
        for &id in fallbacks {
            if !injected.contains(&id) {
                injected.push(id);
            }
        }

        let prev_injected = self
            .injected_by_generic
            .get(&generic)
            .cloned()
            .unwrap_or_default();

        let mut base = shaper.generic_family_ids(generic);
        if !prev_injected.is_empty() {
            base.retain(|id| !prev_injected.contains(id));
        }

        let mut next: Vec<ParleyFamilyId> = Vec::new();
        next.extend_from_slice(&injected);
        for id in base {
            if !next.contains(&id) {
                next.push(id);
            }
        }

        self.injected_by_generic.insert(generic, injected);
        shaper.set_generic_family_ids(generic, &next)
    }
}

pub fn pick_primary_family_id(
    shaper: &mut ParleyShaper,
    overrides: &[String],
    defaults: &'static [&'static str],
) -> Option<ParleyFamilyId> {
    for candidate in overrides {
        if let Some(id) = shaper.resolve_family_id(candidate) {
            return Some(id);
        }
    }
    fallback_policy::first_available_family_id(shaper, defaults)
}

pub fn resolve_common_fallback_ids_and_suffix(
    shaper: &mut ParleyShaper,
    candidates: &[String],
) -> (Vec<ParleyFamilyId>, String) {
    let mut resolved_suffix: Vec<String> = Vec::new();
    let mut fallback_ids: Vec<ParleyFamilyId> = Vec::new();
    let max = fallback_policy::common_fallback_stack_suffix_max_families();

    for family in candidates {
        if let Some(id) = shaper.resolve_family_id(family) {
            let pushed = if !fallback_ids.contains(&id) {
                fallback_ids.push(id);
                true
            } else {
                false
            };
            if pushed && resolved_suffix.len() < max {
                resolved_suffix.push(family.clone());
            }
        }
    }

    (fallback_ids, resolved_suffix.join(", "))
}

pub fn apply_font_families_inner(
    shaper: &mut ParleyShaper,
    policy: &mut fallback_policy::TextFallbackPolicyV1,
    injections: &mut GenericFamilyInjectionState,
    config: &fret_core::TextFontFamilyConfig,
) -> (bool, bool, bool) {
    let prev_mode = policy.common_fallback_mode;

    policy.font_family_config = config.clone();
    policy.refresh_derived(shaper);
    let mode_changed = policy.common_fallback_mode != prev_mode;

    let sans = pick_primary_family_id(
        shaper,
        &config.ui_sans,
        fallback_policy::default_sans_candidates(shaper),
    );
    let serif = pick_primary_family_id(
        shaper,
        &config.ui_serif,
        fallback_policy::default_serif_candidates(shaper),
    );
    let mono = pick_primary_family_id(
        shaper,
        &config.ui_mono,
        fallback_policy::default_monospace_candidates(shaper),
    );

    let (fallback_ids, suffix) = if policy.prefer_common_fallback() {
        resolve_common_fallback_ids_and_suffix(shaper, &policy.common_fallback_candidates)
    } else {
        (Vec::new(), String::new())
    };

    policy.common_fallback_stack_suffix = suffix;
    let suffix_changed =
        shaper.set_common_fallback_stack_suffix(policy.common_fallback_stack_suffix.clone());

    let mut changed = false;
    changed |=
        injections.apply_generic_stack(shaper, ParleyGenericFamily::SansSerif, sans, &fallback_ids);
    changed |=
        injections.apply_generic_stack(shaper, ParleyGenericFamily::SystemUi, sans, &fallback_ids);
    changed |= injections.apply_generic_stack(
        shaper,
        ParleyGenericFamily::UiSansSerif,
        sans,
        &fallback_ids,
    );
    changed |=
        injections.apply_generic_stack(shaper, ParleyGenericFamily::Serif, serif, &fallback_ids);
    changed |=
        injections.apply_generic_stack(shaper, ParleyGenericFamily::UiSerif, serif, &fallback_ids);
    changed |=
        injections.apply_generic_stack(shaper, ParleyGenericFamily::Monospace, mono, &fallback_ids);
    changed |= injections.apply_generic_stack(
        shaper,
        ParleyGenericFamily::UiMonospace,
        mono,
        &fallback_ids,
    );
    changed |=
        injections.apply_generic_stack(shaper, ParleyGenericFamily::Emoji, None, &fallback_ids);

    (changed, suffix_changed, mode_changed)
}
