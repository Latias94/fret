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

pub(super) fn check_out_dir_for_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps(
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

