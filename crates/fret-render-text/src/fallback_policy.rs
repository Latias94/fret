use crate::parley_shaper::ParleyShaper;
use parley::fontique::FamilyId as ParleyFamilyId;
use std::{
    collections::HashSet,
    hash::{Hash as _, Hasher as _},
    sync::OnceLock,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonFallbackMode {
    PreferSystemFallback,
    PreferCommonFallback,
}

#[derive(Debug, Clone)]
pub struct TextFallbackPolicyV1 {
    /// Last applied config inputs (runner-owned, portable).
    pub font_family_config: fret_core::TextFontFamilyConfig,
    /// Last applied shaping locale (BCP47).
    pub locale_bcp47: Option<String>,

    /// Derived, renderer-internal policy state.
    pub common_fallback_mode: CommonFallbackMode,
    pub common_fallback_candidates: Vec<String>,
    pub common_fallback_stack_suffix: String,

    /// Fingerprint of the effective fallback policy, intended for diagnostics + cache invalidation.
    pub fallback_policy_key: u64,
}

impl TextFallbackPolicyV1 {
    pub fn new(shaper: &ParleyShaper) -> Self {
        let mut out = Self {
            font_family_config: fret_core::TextFontFamilyConfig::default(),
            locale_bcp47: None,
            common_fallback_mode: CommonFallbackMode::PreferSystemFallback,
            common_fallback_candidates: Vec::new(),
            common_fallback_stack_suffix: String::new(),
            // Non-zero by default so callers can treat `0` as "unknown/uninitialized" if desired.
            fallback_policy_key: 1,
        };
        out.refresh_derived(shaper);
        out.recompute_key(shaper);
        out
    }

    pub fn prefer_common_fallback(&self) -> bool {
        self.common_fallback_mode == CommonFallbackMode::PreferCommonFallback
    }

    fn platform_default_common_fallback_mode(shaper: &ParleyShaper) -> CommonFallbackMode {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = shaper;
            CommonFallbackMode::PreferCommonFallback
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if shaper.system_fonts_enabled() {
                CommonFallbackMode::PreferSystemFallback
            } else {
                CommonFallbackMode::PreferCommonFallback
            }
        }
    }

    pub fn refresh_derived(&mut self, shaper: &ParleyShaper) {
        self.common_fallback_mode = match self.font_family_config.common_fallback_injection {
            fret_core::TextCommonFallbackInjection::PlatformDefault => {
                Self::platform_default_common_fallback_mode(shaper)
            }
            fret_core::TextCommonFallbackInjection::None => {
                CommonFallbackMode::PreferSystemFallback
            }
            fret_core::TextCommonFallbackInjection::CommonFallback => {
                CommonFallbackMode::PreferCommonFallback
            }
        };

        self.common_fallback_candidates = match self.common_fallback_mode {
            CommonFallbackMode::PreferSystemFallback => Vec::new(),
            CommonFallbackMode::PreferCommonFallback => effective_common_fallback_candidates(
                &self.font_family_config.common_fallback,
                default_common_fallback_families(shaper),
            ),
        };

        self.common_fallback_stack_suffix = match self.common_fallback_mode {
            CommonFallbackMode::PreferSystemFallback => String::new(),
            CommonFallbackMode::PreferCommonFallback => self.common_fallback_candidates.join(", "),
        };
    }

    pub fn recompute_key(&mut self, shaper: &ParleyShaper) {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "fret.text.fallback_policy.v1".hash(&mut hasher);

        shaper.system_fonts_enabled().hash(&mut hasher);
        match self.common_fallback_mode {
            CommonFallbackMode::PreferSystemFallback => 0u8.hash(&mut hasher),
            CommonFallbackMode::PreferCommonFallback => 1u8.hash(&mut hasher),
        }

        self.locale_bcp47
            .as_deref()
            .map(|v| v.to_ascii_lowercase())
            .hash(&mut hasher);

        match self.font_family_config.common_fallback_injection {
            fret_core::TextCommonFallbackInjection::PlatformDefault => 0u8.hash(&mut hasher),
            fret_core::TextCommonFallbackInjection::None => 1u8.hash(&mut hasher),
            fret_core::TextCommonFallbackInjection::CommonFallback => 2u8.hash(&mut hasher),
        }

        normalize_and_hash_family_candidates(&mut hasher, &self.font_family_config.ui_sans);
        normalize_and_hash_family_candidates(&mut hasher, &self.font_family_config.ui_serif);
        normalize_and_hash_family_candidates(&mut hasher, &self.font_family_config.ui_mono);
        normalize_and_hash_family_candidates(&mut hasher, &self.font_family_config.common_fallback);

        for &family in default_common_fallback_families(shaper) {
            family.trim().to_ascii_lowercase().hash(&mut hasher);
        }

        shaper
            .common_fallback_stack_suffix()
            .trim()
            .to_ascii_lowercase()
            .hash(&mut hasher);

        let key = hasher.finish();
        self.fallback_policy_key = if key == 0 { 1 } else { key };
    }

    pub fn diagnostics_snapshot(
        &self,
        frame_id: fret_core::FrameId,
        font_stack_key: u64,
        font_db_revision: u64,
        shaper: &ParleyShaper,
    ) -> fret_core::RendererTextFallbackPolicySnapshot {
        fret_core::RendererTextFallbackPolicySnapshot {
            frame_id,
            font_stack_key,
            font_db_revision,
            fallback_policy_key: self.fallback_policy_key,
            system_fonts_enabled: shaper.system_fonts_enabled(),
            locale_bcp47: self.locale_bcp47.clone(),
            common_fallback_injection: self.font_family_config.common_fallback_injection,
            prefer_common_fallback: self.prefer_common_fallback(),
            configured_ui_sans_families: self.font_family_config.ui_sans.clone(),
            configured_ui_serif_families: self.font_family_config.ui_serif.clone(),
            configured_ui_mono_families: self.font_family_config.ui_mono.clone(),
            configured_common_fallback_families: self.font_family_config.common_fallback.clone(),
            default_ui_sans_candidates: default_sans_candidates(shaper)
                .iter()
                .map(|family| (*family).to_string())
                .collect(),
            default_ui_serif_candidates: default_serif_candidates(shaper)
                .iter()
                .map(|family| (*family).to_string())
                .collect(),
            default_ui_mono_candidates: default_monospace_candidates(shaper)
                .iter()
                .map(|family| (*family).to_string())
                .collect(),
            default_common_fallback_families: default_common_fallback_families(shaper)
                .iter()
                .map(|family| (*family).to_string())
                .collect(),
            common_fallback_stack_suffix: shaper.common_fallback_stack_suffix().to_string(),
            common_fallback_candidates: self.common_fallback_candidates.clone(),
            bundled_profile_contract: bundled_profile_contract_snapshot(),
        }
    }
}

#[cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]
fn merged_static_family_lists(lists: &[&[&'static str]]) -> Box<[&'static str]> {
    let mut seen_lower: HashSet<String> = HashSet::new();
    let mut families: Vec<&'static str> = Vec::new();
    for list in lists {
        for &family in *list {
            let trimmed = family.trim();
            if trimmed.is_empty() {
                continue;
            }
            let key = trimmed.to_ascii_lowercase();
            if seen_lower.insert(key) {
                families.push(trimmed);
            }
        }
    }
    families.into_boxed_slice()
}

#[cfg(target_arch = "wasm32")]
fn bundled_only_default_common_fallback_families() -> &'static [&'static str] {
    static FAMILIES: OnceLock<Box<[&'static str]>> = OnceLock::new();
    FAMILIES.get_or_init(|| {
        let profile = fret_fonts::default_profile();
        merged_static_family_lists(&[profile.ui_sans_families, profile.common_fallback_families])
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn bundled_only_default_common_fallback_families() -> &'static [&'static str] {
    static FAMILIES: OnceLock<Box<[&'static str]>> = OnceLock::new();
    FAMILIES.get_or_init(|| {
        let profile = fret_fonts::default_profile();
        merged_static_family_lists(&[profile.ui_sans_families, profile.common_fallback_families])
    })
}

#[cfg(target_os = "windows")]
fn platform_default_common_fallback_families() -> &'static [&'static str] {
    &[
        // UI
        "Segoe UI",
        "Tahoma",
        // CJK
        "Microsoft YaHei UI",
        "Microsoft YaHei",
        "Yu Gothic UI",
        "Meiryo UI",
        "Meiryo",
        "Nirmala UI",
        // Emoji
        "Segoe UI Emoji",
        "Segoe UI Symbol",
    ]
}

#[cfg(target_os = "macos")]
fn platform_default_common_fallback_families() -> &'static [&'static str] {
    &[
        // UI
        "SF Pro Text",
        ".SF NS Text",
        "Helvetica Neue",
        // CJK
        "PingFang SC",
        "PingFang TC",
        "Hiragino Sans",
        // Emoji
        "Apple Color Emoji",
    ]
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
fn platform_default_common_fallback_families() -> &'static [&'static str] {
    &[
        // UI
        "Noto Sans",
        "DejaVu Sans",
        "Liberation Sans",
        // CJK
        "Noto Sans CJK JP",
        "Noto Sans CJK TC",
    ]
}

#[cfg(not(any(
    target_arch = "wasm32",
    target_os = "windows",
    target_os = "macos",
    all(unix, not(any(target_os = "macos", target_os = "android")))
)))]
fn platform_default_common_fallback_families() -> &'static [&'static str] {
    &[]
}

#[cfg(not(target_arch = "wasm32"))]
fn native_default_common_fallback_families() -> &'static [&'static str] {
    static FAMILIES: OnceLock<Box<[&'static str]>> = OnceLock::new();
    FAMILIES.get_or_init(|| {
        merged_static_family_lists(&[
            platform_default_common_fallback_families(),
            fret_fonts::default_profile().common_fallback_families,
        ])
    })
}

pub fn default_common_fallback_families(shaper: &ParleyShaper) -> &'static [&'static str] {
    // Bundled-only mode should be explicit and deterministic on both wasm and native.
    if !shaper.system_fonts_enabled() {
        return bundled_only_default_common_fallback_families();
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = shaper;
        bundled_only_default_common_fallback_families()
    }
    #[cfg(target_os = "windows")]
    {
        native_default_common_fallback_families()
    }
    #[cfg(target_os = "macos")]
    {
        native_default_common_fallback_families()
    }
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    {
        native_default_common_fallback_families()
    }
    #[cfg(not(any(
        target_arch = "wasm32",
        target_os = "windows",
        target_os = "macos",
        all(unix, not(any(target_os = "macos", target_os = "android")))
    )))]
    {
        let _ = shaper;
        &[]
    }
}

pub fn first_available_family_id(
    shaper: &mut ParleyShaper,
    candidates: &[&str],
) -> Option<ParleyFamilyId> {
    for &name in candidates {
        if let Some(id) = shaper.resolve_family_id(name) {
            return Some(id);
        }
    }
    None
}

pub fn common_fallback_stack_suffix(
    common_fallback_config: &[String],
    defaults: &'static [&'static str],
) -> String {
    effective_common_fallback_candidates(common_fallback_config, defaults).join(", ")
}

pub fn effective_common_fallback_candidates(
    common_fallback_config: &[String],
    defaults: &'static [&'static str],
) -> Vec<String> {
    let mut seen_lower: HashSet<String> = HashSet::new();
    let mut families: Vec<String> = Vec::new();

    let mut push = |name: &str| {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return;
        }
        let key = trimmed.to_ascii_lowercase();
        if seen_lower.insert(key) {
            families.push(trimmed.to_string());
        }
    };

    for family in common_fallback_config {
        push(family);
    }
    for &family in defaults {
        push(family);
    }

    families
}

pub fn common_fallback_stack_suffix_max_families() -> usize {
    static MAX: OnceLock<usize> = OnceLock::new();
    *MAX.get_or_init(|| {
        // Keep the explicit per-style fallback list bounded to avoid pathological slowdowns when
        // users copy-paste huge fallback stacks.
        std::env::var("FRET_TEXT_COMMON_FALLBACK_MAX_FAMILIES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(64)
            .clamp(1, 256)
    })
}

fn normalize_and_hash_family_candidates(
    hasher: &mut std::collections::hash_map::DefaultHasher,
    candidates: &[String],
) {
    let mut out: Vec<String> = Vec::new();
    for c in candidates {
        let trimmed = c.trim();
        if trimmed.is_empty() {
            continue;
        }
        out.push(trimmed.to_ascii_lowercase());
    }
    out.hash(hasher);
}

pub fn default_sans_candidates(shaper: &ParleyShaper) -> &'static [&'static str] {
    if !shaper.system_fonts_enabled() {
        return fret_fonts::default_profile().ui_sans_families;
    }
    #[cfg(target_os = "windows")]
    {
        &["Segoe UI", "Tahoma", "Arial"]
    }
    #[cfg(target_os = "macos")]
    {
        &["SF Pro Text", ".SF NS Text", "Helvetica Neue", "Helvetica"]
    }
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    {
        &["Noto Sans", "DejaVu Sans", "Liberation Sans"]
    }
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        all(unix, not(any(target_os = "macos", target_os = "android")))
    )))]
    {
        let _ = shaper;
        &[]
    }
}

pub fn default_monospace_candidates(shaper: &ParleyShaper) -> &'static [&'static str] {
    if !shaper.system_fonts_enabled() {
        return fret_fonts::default_profile().ui_mono_families;
    }
    #[cfg(target_os = "windows")]
    {
        &["Cascadia Mono", "Consolas", "Courier New"]
    }
    #[cfg(target_os = "macos")]
    {
        &["SF Mono", "Menlo", "Monaco"]
    }
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    {
        &["Noto Sans Mono", "DejaVu Sans Mono", "Liberation Mono"]
    }
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        all(unix, not(any(target_os = "macos", target_os = "android")))
    )))]
    {
        let _ = shaper;
        &[]
    }
}

pub fn default_serif_candidates(shaper: &ParleyShaper) -> &'static [&'static str] {
    if !shaper.system_fonts_enabled() {
        return fret_fonts::default_profile().ui_serif_families;
    }
    #[cfg(target_os = "windows")]
    {
        &["Times New Roman", "Georgia"]
    }
    #[cfg(target_os = "macos")]
    {
        &["New York", "Times New Roman", "Times"]
    }
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    {
        &["DejaVu Serif", "Noto Serif", "Liberation Serif"]
    }
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        all(unix, not(any(target_os = "macos", target_os = "android")))
    )))]
    {
        let _ = shaper;
        &[]
    }
}

fn bundled_profile_contract_snapshot() -> fret_core::RendererBundledFontProfileSnapshot {
    let profile = fret_fonts::default_profile();

    fn role_name(role: fret_fonts::BundledFontRole) -> &'static str {
        match role {
            fret_fonts::BundledFontRole::UiSans => "ui_sans",
            fret_fonts::BundledFontRole::UiSerif => "ui_serif",
            fret_fonts::BundledFontRole::UiMonospace => "ui_monospace",
            fret_fonts::BundledFontRole::EmojiFallback => "emoji_fallback",
            fret_fonts::BundledFontRole::CjkFallback => "cjk_fallback",
        }
    }

    fn generic_name(family: fret_fonts::BundledGenericFamily) -> &'static str {
        match family {
            fret_fonts::BundledGenericFamily::Sans => "sans",
            fret_fonts::BundledGenericFamily::Serif => "serif",
            fret_fonts::BundledGenericFamily::Monospace => "monospace",
        }
    }

    fret_core::RendererBundledFontProfileSnapshot {
        name: profile.name.to_string(),
        provided_roles: profile
            .provided_roles
            .iter()
            .map(|role| role_name(*role).to_string())
            .collect(),
        expected_family_names: profile
            .expected_family_names
            .iter()
            .map(|family| (*family).to_string())
            .collect(),
        guaranteed_generic_families: profile
            .guaranteed_generic_families
            .iter()
            .map(|family| generic_name(*family).to_string())
            .collect(),
        ui_sans_families: profile
            .ui_sans_families
            .iter()
            .map(|family| (*family).to_string())
            .collect(),
        ui_serif_families: profile
            .ui_serif_families
            .iter()
            .map(|family| (*family).to_string())
            .collect(),
        ui_mono_families: profile
            .ui_mono_families
            .iter()
            .map(|family| (*family).to_string())
            .collect(),
        common_fallback_families: profile
            .common_fallback_families
            .iter()
            .map(|family| (*family).to_string())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merged_static_family_lists_preserves_order_and_dedupes_case_insensitively() {
        let families = merged_static_family_lists(&[
            &["Inter", "Noto Sans CJK SC"],
            &["inter", "Noto Color Emoji", "Noto Sans CJK SC"],
        ]);
        assert_eq!(
            families.as_ref(),
            &["Inter", "Noto Sans CJK SC", "Noto Color Emoji"]
        );
    }
}
