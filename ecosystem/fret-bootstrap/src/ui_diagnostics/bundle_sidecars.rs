use std::path::Path;

use fret_diag_protocol::UiScriptResultV1;

use super::{
    UiDiagnosticsService, UiDiagnosticsWindowBundleV1, bundle, bundle_index, write_json_compact,
};

pub(super) fn want_sidecars() -> bool {
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

pub(super) fn write_bundle_sidecars(
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
    let window_map = bundle_index::build_window_map_json(&label, windows);
    let dock_routing = bundle_index::build_dock_routing_json(&label, windows);

    if is_script_dump
        && let Some(script_result) = read_script_result_v1(&service.cfg.script_result_path) {
            let _ = write_json_compact(dir.join("script.result.json"), &script_result);
            if let Some(script_steps) =
                super::script_step_index::build_script_step_index_payload(&index, &script_result)
                && let Some(obj) = index.as_object_mut()
            {
                obj.insert("script".to_string(), script_steps);
            }
        }

    let _ = write_json_compact(dir.join("bundle.index.json"), &index);
    let _ = write_json_compact(dir.join("bundle.meta.json"), &meta);
    let _ = write_json_compact(dir.join("test_ids.index.json"), &test_ids_index);
    let _ = write_json_compact(dir.join("window.map.json"), &window_map);
    let _ = write_json_compact(dir.join("dock.routing.json"), &dock_routing);
}
