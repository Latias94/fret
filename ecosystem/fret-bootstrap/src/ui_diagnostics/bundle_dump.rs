use std::path::{Path, PathBuf};

use fret_diag_protocol::UiScriptResultV1;

use super::{
    UiArtifactStatsV1, UiDiagnosticsBundleV1, UiDiagnosticsBundleV2, UiDiagnosticsService,
    UiDiagnosticsWindowBundleV1, bundle, bundle_index, unix_ms_now, write_json, write_json_compact,
    write_latest_pointer,
};

fn want_sidecars() -> bool {
    std::env::var("FRET_DIAG_BUNDLE_WRITE_INDEX")
        .ok()
        .map(|v| v.trim().to_ascii_lowercase())
        .map(|v| !matches!(v.as_str(), "0" | "false" | "no" | "off"))
        .unwrap_or(true)
}

fn read_script_result_v1(path: &Path) -> Option<UiScriptResultV1> {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<UiScriptResultV1>(&bytes).ok())
        .filter(|r| r.schema_version == 1)
}

fn write_bundle_sidecars(
    service: &UiDiagnosticsService,
    dir: &Path,
    bundle_json_path: &Path,
    windows: &[UiDiagnosticsWindowBundleV1],
    semantics_table: Option<&bundle::UiBundleSemanticsTableV1>,
    is_script_dump: bool,
) {
    let label = bundle_json_path.display().to_string();
    let mut index = bundle_index::build_bundle_index_json(&label, windows, semantics_table);
    let meta = bundle_index::build_bundle_meta_json(&label, windows, semantics_table);
    let test_ids_index = bundle_index::build_test_ids_index_json(&label, windows, semantics_table);

    if is_script_dump {
        if let Some(script_result) = read_script_result_v1(&service.cfg.script_result_path) {
            let _ = write_json_compact(dir.join("script.result.json"), &script_result);
            if let Some(script_steps) =
                super::script_step_index::build_script_step_index_payload(&index, &script_result)
                && let Some(obj) = index.as_object_mut()
            {
                obj.insert("script".to_string(), script_steps);
            }
        }
    }

    let _ = write_json_compact(dir.join("bundle.index.json"), &index);
    let _ = write_json_compact(dir.join("bundle.meta.json"), &meta);
    let _ = write_json_compact(dir.join("test_ids.index.json"), &test_ids_index);
}

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

    let default_schema_version = if is_script_dump {
        bundle::BundleSchemaVersionV1::V2
    } else {
        bundle::BundleSchemaVersionV1::V2
    };
    let schema_version = bundle::BundleSchemaVersionV1::from_env_or_default(default_schema_version);

    let default_semantics_mode = if is_script_dump {
        bundle::BundleSemanticsModeV1::Last
    } else {
        bundle::BundleSemanticsModeV1::Changed
    };
    let semantics_mode = bundle::BundleSemanticsModeV1::from_env_or_default(default_semantics_mode);

    if !cfg!(target_arch = "wasm32") && std::fs::create_dir_all(&dir).is_err() {
        return None;
    }

    let bundle_json_path = dir.join("bundle.json");
    let bundle_json_format = std::env::var("FRET_DIAG_BUNDLE_JSON_FORMAT")
        .ok()
        .map(|v| v.trim().to_ascii_lowercase());
    let want_pretty = matches!(bundle_json_format.as_deref(), Some("pretty"));

    let (bundle_json_bytes, window_count, event_count, snapshot_count, max_snapshots, dump_max) =
        match schema_version {
            bundle::BundleSchemaVersionV1::V1 => {
                let mut bundle = bundle_v1;
                bundle.apply_semantics_mode_v1(semantics_mode);

                let bundle_json_bytes = if !cfg!(target_arch = "wasm32") {
                    let write_result = if want_pretty {
                        write_json(bundle_json_path.clone(), &bundle)
                    } else {
                        write_json_compact(bundle_json_path.clone(), &bundle)
                    };
                    if write_result.is_err() {
                        return None;
                    }
                    std::fs::metadata(&bundle_json_path).ok().map(|m| m.len())
                } else if service.ws_is_configured() {
                    serde_json::to_vec(&bundle).ok().map(|b| b.len() as u64)
                } else {
                    None
                };

                let window_count = bundle.windows.len() as u64;
                let event_count = bundle.windows.iter().map(|w| w.events.len() as u64).sum();
                let snapshot_count = bundle
                    .windows
                    .iter()
                    .map(|w| w.snapshots.len() as u64)
                    .sum();
                let max_snapshots = bundle.config.max_snapshots as u64;
                let dump_max = bundle.config.dump_max_snapshots.map(|n| n as u64);

                #[cfg(feature = "diagnostics-ws")]
                {
                    service.ws_send_bundle_dumped_v1(ts, &dir, &bundle, request_id);
                }

                if !cfg!(target_arch = "wasm32") {
                    if want_sidecars() {
                        write_bundle_sidecars(
                            service,
                            &dir,
                            &bundle_json_path,
                            &bundle.windows,
                            None,
                            is_script_dump,
                        );
                    }
                }

                (
                    bundle_json_bytes,
                    window_count,
                    event_count,
                    snapshot_count,
                    max_snapshots,
                    dump_max,
                )
            }
            bundle::BundleSchemaVersionV1::V2 => {
                let mut bundle = UiDiagnosticsBundleV2::from_v1(bundle_v1);
                bundle.apply_semantics_mode_v1(semantics_mode);

                let bundle_json_bytes = if !cfg!(target_arch = "wasm32") {
                    let write_result = if want_pretty {
                        write_json(bundle_json_path.clone(), &bundle)
                    } else {
                        write_json_compact(bundle_json_path.clone(), &bundle)
                    };
                    if write_result.is_err() {
                        return None;
                    }
                    std::fs::metadata(&bundle_json_path).ok().map(|m| m.len())
                } else if service.ws_is_configured() {
                    serde_json::to_vec(&bundle).ok().map(|b| b.len() as u64)
                } else {
                    None
                };

                let window_count = bundle.windows.len() as u64;
                let event_count = bundle.windows.iter().map(|w| w.events.len() as u64).sum();
                let snapshot_count = bundle
                    .windows
                    .iter()
                    .map(|w| w.snapshots.len() as u64)
                    .sum();
                let max_snapshots = bundle.config.max_snapshots as u64;
                let dump_max = bundle.config.dump_max_snapshots.map(|n| n as u64);

                #[cfg(feature = "diagnostics-ws")]
                {
                    service.ws_send_bundle_dumped_v1(ts, &dir, &bundle, request_id);
                }

                if !cfg!(target_arch = "wasm32") {
                    if want_sidecars() {
                        let semantics_table = bundle.tables.semantics.as_ref();
                        write_bundle_sidecars(
                            service,
                            &dir,
                            &bundle_json_path,
                            &bundle.windows,
                            semantics_table,
                            is_script_dump,
                        );
                    }
                }

                (
                    bundle_json_bytes,
                    window_count,
                    event_count,
                    snapshot_count,
                    max_snapshots,
                    dump_max,
                )
            }
        };

    if !cfg!(target_arch = "wasm32") {
        let _ = write_latest_pointer(&service.cfg.out_dir, &dir);
        if service.cfg.screenshot_on_dump {
            let _ = std::fs::write(dir.join("screenshot.request"), b"1\n");
        }
    }

    service.last_dump_dir = Some(dir.clone());
    service.last_dump_artifact_stats = Some(UiArtifactStatsV1 {
        schema_version: 1,
        bundle_json_bytes,
        window_count,
        event_count,
        snapshot_count,
        max_snapshots,
        dump_max_snapshots: dump_max,
    });

    Some(dir)
}
