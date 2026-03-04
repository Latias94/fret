use std::path::{Path, PathBuf};

fn metadata_mtime_unix_ms(meta: &std::fs::Metadata) -> Option<u64> {
    let modified = meta.modified().ok()?;
    let dur = modified
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .ok()?;
    Some(dur.as_millis().min(u64::MAX as u128) as u64)
}

fn json_file_summary(path: &Path) -> Option<serde_json::Value> {
    let v = crate::read_json_value(path)?;
    let schema_version = v.get("schema_version").and_then(|v| v.as_u64());
    let kind = v
        .get("kind")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let failures_len = v
        .get("failures")
        .and_then(|v| v.as_array())
        .map(|a| a.len() as u64);
    let ok = failures_len.map(|n| n == 0);

    Some(serde_json::json!({
        "schema_version": schema_version,
        "kind": kind,
        "ok": ok,
        "failures_len": failures_len,
    }))
}

fn resource_footprint_summary(path: &Path) -> Option<serde_json::Value> {
    let v = crate::read_json_value(path)?;
    let pid = v.get("pid").and_then(|v| v.as_u64());
    let wall_time_ms = v.get("wall_time_ms").and_then(|v| v.as_u64());
    let killed = v.get("killed").and_then(|v| v.as_bool());
    let note = v
        .get("note")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let cpu_avg_pct_total_cores = v
        .get("cpu")
        .and_then(|v| v.get("avg_cpu_percent_total_cores"))
        .and_then(|v| v.as_f64());
    let cpu_usage_pct_avg = v
        .get("cpu")
        .and_then(|v| v.get("usage_percent_avg"))
        .and_then(|v| v.as_f64());

    let working_set_bytes = v
        .get("memory")
        .and_then(|v| v.get("working_set_bytes"))
        .and_then(|v| v.as_u64());
    let peak_working_set_bytes = v
        .get("memory")
        .and_then(|v| v.get("peak_working_set_bytes"))
        .and_then(|v| v.as_u64());

    let macos_physical_footprint_bytes = v
        .get("macos_vmmap")
        .and_then(|v| v.get("physical_footprint_bytes"))
        .and_then(|v| v.as_u64());
    let macos_physical_footprint_peak_bytes = v
        .get("macos_vmmap")
        .and_then(|v| v.get("physical_footprint_peak_bytes"))
        .and_then(|v| v.as_u64());
    let macos_owned_unmapped_memory_dirty_bytes = v
        .get("macos_vmmap")
        .and_then(|v| v.get("regions"))
        .and_then(|v| v.get("owned_unmapped_memory_dirty_bytes"))
        .and_then(|v| v.as_u64());

    let macos_vmmap_top_dirty_region_type = v
        .get("macos_vmmap")
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.get("regions"))
        .and_then(|v| v.get("top_dirty"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.get("region_type"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let macos_vmmap_top_dirty_region_bytes = v
        .get("macos_vmmap")
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.get("regions"))
        .and_then(|v| v.get("top_dirty"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.get("dirty_bytes"))
        .and_then(|v| v.as_u64());

    let macos_vmmap_top_allocated_malloc_zone = v
        .get("macos_vmmap")
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.get("malloc_zones"))
        .and_then(|v| v.get("top_allocated"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.get("zone"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let macos_vmmap_top_allocated_malloc_bytes = v
        .get("macos_vmmap")
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.get("malloc_zones"))
        .and_then(|v| v.get("top_allocated"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.get("allocated_bytes"))
        .and_then(|v| v.as_u64());

    let macos_vmmap_top_dirty_regions = v
        .get("macos_vmmap")
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.get("regions"))
        .and_then(|v| v.get("top_dirty"))
        .and_then(|v| v.as_array())
        .map(|rows| {
            rows.iter()
                .take(8)
                .map(|row| {
                    serde_json::json!({
                        "region_type": row.get("region_type").and_then(|v| v.as_str()),
                        "region_count": row.get("region_count").and_then(|v| v.as_u64()),
                        "virtual_bytes": row.get("virtual_bytes").and_then(|v| v.as_u64()),
                        "resident_bytes": row.get("resident_bytes").and_then(|v| v.as_u64()),
                        "dirty_bytes": row.get("dirty_bytes").and_then(|v| v.as_u64()),
                    })
                })
                .collect::<Vec<_>>()
        });

    let macos_vmmap_top_resident_regions = v
        .get("macos_vmmap")
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.get("regions"))
        .and_then(|v| v.get("top_resident"))
        .and_then(|v| v.as_array())
        .map(|rows| {
            rows.iter()
                .take(8)
                .map(|row| {
                    serde_json::json!({
                        "region_type": row.get("region_type").and_then(|v| v.as_str()),
                        "region_count": row.get("region_count").and_then(|v| v.as_u64()),
                        "virtual_bytes": row.get("virtual_bytes").and_then(|v| v.as_u64()),
                        "resident_bytes": row.get("resident_bytes").and_then(|v| v.as_u64()),
                        "dirty_bytes": row.get("dirty_bytes").and_then(|v| v.as_u64()),
                    })
                })
                .collect::<Vec<_>>()
        });

    Some(serde_json::json!({
        "pid": pid,
        "wall_time_ms": wall_time_ms,
        "killed": killed,
        "note": note,
        "cpu_avg_percent_total_cores": cpu_avg_pct_total_cores,
        "cpu_usage_percent_avg": cpu_usage_pct_avg,
        "working_set_bytes": working_set_bytes,
        "peak_working_set_bytes": peak_working_set_bytes,
        "macos_physical_footprint_bytes": macos_physical_footprint_bytes,
        "macos_physical_footprint_peak_bytes": macos_physical_footprint_peak_bytes,
        "macos_owned_unmapped_memory_dirty_bytes": macos_owned_unmapped_memory_dirty_bytes,
        "macos_vmmap_top_dirty_region_type": macos_vmmap_top_dirty_region_type,
        "macos_vmmap_top_dirty_region_bytes": macos_vmmap_top_dirty_region_bytes,
        "macos_vmmap_top_dirty_regions": macos_vmmap_top_dirty_regions,
        "macos_vmmap_top_resident_regions": macos_vmmap_top_resident_regions,
        "macos_vmmap_top_allocated_malloc_zone": macos_vmmap_top_allocated_malloc_zone,
        "macos_vmmap_top_allocated_malloc_bytes": macos_vmmap_top_allocated_malloc_bytes,
    }))
}

fn bundle_stats_summary_from_path(path: &Path) -> Option<serde_json::Value> {
    let v = crate::read_json_value(path)?;
    let windows = v.get("windows").and_then(|v| v.as_array())?;
    let first_window = windows.first()?;
    let snapshots = first_window.get("snapshots").and_then(|v| v.as_array())?;
    let last_snapshot = snapshots.last()?;
    let stats = last_snapshot
        .get("debug")
        .and_then(|v| v.get("stats"))
        .and_then(|v| v.as_object())?;

    let get_u64 = |k: &str| stats.get(k).and_then(|v| v.as_u64());
    let get_bool = |k: &str| stats.get(k).and_then(|v| v.as_bool());

    let render_text = last_snapshot
        .get("resource_caches")
        .and_then(|v| v.get("render_text"))
        .and_then(|v| v.as_object());

    let rt_u64 = |k: &str| render_text.and_then(|o| o.get(k)).and_then(|v| v.as_u64());
    let rt_atlas = |k: &str| {
        render_text
            .and_then(|o| o.get(k))
            .and_then(|v| v.as_object())
    };

    let atlas_u64 = |atlas: Option<&serde_json::Map<String, serde_json::Value>>, k: &str| {
        atlas.and_then(|o| o.get(k)).and_then(|v| v.as_u64())
    };

    let sat_mul_u64 =
        |a: u64, b: u64| -> u64 { ((a as u128) * (b as u128)).min(u64::MAX as u128) as u64 };
    let atlas_bytes =
        |atlas: Option<&serde_json::Map<String, serde_json::Value>>, bpp: u64| -> Option<u64> {
            let w = atlas_u64(atlas, "width")?;
            let h = atlas_u64(atlas, "height")?;
            let pages = atlas_u64(atlas, "pages")?;
            Some(sat_mul_u64(sat_mul_u64(sat_mul_u64(w, h), pages), bpp))
        };

    let mask_atlas = rt_atlas("mask_atlas");
    let color_atlas = rt_atlas("color_atlas");
    let subpixel_atlas = rt_atlas("subpixel_atlas");

    let render_text_mask_atlas_bytes_live_estimate = atlas_bytes(mask_atlas, 1);
    let render_text_color_atlas_bytes_live_estimate = atlas_bytes(color_atlas, 4);
    let render_text_subpixel_atlas_bytes_live_estimate = atlas_bytes(subpixel_atlas, 4);
    let render_text_atlas_bytes_live_estimate_total = match (
        render_text_mask_atlas_bytes_live_estimate,
        render_text_color_atlas_bytes_live_estimate,
        render_text_subpixel_atlas_bytes_live_estimate,
    ) {
        (Some(a), Some(b), Some(c)) => Some(a.saturating_add(b).saturating_add(c)),
        _ => None,
    };

    Some(serde_json::json!({
        "bundle_schema_version": v.get("schema_version").and_then(|v| v.as_u64()),
        "window": first_window.get("window").and_then(|v| v.as_u64()),
        "tick_id": last_snapshot.get("tick_id").and_then(|v| v.as_u64()),
        "frame_id": last_snapshot.get("frame_id").and_then(|v| v.as_u64()),
        "wgpu_allocator_tick_id": get_u64("wgpu_allocator_tick_id"),
        "wgpu_allocator_frame_id": get_u64("wgpu_allocator_frame_id"),
        "wgpu_allocator_report_present": get_bool("wgpu_allocator_report_present"),
        "wgpu_allocator_total_allocated_bytes": get_u64("wgpu_allocator_total_allocated_bytes"),
        "wgpu_allocator_total_reserved_bytes": get_u64("wgpu_allocator_total_reserved_bytes"),
        "wgpu_metal_current_allocated_size_present": get_bool("wgpu_metal_current_allocated_size_present"),
        "wgpu_metal_current_allocated_size_bytes": get_u64("wgpu_metal_current_allocated_size_bytes"),
        "renderer_intermediate_peak_in_use_bytes": get_u64("renderer_intermediate_peak_in_use_bytes"),
        "renderer_gpu_images_bytes_estimate": get_u64("renderer_gpu_images_bytes_estimate"),
        "renderer_gpu_render_targets_bytes_estimate": get_u64("renderer_gpu_render_targets_bytes_estimate"),

        "render_text_present": render_text.is_some(),
        "render_text_blobs_live": rt_u64("blobs_live"),
        "render_text_blob_cache_entries": rt_u64("blob_cache_entries"),
        "render_text_shape_cache_entries": rt_u64("shape_cache_entries"),
        "render_text_measure_cache_buckets": rt_u64("measure_cache_buckets"),

        "render_text_mask_atlas_width_px": atlas_u64(mask_atlas, "width"),
        "render_text_mask_atlas_height_px": atlas_u64(mask_atlas, "height"),
        "render_text_mask_atlas_pages": atlas_u64(mask_atlas, "pages"),
        "render_text_mask_atlas_used_px": atlas_u64(mask_atlas, "used_px"),
        "render_text_mask_atlas_capacity_px": atlas_u64(mask_atlas, "capacity_px"),
        "render_text_mask_atlas_bytes_live_estimate": render_text_mask_atlas_bytes_live_estimate,

        "render_text_color_atlas_width_px": atlas_u64(color_atlas, "width"),
        "render_text_color_atlas_height_px": atlas_u64(color_atlas, "height"),
        "render_text_color_atlas_pages": atlas_u64(color_atlas, "pages"),
        "render_text_color_atlas_used_px": atlas_u64(color_atlas, "used_px"),
        "render_text_color_atlas_capacity_px": atlas_u64(color_atlas, "capacity_px"),
        "render_text_color_atlas_bytes_live_estimate": render_text_color_atlas_bytes_live_estimate,

        "render_text_subpixel_atlas_width_px": atlas_u64(subpixel_atlas, "width"),
        "render_text_subpixel_atlas_height_px": atlas_u64(subpixel_atlas, "height"),
        "render_text_subpixel_atlas_pages": atlas_u64(subpixel_atlas, "pages"),
        "render_text_subpixel_atlas_used_px": atlas_u64(subpixel_atlas, "used_px"),
        "render_text_subpixel_atlas_capacity_px": atlas_u64(subpixel_atlas, "capacity_px"),
        "render_text_subpixel_atlas_bytes_live_estimate": render_text_subpixel_atlas_bytes_live_estimate,

        "render_text_atlas_bytes_live_estimate_total": render_text_atlas_bytes_live_estimate_total,
    }))
}

pub(crate) fn write_evidence_index(
    artifacts_root: &Path,
    summary_path: &Path,
    summary_json: Option<&serde_json::Value>,
) -> Result<PathBuf, String> {
    let out_path = artifacts_root.join("evidence.index.json");

    let mut entries: Vec<serde_json::Value> = Vec::new();
    let mut checks: Vec<serde_json::Value> = Vec::new();

    let mut add_file = |name: &str, rel: &str| {
        let path = artifacts_root.join(rel);
        let meta = std::fs::metadata(&path).ok();
        let exists = meta.is_some();
        let size_bytes = meta.as_ref().map(|m| m.len());
        let mtime_unix_ms = meta.as_ref().and_then(metadata_mtime_unix_ms);
        let json = if exists && rel.ends_with(".json") {
            json_file_summary(&path)
        } else {
            None
        };

        entries.push(serde_json::json!({
            "name": name,
            "rel_path": rel,
            "kind": "file",
            "exists": exists,
            "size_bytes": size_bytes,
            "mtime_unix_ms": mtime_unix_ms,
            "json": json,
        }));
    };

    add_file("repro.summary", "repro.summary.json");
    add_file("repro.zip", "repro.zip");
    add_file("resource.footprint", "resource.footprint.json");
    add_file("resource.vmmap_summary", "resource.vmmap_summary.txt");
    add_file("redraw_hitches", "redraw_hitches.log");
    add_file("renderdoc.captures", "renderdoc.captures.json");
    add_file("tracy.note", "tracy.note.md");
    add_file("script", "script.json");
    add_file("script.result", "script.result.json");
    add_file("pick.result", "pick.result.json");
    add_file("screenshots.result", "screenshots.result.json");
    add_file(
        "check.semantics_changed_repainted",
        "check.semantics_changed_repainted.json",
    );
    add_file("check.idle_no_paint", "check.idle_no_paint.json");
    add_file("check.pixels_changed", "check.pixels_changed.json");
    add_file("check.perf_thresholds", "check.perf_thresholds.json");
    add_file("check.perf_hints", "check.perf_hints.json");
    add_file("check.redraw_hitches", "check.redraw_hitches.json");
    add_file(
        "check.wgpu_metal_allocated_size",
        "check.wgpu_metal_allocated_size.json",
    );
    add_file(
        "check.render_text_atlas_bytes",
        "check.render_text_atlas_bytes.json",
    );
    add_file("check.resource_footprint", "check.resource_footprint.json");
    add_file(
        "check.view_cache_reuse_stable",
        "check.view_cache_reuse_stable.json",
    );

    for e in entries.iter() {
        let Some(rel) = e.get("rel_path").and_then(|v| v.as_str()) else {
            continue;
        };
        if !rel.starts_with("check.") || !rel.ends_with(".json") {
            continue;
        }
        if e.get("exists").and_then(|v| v.as_bool()) != Some(true) {
            continue;
        }
        let name = e.get("name").cloned().unwrap_or(serde_json::Value::Null);
        let json = e.get("json").cloned().unwrap_or(serde_json::Value::Null);
        let ok = json.get("ok").cloned().unwrap_or(serde_json::Value::Null);
        checks.push(serde_json::json!({
            "name": name,
            "file": rel,
            "ok": ok,
            "summary": json,
        }));
    }

    let footprint = artifacts_root.join("resource.footprint.json");
    let bundle_stats = summary_json
        .and_then(|v| {
            v.get("selected_bundle_json")
                .or_else(|| v.get("packed_bundle_json"))
        })
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .filter(|p| p.is_file())
        .and_then(|p| bundle_stats_summary_from_path(&p));

    let resources = serde_json::json!({
        "process_footprint": if footprint.is_file() {
            resource_footprint_summary(&footprint)
        } else {
            None
        },
        "bundle_last_frame_stats": bundle_stats,
    });

    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": crate::now_unix_ms(),
        "out_dir": artifacts_root.display().to_string(),
        "summary_file": summary_path.file_name().and_then(|s| s.to_str()).unwrap_or("repro.summary.json"),
        "summary": summary_json.cloned(),
        "entries": entries,
        "checks": checks,
        "resources": resources,
    });

    let _ = crate::write_json_value(&out_path, &payload);
    Ok(out_path)
}
