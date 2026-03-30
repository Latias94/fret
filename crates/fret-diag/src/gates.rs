use std::path::{Path, PathBuf};

use super::util::{now_unix_ms, read_json_value, write_json_value};

#[derive(Default)]
pub(super) struct ResourceFootprintThresholds {
    pub(super) max_working_set_bytes: Option<u64>,
    pub(super) max_peak_working_set_bytes: Option<u64>,
    pub(super) max_macos_physical_footprint_peak_bytes: Option<u64>,
    pub(super) max_macos_owned_unmapped_memory_dirty_bytes: Option<u64>,
    pub(super) max_macos_io_surface_dirty_bytes: Option<u64>,
    pub(super) max_macos_io_accelerator_dirty_bytes: Option<u64>,
    pub(super) max_macos_malloc_small_dirty_bytes: Option<u64>,
    pub(super) max_macos_malloc_dirty_bytes_total: Option<u64>,
    pub(super) max_macos_malloc_zones_total_allocated_bytes: Option<u64>,
    pub(super) max_macos_malloc_zones_total_frag_bytes: Option<u64>,
    pub(super) max_macos_malloc_zones_total_dirty_bytes: Option<u64>,
    pub(super) max_cpu_avg_percent_total_cores: Option<f64>,
}

impl ResourceFootprintThresholds {
    pub(super) fn any(&self) -> bool {
        self.max_working_set_bytes.is_some()
            || self.max_peak_working_set_bytes.is_some()
            || self.max_macos_physical_footprint_peak_bytes.is_some()
            || self.max_macos_owned_unmapped_memory_dirty_bytes.is_some()
            || self.max_macos_io_surface_dirty_bytes.is_some()
            || self.max_macos_io_accelerator_dirty_bytes.is_some()
            || self.max_macos_malloc_small_dirty_bytes.is_some()
            || self.max_macos_malloc_dirty_bytes_total.is_some()
            || self.max_macos_malloc_zones_total_allocated_bytes.is_some()
            || self.max_macos_malloc_zones_total_frag_bytes.is_some()
            || self.max_macos_malloc_zones_total_dirty_bytes.is_some()
            || self.max_cpu_avg_percent_total_cores.is_some()
    }
}

#[derive(Debug, Clone)]
pub(super) struct ResourceFootprintGateResult {
    pub(super) evidence_path: PathBuf,
    pub(super) failures: usize,
}

#[derive(Debug, Clone)]
pub(super) struct WgpuMetalAllocatedSizeGateResult {
    pub(super) evidence_path: PathBuf,
    pub(super) failures: usize,
}

#[derive(Debug, Clone, Default)]
pub(super) struct WgpuHubCountsThresholds {
    pub(super) max_wgpu_hub_buffers: Option<u64>,
    pub(super) max_wgpu_hub_textures: Option<u64>,
    pub(super) max_wgpu_hub_render_pipelines: Option<u64>,
    pub(super) max_wgpu_hub_shader_modules: Option<u64>,
}

impl WgpuHubCountsThresholds {
    pub(super) fn any(&self) -> bool {
        self.max_wgpu_hub_buffers.is_some()
            || self.max_wgpu_hub_textures.is_some()
            || self.max_wgpu_hub_render_pipelines.is_some()
            || self.max_wgpu_hub_shader_modules.is_some()
    }
}

#[derive(Debug, Clone)]
pub(super) struct WgpuHubCountsGateResult {
    pub(super) evidence_path: PathBuf,
    pub(super) failures: usize,
}

#[derive(Debug, Clone)]
pub(super) struct RenderTextAtlasBytesGateResult {
    pub(super) evidence_path: PathBuf,
    pub(super) failures: usize,
}

#[derive(Default)]
pub(super) struct RenderTextFontDbThresholds {
    pub(super) max_render_text_registered_font_blobs_total_bytes: Option<u64>,
    pub(super) max_render_text_registered_font_blobs_count: Option<u64>,
    pub(super) max_render_text_shape_cache_entries: Option<u64>,
    pub(super) max_render_text_blob_cache_entries: Option<u64>,
    pub(super) max_render_text_shape_cache_bytes_estimate_total: Option<u64>,
    pub(super) max_render_text_blob_paint_palette_bytes_estimate_total: Option<u64>,
    pub(super) max_render_text_blob_decorations_bytes_estimate_total: Option<u64>,
}

impl RenderTextFontDbThresholds {
    pub(super) fn any(&self) -> bool {
        self.max_render_text_registered_font_blobs_total_bytes
            .is_some()
            || self.max_render_text_registered_font_blobs_count.is_some()
            || self.max_render_text_shape_cache_entries.is_some()
            || self.max_render_text_blob_cache_entries.is_some()
            || self
                .max_render_text_shape_cache_bytes_estimate_total
                .is_some()
            || self
                .max_render_text_blob_paint_palette_bytes_estimate_total
                .is_some()
            || self
                .max_render_text_blob_decorations_bytes_estimate_total
                .is_some()
    }
}

#[derive(Debug, Clone)]
pub(super) struct RenderTextFontDbGateResult {
    pub(super) evidence_path: PathBuf,
    pub(super) failures: usize,
}

#[derive(Default)]
pub(super) struct RendererGpuBudgetThresholds {
    pub(super) max_renderer_gpu_images_bytes_estimate: Option<u64>,
    pub(super) max_renderer_gpu_render_targets_bytes_estimate: Option<u64>,
    pub(super) max_renderer_intermediate_peak_in_use_bytes: Option<u64>,
}

impl RendererGpuBudgetThresholds {
    pub(super) fn any(&self) -> bool {
        self.max_renderer_gpu_images_bytes_estimate.is_some()
            || self
                .max_renderer_gpu_render_targets_bytes_estimate
                .is_some()
            || self.max_renderer_intermediate_peak_in_use_bytes.is_some()
    }
}

#[derive(Debug, Clone)]
pub(super) struct RendererGpuBudgetsGateResult {
    pub(super) evidence_path: PathBuf,
    pub(super) failures: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LinearBytesVsImagesThreshold {
    pub(super) intercept_bytes: u64,
    pub(super) slope_ppm: u64,
    pub(super) headroom_bytes: u64,
}

#[derive(Debug, Clone)]
pub(super) struct LinearBytesVsImagesGateResult {
    pub(super) evidence_path: PathBuf,
    pub(super) failures: usize,
}

#[derive(Default)]
pub(super) struct CodeEditorMemoryThresholds {
    pub(super) max_code_editor_buffer_len_bytes: Option<u64>,
    pub(super) max_code_editor_undo_text_bytes_estimate_total: Option<u64>,
    pub(super) max_code_editor_row_text_cache_entries: Option<u64>,
    pub(super) max_code_editor_row_rich_cache_entries: Option<u64>,
}

impl CodeEditorMemoryThresholds {
    pub(super) fn any(&self) -> bool {
        self.max_code_editor_buffer_len_bytes.is_some()
            || self
                .max_code_editor_undo_text_bytes_estimate_total
                .is_some()
            || self.max_code_editor_row_text_cache_entries.is_some()
            || self.max_code_editor_row_rich_cache_entries.is_some()
    }
}

#[derive(Debug, Clone)]
pub(super) struct CodeEditorMemoryGateResult {
    pub(super) evidence_path: PathBuf,
    pub(super) failures: usize,
}

pub(super) fn check_wgpu_metal_current_allocated_size_threshold(
    out_dir: &Path,
    bundle_path: Option<&Path>,
    max_wgpu_metal_current_allocated_size_bytes: u64,
) -> Result<WgpuMetalAllocatedSizeGateResult, String> {
    let out_path = out_dir.join("check.wgpu_metal_allocated_size.json");

    let v = bundle_path.and_then(read_json_value);
    let bundle_present = v.is_some();

    let (tick_id, frame_id, present_flag_any, bytes_min, bytes_max, bytes_max_tick_frame) =
        if let Some(v) = v.as_ref() {
            let windows = v.get("windows").and_then(|v| v.as_array());
            let first_window = windows.and_then(|w| w.first());
            let snapshots = first_window
                .and_then(|w| w.get("snapshots"))
                .and_then(|v| v.as_array());
            let last_snapshot = snapshots.and_then(|s| s.last());

            let tick_id = last_snapshot
                .and_then(|s| s.get("tick_id"))
                .and_then(|v| v.as_u64());
            let frame_id = last_snapshot
                .and_then(|s| s.get("frame_id"))
                .and_then(|v| v.as_u64());

            let mut present_flag_any: Option<bool> = None;
            let mut bytes_min: Option<u64> = None;
            let mut bytes_max: Option<u64> = None;
            let mut bytes_max_tick_frame: Option<(u64, u64)> = None;

            if let Some(snapshots) = snapshots {
                for snapshot in snapshots {
                    let snap_tick_id = snapshot.get("tick_id").and_then(|v| v.as_u64());
                    let snap_frame_id = snapshot.get("frame_id").and_then(|v| v.as_u64());
                    let stats = snapshot
                        .get("debug")
                        .and_then(|d| d.get("stats"))
                        .and_then(|v| v.as_object());

                    let present_flag = stats
                        .and_then(|o| o.get("wgpu_metal_current_allocated_size_present"))
                        .and_then(|v| v.as_bool());
                    if let Some(present_flag) = present_flag {
                        present_flag_any = Some(present_flag_any.unwrap_or(false) || present_flag);
                    }

                    let bytes_value = stats
                        .and_then(|o| o.get("wgpu_metal_current_allocated_size_bytes"))
                        .and_then(|v| v.as_u64());
                    if present_flag == Some(true)
                        && let Some(bytes_value) = bytes_value
                    {
                        bytes_min = Some(bytes_min.map_or(bytes_value, |cur| cur.min(bytes_value)));
                        if bytes_max.is_none_or(|cur| bytes_value > cur) {
                            bytes_max = Some(bytes_value);
                            if let (Some(t), Some(f)) = (snap_tick_id, snap_frame_id) {
                                bytes_max_tick_frame = Some((t, f));
                            }
                        }
                    }
                }
            }

            (
                tick_id,
                frame_id,
                present_flag_any,
                bytes_min,
                bytes_max,
                bytes_max_tick_frame,
            )
        } else {
            (None, None, None, None, None, None)
        };

    let missing_reason = if bundle_present {
        "missing_field"
    } else {
        "missing_bundle"
    };

    let mut failures: Vec<serde_json::Value> = Vec::new();
    match (present_flag_any, bytes_max) {
        (Some(true), Some(observed)) if observed > max_wgpu_metal_current_allocated_size_bytes => {
            failures.push(serde_json::json!({
                "kind": "wgpu_metal_current_allocated_size_bytes",
                "threshold": max_wgpu_metal_current_allocated_size_bytes,
                "observed": observed,
                "reason": "exceeded",
            }));
        }
        (Some(true), Some(_)) => {}
        (Some(true), None) => failures.push(serde_json::json!({
            "kind": "wgpu_metal_current_allocated_size_bytes",
            "threshold": max_wgpu_metal_current_allocated_size_bytes,
            "observed": serde_json::Value::Null,
            "reason": missing_reason,
            "field": "windows[0].snapshots[-1].debug.stats.wgpu_metal_current_allocated_size_bytes",
        })),
        (Some(false), _) | (None, _) => failures.push(serde_json::json!({
            "kind": "wgpu_metal_current_allocated_size_bytes",
            "threshold": max_wgpu_metal_current_allocated_size_bytes,
            "observed": serde_json::Value::Null,
            "reason": missing_reason,
            "field": "windows[0].snapshots[-1].debug.stats.wgpu_metal_current_allocated_size_bytes",
        })),
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "wgpu_metal_current_allocated_size_threshold",
        "out_dir": out_dir.display().to_string(),
        "bundle_file": bundle_path
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>"),
        "thresholds": {
            "max_wgpu_metal_current_allocated_size_bytes": max_wgpu_metal_current_allocated_size_bytes,
        },
        "observed": {
            "bundle_present": bundle_present,
            "tick_id": tick_id,
            "frame_id": frame_id,
            "wgpu_metal_current_allocated_size_present": present_flag_any,
            "wgpu_metal_current_allocated_size_bytes_min": bytes_min,
            "wgpu_metal_current_allocated_size_bytes_max": bytes_max,
            "wgpu_metal_current_allocated_size_bytes_max_tick_frame": bytes_max_tick_frame,
        },
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    let failures = payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(WgpuMetalAllocatedSizeGateResult {
        evidence_path: out_path,
        failures,
    })
}

fn linear_allowed_bytes(thr: LinearBytesVsImagesThreshold, images_bytes: u64) -> u64 {
    let LinearBytesVsImagesThreshold {
        intercept_bytes,
        slope_ppm,
        headroom_bytes,
    } = thr;

    let scaled = (images_bytes as u128)
        .saturating_mul(slope_ppm as u128)
        .saturating_add(999_999);
    let scaled = (scaled / 1_000_000).min(u64::MAX as u128) as u64;

    intercept_bytes
        .saturating_add(scaled)
        .saturating_add(headroom_bytes)
}

fn parse_bundle_images_bytes_max_and_last(
    bundle_path: Option<&Path>,
) -> (Option<u64>, Option<u64>, Option<u64>, Option<u64>, bool) {
    let v = bundle_path.and_then(read_json_value);
    let bundle_present = v.is_some();

    let windows = v
        .as_ref()
        .and_then(|v| v.get("windows"))
        .and_then(|v| v.as_array());
    let first_window = windows.and_then(|w| w.first());
    let snapshots = first_window
        .and_then(|w| w.get("snapshots"))
        .and_then(|v| v.as_array());
    let last_snapshot = snapshots.and_then(|s| s.last());

    let tick_id = last_snapshot
        .and_then(|s| s.get("tick_id"))
        .and_then(|v| v.as_u64());
    let frame_id = last_snapshot
        .and_then(|s| s.get("frame_id"))
        .and_then(|v| v.as_u64());

    let mut images_bytes_max: Option<u64> = None;
    let mut images_bytes_last: Option<u64> = None;

    if let Some(snapshots) = snapshots {
        for (ix, snapshot) in snapshots.iter().enumerate() {
            let stats = snapshot
                .get("debug")
                .and_then(|d| d.get("stats"))
                .and_then(|v| v.as_object());
            let v = stats
                .and_then(|o| o.get("renderer_gpu_images_bytes_estimate"))
                .and_then(|v| v.as_u64());
            if let Some(v) = v {
                images_bytes_max = Some(images_bytes_max.map_or(v, |cur| cur.max(v)));
                if ix == snapshots.len().saturating_sub(1) {
                    images_bytes_last = Some(v);
                }
            }
        }
    }

    (
        tick_id,
        frame_id,
        images_bytes_max,
        images_bytes_last,
        bundle_present,
    )
}

pub(super) fn check_macos_owned_unmapped_memory_dirty_bytes_linear_vs_renderer_gpu_images(
    out_dir: &Path,
    footprint_path: &Path,
    bundle_path: Option<&Path>,
    thr: LinearBytesVsImagesThreshold,
) -> Result<LinearBytesVsImagesGateResult, String> {
    let out_path = out_dir.join("check.macos_owned_unmapped_linear_vs_images.json");

    let footprint = read_json_value(footprint_path);
    let footprint_present = footprint.is_some();

    let macos_vmmap_source = if footprint
        .as_ref()
        .and_then(|v| v.get("macos_vmmap_steady"))
        .is_some()
    {
        "steady"
    } else {
        "exit"
    };
    let macos_vmmap_field_prefix = if macos_vmmap_source == "steady" {
        "macos_vmmap_steady"
    } else {
        "macos_vmmap"
    };
    let macos_vmmap = footprint
        .as_ref()
        .and_then(|v| v.get("macos_vmmap_steady").or_else(|| v.get("macos_vmmap")));

    let observed_bytes = macos_vmmap
        .and_then(|v| v.get("regions"))
        .and_then(|v| v.get("owned_unmapped_memory_dirty_bytes"))
        .and_then(|v| v.as_u64());

    let (tick_id, frame_id, images_bytes_max, images_bytes_last, bundle_present) =
        parse_bundle_images_bytes_max_and_last(bundle_path);

    let missing_reason = if !footprint_present || !bundle_present {
        "missing_artifacts"
    } else {
        "missing_fields"
    };

    let mut failures: Vec<serde_json::Value> = Vec::new();
    match (observed_bytes, images_bytes_max) {
        (Some(observed), Some(images)) => {
            let allowed = linear_allowed_bytes(thr, images);
            if observed > allowed {
                failures.push(serde_json::json!({
                    "kind": "macos_owned_unmapped_memory_dirty_bytes_linear_vs_renderer_gpu_images_bytes_estimate",
                    "threshold": {
                        "intercept_bytes": thr.intercept_bytes,
                        "slope_ppm": thr.slope_ppm,
                        "headroom_bytes": thr.headroom_bytes,
                    },
                    "observed": {
                        "owned_unmapped_memory_dirty_bytes": observed,
                        "renderer_gpu_images_bytes_estimate_max": images,
                        "renderer_gpu_images_bytes_estimate_last": images_bytes_last,
                    },
                    "allowed_bytes": allowed,
                    "reason": "exceeded",
                }));
            }
        }
        (None, _) => failures.push(serde_json::json!({
            "kind": "macos_owned_unmapped_memory_dirty_bytes_linear_vs_renderer_gpu_images_bytes_estimate",
            "threshold": {
                "intercept_bytes": thr.intercept_bytes,
                "slope_ppm": thr.slope_ppm,
                "headroom_bytes": thr.headroom_bytes,
            },
            "observed": {
                "owned_unmapped_memory_dirty_bytes": serde_json::Value::Null,
                "renderer_gpu_images_bytes_estimate_max": images_bytes_max,
                "renderer_gpu_images_bytes_estimate_last": images_bytes_last,
            },
            "allowed_bytes": serde_json::Value::Null,
            "reason": missing_reason,
            "field": format!("{macos_vmmap_field_prefix}.regions.owned_unmapped_memory_dirty_bytes"),
        })),
        (_, None) => failures.push(serde_json::json!({
            "kind": "macos_owned_unmapped_memory_dirty_bytes_linear_vs_renderer_gpu_images_bytes_estimate",
            "threshold": {
                "intercept_bytes": thr.intercept_bytes,
                "slope_ppm": thr.slope_ppm,
                "headroom_bytes": thr.headroom_bytes,
            },
            "observed": {
                "owned_unmapped_memory_dirty_bytes": observed_bytes,
                "renderer_gpu_images_bytes_estimate_max": serde_json::Value::Null,
                "renderer_gpu_images_bytes_estimate_last": images_bytes_last,
            },
            "allowed_bytes": serde_json::Value::Null,
            "reason": missing_reason,
            "field": "windows[0].snapshots[-1].debug.stats.renderer_gpu_images_bytes_estimate",
        })),
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "macos_owned_unmapped_memory_dirty_bytes_linear_vs_renderer_gpu_images",
        "out_dir": out_dir.display().to_string(),
        "threshold": {
            "intercept_bytes": thr.intercept_bytes,
            "slope_ppm": thr.slope_ppm,
            "headroom_bytes": thr.headroom_bytes,
        },
        "observed": {
            "footprint_present": footprint_present,
            "bundle_present": bundle_present,
            "tick_id": tick_id,
            "frame_id": frame_id,
            "macos_vmmap_source": macos_vmmap_source,
            "macos_owned_unmapped_memory_dirty_bytes": observed_bytes,
            "renderer_gpu_images_bytes_estimate_max": images_bytes_max,
            "renderer_gpu_images_bytes_estimate_last": images_bytes_last,
            "allowed_bytes_for_images_max": images_bytes_max.map(|x| linear_allowed_bytes(thr, x)),
        },
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    let failures = payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(LinearBytesVsImagesGateResult {
        evidence_path: out_path,
        failures,
    })
}

pub(super) fn check_wgpu_metal_current_allocated_size_bytes_linear_vs_renderer_gpu_images(
    out_dir: &Path,
    bundle_path: Option<&Path>,
    thr: LinearBytesVsImagesThreshold,
) -> Result<LinearBytesVsImagesGateResult, String> {
    let out_path = out_dir.join("check.wgpu_metal_allocated_size_linear_vs_images.json");

    let v = bundle_path.and_then(read_json_value);
    let bundle_present = v.is_some();

    let windows = v
        .as_ref()
        .and_then(|v| v.get("windows"))
        .and_then(|v| v.as_array());
    let first_window = windows.and_then(|w| w.first());
    let snapshots = first_window
        .and_then(|w| w.get("snapshots"))
        .and_then(|v| v.as_array());
    let last_snapshot = snapshots.and_then(|s| s.last());

    let tick_id = last_snapshot
        .and_then(|s| s.get("tick_id"))
        .and_then(|v| v.as_u64());
    let frame_id = last_snapshot
        .and_then(|s| s.get("frame_id"))
        .and_then(|v| v.as_u64());

    let mut present_flag_any: Option<bool> = None;
    let mut bytes_max: Option<u64> = None;
    let mut images_bytes_max: Option<u64> = None;
    let mut images_bytes_last: Option<u64> = None;

    if let Some(snapshots) = snapshots {
        for (ix, snapshot) in snapshots.iter().enumerate() {
            let stats = snapshot
                .get("debug")
                .and_then(|d| d.get("stats"))
                .and_then(|v| v.as_object());

            let present_flag = stats
                .and_then(|o| o.get("wgpu_metal_current_allocated_size_present"))
                .and_then(|v| v.as_bool());
            if let Some(present_flag) = present_flag {
                present_flag_any = Some(present_flag_any.unwrap_or(false) || present_flag);
            }

            let bytes_value = stats
                .and_then(|o| o.get("wgpu_metal_current_allocated_size_bytes"))
                .and_then(|v| v.as_u64());
            if present_flag == Some(true)
                && let Some(bytes_value) = bytes_value
            {
                bytes_max = Some(bytes_max.map_or(bytes_value, |cur| cur.max(bytes_value)));
            }

            let img_value = stats
                .and_then(|o| o.get("renderer_gpu_images_bytes_estimate"))
                .and_then(|v| v.as_u64());
            if let Some(img_value) = img_value {
                images_bytes_max =
                    Some(images_bytes_max.map_or(img_value, |cur| cur.max(img_value)));
                if ix == snapshots.len().saturating_sub(1) {
                    images_bytes_last = Some(img_value);
                }
            }
        }
    }

    let missing_reason = if !bundle_present {
        "missing_artifacts"
    } else {
        "missing_fields"
    };

    let mut failures: Vec<serde_json::Value> = Vec::new();
    match (present_flag_any, bytes_max, images_bytes_max) {
        (Some(true), Some(observed), Some(images)) => {
            let allowed = linear_allowed_bytes(thr, images);
            if observed > allowed {
                failures.push(serde_json::json!({
                    "kind": "wgpu_metal_current_allocated_size_bytes_linear_vs_renderer_gpu_images_bytes_estimate",
                    "threshold": {
                        "intercept_bytes": thr.intercept_bytes,
                        "slope_ppm": thr.slope_ppm,
                        "headroom_bytes": thr.headroom_bytes,
                    },
                    "observed": {
                        "wgpu_metal_current_allocated_size_bytes_max": observed,
                        "renderer_gpu_images_bytes_estimate_max": images,
                        "renderer_gpu_images_bytes_estimate_last": images_bytes_last,
                    },
                    "allowed_bytes": allowed,
                    "reason": "exceeded",
                }));
            }
        }
        (Some(true), None, _) => failures.push(serde_json::json!({
            "kind": "wgpu_metal_current_allocated_size_bytes_linear_vs_renderer_gpu_images_bytes_estimate",
            "threshold": {
                "intercept_bytes": thr.intercept_bytes,
                "slope_ppm": thr.slope_ppm,
                "headroom_bytes": thr.headroom_bytes,
            },
            "observed": {
                "wgpu_metal_current_allocated_size_bytes_max": serde_json::Value::Null,
                "renderer_gpu_images_bytes_estimate_max": images_bytes_max,
                "renderer_gpu_images_bytes_estimate_last": images_bytes_last,
            },
            "allowed_bytes": serde_json::Value::Null,
            "reason": missing_reason,
            "field": "windows[0].snapshots[-1].debug.stats.wgpu_metal_current_allocated_size_bytes",
        })),
        (Some(true), Some(_), None) => failures.push(serde_json::json!({
            "kind": "wgpu_metal_current_allocated_size_bytes_linear_vs_renderer_gpu_images_bytes_estimate",
            "threshold": {
                "intercept_bytes": thr.intercept_bytes,
                "slope_ppm": thr.slope_ppm,
                "headroom_bytes": thr.headroom_bytes,
            },
            "observed": {
                "wgpu_metal_current_allocated_size_bytes_max": bytes_max,
                "renderer_gpu_images_bytes_estimate_max": serde_json::Value::Null,
                "renderer_gpu_images_bytes_estimate_last": images_bytes_last,
            },
            "allowed_bytes": serde_json::Value::Null,
            "reason": missing_reason,
            "field": "windows[0].snapshots[-1].debug.stats.renderer_gpu_images_bytes_estimate",
        })),
        (Some(false), _, _) | (None, _, _) => failures.push(serde_json::json!({
            "kind": "wgpu_metal_current_allocated_size_bytes_linear_vs_renderer_gpu_images_bytes_estimate",
            "threshold": {
                "intercept_bytes": thr.intercept_bytes,
                "slope_ppm": thr.slope_ppm,
                "headroom_bytes": thr.headroom_bytes,
            },
            "observed": {
                "wgpu_metal_current_allocated_size_present": present_flag_any,
                "wgpu_metal_current_allocated_size_bytes_max": bytes_max,
                "renderer_gpu_images_bytes_estimate_max": images_bytes_max,
                "renderer_gpu_images_bytes_estimate_last": images_bytes_last,
            },
            "allowed_bytes": serde_json::Value::Null,
            "reason": missing_reason,
            "field": "windows[0].snapshots[-1].debug.stats.wgpu_metal_current_allocated_size_present",
        })),
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "wgpu_metal_current_allocated_size_bytes_linear_vs_renderer_gpu_images",
        "out_dir": out_dir.display().to_string(),
        "bundle_file": bundle_path
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>"),
        "threshold": {
            "intercept_bytes": thr.intercept_bytes,
            "slope_ppm": thr.slope_ppm,
            "headroom_bytes": thr.headroom_bytes,
        },
        "observed": {
            "bundle_present": bundle_present,
            "tick_id": tick_id,
            "frame_id": frame_id,
            "wgpu_metal_current_allocated_size_present": present_flag_any,
            "wgpu_metal_current_allocated_size_bytes_max": bytes_max,
            "renderer_gpu_images_bytes_estimate_max": images_bytes_max,
            "renderer_gpu_images_bytes_estimate_last": images_bytes_last,
            "allowed_bytes_for_images_max": images_bytes_max.map(|x| linear_allowed_bytes(thr, x)),
        },
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    let failures = payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(LinearBytesVsImagesGateResult {
        evidence_path: out_path,
        failures,
    })
}

pub(super) fn check_wgpu_hub_counts_thresholds(
    out_dir: &Path,
    bundle_path: Option<&Path>,
    thresholds: &WgpuHubCountsThresholds,
) -> Result<WgpuHubCountsGateResult, String> {
    let out_path = out_dir.join("check.wgpu_hub_counts.json");

    let v = bundle_path.and_then(read_json_value);
    let bundle_present = v.is_some();

    let (
        last_tick_id,
        last_frame_id,
        snapshots_len,
        hub_samples,
        buffers_max,
        buffers_max_tick_frame,
        textures_max,
        textures_max_tick_frame,
        render_pipelines_max,
        render_pipelines_max_tick_frame,
        shader_modules_max,
        shader_modules_max_tick_frame,
    ) = if let Some(v) = v.as_ref() {
        let windows = v.get("windows").and_then(|v| v.as_array());
        let first_window = windows.and_then(|w| w.first());
        let snapshots = first_window
            .and_then(|w| w.get("snapshots"))
            .and_then(|v| v.as_array());

        let snapshots_len = snapshots.map(|s| s.len()).unwrap_or(0);

        let mut last_tick_id: Option<u64> = None;
        let mut last_frame_id: Option<u64> = None;

        let mut hub_samples: u64 = 0;

        let mut buffers_max: Option<u64> = None;
        let mut buffers_max_tick_frame: Option<(u64, u64)> = None;
        let mut textures_max: Option<u64> = None;
        let mut textures_max_tick_frame: Option<(u64, u64)> = None;
        let mut render_pipelines_max: Option<u64> = None;
        let mut render_pipelines_max_tick_frame: Option<(u64, u64)> = None;
        let mut shader_modules_max: Option<u64> = None;
        let mut shader_modules_max_tick_frame: Option<(u64, u64)> = None;

        if let Some(snapshots) = snapshots {
            for snapshot in snapshots {
                let snap_tick_id = snapshot.get("tick_id").and_then(|v| v.as_u64());
                let snap_frame_id = snapshot.get("frame_id").and_then(|v| v.as_u64());
                last_tick_id = snap_tick_id;
                last_frame_id = snap_frame_id;

                let stats = snapshot
                    .get("debug")
                    .and_then(|d| d.get("stats"))
                    .and_then(|v| v.as_object());
                let u64_stat = |k: &str| stats.and_then(|o| o.get(k)).and_then(|v| v.as_u64());

                let hub_tick_id = u64_stat("wgpu_hub_tick_id");
                let hub_frame_id = u64_stat("wgpu_hub_frame_id");
                if hub_frame_id.unwrap_or(0) == 0 {
                    continue;
                }
                hub_samples += 1;

                let observed_tick_frame = hub_tick_id
                    .zip(hub_frame_id)
                    .or_else(|| snap_tick_id.zip(snap_frame_id));

                let update_max = |cur: &mut Option<u64>,
                                  cur_tick_frame: &mut Option<(u64, u64)>,
                                  v: Option<u64>| {
                    let Some(v) = v else {
                        return;
                    };
                    if cur.is_none_or(|c| v > c) {
                        *cur = Some(v);
                        *cur_tick_frame = observed_tick_frame;
                    }
                };

                update_max(
                    &mut buffers_max,
                    &mut buffers_max_tick_frame,
                    u64_stat("wgpu_hub_buffers"),
                );
                update_max(
                    &mut textures_max,
                    &mut textures_max_tick_frame,
                    u64_stat("wgpu_hub_textures"),
                );
                update_max(
                    &mut render_pipelines_max,
                    &mut render_pipelines_max_tick_frame,
                    u64_stat("wgpu_hub_render_pipelines"),
                );
                update_max(
                    &mut shader_modules_max,
                    &mut shader_modules_max_tick_frame,
                    u64_stat("wgpu_hub_shader_modules"),
                );
            }
        }

        (
            last_tick_id,
            last_frame_id,
            snapshots_len,
            hub_samples,
            buffers_max,
            buffers_max_tick_frame,
            textures_max,
            textures_max_tick_frame,
            render_pipelines_max,
            render_pipelines_max_tick_frame,
            shader_modules_max,
            shader_modules_max_tick_frame,
        )
    } else {
        (
            None, None, 0, 0, None, None, None, None, None, None, None, None,
        )
    };

    let missing_reason = if bundle_present {
        "missing_field"
    } else {
        "missing_bundle"
    };

    let hub_present = hub_samples > 0;

    let mut failures: Vec<serde_json::Value> = Vec::new();
    let mut check_u64 = |kind: &str, observed: Option<u64>, thr: Option<u64>| {
        let Some(thr) = thr else {
            return;
        };
        match (hub_present, observed) {
            (true, Some(observed)) if observed > thr => failures.push(serde_json::json!({
                "kind": kind,
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            (true, Some(_)) => {}
            (true, None) => failures.push(serde_json::json!({
                "kind": kind,
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": format!("windows[0].snapshots[-1].debug.stats.{kind}"),
            })),
            (false, _) => failures.push(serde_json::json!({
                "kind": kind,
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].debug.stats.wgpu_hub_frame_id",
            })),
        }
    };

    check_u64(
        "wgpu_hub_buffers",
        buffers_max,
        thresholds.max_wgpu_hub_buffers,
    );
    check_u64(
        "wgpu_hub_textures",
        textures_max,
        thresholds.max_wgpu_hub_textures,
    );
    check_u64(
        "wgpu_hub_render_pipelines",
        render_pipelines_max,
        thresholds.max_wgpu_hub_render_pipelines,
    );
    check_u64(
        "wgpu_hub_shader_modules",
        shader_modules_max,
        thresholds.max_wgpu_hub_shader_modules,
    );

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "wgpu_hub_counts_thresholds",
        "out_dir": out_dir.display().to_string(),
        "bundle_file": bundle_path
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>"),
        "thresholds": {
            "max_wgpu_hub_buffers": thresholds.max_wgpu_hub_buffers,
            "max_wgpu_hub_textures": thresholds.max_wgpu_hub_textures,
            "max_wgpu_hub_render_pipelines": thresholds.max_wgpu_hub_render_pipelines,
            "max_wgpu_hub_shader_modules": thresholds.max_wgpu_hub_shader_modules,
        },
        "observed": {
            "bundle_present": bundle_present,
            "tick_id": last_tick_id,
            "frame_id": last_frame_id,
            "snapshots_len": snapshots_len,
            "wgpu_hub_samples": hub_samples,
            "wgpu_hub_present": hub_present,
            "wgpu_hub_buffers_max": buffers_max,
            "wgpu_hub_buffers_max_tick_frame": buffers_max_tick_frame,
            "wgpu_hub_textures_max": textures_max,
            "wgpu_hub_textures_max_tick_frame": textures_max_tick_frame,
            "wgpu_hub_render_pipelines_max": render_pipelines_max,
            "wgpu_hub_render_pipelines_max_tick_frame": render_pipelines_max_tick_frame,
            "wgpu_hub_shader_modules_max": shader_modules_max,
            "wgpu_hub_shader_modules_max_tick_frame": shader_modules_max_tick_frame,
        },
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    let failures = payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(WgpuHubCountsGateResult {
        evidence_path: out_path,
        failures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dir(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.push(format!("fret-diag-{name}-{}-{nonce}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn wgpu_hub_counts_gate_uses_max_across_snapshots() {
        let out_dir = make_temp_dir("wgpu-hub-max");
        let bundle_path = out_dir.join("bundle.json");

        let bundle = serde_json::json!({
            "windows": [{
                "snapshots": [
                    {
                        "tick_id": 1,
                        "frame_id": 10,
                        "debug": { "stats": {
                            "wgpu_hub_tick_id": 1,
                            "wgpu_hub_frame_id": 10,
                            "wgpu_hub_textures": 20
                        }}
                    },
                    {
                        "tick_id": 2,
                        "frame_id": 11,
                        "debug": { "stats": {
                            "wgpu_hub_tick_id": 2,
                            "wgpu_hub_frame_id": 11,
                            "wgpu_hub_textures": 10
                        }}
                    }
                ]
            }]
        });
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap()).unwrap();

        let thresholds = WgpuHubCountsThresholds {
            max_wgpu_hub_buffers: None,
            max_wgpu_hub_textures: Some(15),
            max_wgpu_hub_render_pipelines: None,
            max_wgpu_hub_shader_modules: None,
        };

        let r =
            check_wgpu_hub_counts_thresholds(&out_dir, Some(&bundle_path), &thresholds).unwrap();
        assert_eq!(r.failures, 1);
    }
}

pub(super) fn check_renderer_gpu_budget_thresholds(
    out_dir: &Path,
    bundle_path: Option<&Path>,
    thresholds: &RendererGpuBudgetThresholds,
) -> Result<RendererGpuBudgetsGateResult, String> {
    let out_path = out_dir.join("check.renderer_gpu_budgets.json");

    let v = bundle_path.and_then(read_json_value);
    let bundle_present = v.is_some();

    let (tick_id, frame_id, stats, images_bytes, render_targets_bytes, intermediate_peak_bytes) =
        if let Some(v) = v.as_ref() {
            let windows = v.get("windows").and_then(|v| v.as_array());
            let first_window = windows.and_then(|w| w.first());
            let snapshots = first_window
                .and_then(|w| w.get("snapshots"))
                .and_then(|v| v.as_array());
            let last_snapshot = snapshots.and_then(|s| s.last());
            let stats = last_snapshot
                .and_then(|s| s.get("debug"))
                .and_then(|d| d.get("stats"))
                .and_then(|v| v.as_object());

            let tick_id = last_snapshot
                .and_then(|s| s.get("tick_id"))
                .and_then(|v| v.as_u64());
            let frame_id = last_snapshot
                .and_then(|s| s.get("frame_id"))
                .and_then(|v| v.as_u64());

            let images_bytes = stats
                .and_then(|o| o.get("renderer_gpu_images_bytes_estimate"))
                .and_then(|v| v.as_u64());
            let render_targets_bytes = stats
                .and_then(|o| o.get("renderer_gpu_render_targets_bytes_estimate"))
                .and_then(|v| v.as_u64());
            let intermediate_peak_bytes = stats
                .and_then(|o| o.get("renderer_intermediate_peak_in_use_bytes"))
                .and_then(|v| v.as_u64());

            (
                tick_id,
                frame_id,
                stats.is_some(),
                images_bytes,
                render_targets_bytes,
                intermediate_peak_bytes,
            )
        } else {
            (None, None, false, None, None, None)
        };

    let missing_reason = if bundle_present {
        "missing_field"
    } else {
        "missing_bundle"
    };

    let mut failures: Vec<serde_json::Value> = Vec::new();

    if let Some(thr) = thresholds.max_renderer_gpu_images_bytes_estimate {
        match images_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "renderer_gpu_images_bytes_estimate",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "renderer_gpu_images_bytes_estimate",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].debug.stats.renderer_gpu_images_bytes_estimate",
            })),
        }
    }

    if let Some(thr) = thresholds.max_renderer_gpu_render_targets_bytes_estimate {
        match render_targets_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "renderer_gpu_render_targets_bytes_estimate",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "renderer_gpu_render_targets_bytes_estimate",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].debug.stats.renderer_gpu_render_targets_bytes_estimate",
            })),
        }
    }

    if let Some(thr) = thresholds.max_renderer_intermediate_peak_in_use_bytes {
        match intermediate_peak_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "renderer_intermediate_peak_in_use_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "renderer_intermediate_peak_in_use_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].debug.stats.renderer_intermediate_peak_in_use_bytes",
            })),
        }
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "renderer_gpu_budget_thresholds",
        "out_dir": out_dir.display().to_string(),
        "bundle_file": bundle_path
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>"),
        "thresholds": {
            "max_renderer_gpu_images_bytes_estimate": thresholds.max_renderer_gpu_images_bytes_estimate,
            "max_renderer_gpu_render_targets_bytes_estimate": thresholds.max_renderer_gpu_render_targets_bytes_estimate,
            "max_renderer_intermediate_peak_in_use_bytes": thresholds.max_renderer_intermediate_peak_in_use_bytes,
        },
        "observed": {
            "bundle_present": bundle_present,
            "tick_id": tick_id,
            "frame_id": frame_id,
            "stats_present": stats,
            "renderer_gpu_images_bytes_estimate": images_bytes,
            "renderer_gpu_render_targets_bytes_estimate": render_targets_bytes,
            "renderer_intermediate_peak_in_use_bytes": intermediate_peak_bytes,
        },
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    let failures = payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(RendererGpuBudgetsGateResult {
        evidence_path: out_path,
        failures,
    })
}

pub(super) fn check_render_text_atlas_bytes_live_estimate_total_threshold(
    out_dir: &Path,
    bundle_path: Option<&Path>,
    max_render_text_atlas_bytes_live_estimate_total: u64,
) -> Result<RenderTextAtlasBytesGateResult, String> {
    let out_path = out_dir.join("check.render_text_atlas_bytes.json");

    let v = bundle_path.and_then(read_json_value);
    let bundle_present = v.is_some();

    let (tick_id, frame_id, present_flag, mask_bytes, color_bytes, subpixel_bytes, total_bytes) =
        if let Some(v) = v.as_ref() {
            let windows = v.get("windows").and_then(|v| v.as_array());
            let first_window = windows.and_then(|w| w.first());
            let snapshots = first_window
                .and_then(|w| w.get("snapshots"))
                .and_then(|v| v.as_array());
            let last_snapshot = snapshots.and_then(|s| s.last());

            let tick_id = last_snapshot
                .and_then(|s| s.get("tick_id"))
                .and_then(|v| v.as_u64());
            let frame_id = last_snapshot
                .and_then(|s| s.get("frame_id"))
                .and_then(|v| v.as_u64());

            let render_text = last_snapshot
                .and_then(|s| s.get("resource_caches"))
                .and_then(|v| v.get("render_text"))
                .and_then(|v| v.as_object());

            let rt_atlas = |k: &str| {
                render_text
                    .and_then(|o| o.get(k))
                    .and_then(|v| v.as_object())
            };
            let atlas_u64 = |atlas: Option<&serde_json::Map<String, serde_json::Value>>,
                             k: &str| {
                atlas.and_then(|o| o.get(k)).and_then(|v| v.as_u64())
            };
            let sat_mul_u64 = |a: u64, b: u64| -> u64 {
                ((a as u128) * (b as u128)).min(u64::MAX as u128) as u64
            };
            let atlas_bytes = |atlas: Option<&serde_json::Map<String, serde_json::Value>>,
                               bpp: u64|
             -> Option<u64> {
                let w = atlas_u64(atlas, "width")?;
                let h = atlas_u64(atlas, "height")?;
                let pages = atlas_u64(atlas, "pages")?;
                Some(sat_mul_u64(sat_mul_u64(sat_mul_u64(w, h), pages), bpp))
            };

            let mask_atlas = rt_atlas("mask_atlas");
            let color_atlas = rt_atlas("color_atlas");
            let subpixel_atlas = rt_atlas("subpixel_atlas");

            let mask_bytes = atlas_bytes(mask_atlas, 1);
            let color_bytes = atlas_bytes(color_atlas, 4);
            let subpixel_bytes = atlas_bytes(subpixel_atlas, 4);
            let total_bytes = match (mask_bytes, color_bytes, subpixel_bytes) {
                (Some(a), Some(b), Some(c)) => Some(a.saturating_add(b).saturating_add(c)),
                _ => None,
            };

            (
                tick_id,
                frame_id,
                Some(render_text.is_some()),
                mask_bytes,
                color_bytes,
                subpixel_bytes,
                total_bytes,
            )
        } else {
            (None, None, None, None, None, None, None)
        };

    let missing_reason = if bundle_present {
        "missing_field"
    } else {
        "missing_bundle"
    };

    let mut failures: Vec<serde_json::Value> = Vec::new();
    match (present_flag, total_bytes) {
        (Some(true), Some(observed)) if observed > max_render_text_atlas_bytes_live_estimate_total => {
            failures.push(serde_json::json!({
                "kind": "render_text_atlas_bytes_live_estimate_total",
                "threshold": max_render_text_atlas_bytes_live_estimate_total,
                "observed": observed,
                "reason": "exceeded",
            }));
        }
        (Some(true), Some(_)) => {}
        (Some(true), None) => failures.push(serde_json::json!({
            "kind": "render_text_atlas_bytes_live_estimate_total",
            "threshold": max_render_text_atlas_bytes_live_estimate_total,
            "observed": serde_json::Value::Null,
            "reason": missing_reason,
            "field": "windows[0].snapshots[-1].resource_caches.render_text.*_atlas.{width,height,pages}",
        })),
        (Some(false), _) | (None, _) => failures.push(serde_json::json!({
            "kind": "render_text_atlas_bytes_live_estimate_total",
            "threshold": max_render_text_atlas_bytes_live_estimate_total,
            "observed": serde_json::Value::Null,
            "reason": missing_reason,
            "field": "windows[0].snapshots[-1].resource_caches.render_text",
        })),
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "render_text_atlas_bytes_threshold",
        "out_dir": out_dir.display().to_string(),
        "bundle_file": bundle_path
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>"),
        "thresholds": {
            "max_render_text_atlas_bytes_live_estimate_total": max_render_text_atlas_bytes_live_estimate_total,
        },
        "observed": {
            "bundle_present": bundle_present,
            "tick_id": tick_id,
            "frame_id": frame_id,
            "render_text_present": present_flag,
            "render_text_mask_atlas_bytes_live_estimate": mask_bytes,
            "render_text_color_atlas_bytes_live_estimate": color_bytes,
            "render_text_subpixel_atlas_bytes_live_estimate": subpixel_bytes,
            "render_text_atlas_bytes_live_estimate_total": total_bytes,
        },
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    let failures = payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(RenderTextAtlasBytesGateResult {
        evidence_path: out_path,
        failures,
    })
}

pub(super) fn check_render_text_font_db_thresholds(
    out_dir: &Path,
    bundle_path: Option<&Path>,
    thresholds: &RenderTextFontDbThresholds,
) -> Result<RenderTextFontDbGateResult, String> {
    let out_path = out_dir.join("check.render_text_font_db.json");

    let v = bundle_path.and_then(read_json_value);
    let bundle_present = v.is_some();

    let (tick_id, frame_id, render_text) = if let Some(v) = v.as_ref() {
        let windows = v.get("windows").and_then(|v| v.as_array());
        let first_window = windows.and_then(|w| w.first());
        let snapshots = first_window
            .and_then(|w| w.get("snapshots"))
            .and_then(|v| v.as_array());
        let last_snapshot = snapshots.and_then(|s| s.last());

        let tick_id = last_snapshot
            .and_then(|s| s.get("tick_id"))
            .and_then(|v| v.as_u64());
        let frame_id = last_snapshot
            .and_then(|s| s.get("frame_id"))
            .and_then(|v| v.as_u64());

        let render_text = last_snapshot
            .and_then(|s| s.get("resource_caches"))
            .and_then(|v| v.get("render_text"))
            .and_then(|v| v.as_object());

        (tick_id, frame_id, render_text)
    } else {
        (None, None, None)
    };

    let missing_reason = if bundle_present {
        "missing_field"
    } else {
        "missing_bundle"
    };

    let u64_field = |k: &str| -> Option<u64> {
        render_text
            .as_ref()
            .and_then(|o| o.get(k))
            .and_then(|v| v.as_u64())
    };

    let registered_font_blobs_total_bytes = u64_field("registered_font_blobs_total_bytes");
    let registered_font_blobs_count = u64_field("registered_font_blobs_count");
    let shape_cache_entries = u64_field("shape_cache_entries");
    let blob_cache_entries = u64_field("blob_cache_entries");
    let shape_cache_bytes_estimate_total = u64_field("shape_cache_bytes_estimate_total");
    let blob_paint_palette_bytes_estimate_total =
        u64_field("blob_paint_palette_bytes_estimate_total");
    let blob_decorations_bytes_estimate_total = u64_field("blob_decorations_bytes_estimate_total");

    let mut failures: Vec<serde_json::Value> = Vec::new();

    if let Some(thr) = thresholds.max_render_text_registered_font_blobs_total_bytes {
        match registered_font_blobs_total_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "render_text_registered_font_blobs_total_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "render_text_registered_font_blobs_total_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].resource_caches.render_text.registered_font_blobs_total_bytes",
            })),
        }
    }

    if let Some(thr) = thresholds.max_render_text_registered_font_blobs_count {
        match registered_font_blobs_count {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "render_text_registered_font_blobs_count",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "render_text_registered_font_blobs_count",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].resource_caches.render_text.registered_font_blobs_count",
            })),
        }
    }

    if let Some(thr) = thresholds.max_render_text_shape_cache_entries {
        match shape_cache_entries {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "render_text_shape_cache_entries",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "render_text_shape_cache_entries",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].resource_caches.render_text.shape_cache_entries",
            })),
        }
    }

    if let Some(thr) = thresholds.max_render_text_blob_cache_entries {
        match blob_cache_entries {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "render_text_blob_cache_entries",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "render_text_blob_cache_entries",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].resource_caches.render_text.blob_cache_entries",
            })),
        }
    }

    if let Some(thr) = thresholds.max_render_text_shape_cache_bytes_estimate_total {
        match shape_cache_bytes_estimate_total {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "render_text_shape_cache_bytes_estimate_total",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "render_text_shape_cache_bytes_estimate_total",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].resource_caches.render_text.shape_cache_bytes_estimate_total",
            })),
        }
    }

    if let Some(thr) = thresholds.max_render_text_blob_paint_palette_bytes_estimate_total {
        match blob_paint_palette_bytes_estimate_total {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "render_text_blob_paint_palette_bytes_estimate_total",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "render_text_blob_paint_palette_bytes_estimate_total",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].resource_caches.render_text.blob_paint_palette_bytes_estimate_total",
            })),
        }
    }

    if let Some(thr) = thresholds.max_render_text_blob_decorations_bytes_estimate_total {
        match blob_decorations_bytes_estimate_total {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "render_text_blob_decorations_bytes_estimate_total",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "render_text_blob_decorations_bytes_estimate_total",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].resource_caches.render_text.blob_decorations_bytes_estimate_total",
            })),
        }
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "render_text_font_db_thresholds",
        "out_dir": out_dir.display().to_string(),
        "bundle_file": bundle_path
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>"),
        "thresholds": {
            "max_render_text_registered_font_blobs_total_bytes": thresholds.max_render_text_registered_font_blobs_total_bytes,
            "max_render_text_registered_font_blobs_count": thresholds.max_render_text_registered_font_blobs_count,
            "max_render_text_shape_cache_entries": thresholds.max_render_text_shape_cache_entries,
            "max_render_text_blob_cache_entries": thresholds.max_render_text_blob_cache_entries,
            "max_render_text_shape_cache_bytes_estimate_total": thresholds.max_render_text_shape_cache_bytes_estimate_total,
            "max_render_text_blob_paint_palette_bytes_estimate_total": thresholds.max_render_text_blob_paint_palette_bytes_estimate_total,
            "max_render_text_blob_decorations_bytes_estimate_total": thresholds.max_render_text_blob_decorations_bytes_estimate_total,
        },
        "observed": {
            "bundle_present": bundle_present,
            "tick_id": tick_id,
            "frame_id": frame_id,
            "render_text_present": render_text.is_some(),
            "render_text_registered_font_blobs_total_bytes": registered_font_blobs_total_bytes,
            "render_text_registered_font_blobs_count": registered_font_blobs_count,
            "render_text_shape_cache_entries": shape_cache_entries,
            "render_text_blob_cache_entries": blob_cache_entries,
            "render_text_shape_cache_bytes_estimate_total": shape_cache_bytes_estimate_total,
            "render_text_blob_paint_palette_bytes_estimate_total": blob_paint_palette_bytes_estimate_total,
            "render_text_blob_decorations_bytes_estimate_total": blob_decorations_bytes_estimate_total,
        },
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    let failures = payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(RenderTextFontDbGateResult {
        evidence_path: out_path,
        failures,
    })
}

pub(super) fn check_code_editor_memory_thresholds(
    out_dir: &Path,
    bundle_path: Option<&Path>,
    thresholds: &CodeEditorMemoryThresholds,
) -> Result<CodeEditorMemoryGateResult, String> {
    let out_path = out_dir.join("check.code_editor_memory.json");

    let v = bundle_path.and_then(read_json_value);
    let bundle_present = v.is_some();

    let (tick_id, frame_id, mem, cache_sizes) = if let Some(v) = v.as_ref() {
        let windows = v.get("windows").and_then(|v| v.as_array());
        let first_window = windows.and_then(|w| w.first());
        let snapshots = first_window
            .and_then(|w| w.get("snapshots"))
            .and_then(|v| v.as_array());
        let last_snapshot = snapshots.and_then(|s| s.last());

        let tick_id = last_snapshot
            .and_then(|s| s.get("tick_id"))
            .and_then(|v| v.as_u64());
        let frame_id = last_snapshot
            .and_then(|s| s.get("frame_id"))
            .and_then(|v| v.as_u64());

        let torture = last_snapshot
            .and_then(|s| s.get("app_snapshot"))
            .and_then(|v| v.get("code_editor"))
            .and_then(|v| v.get("torture"));

        let mem = torture
            .and_then(|v| v.get("memory"))
            .and_then(|v| v.as_object());
        let cache_sizes = torture
            .and_then(|v| v.get("cache_sizes"))
            .and_then(|v| v.as_object());

        (tick_id, frame_id, mem, cache_sizes)
    } else {
        (None, None, None, None)
    };

    let missing_reason = if bundle_present {
        "missing_field"
    } else {
        "missing_bundle"
    };

    let buffer_len_bytes = mem
        .as_ref()
        .and_then(|o| o.get("buffer_len_bytes"))
        .and_then(|v| v.as_u64());
    let undo_text_bytes_estimate_total = mem
        .as_ref()
        .and_then(|o| o.get("undo_text_bytes_estimate_total"))
        .and_then(|v| v.as_u64());
    let row_text_cache_entries = cache_sizes
        .as_ref()
        .and_then(|o| o.get("row_text_cache_entries"))
        .and_then(|v| v.as_u64());
    let row_rich_cache_entries = cache_sizes
        .as_ref()
        .and_then(|o| o.get("row_rich_cache_entries"))
        .and_then(|v| v.as_u64());

    let mut failures: Vec<serde_json::Value> = Vec::new();

    if let Some(thr) = thresholds.max_code_editor_buffer_len_bytes {
        match buffer_len_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "code_editor_buffer_len_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "code_editor_buffer_len_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].app_snapshot.code_editor.torture.memory.buffer_len_bytes",
            })),
        }
    }

    if let Some(thr) = thresholds.max_code_editor_undo_text_bytes_estimate_total {
        match undo_text_bytes_estimate_total {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "code_editor_undo_text_bytes_estimate_total",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "code_editor_undo_text_bytes_estimate_total",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].app_snapshot.code_editor.torture.memory.undo_text_bytes_estimate_total",
            })),
        }
    }

    if let Some(thr) = thresholds.max_code_editor_row_text_cache_entries {
        match row_text_cache_entries {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "code_editor_row_text_cache_entries",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "code_editor_row_text_cache_entries",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].app_snapshot.code_editor.torture.cache_sizes.row_text_cache_entries",
            })),
        }
    }

    if let Some(thr) = thresholds.max_code_editor_row_rich_cache_entries {
        match row_rich_cache_entries {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "code_editor_row_rich_cache_entries",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "code_editor_row_rich_cache_entries",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "windows[0].snapshots[-1].app_snapshot.code_editor.torture.cache_sizes.row_rich_cache_entries",
            })),
        }
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "code_editor_memory_thresholds",
        "out_dir": out_dir.display().to_string(),
        "bundle_file": bundle_path
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>"),
        "thresholds": {
            "max_code_editor_buffer_len_bytes": thresholds.max_code_editor_buffer_len_bytes,
            "max_code_editor_undo_text_bytes_estimate_total": thresholds.max_code_editor_undo_text_bytes_estimate_total,
            "max_code_editor_row_text_cache_entries": thresholds.max_code_editor_row_text_cache_entries,
            "max_code_editor_row_rich_cache_entries": thresholds.max_code_editor_row_rich_cache_entries,
        },
        "observed": {
            "bundle_present": bundle_present,
            "tick_id": tick_id,
            "frame_id": frame_id,
            "memory_present": mem.is_some(),
            "cache_sizes_present": cache_sizes.is_some(),
            "code_editor_buffer_len_bytes": buffer_len_bytes,
            "code_editor_undo_text_bytes_estimate_total": undo_text_bytes_estimate_total,
            "code_editor_row_text_cache_entries": row_text_cache_entries,
            "code_editor_row_rich_cache_entries": row_rich_cache_entries,
        },
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    let failures = payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(CodeEditorMemoryGateResult {
        evidence_path: out_path,
        failures,
    })
}

pub(super) fn check_resource_footprint_thresholds(
    out_dir: &Path,
    footprint_path: &Path,
    thresholds: &ResourceFootprintThresholds,
) -> Result<ResourceFootprintGateResult, String> {
    let out_path = out_dir.join("check.resource_footprint.json");
    let v = read_json_value(footprint_path);
    let footprint_present = v.is_some();

    let pid = v
        .as_ref()
        .and_then(|v| v.get("pid"))
        .and_then(|v| v.as_u64());
    let killed = v
        .as_ref()
        .and_then(|v| v.get("killed"))
        .and_then(|v| v.as_bool());
    let wall_time_ms = v
        .as_ref()
        .and_then(|v| v.get("wall_time_ms"))
        .and_then(|v| v.as_u64());
    let logical_cores = v
        .as_ref()
        .and_then(|v| v.get("logical_cores"))
        .and_then(|v| v.as_u64());
    let note = v
        .as_ref()
        .and_then(|v| v.get("note"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let working_set_bytes = v
        .as_ref()
        .and_then(|v| v.get("memory"))
        .and_then(|v| v.get("working_set_bytes"))
        .and_then(|v| v.as_u64());
    let peak_working_set_bytes = v
        .as_ref()
        .and_then(|v| v.get("memory"))
        .and_then(|v| v.get("peak_working_set_bytes"))
        .and_then(|v| v.as_u64());

    let macos_vmmap_source = if v
        .as_ref()
        .and_then(|v| v.get("macos_vmmap_steady"))
        .is_some()
    {
        "steady"
    } else {
        "exit"
    };
    let macos_vmmap_field_prefix = if macos_vmmap_source == "steady" {
        "macos_vmmap_steady"
    } else {
        "macos_vmmap"
    };
    let macos_vmmap = v
        .as_ref()
        .and_then(|v| v.get("macos_vmmap_steady").or_else(|| v.get("macos_vmmap")));

    let macos_physical_footprint_peak_bytes = macos_vmmap
        .and_then(|v| v.get("physical_footprint_peak_bytes"))
        .and_then(|v| v.as_u64());
    let macos_owned_unmapped_memory_dirty_bytes = macos_vmmap
        .and_then(|v| v.get("regions"))
        .and_then(|v| v.get("owned_unmapped_memory_dirty_bytes"))
        .and_then(|v| v.as_u64());
    let macos_io_surface_dirty_bytes = macos_vmmap
        .and_then(|v| v.get("regions"))
        .and_then(|v| v.get("io_surface_dirty_bytes"))
        .and_then(|v| v.as_u64());
    let macos_io_accelerator_dirty_bytes = macos_vmmap
        .and_then(|v| v.get("regions"))
        .and_then(|v| v.get("io_accelerator_dirty_bytes"))
        .and_then(|v| v.as_u64());
    let macos_malloc_small_dirty_bytes = macos_vmmap
        .and_then(|v| v.get("regions"))
        .and_then(|v| v.get("malloc_small_dirty_bytes"))
        .and_then(|v| v.as_u64());
    let macos_malloc_dirty_bytes_total = macos_vmmap
        .and_then(|v| v.get("regions"))
        .and_then(|v| v.get("malloc_dirty_bytes_total"))
        .and_then(|v| v.as_u64());

    let macos_malloc_zones_total_allocated_bytes = macos_vmmap
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.get("malloc_zones"))
        .and_then(|v| v.get("total"))
        .and_then(|v| v.get("allocated_bytes"))
        .and_then(|v| v.as_u64());
    let macos_malloc_zones_total_frag_bytes = macos_vmmap
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.get("malloc_zones"))
        .and_then(|v| v.get("total"))
        .and_then(|v| v.get("frag_bytes"))
        .and_then(|v| v.as_u64());
    let macos_malloc_zones_total_dirty_bytes = macos_vmmap
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.get("malloc_zones"))
        .and_then(|v| v.get("total"))
        .and_then(|v| v.get("dirty_bytes"))
        .and_then(|v| v.as_u64());

    let cpu_avg_percent_total_cores = v
        .as_ref()
        .and_then(|v| v.get("cpu"))
        .and_then(|v| v.get("avg_cpu_percent_total_cores"))
        .and_then(|v| v.as_f64());
    let cpu_samples = v
        .as_ref()
        .and_then(|v| v.get("cpu"))
        .and_then(|v| v.get("samples"))
        .and_then(|v| v.as_u64());
    let cpu_collector = v
        .as_ref()
        .and_then(|v| v.get("cpu"))
        .and_then(|v| v.get("collector"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut failures: Vec<serde_json::Value> = Vec::new();
    let missing_reason = if footprint_present {
        "missing_field"
    } else {
        "missing_footprint"
    };

    if let Some(thr) = thresholds.max_working_set_bytes {
        match working_set_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "working_set_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "working_set_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "memory.working_set_bytes",
            })),
        }
    }

    if let Some(thr) = thresholds.max_peak_working_set_bytes {
        match peak_working_set_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "peak_working_set_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "peak_working_set_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "memory.peak_working_set_bytes",
            })),
        }
    }

    if let Some(thr) = thresholds.max_macos_physical_footprint_peak_bytes {
        match macos_physical_footprint_peak_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "macos_physical_footprint_peak_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "macos_physical_footprint_peak_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": format!("{macos_vmmap_field_prefix}.physical_footprint_peak_bytes"),
            })),
        }
    }

    if let Some(thr) = thresholds.max_macos_owned_unmapped_memory_dirty_bytes {
        match macos_owned_unmapped_memory_dirty_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "macos_owned_unmapped_memory_dirty_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "macos_owned_unmapped_memory_dirty_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": format!("{macos_vmmap_field_prefix}.regions.owned_unmapped_memory_dirty_bytes"),
            })),
        }
    }

    if let Some(thr) = thresholds.max_macos_io_surface_dirty_bytes {
        match macos_io_surface_dirty_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "macos_io_surface_dirty_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "macos_io_surface_dirty_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": format!("{macos_vmmap_field_prefix}.regions.io_surface_dirty_bytes"),
            })),
        }
    }

    if let Some(thr) = thresholds.max_macos_io_accelerator_dirty_bytes {
        match macos_io_accelerator_dirty_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "macos_io_accelerator_dirty_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "macos_io_accelerator_dirty_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": format!("{macos_vmmap_field_prefix}.regions.io_accelerator_dirty_bytes"),
            })),
        }
    }

    if let Some(thr) = thresholds.max_macos_malloc_small_dirty_bytes {
        match macos_malloc_small_dirty_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "macos_malloc_small_dirty_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "macos_malloc_small_dirty_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": format!("{macos_vmmap_field_prefix}.regions.malloc_small_dirty_bytes"),
            })),
        }
    }

    if let Some(thr) = thresholds.max_macos_malloc_dirty_bytes_total {
        match macos_malloc_dirty_bytes_total {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "macos_malloc_dirty_bytes_total",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "macos_malloc_dirty_bytes_total",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": format!("{macos_vmmap_field_prefix}.regions.malloc_dirty_bytes_total"),
            })),
        }
    }

    if let Some(thr) = thresholds.max_macos_malloc_zones_total_allocated_bytes {
        match macos_malloc_zones_total_allocated_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "macos_malloc_zones_total_allocated_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "macos_malloc_zones_total_allocated_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": format!("{macos_vmmap_field_prefix}.tables.malloc_zones.total.allocated_bytes"),
            })),
        }
    }

    if let Some(thr) = thresholds.max_macos_malloc_zones_total_frag_bytes {
        match macos_malloc_zones_total_frag_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "macos_malloc_zones_total_frag_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "macos_malloc_zones_total_frag_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": format!("{macos_vmmap_field_prefix}.tables.malloc_zones.total.frag_bytes"),
            })),
        }
    }

    if let Some(thr) = thresholds.max_macos_malloc_zones_total_dirty_bytes {
        match macos_malloc_zones_total_dirty_bytes {
            Some(observed) if observed > thr => failures.push(serde_json::json!({
                "kind": "macos_malloc_zones_total_dirty_bytes",
                "threshold": thr,
                "observed": observed,
                "reason": "exceeded",
            })),
            Some(_) => {}
            None => failures.push(serde_json::json!({
                "kind": "macos_malloc_zones_total_dirty_bytes",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": format!("{macos_vmmap_field_prefix}.tables.malloc_zones.total.dirty_bytes"),
            })),
        }
    }

    if let Some(thr) = thresholds.max_cpu_avg_percent_total_cores {
        match cpu_avg_percent_total_cores {
            Some(observed) => {
                if cpu_samples == Some(0) && cpu_collector.as_deref() == Some("sysinfo") {
                    failures.push(serde_json::json!({
                        "kind": "cpu_avg_percent_total_cores",
                        "threshold": thr,
                        "observed": observed,
                        "reason": "insufficient_samples",
                        "samples": cpu_samples,
                    }));
                } else if observed > thr {
                    failures.push(serde_json::json!({
                        "kind": "cpu_avg_percent_total_cores",
                        "threshold": thr,
                        "observed": observed,
                        "reason": "exceeded",
                    }));
                }
            }
            None => failures.push(serde_json::json!({
                "kind": "cpu_avg_percent_total_cores",
                "threshold": thr,
                "observed": serde_json::Value::Null,
                "reason": missing_reason,
                "field": "cpu.avg_cpu_percent_total_cores",
            })),
        }
    }

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "resource_footprint_thresholds",
        "out_dir": out_dir.display().to_string(),
        "footprint_file": footprint_path.file_name().and_then(|s| s.to_str()).unwrap_or("resource.footprint.json"),
        "thresholds": {
            "max_working_set_bytes": thresholds.max_working_set_bytes,
            "max_peak_working_set_bytes": thresholds.max_peak_working_set_bytes,
            "max_macos_physical_footprint_peak_bytes": thresholds.max_macos_physical_footprint_peak_bytes,
            "max_macos_owned_unmapped_memory_dirty_bytes": thresholds.max_macos_owned_unmapped_memory_dirty_bytes,
            "max_macos_io_surface_dirty_bytes": thresholds.max_macos_io_surface_dirty_bytes,
            "max_macos_io_accelerator_dirty_bytes": thresholds.max_macos_io_accelerator_dirty_bytes,
            "max_macos_malloc_small_dirty_bytes": thresholds.max_macos_malloc_small_dirty_bytes,
            "max_macos_malloc_dirty_bytes_total": thresholds.max_macos_malloc_dirty_bytes_total,
            "max_macos_malloc_zones_total_allocated_bytes": thresholds.max_macos_malloc_zones_total_allocated_bytes,
            "max_macos_malloc_zones_total_frag_bytes": thresholds.max_macos_malloc_zones_total_frag_bytes,
            "max_macos_malloc_zones_total_dirty_bytes": thresholds.max_macos_malloc_zones_total_dirty_bytes,
            "max_cpu_avg_percent_total_cores": thresholds.max_cpu_avg_percent_total_cores,
        },
        "observed": {
            "present": footprint_present,
            "pid": pid,
            "killed": killed,
            "wall_time_ms": wall_time_ms,
            "logical_cores": logical_cores,
            "note": note,
            "cpu_collector": cpu_collector,
            "cpu_samples": cpu_samples,
            "cpu_avg_percent_total_cores": cpu_avg_percent_total_cores,
            "working_set_bytes": working_set_bytes,
            "peak_working_set_bytes": peak_working_set_bytes,
            "macos_vmmap_source": macos_vmmap_source,
            "macos_physical_footprint_peak_bytes": macos_physical_footprint_peak_bytes,
            "macos_owned_unmapped_memory_dirty_bytes": macos_owned_unmapped_memory_dirty_bytes,
            "macos_io_surface_dirty_bytes": macos_io_surface_dirty_bytes,
            "macos_io_accelerator_dirty_bytes": macos_io_accelerator_dirty_bytes,
            "macos_malloc_small_dirty_bytes": macos_malloc_small_dirty_bytes,
            "macos_malloc_dirty_bytes_total": macos_malloc_dirty_bytes_total,
            "macos_malloc_zones_total_allocated_bytes": macos_malloc_zones_total_allocated_bytes,
            "macos_malloc_zones_total_frag_bytes": macos_malloc_zones_total_frag_bytes,
            "macos_malloc_zones_total_dirty_bytes": macos_malloc_zones_total_dirty_bytes,
        },
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    let failures = payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(ResourceFootprintGateResult {
        evidence_path: out_path,
        failures,
    })
}

#[derive(Debug, Clone)]
pub(super) struct RedrawHitchesGateResult {
    pub(super) evidence_path: PathBuf,
    pub(super) failures: usize,
}

#[derive(Debug, Clone)]
struct RedrawHitchRecord {
    line_no: usize,
    ts_unix_ms: Option<u64>,
    tick_id: Option<u64>,
    frame_id: Option<u64>,
    total_ms: u64,
    prepare_ms: Option<u64>,
    render_ms: Option<u64>,
    record_ms: Option<u64>,
    present_ms: Option<u64>,
    scene_ops: Option<u64>,
    line: String,
}

pub(super) fn check_redraw_hitches_max_total_ms(
    out_dir: &Path,
    max_total_ms: u64,
) -> Result<RedrawHitchesGateResult, String> {
    let log_path = out_dir.join("redraw_hitches.log");
    let out_path = out_dir.join("check.redraw_hitches.json");

    let parse_u64_after = |s: &str, needle: &str| -> Option<u64> {
        let start = s.find(needle)? + needle.len();
        let bytes = s.as_bytes();
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end == start {
            return None;
        }
        s.get(start..end)?.parse::<u64>().ok()
    };

    let parse_opt_u64_dbg = |s: &str, key: &str| -> Option<u64> {
        let needle = format!("{key}=Some(");
        let start = s.find(&needle)? + needle.len();
        let bytes = s.as_bytes();
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end == start {
            return None;
        }
        s.get(start..end)?.parse::<u64>().ok()
    };

    let parse_ts = |s: &str| -> Option<u64> {
        let s = s.strip_prefix('[')?;
        let end = s.find(']')?;
        s.get(0..end)?.parse::<u64>().ok()
    };

    let truncate = |s: &str, max_chars: usize| -> String {
        if s.chars().count() <= max_chars {
            return s.to_string();
        }
        s.chars().take(max_chars).collect()
    };

    let contents = std::fs::read_to_string(&log_path).ok();
    let present = contents.is_some();

    let mut records: Vec<RedrawHitchRecord> = Vec::new();
    if let Some(contents) = contents.as_ref() {
        for (idx, line) in contents.lines().enumerate() {
            let Some(total_ms) = parse_u64_after(line, "total_ms=") else {
                continue;
            };
            records.push(RedrawHitchRecord {
                line_no: idx.saturating_add(1),
                ts_unix_ms: parse_ts(line),
                tick_id: parse_u64_after(line, "tick_id="),
                frame_id: parse_u64_after(line, "frame_id="),
                total_ms,
                prepare_ms: parse_opt_u64_dbg(line, "prepare_ms"),
                render_ms: parse_opt_u64_dbg(line, "render_ms"),
                record_ms: parse_opt_u64_dbg(line, "record_ms"),
                present_ms: parse_opt_u64_dbg(line, "present_ms"),
                scene_ops: parse_u64_after(line, "scene_ops="),
                line: truncate(line, 400),
            });
        }
    }

    let mut totals: Vec<u64> = records.iter().map(|r| r.total_ms).collect();
    totals.sort_unstable();

    let max_observed = totals.last().copied();
    let avg_observed = if totals.is_empty() {
        None
    } else {
        Some(totals.iter().sum::<u64>() as f64 / totals.len() as f64)
    };
    let p95_observed = if totals.is_empty() {
        None
    } else {
        let idx = (totals.len().saturating_sub(1)) * 95 / 100;
        totals.get(idx).copied()
    };

    let mut failures: Vec<serde_json::Value> = Vec::new();
    if !present {
        failures.push(serde_json::json!({
            "kind": "log_file",
            "reason": "missing_log",
            "file": log_path.file_name().and_then(|s| s.to_str()).unwrap_or("redraw_hitches.log"),
        }));
    } else if records.is_empty() {
        failures.push(serde_json::json!({
            "kind": "parse",
            "reason": "no_records",
            "field": "total_ms",
        }));
    }

    if let Some(observed) = max_observed
        && observed > max_total_ms
    {
        failures.push(serde_json::json!({
            "kind": "max_total_ms",
            "threshold": max_total_ms,
            "observed": observed,
            "reason": "exceeded",
        }));
    }

    records.sort_by(|a, b| b.total_ms.cmp(&a.total_ms));
    let top = records
        .iter()
        .take(10)
        .map(|r| {
            serde_json::json!({
                "line_no": r.line_no,
                "ts_unix_ms": r.ts_unix_ms,
                "tick_id": r.tick_id,
                "frame_id": r.frame_id,
                "total_ms": r.total_ms,
                "prepare_ms": r.prepare_ms,
                "render_ms": r.render_ms,
                "record_ms": r.record_ms,
                "present_ms": r.present_ms,
                "scene_ops": r.scene_ops,
                "line": r.line,
            })
        })
        .collect::<Vec<_>>();

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "redraw_hitches_thresholds",
        "out_dir": out_dir.display().to_string(),
        "log_file": log_path.file_name().and_then(|s| s.to_str()).unwrap_or("redraw_hitches.log"),
        "thresholds": {
            "max_total_ms": max_total_ms,
        },
        "observed": {
            "present": present,
            "records": totals.len(),
            "max_total_ms": max_observed,
            "p95_total_ms": p95_observed,
            "avg_total_ms": avg_observed,
        },
        "top": top,
        "failures": failures,
    });
    let _ = write_json_value(&out_path, &payload);

    let failures = payload
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(RedrawHitchesGateResult {
        evidence_path: out_path,
        failures,
    })
}
