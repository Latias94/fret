use std::path::{Path, PathBuf};

use crate::util::write_json_value;

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
        let bundle_json = path.join("bundle.json");
        if !bundle_json.is_file() {
            continue;
        }
        match &best {
            Some((best_ts, _)) if *best_ts >= ts => {}
            _ => best = Some((ts, path)),
        }
    }
    best.map(|(_, p)| p)
}

fn bundle_last_snapshot_by_frame_id<'a>(
    bundle: &'a serde_json::Value,
) -> Option<&'a serde_json::Value> {
    let windows = bundle.get("windows")?.as_array()?;
    let w = windows.first()?;
    let snaps = w.get("snapshots")?.as_array()?;
    snaps
        .iter()
        .filter_map(|s| Some((s.get("frame_id")?.as_u64()?, s)))
        .max_by_key(|(frame_id, _)| *frame_id)
        .map(|(_, s)| s)
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

    let before_path = before_dir.join("bundle.json");
    let after_path = after_dir.join("bundle.json");

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

    fn bundle_last_text_policy_key(bundle: &serde_json::Value) -> Option<u64> {
        let best = bundle_last_snapshot_by_frame_id(bundle)?;

        let policy = best
            .get("resource_caches")?
            .get("render_text_fallback_policy")?
            .as_object()?;
        policy.get("fallback_policy_key")?.as_u64()
    }

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

    let before_path = before_dir.join("bundle.json");
    let after_path = after_dir.join("bundle.json");

    let before_bytes = std::fs::read(&before_path).map_err(|e| e.to_string())?;
    let after_bytes = std::fs::read(&after_path).map_err(|e| e.to_string())?;
    let before_bundle: serde_json::Value =
        serde_json::from_slice(&before_bytes).map_err(|e| e.to_string())?;
    let after_bundle: serde_json::Value =
        serde_json::from_slice(&after_bytes).map_err(|e| e.to_string())?;

    let before_key = bundle_last_text_policy_key(&before_bundle).ok_or_else(|| {
        format!(
            "ui-gallery text fallback policy gate expected renderer text fallback policy snapshot in bundle\n  bundle: {}",
            before_path.display()
        )
    })?;
    let after_key = bundle_last_text_policy_key(&after_bundle).ok_or_else(|| {
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
        "before": { "fallback_policy_key": before_key },
        "after": { "fallback_policy_key": after_key },
    });
    let _ = write_json_value(&evidence_path, &payload);

    if before_key == after_key {
        return Err(format!(
            "ui-gallery text fallback policy gate failed: expected fallback_policy_key to change after settings apply\n  before: fallback_policy_key={}\n  after:  fallback_policy_key={}\n  evidence: {}",
            before_key,
            after_key,
            evidence_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_out_dir_for_ui_gallery_text_mixed_script_bundled_fallback_conformance(
    out_dir: &Path,
) -> Result<(), String> {
    const LABEL: &str = "ui-gallery-text-mixed-script-bundled-fallback-conformance";

    fn bundle_last_text_policy_snapshot(
        bundle: &serde_json::Value,
    ) -> Option<(bool, bool, Vec<String>)> {
        let best = bundle_last_snapshot_by_frame_id(bundle)?;

        let policy = best
            .get("resource_caches")?
            .get("render_text_fallback_policy")?
            .as_object()?;
        let system_fonts_enabled = policy.get("system_fonts_enabled")?.as_bool()?;
        let prefer_common_fallback = policy.get("prefer_common_fallback")?.as_bool()?;
        let candidates = policy
            .get("common_fallback_candidates")
            .and_then(|v| v.as_array())
            .map(|v| {
                v.iter()
                    .filter_map(|s| s.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        Some((system_fonts_enabled, prefer_common_fallback, candidates))
    }

    fn bundle_last_text_missing_glyphs(bundle: &serde_json::Value) -> Option<u64> {
        let best = bundle_last_snapshot_by_frame_id(bundle)?;

        let render_text = best
            .get("resource_caches")?
            .get("render_text")?
            .as_object()?;
        render_text.get("frame_missing_glyphs")?.as_u64()
    }

    let dir = find_latest_labeled_bundle_dir(out_dir, LABEL).ok_or_else(|| {
        format!(
            "ui-gallery text mixed-script bundled fallback gate expected a capture_bundle label={LABEL} under out_dir, but none was found\n  out_dir: {}",
            out_dir.display()
        )
    })?;

    let bundle_path = dir.join("bundle.json");
    let bytes = std::fs::read(&bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let (system_fonts_enabled, prefer_common_fallback, candidates) =
        bundle_last_text_policy_snapshot(&bundle).ok_or_else(|| {
            format!(
                "ui-gallery text mixed-script bundled fallback gate expected renderer text fallback policy snapshot in bundle\n  bundle: {}",
                bundle_path.display()
            )
        })?;
    let missing_glyphs = bundle_last_text_missing_glyphs(&bundle).ok_or_else(|| {
        format!(
            "ui-gallery text mixed-script bundled fallback gate expected renderer text perf snapshot in bundle\n  bundle: {}",
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
            "system_fonts_enabled": system_fonts_enabled,
            "prefer_common_fallback": prefer_common_fallback,
            "common_fallback_candidates": candidates,
        },
        "render_text": { "frame_missing_glyphs": missing_glyphs },
    });
    let _ = write_json_value(&evidence_path, &payload);

    if system_fonts_enabled {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected system_fonts_enabled=false for a deterministic (bundled-only) fallback baseline\n  hint: rerun with --env FRET_TEXT_SYSTEM_FONTS=0 and ensure bundled fonts are loaded (FRET_UI_GALLERY_BOOTSTRAP_FONTS=1)\n  evidence: {}",
            evidence_path.display()
        ));
    }
    if !prefer_common_fallback {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected prefer_common_fallback=true when system fonts are disabled\n  evidence: {}",
            evidence_path.display()
        ));
    }

    const EXPECTED: &[&str] = &["Noto Sans CJK SC", "Noto Sans Arabic", "Noto Color Emoji"];
    for &family in EXPECTED {
        if !candidates.iter().any(|c| c == family) {
            return Err(format!(
                "ui-gallery text mixed-script bundled fallback gate failed: expected common_fallback_candidates to include {family:?}\n  evidence: {}",
                evidence_path.display()
            ));
        }
    }

    if missing_glyphs != 0 {
        return Err(format!(
            "ui-gallery text mixed-script bundled fallback gate failed: expected frame_missing_glyphs=0 under bundled fonts\n  observed: {}\n  evidence: {}",
            missing_glyphs,
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

    fn bundle_last_text_policy_key_and_locale(
        bundle: &serde_json::Value,
    ) -> Option<(u64, Option<String>)> {
        let best = bundle_last_snapshot_by_frame_id(bundle)?;

        let policy = best
            .get("resource_caches")?
            .get("render_text_fallback_policy")?
            .as_object()?;
        let key = policy.get("fallback_policy_key")?.as_u64()?;
        let locale_bcp47 = policy
            .get("locale_bcp47")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        Some((key, locale_bcp47))
    }

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

    let before_path = before_dir.join("bundle.json");
    let after_path = after_dir.join("bundle.json");

    let before_bytes = std::fs::read(&before_path).map_err(|e| e.to_string())?;
    let after_bytes = std::fs::read(&after_path).map_err(|e| e.to_string())?;
    let before_bundle: serde_json::Value =
        serde_json::from_slice(&before_bytes).map_err(|e| e.to_string())?;
    let after_bundle: serde_json::Value =
        serde_json::from_slice(&after_bytes).map_err(|e| e.to_string())?;

    let before = bundle_last_text_policy_key_and_locale(&before_bundle).ok_or_else(|| {
        format!(
            "ui-gallery text fallback policy locale gate expected renderer text fallback policy snapshot in bundle\n  bundle: {}",
            before_path.display()
        )
    })?;
    let after = bundle_last_text_policy_key_and_locale(&after_bundle).ok_or_else(|| {
        format!(
            "ui-gallery text fallback policy locale gate expected renderer text fallback policy snapshot in bundle\n  bundle: {}",
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
        "before": { "fallback_policy_key": before.0, "locale_bcp47": before.1 },
        "after": { "fallback_policy_key": after.0, "locale_bcp47": after.1 },
    });
    let _ = write_json_value(&evidence_path, &payload);

    if before.1.as_deref() != Some("en-US") {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected locale_bcp47 to be en-US in the BEFORE capture\n  observed: {:?}\n  evidence: {}",
            before.1,
            evidence_path.display()
        ));
    }
    if after.1.as_deref() != Some("zh-CN") {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected locale_bcp47 to be zh-CN in the AFTER capture\n  observed: {:?}\n  evidence: {}",
            after.1,
            evidence_path.display()
        ));
    }
    if before.0 == after.0 {
        return Err(format!(
            "ui-gallery text fallback policy locale gate failed: expected fallback_policy_key to change when locale changes\n  before: {}\n  after: {}\n  evidence: {}",
            before.0,
            after.0,
            evidence_path.display()
        ));
    }

    Ok(())
}
