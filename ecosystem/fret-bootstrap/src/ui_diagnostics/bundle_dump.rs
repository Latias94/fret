use std::path::PathBuf;

use super::bundle_dump_policy::{
    apply_dump_semantics_policy_to_semantics_table, apply_dump_semantics_policy_to_windows,
    resolve_dump_semantics_policy,
};
use super::bundle_sidecars::{want_sidecars, write_bundle_sidecars};
use super::{
    UiArtifactStatsV1, UiDiagnosticsBundleConfigV1, UiDiagnosticsBundleV1, UiDiagnosticsBundleV2,
    UiDiagnosticsService, UiDiagnosticsWindowBundleV1, bundle, unix_ms_now, write_json,
    write_json_compact, write_latest_pointer,
};

pub(super) fn dump_bundle_with_options(
    service: &mut UiDiagnosticsService,
    label: Option<&str>,
    dump_max_snapshots_override: Option<usize>,
    request_id: Option<u64>,
) -> Option<PathBuf> {
    #[cfg(not(feature = "diagnostics-ws"))]
    let _ = request_id;

    let ts = unix_ms_now();
    let mut dir_name = ts.to_string();
    if let Some(label) = label {
        if !label.is_empty() {
            dir_name = format!("{dir_name}-{label}");
        }
    }

    let dir = service.cfg.out_dir.join(dir_name);

    let is_script_dump = label.is_some()
        && (!service.active_scripts.is_empty()
            || service.pending_script.is_some()
            || service.pending_script_run_id.is_some()
            || label.unwrap_or_default().trim().starts_with("script-step-"));
    let dump_max_snapshots = {
        let default = if is_script_dump {
            service.cfg.script_dump_max_snapshots
        } else {
            service.cfg.max_snapshots
        };

        match dump_max_snapshots_override {
            Some(want) => {
                if service.cfg.max_snapshots == 0 {
                    0
                } else {
                    want.clamp(1, service.cfg.max_snapshots)
                }
            }
            None => default,
        }
    };

    let bundle_v1 = UiDiagnosticsBundleV1::from_service(ts, &dir, service, dump_max_snapshots);

    let default_semantics_mode = if is_script_dump {
        bundle::BundleSemanticsModeV1::Last
    } else {
        bundle::BundleSemanticsModeV1::Changed
    };
    let semantics_mode = bundle::BundleSemanticsModeV1::from_env_or_default(default_semantics_mode);

    let dump_semantics_policy = resolve_dump_semantics_policy(&service.cfg, is_script_dump);

    if !cfg!(target_arch = "wasm32") && std::fs::create_dir_all(&dir).is_err() {
        return None;
    }

    let bundle_json_path = dir.join("bundle.json");
    let bundle_json_format = std::env::var("FRET_DIAG_BUNDLE_JSON_FORMAT")
        .ok()
        .map(|v| v.trim().to_ascii_lowercase());
    let want_pretty = matches!(bundle_json_format.as_deref(), Some("pretty"));

    let stats = dump_schema_v2(
        service,
        ts,
        &dir,
        &bundle_json_path,
        want_pretty,
        bundle_v1,
        semantics_mode,
        &dump_semantics_policy,
        is_script_dump,
        request_id,
    )?;

    if !cfg!(target_arch = "wasm32") {
        let _ = write_latest_pointer(&service.cfg.out_dir, &dir);
        if service.cfg.screenshot_on_dump {
            let _ = std::fs::write(dir.join("screenshot.request"), b"1\n");
        }
    }

    service.last_dump_dir = Some(dir.clone());
    service.last_dump_artifact_stats = Some(stats);

    Some(dir)
}

fn write_bundle_json_bytes<T: serde::Serialize>(
    service: &UiDiagnosticsService,
    bundle_json_path: &PathBuf,
    want_pretty: bool,
    bundle: &T,
) -> Option<Option<u64>> {
    if !cfg!(target_arch = "wasm32") {
        let write_result = if want_pretty {
            write_json(bundle_json_path.clone(), bundle)
        } else {
            write_json_compact(bundle_json_path.clone(), bundle)
        };
        if write_result.is_err() {
            return None;
        }
        return Some(std::fs::metadata(bundle_json_path).ok().map(|m| m.len()));
    }

    if service.ws_is_configured() {
        return Some(serde_json::to_vec(bundle).ok().map(|b| b.len() as u64));
    }

    Some(None)
}

fn finalize_dump(
    service: &mut UiDiagnosticsService,
    exported_unix_ms: u64,
    dir: &PathBuf,
    bundle_json_path: &PathBuf,
    want_pretty: bool,
    bundle: &impl serde::Serialize,
    windows: &[UiDiagnosticsWindowBundleV1],
    semantics_table: Option<&bundle::UiBundleSemanticsTableV1>,
    config: &UiDiagnosticsBundleConfigV1,
    is_script_dump: bool,
    request_id: Option<u64>,
) -> Option<UiArtifactStatsV1> {
    #[cfg(not(feature = "diagnostics-ws"))]
    let _ = exported_unix_ms;
    #[cfg(not(feature = "diagnostics-ws"))]
    let _ = request_id;

    let bundle_json_bytes =
        write_bundle_json_bytes(service, bundle_json_path, want_pretty, bundle)?;

    let window_count = windows.len() as u64;
    let event_count = windows.iter().map(|w| w.events.len() as u64).sum();
    let snapshot_count = windows.iter().map(|w| w.snapshots.len() as u64).sum();
    let max_snapshots = config.max_snapshots as u64;
    let dump_max_snapshots = config.dump_max_snapshots.map(|n| n as u64);

    #[cfg(feature = "diagnostics-ws")]
    {
        service.ws_send_bundle_dumped_v1(exported_unix_ms, dir, bundle, request_id);
    }

    if !cfg!(target_arch = "wasm32") && want_sidecars() {
        write_bundle_sidecars(
            service,
            dir,
            bundle_json_path,
            windows,
            semantics_table,
            is_script_dump,
        );
    }

    Some(UiArtifactStatsV1 {
        schema_version: 1,
        bundle_json_bytes,
        window_count,
        event_count,
        snapshot_count,
        max_snapshots,
        dump_max_snapshots,
    })
}

fn dump_schema_v2(
    service: &mut UiDiagnosticsService,
    exported_unix_ms: u64,
    dir: &PathBuf,
    bundle_json_path: &PathBuf,
    want_pretty: bool,
    bundle_v1: UiDiagnosticsBundleV1,
    semantics_mode: bundle::BundleSemanticsModeV1,
    dump_semantics_policy: &super::bundle_dump_policy::DumpSemanticsPolicy,
    is_script_dump: bool,
    request_id: Option<u64>,
) -> Option<UiArtifactStatsV1> {
    let mut bundle = UiDiagnosticsBundleV2::from_v1(bundle_v1);
    bundle.apply_semantics_mode_v1(semantics_mode);
    bundle.config.max_semantics_nodes = dump_semantics_policy.max_nodes;
    apply_dump_semantics_policy_to_windows(&mut bundle.windows, dump_semantics_policy);
    if let Some(table) = bundle.tables.semantics.as_mut() {
        apply_dump_semantics_policy_to_semantics_table(table, dump_semantics_policy);
    }

    let semantics_table = bundle.tables.semantics.as_ref();
    let stats = finalize_dump(
        service,
        exported_unix_ms,
        dir,
        bundle_json_path,
        want_pretty,
        &bundle,
        &bundle.windows,
        semantics_table,
        &bundle.config,
        is_script_dump,
        request_id,
    )?;

    if !cfg!(target_arch = "wasm32") && service.cfg.write_bundle_schema2 {
        // Opt-in compact artifact for schema2-first workflows.
        //
        // We intentionally keep this file name stable (`bundle.schema2.json`) so tooling can
        // consume it directly without reparsing raw `bundle.json`.
        bundle.apply_semantics_mode_v1(bundle::BundleSemanticsModeV1::Last);
        let _ = write_json_compact(dir.join("bundle.schema2.json"), &bundle);
    }

    Some(stats)
}
