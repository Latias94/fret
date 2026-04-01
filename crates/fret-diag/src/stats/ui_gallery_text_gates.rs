use std::path::{Path, PathBuf};

use crate::util::write_json_value;

#[derive(Debug, Clone, PartialEq, Eq)]
struct BundledProfileContractEvidence {
    name: String,
    expected_family_names: Vec<String>,
    ui_sans_families: Vec<String>,
    ui_serif_families: Vec<String>,
    ui_mono_families: Vec<String>,
    common_fallback_families: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MixedScriptBundledFallbackEvidence {
    system_fonts_enabled: bool,
    prefer_common_fallback: bool,
    configured_common_fallback_families: Vec<String>,
    common_fallback_candidates: Vec<String>,
    default_ui_sans_candidates: Vec<String>,
    default_ui_serif_candidates: Vec<String>,
    default_ui_mono_candidates: Vec<String>,
    default_common_fallback_families: Vec<String>,
    bundled_profile: BundledProfileContractEvidence,
    font_trace_entry_count: usize,
    font_trace_families: Vec<String>,
    font_trace_common_fallback_families: Vec<String>,
    sample_trace_frame_id: Option<u64>,
    sample_trace_frame_missing_glyphs: Option<u64>,
    latin_families: Vec<String>,
    latin_classes: Vec<String>,
    cjk_families: Vec<String>,
    cjk_classes: Vec<String>,
    emoji_families: Vec<String>,
    emoji_classes: Vec<String>,
    mixed_families: Vec<String>,
    mixed_classes: Vec<String>,
    registered_font_blobs_count: u64,
    registered_font_blobs_total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocaleChangeConformanceEvidence {
    fallback_policy_key: u64,
    locale_bcp47: Option<String>,
    system_fonts_enabled: bool,
    prefer_common_fallback: bool,
    prefer_common_fallback_for_generics: bool,
    common_fallback_injection: Option<String>,
    missing_glyphs: u64,
    font_trace_entry_count: usize,
    font_trace_locales: Vec<String>,
    common_fallback_candidates: Vec<String>,
    sample_trace_frame_id: Option<u64>,
    sample_trace_locales: Vec<String>,
    latin_families: Vec<String>,
    latin_classes: Vec<String>,
    cjk_families: Vec<String>,
    cjk_classes: Vec<String>,
    emoji_families: Vec<String>,
    emoji_classes: Vec<String>,
    mixed_families: Vec<String>,
    mixed_classes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SettingsChangeConformanceEvidence {
    fallback_policy_key: u64,
    system_fonts_enabled: bool,
    prefer_common_fallback: bool,
    common_fallback_injection: Option<String>,
    configured_common_fallback_families: Vec<String>,
    common_fallback_candidates: Vec<String>,
    common_fallback_stack_suffix: String,
}

const LOCALE_CHANGE_TRACE_SAMPLE_LATIN: &str = "m";
const LOCALE_CHANGE_TRACE_SAMPLE_CJK: &str = "你";
const LOCALE_CHANGE_TRACE_SAMPLE_EMOJI: &str = "\u{1F600}";
const LOCALE_CHANGE_TRACE_SAMPLE_MIXED: &str = "m你\u{1F600}";

fn find_latest_labeled_bundle_dir(out_dir: &Path, label: &str) -> Option<PathBuf> {
    let suffix = format!("-{label}");
    let mut best: Option<(u64, PathBuf)> = None;
    let entries = std::fs::read_dir(out_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name()?.to_str()?.to_string();
        if !name.ends_with(&suffix) {
            continue;
        }
        let ts = name.split('-').next()?.parse::<u64>().ok()?;
        let bundle = crate::resolve_bundle_artifact_path(&path);
        if !bundle.is_file() {
            continue;
        }
        match &best {
            Some((best_ts, _)) if *best_ts >= ts => {}
            _ => best = Some((ts, path)),
        }
    }
    best.map(|(_, p)| p)
}

fn bundle_last_snapshot_by_frame_id(bundle: &serde_json::Value) -> Option<&serde_json::Value> {
    let windows = bundle.get("windows")?.as_array()?;
    let w = windows.first()?;
    let snaps = w.get("snapshots")?.as_array()?;
    snaps
        .iter()
        .filter_map(|s| Some((s.get("frame_id")?.as_u64()?, s)))
        .max_by_key(|(frame_id, _)| *frame_id)
        .map(|(_, s)| s)
}

fn json_string_vec(value: Option<&serde_json::Value>) -> Vec<String> {
    value
        .and_then(|v| v.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn push_unique_case_insensitive(out: &mut Vec<String>, value: &str) {
    if out
        .iter()
        .any(|existing| existing.eq_ignore_ascii_case(value))
    {
        return;
    }
    out.push(value.to_string());
}

fn merge_case_insensitive_preserve_order(left: &[String], right: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for family in left.iter().chain(right.iter()) {
        push_unique_case_insensitive(&mut out, family);
    }
    out
}

fn contains_case_insensitive(values: &[String], needle: &str) -> bool {
    values
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(needle))
}

fn bundle_last_text_mixed_script_evidence(
    bundle: &serde_json::Value,
) -> Option<MixedScriptBundledFallbackEvidence> {
    let best = bundle_last_snapshot_by_frame_id(bundle)?;
    let snapshots = bundle
        .get("windows")?
        .as_array()?
        .first()?
        .get("snapshots")?
        .as_array()?;

    let resource_caches = best.get("resource_caches")?.as_object()?;
    let render_text = resource_caches.get("render_text")?.as_object()?;
    let policy = resource_caches
        .get("render_text_fallback_policy")?
        .as_object()?;
    let bundled_profile = policy.get("bundled_profile_contract")?.as_object()?;

    let latest_sample_trace = snapshots
        .iter()
        .filter_map(|snapshot| {
            let frame_id = snapshot.get("frame_id")?.as_u64()?;
            let render_text = snapshot
                .get("resource_caches")?
                .get("render_text")?
                .as_object()?;
            let entries = snapshot
                .get("resource_caches")?
                .get("render_text_font_trace")?
                .get("entries")?
                .as_array()?;

            let latin = entries.iter().find(|entry| {
                entry.get("text_preview").and_then(|v| v.as_str())
                    == Some(LOCALE_CHANGE_TRACE_SAMPLE_LATIN)
            })?;
            let cjk = entries.iter().find(|entry| {
                entry.get("text_preview").and_then(|v| v.as_str())
                    == Some(LOCALE_CHANGE_TRACE_SAMPLE_CJK)
            })?;
            let emoji = entries.iter().find(|entry| {
                entry.get("text_preview").and_then(|v| v.as_str())
                    == Some(LOCALE_CHANGE_TRACE_SAMPLE_EMOJI)
            })?;
            let mixed = entries.iter().find(|entry| {
                entry.get("text_preview").and_then(|v| v.as_str())
                    == Some(LOCALE_CHANGE_TRACE_SAMPLE_MIXED)
            })?;

            let entry_families_and_classes = |entry: &serde_json::Value| {
                entry
                    .get("families")
                    .and_then(|v| v.as_array())
                    .map(|families| {
                        let mut family_names = Vec::new();
                        let mut classes = Vec::new();
                        for family in families {
                            if let Some(name) = family.get("family").and_then(|v| v.as_str()) {
                                family_names.push(name.to_string());
                                classes.push(
                                    family
                                        .get("class")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown")
                                        .to_string(),
                                );
                            }
                        }
                        (family_names, classes)
                    })
                    .unwrap_or_default()
            };

            let (latin_families, latin_classes) = entry_families_and_classes(latin);
            let (cjk_families, cjk_classes) = entry_families_and_classes(cjk);
            let (emoji_families, emoji_classes) = entry_families_and_classes(emoji);
            let (mixed_families, mixed_classes) = entry_families_and_classes(mixed);

            let mut font_trace_families = Vec::new();
            let mut font_trace_common_fallback_families = Vec::new();
            for (families, classes) in [
                (&latin_families, &latin_classes),
                (&cjk_families, &cjk_classes),
                (&emoji_families, &emoji_classes),
                (&mixed_families, &mixed_classes),
            ] {
                for (family, class) in families.iter().zip(classes.iter()) {
                    push_unique_case_insensitive(&mut font_trace_families, family);
                    if class == "common_fallback" {
                        push_unique_case_insensitive(
                            &mut font_trace_common_fallback_families,
                            family,
                        );
                    }
                }
            }

            Some((
                frame_id,
                render_text.get("frame_missing_glyphs")?.as_u64()?,
                entries.len(),
                font_trace_families,
                font_trace_common_fallback_families,
                latin_families,
                latin_classes,
                cjk_families,
                cjk_classes,
                emoji_families,
                emoji_classes,
                mixed_families,
                mixed_classes,
            ))
        })
        .max_by_key(|(frame_id, _, _, _, _, _, _, _, _, _, _, _, _)| *frame_id);

    Some(MixedScriptBundledFallbackEvidence {
        system_fonts_enabled: policy.get("system_fonts_enabled")?.as_bool()?,
        prefer_common_fallback: policy.get("prefer_common_fallback")?.as_bool()?,
        configured_common_fallback_families: json_string_vec(
            policy.get("configured_common_fallback_families"),
        ),
        common_fallback_candidates: json_string_vec(policy.get("common_fallback_candidates")),
        default_ui_sans_candidates: json_string_vec(policy.get("default_ui_sans_candidates")),
        default_ui_serif_candidates: json_string_vec(policy.get("default_ui_serif_candidates")),
        default_ui_mono_candidates: json_string_vec(policy.get("default_ui_mono_candidates")),
        default_common_fallback_families: json_string_vec(
            policy.get("default_common_fallback_families"),
        ),
        bundled_profile: BundledProfileContractEvidence {
            name: bundled_profile
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            expected_family_names: json_string_vec(bundled_profile.get("expected_family_names")),
            ui_sans_families: json_string_vec(bundled_profile.get("ui_sans_families")),
            ui_serif_families: json_string_vec(bundled_profile.get("ui_serif_families")),
            ui_mono_families: json_string_vec(bundled_profile.get("ui_mono_families")),
            common_fallback_families: json_string_vec(
                bundled_profile.get("common_fallback_families"),
            ),
        },
        font_trace_entry_count: latest_sample_trace
            .as_ref()
            .map(|(_, _, entry_count, _, _, _, _, _, _, _, _, _, _)| *entry_count)
            .unwrap_or_default(),
        font_trace_families: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, families, _, _, _, _, _, _, _, _, _)| families.clone())
            .unwrap_or_default(),
        font_trace_common_fallback_families: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, families, _, _, _, _, _, _, _, _)| families.clone())
            .unwrap_or_default(),
        sample_trace_frame_id: latest_sample_trace
            .as_ref()
            .map(|(frame_id, _, _, _, _, _, _, _, _, _, _, _, _)| *frame_id),
        sample_trace_frame_missing_glyphs: latest_sample_trace
            .as_ref()
            .map(|(_, missing_glyphs, _, _, _, _, _, _, _, _, _, _, _)| *missing_glyphs),
        latin_families: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, _, families, _, _, _, _, _, _, _)| families.clone())
            .unwrap_or_default(),
        latin_classes: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, _, _, classes, _, _, _, _, _, _)| classes.clone())
            .unwrap_or_default(),
        cjk_families: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, _, _, _, families, _, _, _, _, _)| families.clone())
            .unwrap_or_default(),
        cjk_classes: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, _, _, _, _, classes, _, _, _, _)| classes.clone())
            .unwrap_or_default(),
        emoji_families: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, _, _, _, _, _, families, _, _, _)| families.clone())
            .unwrap_or_default(),
        emoji_classes: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, _, _, _, _, _, _, classes, _, _)| classes.clone())
            .unwrap_or_default(),
        mixed_families: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, _, _, _, _, _, _, _, families, _)| families.clone())
            .unwrap_or_default(),
        mixed_classes: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, _, _, _, _, _, _, _, _, classes)| classes.clone())
            .unwrap_or_default(),
        registered_font_blobs_count: render_text.get("registered_font_blobs_count")?.as_u64()?,
        registered_font_blobs_total_bytes: render_text
            .get("registered_font_blobs_total_bytes")?
            .as_u64()?,
    })
}

fn bundle_last_text_locale_change_evidence(
    bundle: &serde_json::Value,
) -> Option<LocaleChangeConformanceEvidence> {
    let best = bundle_last_snapshot_by_frame_id(bundle)?;

    let resource_caches = best.get("resource_caches")?.as_object()?;
    let render_text = resource_caches.get("render_text")?.as_object()?;
    let policy = resource_caches
        .get("render_text_fallback_policy")?
        .as_object()?;
    let snapshots = bundle
        .get("windows")?
        .as_array()?
        .first()?
        .get("snapshots")?
        .as_array()?;

    let mut font_trace_locales = Vec::new();
    let font_trace_entry_count = resource_caches
        .get("render_text_font_trace")
        .and_then(|v| v.get("entries"))
        .and_then(|v| v.as_array())
        .map(|entries| {
            for entry in entries {
                let Some(locale) = entry.get("locale_bcp47").and_then(|v| v.as_str()) else {
                    continue;
                };
                push_unique_case_insensitive(&mut font_trace_locales, locale);
            }
            entries.len()
        })
        .unwrap_or_default();

    let latest_sample_trace = snapshots
        .iter()
        .filter_map(|snapshot| {
            let frame_id = snapshot.get("frame_id")?.as_u64()?;
            let entries = snapshot
                .get("resource_caches")?
                .get("render_text_font_trace")?
                .get("entries")?
                .as_array()?;

            let latin = entries.iter().find(|entry| {
                entry.get("text_preview").and_then(|v| v.as_str())
                    == Some(LOCALE_CHANGE_TRACE_SAMPLE_LATIN)
            })?;
            let cjk = entries.iter().find(|entry| {
                entry.get("text_preview").and_then(|v| v.as_str())
                    == Some(LOCALE_CHANGE_TRACE_SAMPLE_CJK)
            })?;
            let emoji = entries.iter().find(|entry| {
                entry.get("text_preview").and_then(|v| v.as_str())
                    == Some(LOCALE_CHANGE_TRACE_SAMPLE_EMOJI)
            })?;
            let mixed = entries.iter().find(|entry| {
                entry.get("text_preview").and_then(|v| v.as_str())
                    == Some(LOCALE_CHANGE_TRACE_SAMPLE_MIXED)
            })?;

            let mut sample_locales = Vec::new();
            for entry in [latin, cjk, emoji, mixed] {
                if let Some(locale) = entry.get("locale_bcp47").and_then(|v| v.as_str()) {
                    push_unique_case_insensitive(&mut sample_locales, locale);
                }
            }

            let entry_families_and_classes = |entry: &serde_json::Value| {
                entry
                    .get("families")
                    .and_then(|v| v.as_array())
                    .map(|families| {
                        let mut family_names = Vec::new();
                        let mut classes = Vec::new();
                        for family in families {
                            if let Some(name) = family.get("family").and_then(|v| v.as_str()) {
                                family_names.push(name.to_string());
                                classes.push(
                                    family
                                        .get("class")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown")
                                        .to_string(),
                                );
                            }
                        }
                        (family_names, classes)
                    })
                    .unwrap_or_default()
            };

            let (latin_families, latin_classes) = entry_families_and_classes(latin);
            let (cjk_families, cjk_classes) = entry_families_and_classes(cjk);
            let (emoji_families, emoji_classes) = entry_families_and_classes(emoji);
            let (mixed_families, mixed_classes) = entry_families_and_classes(mixed);

            Some((
                frame_id,
                sample_locales,
                latin_families,
                latin_classes,
                cjk_families,
                cjk_classes,
                emoji_families,
                emoji_classes,
                mixed_families,
                mixed_classes,
            ))
        })
        .max_by_key(|(frame_id, _, _, _, _, _, _, _, _, _)| *frame_id);

    Some(LocaleChangeConformanceEvidence {
        fallback_policy_key: policy.get("fallback_policy_key")?.as_u64()?,
        locale_bcp47: policy
            .get("locale_bcp47")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        system_fonts_enabled: policy.get("system_fonts_enabled")?.as_bool()?,
        prefer_common_fallback: policy.get("prefer_common_fallback")?.as_bool()?,
        prefer_common_fallback_for_generics: policy
            .get("prefer_common_fallback_for_generics")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        common_fallback_injection: policy
            .get("common_fallback_injection")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        missing_glyphs: render_text.get("frame_missing_glyphs")?.as_u64()?,
        font_trace_entry_count,
        font_trace_locales,
        common_fallback_candidates: json_string_vec(policy.get("common_fallback_candidates")),
        sample_trace_frame_id: latest_sample_trace
            .as_ref()
            .map(|(frame_id, _, _, _, _, _, _, _, _, _)| *frame_id),
        sample_trace_locales: latest_sample_trace
            .as_ref()
            .map(|(_, locales, _, _, _, _, _, _, _, _)| locales.clone())
            .unwrap_or_default(),
        latin_families: latest_sample_trace
            .as_ref()
            .map(|(_, _, families, _, _, _, _, _, _, _)| families.clone())
            .unwrap_or_default(),
        latin_classes: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, classes, _, _, _, _, _, _)| classes.clone())
            .unwrap_or_default(),
        cjk_families: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, families, _, _, _, _, _)| families.clone())
            .unwrap_or_default(),
        cjk_classes: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, _, classes, _, _, _, _)| classes.clone())
            .unwrap_or_default(),
        emoji_families: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, _, _, families, _, _, _)| families.clone())
            .unwrap_or_default(),
        emoji_classes: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, _, _, _, classes, _, _)| classes.clone())
            .unwrap_or_default(),
        mixed_families: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, _, _, _, _, families, _)| families.clone())
            .unwrap_or_default(),
        mixed_classes: latest_sample_trace
            .as_ref()
            .map(|(_, _, _, _, _, _, _, _, _, classes)| classes.clone())
            .unwrap_or_default(),
    })
}

fn bundle_last_text_settings_change_evidence(
    bundle: &serde_json::Value,
) -> Option<SettingsChangeConformanceEvidence> {
    let best = bundle_last_snapshot_by_frame_id(bundle)?;

    let policy = best
        .get("resource_caches")?
        .get("render_text_fallback_policy")?
        .as_object()?;

    Some(SettingsChangeConformanceEvidence {
        fallback_policy_key: policy.get("fallback_policy_key")?.as_u64()?,
        system_fonts_enabled: policy.get("system_fonts_enabled")?.as_bool()?,
        prefer_common_fallback: policy.get("prefer_common_fallback")?.as_bool()?,
        common_fallback_injection: policy
            .get("common_fallback_injection")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        configured_common_fallback_families: json_string_vec(
            policy.get("configured_common_fallback_families"),
        ),
        common_fallback_candidates: json_string_vec(policy.get("common_fallback_candidates")),
        common_fallback_stack_suffix: policy
            .get("common_fallback_stack_suffix")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
    })
}

pub(crate) fn check_out_dir_for_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps(
    out_dir: &Path,
) -> Result<(), String> {
    const BEFORE_LABEL: &str = "ui-gallery-text-rescan-system-fonts-before";
    const AFTER_LABEL: &str = "ui-gallery-text-rescan-system-fonts-after";

    fn bundle_last_text_keys(bundle: &serde_json::Value) -> Option<(u64, u64)> {
        let best = bundle_last_snapshot_by_frame_id(bundle)?;

        let render_text = best
            .get("resource_caches")?
            .get("render_text")?
            .as_object()?;
        let font_stack_key = render_text.get("font_stack_key")?.as_u64()?;
        let font_db_revision = render_text.get("font_db_revision")?.as_u64()?;
        Some((font_stack_key, font_db_revision))
    }

    let before_dir = find_latest_labeled_bundle_dir(out_dir, BEFORE_LABEL).ok_or_else(|| {
        format!(
            "ui-gallery text rescan gate expected a capture_bundle label={BEFORE_LABEL} under out_dir, but none was found\n  out_dir: {}",
            out_dir.display()
        )
    })?;
    let after_dir = find_latest_labeled_bundle_dir(out_dir, AFTER_LABEL).ok_or_else(|| {
        format!(
            "ui-gallery text rescan gate expected a capture_bundle label={AFTER_LABEL} under out_dir, but none was found\n  out_dir: {}",
            out_dir.display()
        )
    })?;

    let before_path = crate::resolve_bundle_artifact_path(&before_dir);
    let after_path = crate::resolve_bundle_artifact_path(&after_dir);

    let before_bytes = std::fs::read(&before_path).map_err(|e| e.to_string())?;
    let after_bytes = std::fs::read(&after_path).map_err(|e| e.to_string())?;
    let before_bundle: serde_json::Value =
        serde_json::from_slice(&before_bytes).map_err(|e| e.to_string())?;
    let after_bundle: serde_json::Value =
        serde_json::from_slice(&after_bytes).map_err(|e| e.to_string())?;

    let before_keys = bundle_last_text_keys(&before_bundle).ok_or_else(|| {
        format!(
            "ui-gallery text rescan gate expected renderer text perf snapshot in bundle\n  bundle: {}",
            before_path.display()
        )
    })?;
    let after_keys = bundle_last_text_keys(&after_bundle).ok_or_else(|| {
        format!(
            "ui-gallery text rescan gate expected renderer text perf snapshot in bundle\n  bundle: {}",
            after_path.display()
        )
    })?;

    let evidence_path =
        out_dir.join("check.ui_gallery_text_rescan_system_fonts_font_stack_key_bumps.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "before_dir": before_dir.display().to_string(),
        "after_dir": after_dir.display().to_string(),
        "before_bundle": before_path.display().to_string(),
        "after_bundle": after_path.display().to_string(),
        "before": { "font_stack_key": before_keys.0, "font_db_revision": before_keys.1 },
        "after": { "font_stack_key": after_keys.0, "font_db_revision": after_keys.1 },
    });
    let _ = write_json_value(&evidence_path, &payload);

    if before_keys == after_keys {
        return Err(format!(
            "ui-gallery text rescan gate failed: expected font keys to change after rescan\n  before: font_stack_key={} font_db_revision={}\n  after:  font_stack_key={} font_db_revision={}\n  evidence: {}",
            before_keys.0,
            before_keys.1,
            after_keys.0,
            after_keys.1,
            evidence_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_settings_change(
    out_dir: &Path,
) -> Result<(), String> {
    const BEFORE_LABEL: &str = "ui-gallery-text-fallback-policy-before";
    const AFTER_LABEL: &str = "ui-gallery-text-fallback-policy-after";

    let before_dir = find_latest_labeled_bundle_dir(out_dir, BEFORE_LABEL).ok_or_else(|| {
        format!(
            "ui-gallery text fallback policy gate expected a capture_bundle label={BEFORE_LABEL} under out_dir, but none was found\n  out_dir: {}",
            out_dir.display()
        )
    })?;
    let after_dir = find_latest_labeled_bundle_dir(out_dir, AFTER_LABEL).ok_or_else(|| {
        format!(
            "ui-gallery text fallback policy gate expected a capture_bundle label={AFTER_LABEL} under out_dir, but none was found\n  out_dir: {}",
            out_dir.display()
        )
    })?;

    let before_path = crate::resolve_bundle_artifact_path(&before_dir);
    let after_path = crate::resolve_bundle_artifact_path(&after_dir);

    let before_bytes = std::fs::read(&before_path).map_err(|e| e.to_string())?;
    let after_bytes = std::fs::read(&after_path).map_err(|e| e.to_string())?;
    let before_bundle: serde_json::Value =
        serde_json::from_slice(&before_bytes).map_err(|e| e.to_string())?;
    let after_bundle: serde_json::Value =
        serde_json::from_slice(&after_bytes).map_err(|e| e.to_string())?;

    let before = bundle_last_text_settings_change_evidence(&before_bundle).ok_or_else(|| {
        format!(
            "ui-gallery text fallback policy gate expected renderer text fallback policy snapshot in bundle\n  bundle: {}",
            before_path.display()
        )
    })?;
    let after = bundle_last_text_settings_change_evidence(&after_bundle).ok_or_else(|| {
        format!(
            "ui-gallery text fallback policy gate expected renderer text fallback policy snapshot in bundle\n  bundle: {}",
            after_path.display()
        )
    })?;

    let evidence_path =
        out_dir.join("check.ui_gallery_text_fallback_policy_key_bumps_on_settings_change.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "before_dir": before_dir.display().to_string(),
        "after_dir": after_dir.display().to_string(),
        "before_bundle": before_path.display().to_string(),
        "after_bundle": after_path.display().to_string(),
        "before": {
            "fallback_policy_key": before.fallback_policy_key,
            "system_fonts_enabled": before.system_fonts_enabled,
            "prefer_common_fallback": before.prefer_common_fallback,
            "common_fallback_injection": before.common_fallback_injection,
            "configured_common_fallback_families": before.configured_common_fallback_families,
            "common_fallback_candidates": before.common_fallback_candidates,
            "common_fallback_stack_suffix": before.common_fallback_stack_suffix,
        },
        "after": {
            "fallback_policy_key": after.fallback_policy_key,
            "system_fonts_enabled": after.system_fonts_enabled,
            "prefer_common_fallback": after.prefer_common_fallback,
            "common_fallback_injection": after.common_fallback_injection,
            "configured_common_fallback_families": after.configured_common_fallback_families,
            "common_fallback_candidates": after.common_fallback_candidates,
            "common_fallback_stack_suffix": after.common_fallback_stack_suffix,
        },
    });
    let _ = write_json_value(&evidence_path, &payload);

    if !before.system_fonts_enabled || !after.system_fonts_enabled {
        return Err(format!(
            "ui-gallery text fallback policy gate failed: expected system_fonts_enabled=true in both captures for the native settings baseline\n  before: {}\n  after: {}\n  evidence: {}",
            before.system_fonts_enabled,
            after.system_fonts_enabled,
            evidence_path.display()
        ));
    }
    if before.common_fallback_injection.as_deref() != Some("platform_default") {
        return Err(format!(
            "ui-gallery text fallback policy gate failed: expected common_fallback_injection=platform_default in the BEFORE capture\n  observed: {:?}\n  evidence: {}",
            before.common_fallback_injection,
            evidence_path.display()
        ));
    }
    if after.common_fallback_injection.as_deref() != Some("common_fallback") {
        return Err(format!(
            "ui-gallery text fallback policy gate failed: expected common_fallback_injection=common_fallback in the AFTER capture\n  observed: {:?}\n  evidence: {}",
            after.common_fallback_injection,
            evidence_path.display()
        ));
    }
    if before.prefer_common_fallback {
        return Err(format!(
            "ui-gallery text fallback policy gate failed: expected prefer_common_fallback=false in the BEFORE capture\n  observed: {}\n  evidence: {}",
            before.prefer_common_fallback,
            evidence_path.display()
        ));
    }
    if !after.prefer_common_fallback {
        return Err(format!(
            "ui-gallery text fallback policy gate failed: expected prefer_common_fallback=true in the AFTER capture\n  observed: {}\n  evidence: {}",
            after.prefer_common_fallback,
            evidence_path.display()
        ));
    }
    if !before.common_fallback_candidates.is_empty() {
        return Err(format!(
            "ui-gallery text fallback policy gate failed: expected common_fallback_candidates=[] in the BEFORE capture on the platform-default/system-fallback lane\n  observed: {:?}\n  evidence: {}",
            before.common_fallback_candidates,
            evidence_path.display()
        ));
    }
    if after.common_fallback_candidates.is_empty() {
        return Err(format!(
            "ui-gallery text fallback policy gate failed: expected non-empty common_fallback_candidates in the AFTER capture after selecting common_fallback injection\n  configured_common_fallback_families: {:?}\n  evidence: {}",
            after.configured_common_fallback_families,
            evidence_path.display()
        ));
    }
    if !before.common_fallback_stack_suffix.is_empty() {
        return Err(format!(
            "ui-gallery text fallback policy gate failed: expected an empty common_fallback_stack_suffix in the BEFORE capture\n  observed: {:?}\n  evidence: {}",
            before.common_fallback_stack_suffix,
            evidence_path.display()
        ));
    }
    if after.common_fallback_stack_suffix.is_empty() {
        return Err(format!(
            "ui-gallery text fallback policy gate failed: expected a non-empty common_fallback_stack_suffix in the AFTER capture\n  common_fallback_candidates: {:?}\n  evidence: {}",
            after.common_fallback_candidates,
            evidence_path.display()
        ));
    }
    if before.fallback_policy_key == after.fallback_policy_key {
        return Err(format!(
            "ui-gallery text fallback policy gate failed: expected fallback_policy_key to change after settings apply\n  before: fallback_policy_key={}\n  after:  fallback_policy_key={}\n  evidence: {}",
            before.fallback_policy_key,
            after.fallback_policy_key,
            evidence_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_out_dir_for_ui_gallery_text_mixed_script_bundled_fallback_conformance(
    out_dir: &Path,
) -> Result<(), String> {
    const LABEL: &str = "ui-gallery-text-mixed-script-bundled-fallback-conformance";

    let dir = find_latest_labeled_bundle_dir(out_dir, LABEL).ok_or_else(|| {
        format!(
            "ui-gallery text mixed-script bundled fallback gate expected a capture_bundle label={LABEL} under out_dir, but none was found\n  out_dir: {}",
            out_dir.display()
        )
    })?;

    let bundle_path = crate::resolve_bundle_artifact_path(&dir);
    let bytes = std::fs::read(&bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let evidence = bundle_last_text_mixed_script_evidence(&bundle).ok_or_else(|| {
        format!(
            "ui-gallery text mixed-script bundled fallback gate expected renderer text fallback policy + trace + perf snapshots in bundle\n  bundle: {}",
            bundle_path.display()
        )
    })?;

    let evidence_path =
        out_dir.join("check.ui_gallery_text_mixed_script_bundled_fallback_conformance.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "bundle_dir": dir.display().to_string(),
        "bundle": bundle_path.display().to_string(),
        "fallback_policy": {
            "system_fonts_enabled": evidence.system_fonts_enabled,
            "prefer_common_fallback": evidence.prefer_common_fallback,
            "configured_common_fallback_families": evidence.configured_common_fallback_families,
            "common_fallback_candidates": evidence.common_fallback_candidates,
            "default_ui_sans_candidates": evidence.default_ui_sans_candidates,
            "default_ui_serif_candidates": evidence.default_ui_serif_candidates,
            "default_ui_mono_candidates": evidence.default_ui_mono_candidates,
            "default_common_fallback_families": evidence.default_common_fallback_families,
            "bundled_profile_contract": {
                "name": evidence.bundled_profile.name,
                "expected_family_names": evidence.bundled_profile.expected_family_names,
                "ui_sans_families": evidence.bundled_profile.ui_sans_families,
                "ui_serif_families": evidence.bundled_profile.ui_serif_families,
                "ui_mono_families": evidence.bundled_profile.ui_mono_families,
                "common_fallback_families": evidence.bundled_profile.common_fallback_families,
            },
        },
        "font_trace": {
            "entry_count": evidence.font_trace_entry_count,
            "families": evidence.font_trace_families,
            "common_fallback_families": evidence.font_trace_common_fallback_families,
            "sample_trace_frame_id": evidence.sample_trace_frame_id,
            "sample_trace_frame_missing_glyphs": evidence.sample_trace_frame_missing_glyphs,
            "latin_families": evidence.latin_families,
            "latin_classes": evidence.latin_classes,
            "cjk_families": evidence.cjk_families,
            "cjk_classes": evidence.cjk_classes,
            "emoji_families": evidence.emoji_families,
            "emoji_classes": evidence.emoji_classes,
            "mixed_families": evidence.mixed_families,
            "mixed_classes": evidence.mixed_classes,
        },
        "render_text": {
            "registered_font_blobs_count": evidence.registered_font_blobs_count,
            "registered_font_blobs_total_bytes": evidence.registered_font_blobs_total_bytes,
        },
    });
    let _ = write_json_value(&evidence_path, &payload);

    if evidence.system_fonts_enabled {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected system_fonts_enabled=false for a deterministic (bundled-only) fallback baseline\n  hint: rerun with --env FRET_TEXT_SYSTEM_FONTS=0 and synchronize the bundled startup baseline into the UI gallery font catalog (FRET_UI_GALLERY_BOOTSTRAP_FONTS=1)\n  evidence: {}",
            evidence_path.display()
        ));
    }
    if !evidence.prefer_common_fallback {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected prefer_common_fallback=true when system fonts are disabled\n  evidence: {}",
            evidence_path.display()
        ));
    }

    if evidence.bundled_profile.name.is_empty() {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected bundled_profile_contract.name to be non-empty\n  evidence: {}",
            evidence_path.display()
        ));
    }

    if evidence.default_ui_sans_candidates != evidence.bundled_profile.ui_sans_families {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected default_ui_sans_candidates to match bundled_profile_contract.ui_sans_families under bundled-only mode\n  observed: {:?}\n  expected: {:?}\n  evidence: {}",
            evidence.default_ui_sans_candidates,
            evidence.bundled_profile.ui_sans_families,
            evidence_path.display()
        ));
    }
    if evidence.default_ui_serif_candidates != evidence.bundled_profile.ui_serif_families {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected default_ui_serif_candidates to match bundled_profile_contract.ui_serif_families under bundled-only mode\n  observed: {:?}\n  expected: {:?}\n  evidence: {}",
            evidence.default_ui_serif_candidates,
            evidence.bundled_profile.ui_serif_families,
            evidence_path.display()
        ));
    }
    if evidence.default_ui_mono_candidates != evidence.bundled_profile.ui_mono_families {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected default_ui_mono_candidates to match bundled_profile_contract.ui_mono_families under bundled-only mode\n  observed: {:?}\n  expected: {:?}\n  evidence: {}",
            evidence.default_ui_mono_candidates,
            evidence.bundled_profile.ui_mono_families,
            evidence_path.display()
        ));
    }

    let expected_default_common_fallback = merge_case_insensitive_preserve_order(
        &evidence.bundled_profile.ui_sans_families,
        &evidence.bundled_profile.common_fallback_families,
    );
    if evidence.default_common_fallback_families != expected_default_common_fallback {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected default_common_fallback_families to be derived from bundled_profile_contract.ui_sans_families + common_fallback_families under bundled-only mode\n  observed: {:?}\n  expected: {:?}\n  evidence: {}",
            evidence.default_common_fallback_families,
            expected_default_common_fallback,
            evidence_path.display()
        ));
    }
    let expected_common_fallback_candidates = merge_case_insensitive_preserve_order(
        &evidence.configured_common_fallback_families,
        &evidence.default_common_fallback_families,
    );
    if evidence.common_fallback_candidates != expected_common_fallback_candidates {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected common_fallback_candidates to match configured_common_fallback_families + default_common_fallback_families (case-insensitive, preserve order)\n  observed: {:?}\n  expected: {:?}\n  evidence: {}",
            evidence.common_fallback_candidates,
            expected_common_fallback_candidates,
            evidence_path.display()
        ));
    }

    if evidence.bundled_profile.common_fallback_families.is_empty() {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected bundled_profile_contract.common_fallback_families to be non-empty for the mixed-script harness\n  evidence: {}",
            evidence_path.display()
        ));
    }

    let expected_profile_family_names = merge_case_insensitive_preserve_order(
        &merge_case_insensitive_preserve_order(
            &merge_case_insensitive_preserve_order(
                &evidence.bundled_profile.ui_sans_families,
                &evidence.bundled_profile.ui_serif_families,
            ),
            &evidence.bundled_profile.ui_mono_families,
        ),
        &evidence.bundled_profile.common_fallback_families,
    );
    for family in &expected_profile_family_names {
        if !evidence
            .bundled_profile
            .expected_family_names
            .iter()
            .any(|candidate| candidate == family)
        {
            return Err(format!(
                "ui-gallery text mixed-script bundled fallback gate failed: expected bundled_profile_contract.expected_family_names to include {family:?}\n  evidence: {}",
                evidence_path.display()
            ));
        }
    }
    for family in &evidence.bundled_profile.common_fallback_families {
        if !evidence
            .common_fallback_candidates
            .iter()
            .any(|candidate| candidate == family)
        {
            return Err(format!(
                "ui-gallery text mixed-script bundled fallback gate failed: expected common_fallback_candidates to include {family:?}\n  evidence: {}",
                evidence_path.display()
            ));
        }
    }

    if evidence.font_trace_entry_count == 0 {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected bundle-scoped font trace entries for the scripted mixed-script sample\n  evidence: {}",
            evidence_path.display()
        ));
    }
    if evidence.sample_trace_frame_id.is_none() {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected sample-aware font trace evidence for `m`, `你`, `😀`, and `m你😀`\n  evidence: {}",
            evidence_path.display()
        ));
    }
    if evidence.font_trace_families.is_empty() {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected font trace to record at least one resolved family for the scripted mixed-script sample\n  evidence: {}",
            evidence_path.display()
        ));
    }
    for family in &evidence.font_trace_families {
        if !contains_case_insensitive(&evidence.bundled_profile.expected_family_names, family) {
            return Err(format!(
                "ui-gallery text mixed-script bundled fallback gate failed: expected font trace families to stay within bundled_profile_contract.expected_family_names under bundled-only mode\n  observed family: {:?}\n  allowed: {:?}\n  evidence: {}",
                family,
                evidence.bundled_profile.expected_family_names,
                evidence_path.display()
            ));
        }
    }
    if evidence.font_trace_common_fallback_families.is_empty() {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected font trace to record at least one common_fallback family usage for the scripted mixed-script sample\n  evidence: {}",
            evidence_path.display()
        ));
    }
    for family in &evidence.font_trace_common_fallback_families {
        if !contains_case_insensitive(&evidence.common_fallback_candidates, family) {
            return Err(format!(
                "ui-gallery text mixed-script bundled fallback gate failed: expected font trace common_fallback families to stay within common_fallback_candidates\n  observed family: {:?}\n  allowed: {:?}\n  evidence: {}",
                family,
                evidence.common_fallback_candidates,
                evidence_path.display()
            ));
        }
    }

    if evidence.registered_font_blobs_count == 0 || evidence.registered_font_blobs_total_bytes == 0
    {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected registered_font_blobs counters to stay populated under bundled-only mode\n  observed: count={} total_bytes={}\n  evidence: {}",
            evidence.registered_font_blobs_count,
            evidence.registered_font_blobs_total_bytes,
            evidence_path.display()
        ));
    }

    if evidence.sample_trace_frame_missing_glyphs != Some(0) {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected the mixed-script sample frame to stay tofu-free under bundled fonts\n  sample_trace_frame_id: {:?}\n  observed_frame_missing_glyphs: {:?}\n  evidence: {}",
            evidence.sample_trace_frame_id,
            evidence.sample_trace_frame_missing_glyphs,
            evidence_path.display()
        ));
    }

    let latin_class = evidence.latin_classes.first().map(String::as_str);
    let cjk_class = evidence.cjk_classes.first().map(String::as_str);
    let emoji_class = evidence.emoji_classes.first().map(String::as_str);
    if latin_class != Some("requested")
        || cjk_class != Some("common_fallback")
        || emoji_class != Some("common_fallback")
    {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected sample trace classes to stay on the requested/common_fallback lanes under bundled-only mode\n  latin_classes: {:?}\n  cjk_classes: {:?}\n  emoji_classes: {:?}\n  evidence: {}",
            evidence.latin_classes,
            evidence.cjk_classes,
            evidence.emoji_classes,
            evidence_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_locale_change(
    out_dir: &Path,
) -> Result<(), String> {
    const BEFORE_LABEL: &str = "ui-gallery-text-fallback-policy-locale-before";
    const AFTER_LABEL: &str = "ui-gallery-text-fallback-policy-locale-after";

    let before_dir = find_latest_labeled_bundle_dir(out_dir, BEFORE_LABEL).ok_or_else(|| {
        format!(
            "ui-gallery text fallback policy locale gate expected a capture_bundle label={BEFORE_LABEL} under out_dir, but none was found\n  out_dir: {}",
            out_dir.display()
        )
    })?;
    let after_dir = find_latest_labeled_bundle_dir(out_dir, AFTER_LABEL).ok_or_else(|| {
        format!(
            "ui-gallery text fallback policy locale gate expected a capture_bundle label={AFTER_LABEL} under out_dir, but none was found\n  out_dir: {}",
            out_dir.display()
        )
    })?;

    let before_path = crate::resolve_bundle_artifact_path(&before_dir);
    let after_path = crate::resolve_bundle_artifact_path(&after_dir);

    let before_bytes = std::fs::read(&before_path).map_err(|e| e.to_string())?;
    let after_bytes = std::fs::read(&after_path).map_err(|e| e.to_string())?;
    let before_bundle: serde_json::Value =
        serde_json::from_slice(&before_bytes).map_err(|e| e.to_string())?;
    let after_bundle: serde_json::Value =
        serde_json::from_slice(&after_bytes).map_err(|e| e.to_string())?;

    let before = bundle_last_text_locale_change_evidence(&before_bundle).ok_or_else(|| {
        format!(
            "ui-gallery text fallback policy locale gate expected renderer text fallback policy + trace + perf snapshots in bundle\n  bundle: {}",
            before_path.display()
        )
    })?;
    let after = bundle_last_text_locale_change_evidence(&after_bundle).ok_or_else(|| {
        format!(
            "ui-gallery text fallback policy locale gate expected renderer text fallback policy + trace + perf snapshots in bundle\n  bundle: {}",
            after_path.display()
        )
    })?;

    let evidence_path =
        out_dir.join("check.ui_gallery_text_fallback_policy_key_bumps_on_locale_change.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "before_dir": before_dir.display().to_string(),
        "after_dir": after_dir.display().to_string(),
        "before_bundle": before_path.display().to_string(),
        "after_bundle": after_path.display().to_string(),
        "before": {
            "fallback_policy_key": before.fallback_policy_key,
            "locale_bcp47": before.locale_bcp47,
            "system_fonts_enabled": before.system_fonts_enabled,
            "prefer_common_fallback": before.prefer_common_fallback,
            "prefer_common_fallback_for_generics": before.prefer_common_fallback_for_generics,
            "common_fallback_injection": before.common_fallback_injection,
            "common_fallback_candidates": before.common_fallback_candidates,
            "frame_missing_glyphs": before.missing_glyphs,
            "font_trace_entry_count": before.font_trace_entry_count,
            "font_trace_locales": before.font_trace_locales,
            "sample_trace_frame_id": before.sample_trace_frame_id,
            "sample_trace_locales": before.sample_trace_locales,
            "latin_families": before.latin_families,
            "latin_classes": before.latin_classes,
            "cjk_families": before.cjk_families,
            "cjk_classes": before.cjk_classes,
            "emoji_families": before.emoji_families,
            "emoji_classes": before.emoji_classes,
            "mixed_families": before.mixed_families,
            "mixed_classes": before.mixed_classes,
        },
        "after": {
            "fallback_policy_key": after.fallback_policy_key,
            "locale_bcp47": after.locale_bcp47,
            "system_fonts_enabled": after.system_fonts_enabled,
            "prefer_common_fallback": after.prefer_common_fallback,
            "prefer_common_fallback_for_generics": after.prefer_common_fallback_for_generics,
            "common_fallback_injection": after.common_fallback_injection,
            "common_fallback_candidates": after.common_fallback_candidates,
            "frame_missing_glyphs": after.missing_glyphs,
            "font_trace_entry_count": after.font_trace_entry_count,
            "font_trace_locales": after.font_trace_locales,
            "sample_trace_frame_id": after.sample_trace_frame_id,
            "sample_trace_locales": after.sample_trace_locales,
            "latin_families": after.latin_families,
            "latin_classes": after.latin_classes,
            "cjk_families": after.cjk_families,
            "cjk_classes": after.cjk_classes,
            "emoji_families": after.emoji_families,
            "emoji_classes": after.emoji_classes,
            "mixed_families": after.mixed_families,
            "mixed_classes": after.mixed_classes,
        },
    });
    let _ = write_json_value(&evidence_path, &payload);

    if !before.system_fonts_enabled || !after.system_fonts_enabled {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected system_fonts_enabled=true in both captures for the native system-font baseline\n  before: {}\n  after: {}\n  evidence: {}",
            before.system_fonts_enabled,
            after.system_fonts_enabled,
            evidence_path.display()
        ));
    }
    if before.common_fallback_injection.as_deref() != Some("platform_default")
        || after.common_fallback_injection.as_deref() != Some("platform_default")
    {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected common_fallback_injection=platform_default in both captures so the script exercises the native hybrid lane\n  before: {:?}\n  after: {:?}\n  evidence: {}",
            before.common_fallback_injection,
            after.common_fallback_injection,
            evidence_path.display()
        ));
    }
    if before.prefer_common_fallback || after.prefer_common_fallback {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected prefer_common_fallback=false in both captures for the native platform-default/system-fallback baseline\n  before: {}\n  after: {}\n  evidence: {}",
            before.prefer_common_fallback,
            after.prefer_common_fallback,
            evidence_path.display()
        ));
    }
    if !before.prefer_common_fallback_for_generics || !after.prefer_common_fallback_for_generics {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected prefer_common_fallback_for_generics=true in both captures for the native platform-default generic no-tofu baseline\n  before: {}\n  after: {}\n  evidence: {}",
            before.prefer_common_fallback_for_generics,
            after.prefer_common_fallback_for_generics,
            evidence_path.display()
        ));
    }
    if before.common_fallback_candidates.is_empty() || after.common_fallback_candidates.is_empty() {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected non-empty common_fallback_candidates in both captures so the generic mixed-script sample keeps the renderer-owned no-tofu baseline\n  before: {:?}\n  after: {:?}\n  evidence: {}",
            before.common_fallback_candidates,
            after.common_fallback_candidates,
            evidence_path.display()
        ));
    }
    if before.locale_bcp47.as_deref() != Some("en-US") {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected locale_bcp47 to be en-US in the BEFORE capture\n  observed: {:?}\n  evidence: {}",
            before.locale_bcp47,
            evidence_path.display()
        ));
    }
    if after.locale_bcp47.as_deref() != Some("zh-CN") {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected locale_bcp47 to be zh-CN in the AFTER capture\n  observed: {:?}\n  evidence: {}",
            after.locale_bcp47,
            evidence_path.display()
        ));
    }
    if before.font_trace_entry_count == 0 || after.font_trace_entry_count == 0 {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected bundle-scoped font trace entries in both captures after enabling FRET_TEXT_FONT_TRACE_ALL\n  before: {}\n  after: {}\n  evidence: {}",
            before.font_trace_entry_count,
            after.font_trace_entry_count,
            evidence_path.display()
        ));
    }
    if before.sample_trace_frame_id.is_none() || after.sample_trace_frame_id.is_none() {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected the mixed-script sample traces (`m`, `你`, `😀`, `m你😀`) to appear in both captures\n  before_frame: {:?}\n  after_frame: {:?}\n  evidence: {}",
            before.sample_trace_frame_id,
            after.sample_trace_frame_id,
            evidence_path.display()
        ));
    }
    if before.sample_trace_locales != vec!["en-US".to_string()] {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected BEFORE mixed-script sample trace locales to settle to [\"en-US\"]\n  observed: {:?}\n  evidence: {}",
            before.sample_trace_locales,
            evidence_path.display()
        ));
    }
    if after.sample_trace_locales != vec!["zh-CN".to_string()] {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected AFTER mixed-script sample trace locales to settle to [\"zh-CN\"]\n  observed: {:?}\n  evidence: {}",
            after.sample_trace_locales,
            evidence_path.display()
        ));
    }
    let validate_sample_families = |label: &str,
                                    evidence: &LocaleChangeConformanceEvidence|
     -> Result<(), String> {
        let latin = evidence.latin_families.first().ok_or_else(|| {
            format!(
                "ui-gallery text fallback policy locale gate failed: expected a latin sample family in the {label} capture\n  evidence: {}",
                evidence_path.display()
            )
        })?;
        let cjk = evidence.cjk_families.first().ok_or_else(|| {
            format!(
                "ui-gallery text fallback policy locale gate failed: expected a cjk sample family in the {label} capture\n  evidence: {}",
                evidence_path.display()
            )
        })?;
        let emoji = evidence.emoji_families.first().ok_or_else(|| {
            format!(
                "ui-gallery text fallback policy locale gate failed: expected an emoji sample family in the {label} capture\n  evidence: {}",
                evidence_path.display()
            )
        })?;

        if !contains_case_insensitive(&evidence.common_fallback_candidates, cjk)
            || !contains_case_insensitive(&evidence.common_fallback_candidates, emoji)
        {
            return Err(format!(
                "ui-gallery text fallback policy locale gate failed: expected cjk/emoji sample families in the {label} capture to resolve inside common_fallback_candidates on the generic no-tofu lane\n  cjk: {:?}\n  emoji: {:?}\n  candidates: {:?}\n  evidence: {}",
                evidence.cjk_families,
                evidence.emoji_families,
                evidence.common_fallback_candidates,
                evidence_path.display()
            ));
        }

        let latin_class = evidence.latin_classes.first().map(String::as_str);
        let cjk_class = evidence.cjk_classes.first().map(String::as_str);
        let emoji_class = evidence.emoji_classes.first().map(String::as_str);
        if latin_class != Some("requested")
            || cjk_class != Some("common_fallback")
            || emoji_class != Some("common_fallback")
        {
            return Err(format!(
                "ui-gallery text fallback policy locale gate failed: expected cjk/emoji sample trace classes in the {label} capture to stay on the generic common-fallback lane while the latin sample remains requested\n  latin_classes: {:?}\n  cjk_classes: {:?}\n  emoji_classes: {:?}\n  evidence: {}",
                evidence.latin_classes,
                evidence.cjk_classes,
                evidence.emoji_classes,
                evidence_path.display()
            ));
        }

        let latin_ix = evidence
            .mixed_families
            .iter()
            .position(|family| family.eq_ignore_ascii_case(latin));
        let cjk_ix = evidence
            .mixed_families
            .iter()
            .position(|family| family.eq_ignore_ascii_case(cjk));
        let emoji_ix = evidence
            .mixed_families
            .iter()
            .position(|family| family.eq_ignore_ascii_case(emoji));

        match (latin_ix, cjk_ix, emoji_ix) {
            (Some(latin_ix), Some(cjk_ix), Some(emoji_ix))
                if latin_ix < cjk_ix && cjk_ix < emoji_ix =>
            {
                let mixed_latin_class = evidence.mixed_classes.get(latin_ix).map(String::as_str);
                let mixed_cjk_class = evidence.mixed_classes.get(cjk_ix).map(String::as_str);
                let mixed_emoji_class = evidence.mixed_classes.get(emoji_ix).map(String::as_str);
                if mixed_latin_class == Some("requested")
                    && mixed_cjk_class == Some("common_fallback")
                    && mixed_emoji_class == Some("common_fallback")
                {
                    Ok(())
                } else {
                    Err(format!(
                        "ui-gallery text fallback policy locale gate failed: expected the mixed-script trace classes to stay requested -> common_fallback -> common_fallback in the {label} capture\n  mixed_classes: {:?}\n  evidence: {}",
                        evidence.mixed_classes,
                        evidence_path.display()
                    ))
                }
            }
            _ => Err(format!(
                "ui-gallery text fallback policy locale gate failed: expected the mixed-script trace to preserve latin -> cjk -> emoji family order in the {label} capture\n  latin: {:?}\n  cjk: {:?}\n  emoji: {:?}\n  mixed: {:?}\n  evidence: {}",
                evidence.latin_families,
                evidence.cjk_families,
                evidence.emoji_families,
                evidence.mixed_families,
                evidence_path.display()
            )),
        }
    };
    validate_sample_families("BEFORE", &before)?;
    validate_sample_families("AFTER", &after)?;
    if before.missing_glyphs != 0 || after.missing_glyphs != 0 {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected frame_missing_glyphs=0 in both captures on the mixed-script page\n  before: {}\n  after: {}\n  evidence: {}",
            before.missing_glyphs,
            after.missing_glyphs,
            evidence_path.display()
        ));
    }
    if before.fallback_policy_key == after.fallback_policy_key {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected fallback_policy_key to change when locale changes\n  before: {}\n  after: {}\n  evidence: {}",
            before.fallback_policy_key,
            after.fallback_policy_key,
            evidence_path.display()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now_unix_ms() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_millis()
    }

    fn unique_tmp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "fret_diag_ui_gallery_text_gates_{label}_{}_{}",
            std::process::id(),
            now_unix_ms()
        ));
        std::fs::create_dir_all(&path).expect("create tmp dir");
        path
    }

    fn write_labeled_bundle(out_dir: &Path, label: &str, bundle: &serde_json::Value) {
        let dir = out_dir.join(format!("{}-{label}", now_unix_ms()));
        std::fs::create_dir_all(&dir).expect("create bundle dir");
        std::fs::write(
            dir.join("bundle.json"),
            serde_json::to_vec_pretty(bundle).expect("serialize bundle"),
        )
        .expect("write bundle");
    }

    fn bundled_mixed_script_bundle() -> serde_json::Value {
        serde_json::json!({
            "windows": [{
                "snapshots": [
                    {
                        "frame_id": 7,
                        "resource_caches": {
                            "render_text": {
                                "frame_missing_glyphs": 0,
                                "registered_font_blobs_count": 4,
                                "registered_font_blobs_total_bytes": 65536
                            },
                            "render_text_fallback_policy": {
                                "system_fonts_enabled": false,
                                "prefer_common_fallback": true,
                                "configured_common_fallback_families": [],
                                "common_fallback_candidates": [
                                    "Inter",
                                    "Noto Sans CJK SC",
                                    "Noto Sans Arabic",
                                    "Noto Color Emoji"
                                ],
                                "default_ui_sans_candidates": ["Inter"],
                                "default_ui_serif_candidates": ["Roboto Slab"],
                                "default_ui_mono_candidates": ["JetBrains Mono"],
                                "default_common_fallback_families": [
                                    "Inter",
                                    "Noto Sans CJK SC",
                                    "Noto Sans Arabic",
                                    "Noto Color Emoji"
                                ],
                                "bundled_profile_contract": {
                                    "name": "default",
                                    "expected_family_names": [
                                        "Inter",
                                        "Roboto Slab",
                                        "JetBrains Mono",
                                        "Noto Sans CJK SC",
                                        "Noto Sans Arabic",
                                        "Noto Color Emoji"
                                    ],
                                    "ui_sans_families": ["Inter"],
                                    "ui_serif_families": ["Roboto Slab"],
                                    "ui_mono_families": ["JetBrains Mono"],
                                    "common_fallback_families": [
                                        "Noto Sans CJK SC",
                                        "Noto Sans Arabic",
                                        "Noto Color Emoji"
                                    ]
                                }
                            },
                            "render_text_font_trace": {
                                "entries": [
                                    {
                                        "text_preview": "m",
                                        "families": [
                                            {
                                                "family": "Inter",
                                                "class": "requested",
                                                "glyphs": 1,
                                                "missing_glyphs": 0
                                            }
                                        ]
                                    },
                                    {
                                        "text_preview": "你",
                                        "families": [
                                            {
                                                "family": "Noto Sans CJK SC",
                                                "class": "common_fallback",
                                                "glyphs": 1,
                                                "missing_glyphs": 0
                                            }
                                        ]
                                    },
                                    {
                                        "text_preview": "😀",
                                        "families": [
                                            {
                                                "family": "Noto Color Emoji",
                                                "class": "common_fallback",
                                                "glyphs": 1,
                                                "missing_glyphs": 0
                                            }
                                        ]
                                    },
                                    {
                                        "text_preview": "m你😀",
                                        "families": [
                                            {
                                                "family": "Inter",
                                                "class": "requested",
                                                "glyphs": 1,
                                                "missing_glyphs": 0
                                            },
                                            {
                                                "family": "Noto Sans CJK SC",
                                                "class": "common_fallback",
                                                "glyphs": 1,
                                                "missing_glyphs": 0
                                            },
                                            {
                                                "family": "Noto Color Emoji",
                                                "class": "common_fallback",
                                                "glyphs": 1,
                                                "missing_glyphs": 0
                                            }
                                        ]
                                    }
                                ]
                            }
                        }
                    },
                    {
                        "frame_id": 8,
                        "resource_caches": {
                            "render_text": {
                                "frame_missing_glyphs": 0,
                                "registered_font_blobs_count": 4,
                                "registered_font_blobs_total_bytes": 65536
                            },
                            "render_text_fallback_policy": {
                                "system_fonts_enabled": false,
                                "prefer_common_fallback": true,
                                "configured_common_fallback_families": [],
                                "common_fallback_candidates": [
                                    "Inter",
                                    "Noto Sans CJK SC",
                                    "Noto Sans Arabic",
                                    "Noto Color Emoji"
                                ],
                                "default_ui_sans_candidates": ["Inter"],
                                "default_ui_serif_candidates": ["Roboto Slab"],
                                "default_ui_mono_candidates": ["JetBrains Mono"],
                                "default_common_fallback_families": [
                                    "Inter",
                                    "Noto Sans CJK SC",
                                    "Noto Sans Arabic",
                                    "Noto Color Emoji"
                                ],
                                "bundled_profile_contract": {
                                    "name": "default",
                                    "expected_family_names": [
                                        "Inter",
                                        "Roboto Slab",
                                        "JetBrains Mono",
                                        "Noto Sans CJK SC",
                                        "Noto Sans Arabic",
                                        "Noto Color Emoji"
                                    ],
                                    "ui_sans_families": ["Inter"],
                                    "ui_serif_families": ["Roboto Slab"],
                                    "ui_mono_families": ["JetBrains Mono"],
                                    "common_fallback_families": [
                                        "Noto Sans CJK SC",
                                        "Noto Sans Arabic",
                                        "Noto Color Emoji"
                                    ]
                                }
                            },
                            "render_text_font_trace": {
                                "entries": [{
                                    "text_preview": "theme=zinc/light view_cache=0 layout_us=1593 paint_us=128",
                                    "families": [
                                        {
                                            "family": "Inter",
                                            "class": "requested",
                                            "glyphs": 8,
                                            "missing_glyphs": 0
                                        }
                                    ]
                                }]
                            }
                        }
                    }
                ]
            }]
        })
    }

    #[test]
    fn mixed_script_bundled_fallback_gate_accepts_profile_backed_snapshot() {
        let out_dir = unique_tmp_dir("pass");
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-mixed-script-bundled-fallback-conformance",
            &bundled_mixed_script_bundle(),
        );

        check_out_dir_for_ui_gallery_text_mixed_script_bundled_fallback_conformance(&out_dir)
            .expect("gate should pass");
        assert!(
            out_dir
                .join("check.ui_gallery_text_mixed_script_bundled_fallback_conformance.json")
                .is_file()
        );
    }

    #[test]
    fn mixed_script_bundled_fallback_gate_rejects_profile_drift_in_default_common_fallback() {
        let out_dir = unique_tmp_dir("fail_profile_drift");
        let mut bundle = bundled_mixed_script_bundle();
        for snapshot in bundle["windows"][0]["snapshots"]
            .as_array_mut()
            .expect("snapshots array")
        {
            let defaults =
                snapshot["resource_caches"]["render_text_fallback_policy"]
                    ["default_common_fallback_families"]
                    .as_array_mut()
                    .expect("default_common_fallback_families array");
            defaults.clear();
            defaults.push(serde_json::json!("Noto Sans CJK SC"));
        }

        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-mixed-script-bundled-fallback-conformance",
            &bundle,
        );

        let err =
            check_out_dir_for_ui_gallery_text_mixed_script_bundled_fallback_conformance(&out_dir)
                .expect_err("gate should fail");
        assert!(
            err.contains("expected default_common_fallback_families to be derived"),
            "{err}"
        );
    }

    #[test]
    fn mixed_script_bundled_fallback_gate_accepts_curated_common_fallback_overrides() {
        let out_dir = unique_tmp_dir("pass_curated_overrides");
        let mut bundle = bundled_mixed_script_bundle();
        for snapshot in bundle["windows"][0]["snapshots"]
            .as_array_mut()
            .expect("snapshots array")
        {
            snapshot["resource_caches"]["render_text_fallback_policy"]["configured_common_fallback_families"] =
                serde_json::json!(["Noto Sans CJK SC", "Segoe UI Emoji", "Segoe UI Symbol"]);
            snapshot["resource_caches"]["render_text_fallback_policy"]["common_fallback_candidates"] =
                serde_json::json!([
                    "Noto Sans CJK SC",
                    "Segoe UI Emoji",
                    "Segoe UI Symbol",
                    "Inter",
                    "Noto Sans Arabic",
                    "Noto Color Emoji"
                ]);
        }

        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-mixed-script-bundled-fallback-conformance",
            &bundle,
        );

        check_out_dir_for_ui_gallery_text_mixed_script_bundled_fallback_conformance(&out_dir)
            .expect("gate should accept configured common fallback overrides");
    }

    #[test]
    fn mixed_script_bundled_fallback_gate_accepts_trace_common_fallback_from_default_ui_family() {
        let out_dir = unique_tmp_dir("pass_default_ui_trace");
        let mut bundle = bundled_mixed_script_bundle();
        let entries = bundle["windows"][0]["snapshots"][0]["resource_caches"]["render_text_font_trace"]
            ["entries"]
            .as_array_mut()
            .expect("entries array");
        for entry in entries {
            let text = entry["text_preview"].as_str().expect("text_preview");
            entry["families"] = match text {
                "m" => serde_json::json!([
                    {
                        "family": "Inter",
                        "class": "requested",
                        "glyphs": 1,
                        "missing_glyphs": 0
                    }
                ]),
                "m你😀" => serde_json::json!([
                    {
                        "family": "Inter",
                        "class": "requested",
                        "glyphs": 1,
                        "missing_glyphs": 0
                    },
                    {
                        "family": "Inter",
                        "class": "common_fallback",
                        "glyphs": 2,
                        "missing_glyphs": 0
                    }
                ]),
                _ => serde_json::json!([
                    {
                        "family": "Inter",
                        "class": "common_fallback",
                        "glyphs": 1,
                        "missing_glyphs": 0
                    }
                ]),
            };
        }

        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-mixed-script-bundled-fallback-conformance",
            &bundle,
        );

        check_out_dir_for_ui_gallery_text_mixed_script_bundled_fallback_conformance(&out_dir)
            .expect("gate should accept default ui family when it participates in common fallback");
    }

    #[test]
    fn mixed_script_bundled_fallback_gate_rejects_trace_family_outside_profile_contract() {
        let out_dir = unique_tmp_dir("fail_trace_family_outside_profile");
        let mut bundle = bundled_mixed_script_bundle();
        let entries = bundle["windows"][0]["snapshots"][0]["resource_caches"]["render_text_font_trace"]
            ["entries"]
            .as_array_mut()
            .expect("entries array");
        for entry in entries {
            let text = entry["text_preview"].as_str().expect("text_preview");
            entry["families"] = match text {
                "m" => serde_json::json!([
                    {
                        "family": "Inter",
                        "class": "requested",
                        "glyphs": 1,
                        "missing_glyphs": 0
                    }
                ]),
                "m你😀" => serde_json::json!([
                    {
                        "family": "Inter",
                        "class": "requested",
                        "glyphs": 1,
                        "missing_glyphs": 0
                    },
                    {
                        "family": "System UI",
                        "class": "common_fallback",
                        "glyphs": 2,
                        "missing_glyphs": 0
                    }
                ]),
                _ => serde_json::json!([
                    {
                        "family": "System UI",
                        "class": "common_fallback",
                        "glyphs": 1,
                        "missing_glyphs": 0
                    }
                ]),
            };
        }

        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-mixed-script-bundled-fallback-conformance",
            &bundle,
        );

        let err =
            check_out_dir_for_ui_gallery_text_mixed_script_bundled_fallback_conformance(&out_dir)
                .expect_err("gate should reject trace families outside bundled profile contract");
        assert!(
            err.contains("expected font trace families to stay within bundled_profile_contract.expected_family_names"),
            "{err}"
        );
    }

    fn settings_change_bundle(
        injection: &str,
        prefer_common_fallback: bool,
        candidates: &[&str],
        key: u64,
    ) -> serde_json::Value {
        let common_fallback_candidates = candidates
            .iter()
            .map(|family| (*family).to_string())
            .collect::<Vec<_>>();
        let stack_suffix = if prefer_common_fallback {
            common_fallback_candidates.join(", ")
        } else {
            String::new()
        };

        serde_json::json!({
            "windows": [{
                "snapshots": [{
                    "frame_id": 5,
                    "resource_caches": {
                        "render_text_fallback_policy": {
                            "fallback_policy_key": key,
                            "system_fonts_enabled": true,
                            "prefer_common_fallback": prefer_common_fallback,
                            "common_fallback_injection": injection,
                            "configured_common_fallback_families": [],
                            "common_fallback_candidates": common_fallback_candidates,
                            "common_fallback_stack_suffix": stack_suffix
                        }
                    }
                }]
            }]
        })
    }

    #[test]
    fn settings_change_gate_accepts_platform_default_to_common_fallback_transition() {
        let out_dir = unique_tmp_dir("pass_settings_change");
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-before",
            &settings_change_bundle("platform_default", false, &[], 10),
        );
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-after",
            &settings_change_bundle(
                "common_fallback",
                true,
                &["Inter", "Noto Sans CJK SC", "Noto Color Emoji"],
                11,
            ),
        );

        check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_settings_change(&out_dir)
            .expect("gate should accept the settings-driven lane transition");
    }

    #[test]
    fn settings_change_gate_rejects_before_capture_outside_platform_default_baseline() {
        let out_dir = unique_tmp_dir("fail_settings_before_baseline");
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-before",
            &settings_change_bundle("common_fallback", true, &["Inter", "Noto Sans CJK SC"], 10),
        );
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-after",
            &settings_change_bundle(
                "common_fallback",
                true,
                &["Inter", "Noto Sans CJK SC", "Noto Color Emoji"],
                11,
            ),
        );

        let err = check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_settings_change(
            &out_dir,
        )
        .expect_err("gate should reject a stale BEFORE lane");
        assert!(
            err.contains(
                "expected common_fallback_injection=platform_default in the BEFORE capture"
            ),
            "{err}"
        );
    }

    #[test]
    fn settings_change_gate_rejects_after_capture_without_common_fallback_candidates() {
        let out_dir = unique_tmp_dir("fail_settings_after_candidates");
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-before",
            &settings_change_bundle("platform_default", false, &[], 10),
        );
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-after",
            &settings_change_bundle("common_fallback", true, &[], 11),
        );

        let err = check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_settings_change(
            &out_dir,
        )
        .expect_err("gate should reject missing common fallback candidates");
        assert!(
            err.contains("expected non-empty common_fallback_candidates in the AFTER capture"),
            "{err}"
        );
    }

    fn locale_change_bundle(locale: &str, key: u64) -> serde_json::Value {
        serde_json::json!({
            "windows": [{
                "snapshots": [{
                    "frame_id": 3,
                    "resource_caches": {
                        "render_text": {
                            "frame_missing_glyphs": 0
                        },
                        "render_text_fallback_policy": {
                            "fallback_policy_key": key,
                            "locale_bcp47": locale,
                            "system_fonts_enabled": true,
                            "prefer_common_fallback": false,
                            "prefer_common_fallback_for_generics": true,
                            "common_fallback_injection": "platform_default",
                            "common_fallback_candidates": ["Noto Sans CJK SC", "Segoe UI Emoji"]
                        },
                        "render_text_font_trace": {
                            "entries": [
                                {
                                    "text_preview": "m",
                                    "locale_bcp47": locale,
                                    "families": [{
                                        "family": "Inter",
                                        "class": "requested",
                                        "glyphs": 1,
                                        "missing_glyphs": 0
                                    }]
                                },
                                {
                                    "text_preview": "你",
                                    "locale_bcp47": locale,
                                    "families": [{
                                        "family": "Noto Sans CJK SC",
                                        "class": "common_fallback",
                                        "glyphs": 1,
                                        "missing_glyphs": 0
                                    }]
                                },
                                {
                                    "text_preview": "😀",
                                    "locale_bcp47": locale,
                                    "families": [{
                                        "family": "Segoe UI Emoji",
                                        "class": "common_fallback",
                                        "glyphs": 1,
                                        "missing_glyphs": 0
                                    }]
                                },
                                {
                                    "text_preview": "m你😀",
                                    "locale_bcp47": locale,
                                    "families": [
                                        {
                                            "family": "Inter",
                                            "class": "requested",
                                            "glyphs": 1,
                                            "missing_glyphs": 0
                                        },
                                        {
                                            "family": "Noto Sans CJK SC",
                                            "class": "common_fallback",
                                            "glyphs": 1,
                                            "missing_glyphs": 0
                                        },
                                        {
                                            "family": "Segoe UI Emoji",
                                            "class": "common_fallback",
                                            "glyphs": 1,
                                            "missing_glyphs": 0
                                        }
                                    ]
                                }
                            ]
                        }
                    }
                }]
            }]
        })
    }

    #[test]
    fn locale_change_gate_accepts_mixed_script_trace_evidence() {
        let out_dir = unique_tmp_dir("pass_locale_change");
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-locale-before",
            &locale_change_bundle("en-US", 10),
        );
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-locale-after",
            &locale_change_bundle("zh-CN", 11),
        );

        check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_locale_change(&out_dir)
            .expect("gate should accept locale change evidence with trace coverage");
    }

    #[test]
    fn locale_change_gate_rejects_trace_locale_drift() {
        let out_dir = unique_tmp_dir("fail_locale_trace_drift");
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-locale-before",
            &locale_change_bundle("en-US", 10),
        );
        let mut after = locale_change_bundle("zh-CN", 11);
        after["windows"][0]["snapshots"][0]["resource_caches"]["render_text_font_trace"]["entries"]
            [0]["locale_bcp47"] = serde_json::json!("en-US");
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-locale-after",
            &after,
        );

        let err =
            check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_locale_change(&out_dir)
                .expect_err("gate should reject stale font trace locales");
        assert!(
            err.contains(
                "expected AFTER mixed-script sample trace locales to settle to [\"zh-CN\"]"
            ),
            "{err}"
        );
    }

    #[test]
    fn locale_change_gate_rejects_mixed_family_order_drift() {
        let out_dir = unique_tmp_dir("fail_locale_family_order");
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-locale-before",
            &locale_change_bundle("en-US", 10),
        );
        let mut after = locale_change_bundle("zh-CN", 11);
        after["windows"][0]["snapshots"][0]["resource_caches"]["render_text_font_trace"]["entries"]
            [3]["families"] = serde_json::json!([
            {
                "family": "Noto Sans CJK SC",
                "class": "common_fallback",
                "glyphs": 1,
                "missing_glyphs": 0
            },
            {
                "family": "Inter",
                "class": "requested",
                "glyphs": 1,
                "missing_glyphs": 0
            },
            {
                "family": "Segoe UI Emoji",
                "class": "common_fallback",
                "glyphs": 1,
                "missing_glyphs": 0
            }
        ]);
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-locale-after",
            &after,
        );

        let err =
            check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_locale_change(&out_dir)
                .expect_err("gate should reject mixed-script family ordering drift");
        assert!(
            err.contains(
                "expected the mixed-script trace to preserve latin -> cjk -> emoji family order"
            ),
            "{err}"
        );
    }

    #[test]
    fn locale_change_gate_rejects_trace_class_drift() {
        let out_dir = unique_tmp_dir("fail_locale_trace_class_drift");
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-locale-before",
            &locale_change_bundle("en-US", 10),
        );
        let mut after = locale_change_bundle("zh-CN", 11);
        after["windows"][0]["snapshots"][0]["resource_caches"]["render_text_font_trace"]["entries"]
            [1]["families"][0]["class"] = serde_json::json!("system_fallback");
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-locale-after",
            &after,
        );

        let err =
            check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_locale_change(&out_dir)
                .expect_err("gate should reject stale sample trace classes");
        assert!(
            err.contains("expected cjk/emoji sample trace classes in the AFTER capture"),
            "{err}"
        );
    }

    #[test]
    fn locale_change_gate_rejects_missing_generic_no_tofu_flag() {
        let out_dir = unique_tmp_dir("fail_locale_missing_generic_flag");
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-locale-before",
            &locale_change_bundle("en-US", 10),
        );
        let mut after = locale_change_bundle("zh-CN", 11);
        after["windows"][0]["snapshots"][0]["resource_caches"]["render_text_fallback_policy"]["prefer_common_fallback_for_generics"] =
            serde_json::json!(false);
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-locale-after",
            &after,
        );

        let err =
            check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_locale_change(&out_dir)
                .expect_err("gate should reject missing generic no-tofu flag");
        assert!(
            err.contains("expected prefer_common_fallback_for_generics=true in both captures"),
            "{err}"
        );
    }

    #[test]
    fn locale_change_gate_rejects_curated_common_fallback_lane() {
        let out_dir = unique_tmp_dir("fail_locale_curated_lane");
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-locale-before",
            &locale_change_bundle("en-US", 10),
        );
        let mut after = locale_change_bundle("zh-CN", 11);
        after["windows"][0]["snapshots"][0]["resource_caches"]["render_text_fallback_policy"]["prefer_common_fallback"] =
            serde_json::json!(true);
        after["windows"][0]["snapshots"][0]["resource_caches"]["render_text_fallback_policy"]["common_fallback_injection"] =
            serde_json::json!("common_fallback");
        after["windows"][0]["snapshots"][0]["resource_caches"]["render_text_fallback_policy"]["common_fallback_candidates"] =
            serde_json::json!(["Noto Sans CJK SC", "Segoe UI Emoji"]);
        write_labeled_bundle(
            &out_dir,
            "ui-gallery-text-fallback-policy-locale-after",
            &after,
        );

        let err =
            check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_locale_change(&out_dir)
                .expect_err("gate should reject the curated common fallback lane");
        assert!(
            err.contains("expected common_fallback_injection=platform_default in both captures"),
            "{err}"
        );
    }
}
