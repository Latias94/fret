use crate::{fallback_policy, fallback_policy::TextFallbackPolicyV1, parley_shaper::ParleyShaper};
use std::{
    collections::{HashSet, VecDeque},
    sync::OnceLock,
};

#[derive(Debug, Default, Clone)]
pub struct FontTraceState {
    active: bool,
    entries: VecDeque<fret_core::RendererTextFontTraceEntry>,
}

#[derive(Debug, Clone)]
pub struct FontTraceFamilyResolved {
    family: String,
    glyphs: u32,
    missing_glyphs: u32,
}

impl FontTraceFamilyResolved {
    pub fn new(family: String, glyphs: u32, missing_glyphs: u32) -> Self {
        Self {
            family,
            glyphs,
            missing_glyphs,
        }
    }

    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn glyphs(&self) -> u32 {
        self.glyphs
    }

    pub fn missing_glyphs(&self) -> u32 {
        self.missing_glyphs
    }

    pub fn into_parts(self) -> (String, u32, u32) {
        (self.family, self.glyphs, self.missing_glyphs)
    }
}

impl FontTraceState {
    pub fn begin_frame(&mut self) {
        self.active = true;
        self.entries.clear();
    }

    pub fn snapshot(
        &self,
        frame_id: fret_core::FrameId,
    ) -> fret_core::RendererTextFontTraceSnapshot {
        fret_core::RendererTextFontTraceSnapshot {
            frame_id,
            entries: self.entries.iter().cloned().collect(),
        }
    }

    pub fn maybe_record(
        &mut self,
        text: &str,
        style: &fret_core::TextStyle,
        constraints: fret_core::TextConstraints,
        fallback_policy: &TextFallbackPolicyV1,
        shaper: &ParleyShaper,
        missing_glyphs: u32,
        families: Vec<FontTraceFamilyResolved>,
    ) {
        if !self.active {
            return;
        }

        let record_all = font_trace_record_all();
        if !record_all && missing_glyphs == 0 {
            return;
        }

        let max_entries = font_trace_entries_limit();
        if max_entries == 0 {
            return;
        }

        let max_text_bytes = font_trace_max_text_bytes();
        let text_preview = truncate_text_preview(text, max_text_bytes);

        let mut common_fallback_lower: HashSet<String> = HashSet::new();
        if fallback_policy.prefer_common_fallback() {
            for f in fallback_policy.common_fallback_candidates() {
                common_fallback_lower.insert(f.trim().to_ascii_lowercase());
            }
        }
        let requested_generic_lower =
            requested_generic_lower_families(&style.font, fallback_policy, shaper);

        let mut usages: Vec<fret_core::RendererTextFontTraceFamilyUsage> =
            Vec::with_capacity(families.len().max(1));
        for family in families {
            let class = classify_trace_family(
                &style.font,
                family.family(),
                &requested_generic_lower,
                &common_fallback_lower,
            );
            let (family, glyphs, missing_glyphs) = family.into_parts();
            usages.push(fret_core::RendererTextFontTraceFamilyUsage {
                family,
                glyphs,
                missing_glyphs,
                class,
            });
        }

        let entry = fret_core::RendererTextFontTraceEntry {
            text_preview,
            text_len_bytes: text.len().min(u32::MAX as usize) as u32,
            font: style.font.clone(),
            font_size: style.size,
            scale_factor: constraints.scale_factor,
            wrap: constraints.wrap,
            overflow: constraints.overflow,
            max_width: constraints.max_width,
            locale_bcp47: fallback_policy.locale_bcp47().map(str::to_string),
            missing_glyphs,
            families: usages,
        };

        self.entries.push_back(entry);
        while self.entries.len() > max_entries {
            self.entries.pop_front();
        }
    }
}

fn font_trace_record_all() -> bool {
    static FLAG: OnceLock<bool> = OnceLock::new();
    *FLAG.get_or_init(|| {
        std::env::var("FRET_TEXT_FONT_TRACE_ALL")
            .ok()
            .is_some_and(|v| !v.trim().is_empty() && v.trim() != "0")
    })
}

fn font_trace_entries_limit() -> usize {
    static LIMIT: OnceLock<usize> = OnceLock::new();
    *LIMIT.get_or_init(|| {
        std::env::var("FRET_TEXT_FONT_TRACE_ENTRIES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(64)
            .min(4096)
    })
}

fn font_trace_max_text_bytes() -> usize {
    static LIMIT: OnceLock<usize> = OnceLock::new();
    *LIMIT.get_or_init(|| {
        std::env::var("FRET_TEXT_FONT_TRACE_MAX_TEXT_BYTES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(256)
            .clamp(16, 16 * 1024)
    })
}

fn truncate_text_preview(text: &str, max_bytes: usize) -> String {
    if max_bytes == 0 || text.len() <= max_bytes {
        return text.to_string();
    }

    let mut end = max_bytes.min(text.len());
    while end > 0 && !text.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    let mut out = text[..end].to_string();
    out.push('…');
    out
}

fn classify_trace_family(
    requested: &fret_core::FontId,
    family: &str,
    requested_generic_lower: &HashSet<String>,
    common_fallback_lower: &HashSet<String>,
) -> fret_core::RendererTextFontTraceFamilyClass {
    let family_lower = family.trim().to_ascii_lowercase();
    if family_lower.is_empty() {
        return fret_core::RendererTextFontTraceFamilyClass::Unknown;
    }

    let is_requested_generic = requested_generic_lower.contains(&family_lower);
    let is_common = common_fallback_lower.contains(&family_lower);
    match requested {
        fret_core::FontId::Family(name) => {
            if name.eq_ignore_ascii_case(family) {
                fret_core::RendererTextFontTraceFamilyClass::Requested
            } else if is_common {
                fret_core::RendererTextFontTraceFamilyClass::CommonFallback
            } else {
                fret_core::RendererTextFontTraceFamilyClass::SystemFallback
            }
        }
        _ => {
            if is_requested_generic {
                fret_core::RendererTextFontTraceFamilyClass::Requested
            } else if is_common {
                fret_core::RendererTextFontTraceFamilyClass::CommonFallback
            } else {
                fret_core::RendererTextFontTraceFamilyClass::SystemFallback
            }
        }
    }
}

fn requested_generic_lower_families(
    requested: &fret_core::FontId,
    fallback_policy: &TextFallbackPolicyV1,
    shaper: &ParleyShaper,
) -> HashSet<String> {
    let (configured, defaults): (&[String], &[&str]) = match requested {
        fret_core::FontId::Ui => (
            &fallback_policy.font_family_config().ui_sans,
            fallback_policy::default_sans_candidates(shaper),
        ),
        fret_core::FontId::Serif => (
            &fallback_policy.font_family_config().ui_serif,
            fallback_policy::default_serif_candidates(shaper),
        ),
        fret_core::FontId::Monospace => (
            &fallback_policy.font_family_config().ui_mono,
            fallback_policy::default_monospace_candidates(shaper),
        ),
        fret_core::FontId::Family(_) => return HashSet::new(),
    };

    let mut families = HashSet::new();
    for family in configured {
        let family = family.trim().to_ascii_lowercase();
        if !family.is_empty() {
            families.insert(family);
        }
    }
    for family in defaults {
        let family = family.trim().to_ascii_lowercase();
        if !family.is_empty() {
            families.insert(family);
        }
    }
    families
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_requested_lane_includes_configured_and_default_candidates() {
        let shaper = ParleyShaper::new_without_system_fonts();
        let mut policy = TextFallbackPolicyV1::new(&shaper);
        let mut config = policy.font_family_config().clone();
        config.ui_sans = vec!["Custom UI".to_string()];
        policy.set_font_family_config(config);

        let families = requested_generic_lower_families(&fret_core::FontId::Ui, &policy, &shaper);
        assert!(families.contains("custom ui"));
        for family in fret_fonts::default_profile().ui_sans_families {
            assert!(
                families.contains(&family.to_ascii_lowercase()),
                "expected requested generic lane to include bundled default sans family {family:?}"
            );
        }
    }

    #[test]
    fn generic_requested_class_beats_common_fallback_overlap() {
        let requested_generic_lower = HashSet::from([String::from("inter")]);
        let common_fallback_lower = HashSet::from([String::from("inter")]);

        let class = classify_trace_family(
            &fret_core::FontId::Ui,
            "Inter",
            &requested_generic_lower,
            &common_fallback_lower,
        );

        assert_eq!(
            class,
            fret_core::RendererTextFontTraceFamilyClass::Requested
        );
    }

    #[test]
    fn generic_nonrequested_noncommon_family_is_system_fallback() {
        let requested_generic_lower = HashSet::from([String::from("inter")]);
        let common_fallback_lower = HashSet::from([String::from("noto sans cjk sc")]);

        let class = classify_trace_family(
            &fret_core::FontId::Ui,
            "Segoe UI Emoji",
            &requested_generic_lower,
            &common_fallback_lower,
        );

        assert_eq!(
            class,
            fret_core::RendererTextFontTraceFamilyClass::SystemFallback
        );
    }
}
