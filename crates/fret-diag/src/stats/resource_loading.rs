use super::*;
use crate::util::{now_unix_ms, write_json_value};

struct ResourceLoadingCounterGateSpec {
    kind: &'static str,
    counter_field: &'static str,
    evidence_file: &'static str,
    counter_pointer: &'static str,
}

const ASSET_LOAD_MISSING_BUNDLE_ASSETS_MAX: ResourceLoadingCounterGateSpec =
    ResourceLoadingCounterGateSpec {
        kind: "asset_load_missing_bundle_assets_max",
        counter_field: "missing_bundle_asset_requests",
        evidence_file: "check.asset_load_missing_bundle_assets_max.json",
        counter_pointer: "debug.resource_loading.asset_load.missing_bundle_asset_requests",
    };

const ASSET_LOAD_STALE_MANIFEST_MAX: ResourceLoadingCounterGateSpec =
    ResourceLoadingCounterGateSpec {
        kind: "asset_load_stale_manifest_max",
        counter_field: "stale_manifest_requests",
        evidence_file: "check.asset_load_stale_manifest_max.json",
        counter_pointer: "debug.resource_loading.asset_load.stale_manifest_requests",
    };

const ASSET_LOAD_UNSUPPORTED_FILE_MAX: ResourceLoadingCounterGateSpec =
    ResourceLoadingCounterGateSpec {
        kind: "asset_load_unsupported_file_max",
        counter_field: "unsupported_file_requests",
        evidence_file: "check.asset_load_unsupported_file_max.json",
        counter_pointer: "debug.resource_loading.asset_load.unsupported_file_requests",
    };

const ASSET_LOAD_UNSUPPORTED_URL_MAX: ResourceLoadingCounterGateSpec =
    ResourceLoadingCounterGateSpec {
        kind: "asset_load_unsupported_url_max",
        counter_field: "unsupported_url_requests",
        evidence_file: "check.asset_load_unsupported_url_max.json",
        counter_pointer: "debug.resource_loading.asset_load.unsupported_url_requests",
    };

const ASSET_LOAD_EXTERNAL_REFERENCE_UNAVAILABLE_MAX: ResourceLoadingCounterGateSpec =
    ResourceLoadingCounterGateSpec {
        kind: "asset_load_external_reference_unavailable_max",
        counter_field: "external_reference_unavailable_requests",
        evidence_file: "check.asset_load_external_reference_unavailable_max.json",
        counter_pointer: "debug.resource_loading.asset_load.external_reference_unavailable_requests",
    };

const ASSET_LOAD_REVISION_CHANGES_MAX: ResourceLoadingCounterGateSpec =
    ResourceLoadingCounterGateSpec {
        kind: "asset_load_revision_changes_max",
        counter_field: "revision_change_requests",
        evidence_file: "check.asset_load_revision_changes_max.json",
        counter_pointer: "debug.resource_loading.asset_load.revision_change_requests",
    };

pub(crate) fn check_bundle_for_asset_load_missing_bundle_assets_max(
    bundle_path: &Path,
    max_allowed: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_resource_loading_counter_max_json(
        &bundle,
        bundle_path,
        &ASSET_LOAD_MISSING_BUNDLE_ASSETS_MAX,
        max_allowed,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_asset_load_stale_manifest_max(
    bundle_path: &Path,
    max_allowed: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_resource_loading_counter_max_json(
        &bundle,
        bundle_path,
        &ASSET_LOAD_STALE_MANIFEST_MAX,
        max_allowed,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_asset_load_unsupported_file_max(
    bundle_path: &Path,
    max_allowed: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_resource_loading_counter_max_json(
        &bundle,
        bundle_path,
        &ASSET_LOAD_UNSUPPORTED_FILE_MAX,
        max_allowed,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_asset_load_unsupported_url_max(
    bundle_path: &Path,
    max_allowed: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_resource_loading_counter_max_json(
        &bundle,
        bundle_path,
        &ASSET_LOAD_UNSUPPORTED_URL_MAX,
        max_allowed,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_asset_load_external_reference_unavailable_max(
    bundle_path: &Path,
    max_allowed: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_resource_loading_counter_max_json(
        &bundle,
        bundle_path,
        &ASSET_LOAD_EXTERNAL_REFERENCE_UNAVAILABLE_MAX,
        max_allowed,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_asset_load_revision_changes_max(
    bundle_path: &Path,
    max_allowed: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_resource_loading_counter_max_json(
        &bundle,
        bundle_path,
        &ASSET_LOAD_REVISION_CHANGES_MAX,
        max_allowed,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_bundled_font_baseline_source(
    bundle_path: &Path,
    expected_source: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_bundled_font_baseline_source_json(
        &bundle,
        bundle_path,
        expected_source,
        warmup_frames,
    )
}

fn check_bundle_for_resource_loading_counter_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    spec: &ResourceLoadingCounterGateSpec,
    max_allowed: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut resource_loading_snapshots: u64 = 0;
    let mut metric_snapshots: u64 = 0;
    let mut max_observed: Option<u64> = None;
    let mut last_observed: Option<serde_json::Value> = None;

    for window in windows {
        let window_id = window.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snapshots = window
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for snapshot in snapshots {
            let frame_id = snapshot
                .get("frame_id")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let resource_loading = snapshot
                .get("debug")
                .and_then(|v| v.get("resource_loading"))
                .and_then(|v| v.as_object());
            let Some(resource_loading) = resource_loading else {
                continue;
            };
            resource_loading_snapshots = resource_loading_snapshots.saturating_add(1);

            let counter = resource_loading
                .get("asset_load")
                .and_then(|v| v.get(spec.counter_field))
                .and_then(|v| v.as_u64());
            let Some(counter) = counter else {
                continue;
            };
            metric_snapshots = metric_snapshots.saturating_add(1);
            max_observed = Some(max_observed.map_or(counter, |prev| prev.max(counter)));
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": snapshot.get("tick_id").and_then(|v| v.as_u64()),
                "frame_id": frame_id,
                "value": counter,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(spec.evidence_file);
    let (bundle_artifact, bundle_json) = super::bundle_artifact_alias_pair(bundle_path);
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": spec.kind,
        "bundle_artifact": bundle_artifact,
        "bundle_json": bundle_json,
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "resource_loading_snapshots": resource_loading_snapshots,
        "metric_snapshots": metric_snapshots,
        "max_allowed": max_allowed,
        "max_observed": max_observed,
        "last_observed": last_observed,
        "expected": {
            "counter_pointer": spec.counter_pointer,
            "max_allowed": max_allowed,
        }
    });
    write_json_value(&evidence_path, &payload)?;

    if resource_loading_snapshots == 0 {
        return Err(format!(
            "{} gate requires debug.resource_loading snapshots after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            spec.kind,
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if metric_snapshots == 0 {
        return Err(format!(
            "{} gate requires {} after warmup, but no matching metric snapshots were observed (warmup_frames={warmup_frames}, resource_loading_snapshots={resource_loading_snapshots})\n  bundle: {}\n  evidence: {}",
            spec.kind,
            spec.counter_pointer,
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if max_observed.unwrap_or(0) <= max_allowed {
        return Ok(());
    }

    Err(format!(
        "{} gate failed (expected {} <= {}, observed max={})\n  bundle: {}\n  evidence: {}",
        spec.kind,
        spec.counter_pointer,
        max_allowed,
        max_observed.unwrap_or(0),
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_bundled_font_baseline_source_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    expected_source: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut resource_loading_snapshots: u64 = 0;
    let mut font_environment_snapshots: u64 = 0;
    let mut matched: bool = false;
    let mut last_observed: Option<serde_json::Value> = None;

    for window in windows {
        let window_id = window.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snapshots = window
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for snapshot in snapshots {
            let frame_id = snapshot
                .get("frame_id")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let resource_loading = snapshot
                .get("debug")
                .and_then(|v| v.get("resource_loading"))
                .and_then(|v| v.as_object());
            let Some(resource_loading) = resource_loading else {
                continue;
            };
            resource_loading_snapshots = resource_loading_snapshots.saturating_add(1);

            let font_environment = resource_loading
                .get("font_environment")
                .and_then(|v| v.as_object());
            let Some(font_environment) = font_environment else {
                continue;
            };
            font_environment_snapshots = font_environment_snapshots.saturating_add(1);

            let source = font_environment
                .get("bundled_baseline_source")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": snapshot.get("tick_id").and_then(|v| v.as_u64()),
                "frame_id": frame_id,
                "bundled_baseline_source": source,
                "bundled_profile_name": font_environment.get("bundled_profile_name").cloned(),
                "bundled_asset_bundle": font_environment.get("bundled_asset_bundle").cloned(),
                "bundled_asset_keys": font_environment.get("bundled_asset_keys").cloned(),
            }));

            if source == expected_source {
                matched = true;
                break;
            }
        }
        if matched {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.bundled_font_baseline_source.json");
    let (bundle_artifact, bundle_json) = super::bundle_artifact_alias_pair(bundle_path);
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "bundled_font_baseline_source",
        "bundle_artifact": bundle_artifact,
        "bundle_json": bundle_json,
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "resource_loading_snapshots": resource_loading_snapshots,
        "font_environment_snapshots": font_environment_snapshots,
        "expected_source": expected_source,
        "matched": matched,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if resource_loading_snapshots == 0 {
        return Err(format!(
            "bundled_font_baseline_source gate requires debug.resource_loading snapshots after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if font_environment_snapshots == 0 {
        return Err(format!(
            "bundled_font_baseline_source gate requires debug.resource_loading.font_environment after warmup, but none were observed (warmup_frames={warmup_frames}, resource_loading_snapshots={resource_loading_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matched {
        return Ok(());
    }

    Err(format!(
        "bundled_font_baseline_source gate failed (expected source={expected_source})\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

#[cfg(test)]
pub(crate) fn check_bundle_for_asset_load_missing_bundle_assets_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    max_allowed: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    check_bundle_for_resource_loading_counter_max_json(
        bundle,
        bundle_path,
        &ASSET_LOAD_MISSING_BUNDLE_ASSETS_MAX,
        max_allowed,
        warmup_frames,
    )
}

#[cfg(test)]
pub(crate) fn check_bundle_for_asset_load_stale_manifest_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    max_allowed: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    check_bundle_for_resource_loading_counter_max_json(
        bundle,
        bundle_path,
        &ASSET_LOAD_STALE_MANIFEST_MAX,
        max_allowed,
        warmup_frames,
    )
}

#[cfg(test)]
pub(crate) fn check_bundle_for_asset_load_unsupported_file_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    max_allowed: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    check_bundle_for_resource_loading_counter_max_json(
        bundle,
        bundle_path,
        &ASSET_LOAD_UNSUPPORTED_FILE_MAX,
        max_allowed,
        warmup_frames,
    )
}

#[cfg(test)]
pub(crate) fn check_bundle_for_asset_load_unsupported_url_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    max_allowed: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    check_bundle_for_resource_loading_counter_max_json(
        bundle,
        bundle_path,
        &ASSET_LOAD_UNSUPPORTED_URL_MAX,
        max_allowed,
        warmup_frames,
    )
}

#[cfg(test)]
pub(crate) fn check_bundle_for_asset_load_external_reference_unavailable_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    max_allowed: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    check_bundle_for_resource_loading_counter_max_json(
        bundle,
        bundle_path,
        &ASSET_LOAD_EXTERNAL_REFERENCE_UNAVAILABLE_MAX,
        max_allowed,
        warmup_frames,
    )
}

#[cfg(test)]
pub(crate) fn check_bundle_for_asset_load_revision_changes_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    max_allowed: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    check_bundle_for_resource_loading_counter_max_json(
        bundle,
        bundle_path,
        &ASSET_LOAD_REVISION_CHANGES_MAX,
        max_allowed,
        warmup_frames,
    )
}
