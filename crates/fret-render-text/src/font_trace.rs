use crate::fallback_policy::TextFallbackPolicyV1;
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
    pub family: String,
    pub glyphs: u32,
    pub missing_glyphs: u32,
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
            for f in &fallback_policy.common_fallback_candidates {
                common_fallback_lower.insert(f.trim().to_ascii_lowercase());
            }
        }

        let mut usages: Vec<fret_core::RendererTextFontTraceFamilyUsage> =
            Vec::with_capacity(families.len().max(1));
        for family in families {
            let class = classify_trace_family(&style.font, &family.family, &common_fallback_lower);
            usages.push(fret_core::RendererTextFontTraceFamilyUsage {
                family: family.family,
                glyphs: family.glyphs,
                missing_glyphs: family.missing_glyphs,
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
            locale_bcp47: fallback_policy.locale_bcp47.clone(),
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
    common_fallback_lower: &HashSet<String>,
) -> fret_core::RendererTextFontTraceFamilyClass {
    let is_common = common_fallback_lower.contains(&family.trim().to_ascii_lowercase());
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
            if is_common {
                fret_core::RendererTextFontTraceFamilyClass::CommonFallback
            } else {
                fret_core::RendererTextFontTraceFamilyClass::Unknown
            }
        }
    }
}
