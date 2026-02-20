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
                default_common_fallback_families(),
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

        for &family in default_common_fallback_families() {
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
}

pub fn default_common_fallback_families() -> &'static [&'static str] {
    // For Web/WASM, there are no system fonts. If the app bundles fonts (e.g. via `fret-fonts`
    // feature flags), include those families in the fallback chain so mixed-script text works
    // without explicit per-span font selection.
    #[cfg(target_arch = "wasm32")]
    {
        &[
            // UI (bundled in `fret-fonts` bootstrap)
            "Inter",
            // CJK (bundled via `fret-fonts/cjk-lite`)
            "Noto Sans CJK SC",
            // Emoji (bundled via `fret-fonts/emoji`)
            "Noto Color Emoji",
        ]
    }
    #[cfg(target_os = "windows")]
    {
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
            // Bundled/portable fallbacks (if available)
            "Noto Sans CJK SC",
            // Emoji
            "Segoe UI Emoji",
            "Segoe UI Symbol",
            "Noto Color Emoji",
        ]
    }
    #[cfg(target_os = "macos")]
    {
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
            // Bundled/portable fallbacks (if available)
            "Noto Sans CJK SC",
            "Noto Color Emoji",
        ]
    }
    #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
    {
        &[
            // UI
            "Noto Sans",
            "DejaVu Sans",
            "Liberation Sans",
            // CJK
            "Noto Sans CJK SC",
            "Noto Sans CJK JP",
            "Noto Sans CJK TC",
            // Emoji
            "Noto Color Emoji",
        ]
    }
    #[cfg(not(any(
        target_arch = "wasm32",
        target_os = "windows",
        target_os = "macos",
        all(unix, not(any(target_os = "macos", target_os = "android")))
    )))]
    {
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

pub fn default_sans_candidates() -> &'static [&'static str] {
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
        &[]
    }
}

pub fn default_monospace_candidates() -> &'static [&'static str] {
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
        &[]
    }
}

pub fn default_serif_candidates() -> &'static [&'static str] {
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
        &[]
    }
}
