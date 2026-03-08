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

    let macos_vmmap_source = if v.get("macos_vmmap_steady").is_some() {
        "steady"
    } else {
        "exit"
    };
    let macos_vmmap = v.get("macos_vmmap_steady").or_else(|| v.get("macos_vmmap"));

    let macos_physical_footprint_bytes = macos_vmmap
        .and_then(|v| v.get("physical_footprint_bytes"))
        .and_then(|v| v.as_u64());
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

    let macos_vmmap_top_dirty_region_type = macos_vmmap
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.get("regions"))
        .and_then(|v| v.get("top_dirty"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.get("region_type"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let macos_vmmap_top_dirty_region_bytes = macos_vmmap
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.get("regions"))
        .and_then(|v| v.get("top_dirty"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.get("dirty_bytes"))
        .and_then(|v| v.as_u64());

    let macos_vmmap_top_allocated_malloc_zone = macos_vmmap
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.get("malloc_zones"))
        .and_then(|v| v.get("top_allocated"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.get("zone"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let macos_vmmap_top_allocated_malloc_bytes = macos_vmmap
        .and_then(|v| v.get("tables"))
        .and_then(|v| v.get("malloc_zones"))
        .and_then(|v| v.get("top_allocated"))
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.get("allocated_bytes"))
        .and_then(|v| v.as_u64());

    let macos_vmmap_top_dirty_regions = macos_vmmap
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

    let macos_vmmap_top_resident_regions = macos_vmmap
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
        "macos_vmmap_source": macos_vmmap_source,
        "macos_physical_footprint_bytes": macos_physical_footprint_bytes,
        "macos_physical_footprint_peak_bytes": macos_physical_footprint_peak_bytes,
        "macos_owned_unmapped_memory_dirty_bytes": macos_owned_unmapped_memory_dirty_bytes,
        "macos_io_surface_dirty_bytes": macos_io_surface_dirty_bytes,
        "macos_io_accelerator_dirty_bytes": macos_io_accelerator_dirty_bytes,
        "macos_malloc_small_dirty_bytes": macos_malloc_small_dirty_bytes,
        "macos_malloc_dirty_bytes_total": macos_malloc_dirty_bytes_total,
        "macos_malloc_zones_total_allocated_bytes": macos_malloc_zones_total_allocated_bytes,
        "macos_malloc_zones_total_frag_bytes": macos_malloc_zones_total_frag_bytes,
        "macos_malloc_zones_total_dirty_bytes": macos_malloc_zones_total_dirty_bytes,
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

    let snapshot_stats_u64 = |snapshot: &serde_json::Value, k: &str| -> Option<u64> {
        snapshot
            .get("debug")
            .and_then(|v| v.get("stats"))
            .and_then(|v| v.get(k))
            .and_then(|v| v.as_u64())
    };
    let snapshot_stats_bool = |snapshot: &serde_json::Value, k: &str| -> Option<bool> {
        snapshot
            .get("debug")
            .and_then(|v| v.get("stats"))
            .and_then(|v| v.get(k))
            .and_then(|v| v.as_bool())
    };

    let get_u64 = |k: &str| stats.get(k).and_then(|v| v.as_u64());
    let get_bool = |k: &str| stats.get(k).and_then(|v| v.as_bool());

    let semantics_nodes_json = last_snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("nodes"))
        .and_then(|v| v.as_array());
    let semantics_nodes = semantics_nodes_json.map(|v| v.len() as u64);
    let current_page_test_id = last_snapshot
        .get("app_snapshot")
        .filter(|snapshot| snapshot.get("kind").and_then(|v| v.as_str()) == Some("fret_ui_gallery"))
        .and_then(|snapshot| snapshot.get("selected_page"))
        .and_then(|v| v.as_str())
        .map(|selected_page| format!("ui-gallery-page-{}", selected_page.replace('_', "-")));

    let (
        ui_gallery_nav_scroll_semantics_subtree_nodes,
        ui_gallery_page_overlay_semantics_subtree_nodes,
        ui_gallery_command_palette_semantics_subtree_nodes,
        ui_gallery_settings_open_semantics_subtree_nodes,
        ui_gallery_workspace_frame_semantics_subtree_nodes,
        ui_gallery_top_bar_semantics_subtree_nodes,
        ui_gallery_workspace_tabstrip_semantics_subtree_nodes,
        ui_gallery_status_bar_semantics_subtree_nodes,
        ui_gallery_content_shell_semantics_subtree_nodes,
        ui_gallery_content_header_semantics_subtree_nodes,
        ui_gallery_content_scroll_semantics_subtree_nodes,
        ui_gallery_page_preview_semantics_subtree_nodes,
        ui_gallery_current_page_semantics_subtree_nodes,
        ui_gallery_card_section_demo_semantics_subtree_nodes,
        ui_gallery_card_section_usage_semantics_subtree_nodes,
        ui_gallery_card_section_size_semantics_subtree_nodes,
        ui_gallery_card_section_card_content_semantics_subtree_nodes,
        ui_gallery_card_section_meeting_notes_semantics_subtree_nodes,
        ui_gallery_card_section_image_semantics_subtree_nodes,
        ui_gallery_card_section_rtl_semantics_subtree_nodes,
        ui_gallery_card_section_compositions_semantics_subtree_nodes,
        ui_gallery_card_section_notes_semantics_subtree_nodes,
    ) = if let Some(nodes) = semantics_nodes_json {
        use std::collections::HashMap;
        let mut children: HashMap<u64, Vec<u64>> = HashMap::new();
        let mut roots_by_test_id: HashMap<String, u64> = HashMap::new();
        for node in nodes {
            let id = node.get("id").and_then(|v| v.as_u64());
            let parent = node.get("parent").and_then(|v| v.as_u64());
            if let Some(id) = id {
                children.entry(id).or_default();
                if let Some(parent) = parent {
                    children.entry(parent).or_default().push(id);
                }
                if let Some(test_id) = node.get("test_id").and_then(|v| v.as_str()) {
                    roots_by_test_id.insert(test_id.to_string(), id);
                }
            }
        }

        let subtree_size = |root: Option<u64>| -> Option<u64> {
            let root = root?;
            let mut stack = vec![root];
            let mut seen = std::collections::BTreeSet::new();
            while let Some(cur) = stack.pop() {
                if !seen.insert(cur) {
                    continue;
                }
                if let Some(kids) = children.get(&cur) {
                    for child in kids {
                        stack.push(*child);
                    }
                }
            }
            Some(seen.len() as u64)
        };
        let subtree_size_by_test_id =
            |test_id: &str| subtree_size(roots_by_test_id.get(test_id).copied());

        (
            subtree_size_by_test_id("ui-gallery-nav-scroll"),
            subtree_size_by_test_id("ui-gallery-page-overlay"),
            subtree_size_by_test_id("ui-gallery-command-palette"),
            subtree_size_by_test_id("ui-gallery-settings-open"),
            subtree_size_by_test_id("ui-gallery-workspace-frame"),
            subtree_size_by_test_id("ui-gallery-top-bar"),
            subtree_size_by_test_id("ui-gallery-workspace-tabstrip"),
            subtree_size_by_test_id("ui-gallery-status-bar"),
            subtree_size_by_test_id("ui-gallery-content-shell"),
            subtree_size_by_test_id("ui-gallery-content-header"),
            subtree_size_by_test_id("ui-gallery-content-scroll"),
            subtree_size_by_test_id("ui-gallery-page-preview"),
            subtree_size(
                current_page_test_id
                    .as_deref()
                    .and_then(|test_id| roots_by_test_id.get(test_id).copied()),
            ),
            subtree_size_by_test_id("ui-gallery-card-section-demo"),
            subtree_size_by_test_id("ui-gallery-card-section-usage"),
            subtree_size_by_test_id("ui-gallery-card-section-size"),
            subtree_size_by_test_id("ui-gallery-card-section-card-content"),
            subtree_size_by_test_id("ui-gallery-card-section-meeting-notes"),
            subtree_size_by_test_id("ui-gallery-card-section-image"),
            subtree_size_by_test_id("ui-gallery-card-section-rtl"),
            subtree_size_by_test_id("ui-gallery-card-section-compositions"),
            subtree_size_by_test_id("ui-gallery-card-section-notes"),
        )
    } else {
        (
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
        )
    };
    let layers_in_paint_order = last_snapshot
        .get("debug")
        .and_then(|v| v.get("layers_in_paint_order"))
        .and_then(|v| v.as_array())
        .map(|v| v.len() as u64);
    let scroll_nodes = last_snapshot
        .get("debug")
        .and_then(|v| v.get("scroll_nodes"))
        .and_then(|v| v.as_array())
        .map(|v| v.len() as u64);
    let element_runtime = last_snapshot
        .get("debug")
        .and_then(|v| v.get("element_runtime"))
        .and_then(|v| v.as_object());
    let er_u64 = |k: &str| {
        element_runtime
            .and_then(|o| o.get(k))
            .and_then(|v| v.as_u64())
    };
    let er_len = |k: &str| -> Option<u64> {
        element_runtime
            .and_then(|o| o.get(k))
            .and_then(|v| v.as_array())
            .map(|v| v.len() as u64)
    };
    let er_pair_second_sum = |k: &str| -> Option<u64> {
        let items = element_runtime
            .and_then(|o| o.get(k))
            .and_then(|v| v.as_array())?;
        Some(items.iter().fold(0u64, |acc, item| {
            let next = item
                .as_array()
                .and_then(|entry| entry.get(1))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            acc.saturating_add(next)
        }))
    };
    let er_len_zero_default = |k: &str| -> Option<u64> {
        let runtime = element_runtime?;
        Some(
            runtime
                .get(k)
                .and_then(|v| v.as_array())
                .map_or(0, |items| items.len() as u64),
        )
    };
    let er_object_array_u64_sum_zero_default = |k: &str, field: &str| -> Option<u64> {
        let runtime = element_runtime?;
        let items = runtime.get(k).and_then(|v| v.as_array());
        Some(items.map_or(0u64, |items| {
            items.iter().fold(0u64, |acc, item| {
                let next = item
                    .as_object()
                    .and_then(|entry| entry.get(field))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                acc.saturating_add(next)
            })
        }))
    };
    let er_object_array_u64_max_zero_default = |k: &str, field: &str| -> Option<u64> {
        let runtime = element_runtime?;
        let items = runtime.get(k).and_then(|v| v.as_array());
        Some(items.map_or(0u64, |items| {
            items.iter().fold(0u64, |acc, item| {
                let next = item
                    .as_object()
                    .and_then(|entry| entry.get(field))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                acc.max(next)
            })
        }))
    };

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

    let mut wgpu_metal_current_allocated_size_bytes_min: Option<u64> = None;
    let mut wgpu_metal_current_allocated_size_bytes_max: Option<u64> = None;
    for snapshot in snapshots {
        if snapshot_stats_bool(snapshot, "wgpu_metal_current_allocated_size_present") != Some(true)
        {
            continue;
        }
        let Some(v) = snapshot_stats_u64(snapshot, "wgpu_metal_current_allocated_size_bytes")
        else {
            continue;
        };
        wgpu_metal_current_allocated_size_bytes_min =
            Some(wgpu_metal_current_allocated_size_bytes_min.map_or(v, |cur| cur.min(v)));
        wgpu_metal_current_allocated_size_bytes_max =
            Some(wgpu_metal_current_allocated_size_bytes_max.map_or(v, |cur| cur.max(v)));
    }

    let app_snapshot_shell = last_snapshot
        .get("app_snapshot")
        .filter(|snapshot| snapshot.get("kind").and_then(|v| v.as_str()) == Some("fret_ui_gallery"))
        .and_then(|snapshot| snapshot.get("shell"))
        .and_then(|v| v.as_object());
    let shell_u64 = |k: &str| {
        app_snapshot_shell
            .and_then(|shell| shell.get(k))
            .and_then(|v| v.as_u64())
    };

    Some(serde_json::json!({
        "bundle_schema_version": v.get("schema_version").and_then(|v| v.as_u64()),
        "window": first_window.get("window").and_then(|v| v.as_u64()),
        "tick_id": last_snapshot.get("tick_id").and_then(|v| v.as_u64()),
        "frame_id": last_snapshot.get("frame_id").and_then(|v| v.as_u64()),

        "wgpu_hub_tick_id": get_u64("wgpu_hub_tick_id"),
        "wgpu_hub_frame_id": get_u64("wgpu_hub_frame_id"),
        "wgpu_hub_adapters": get_u64("wgpu_hub_adapters"),
        "wgpu_hub_devices": get_u64("wgpu_hub_devices"),
        "wgpu_hub_queues": get_u64("wgpu_hub_queues"),
        "wgpu_hub_command_encoders": get_u64("wgpu_hub_command_encoders"),
        "wgpu_hub_buffers": get_u64("wgpu_hub_buffers"),
        "wgpu_hub_textures": get_u64("wgpu_hub_textures"),
        "wgpu_hub_texture_views": get_u64("wgpu_hub_texture_views"),
        "wgpu_hub_samplers": get_u64("wgpu_hub_samplers"),
        "wgpu_hub_shader_modules": get_u64("wgpu_hub_shader_modules"),
        "wgpu_hub_render_pipelines": get_u64("wgpu_hub_render_pipelines"),
        "wgpu_hub_compute_pipelines": get_u64("wgpu_hub_compute_pipelines"),

        "wgpu_allocator_tick_id": get_u64("wgpu_allocator_tick_id"),
        "wgpu_allocator_frame_id": get_u64("wgpu_allocator_frame_id"),
        "wgpu_allocator_sample_present": get_bool("wgpu_allocator_sample_present"),
        "wgpu_allocator_report_present": get_bool("wgpu_allocator_report_present"),
        "wgpu_allocator_total_allocated_bytes": get_u64("wgpu_allocator_total_allocated_bytes"),
        "wgpu_allocator_total_reserved_bytes": get_u64("wgpu_allocator_total_reserved_bytes"),
        "wgpu_metal_current_allocated_size_present": get_bool("wgpu_metal_current_allocated_size_present"),
        "wgpu_metal_current_allocated_size_bytes": get_u64("wgpu_metal_current_allocated_size_bytes"),
        "wgpu_metal_current_allocated_size_bytes_min": wgpu_metal_current_allocated_size_bytes_min,
        "wgpu_metal_current_allocated_size_bytes_max": wgpu_metal_current_allocated_size_bytes_max,
        "renderer_intermediate_peak_in_use_bytes": get_u64("renderer_intermediate_peak_in_use_bytes"),
        "ui_frame_arena_capacity_estimate_bytes": get_u64("frame_arena_capacity_estimate_bytes"),
        "ui_layout_nodes_visited": get_u64("layout_nodes_visited"),
        "ui_prepaint_nodes_visited": get_u64("prepaint_nodes_visited"),
        "ui_interaction_records": get_u64("interaction_records"),
        "ui_layout_invalidations_count": get_u64("layout_invalidations_count"),
        "ui_view_cache_roots_total": get_u64("view_cache_roots_total"),
        "ui_view_cache_roots_reused": get_u64("view_cache_roots_reused"),
        "ui_semantics_nodes": semantics_nodes,
        "ui_element_runtime_observed_models_count": er_len("observed_models"),
        "ui_element_runtime_observed_globals_count": er_len("observed_globals"),
        "ui_element_runtime_observed_environment_count": er_len("observed_environment"),
        "ui_element_runtime_view_cache_reuse_roots_count": er_len("view_cache_reuse_roots"),
        "ui_element_runtime_view_cache_reuse_root_elements_total": er_pair_second_sum("view_cache_reuse_root_element_counts"),
        "ui_element_runtime_continuous_frame_lease_owners_count": er_len_zero_default("continuous_frame_leases"),
        "ui_element_runtime_continuous_frame_lease_count_total": er_object_array_u64_sum_zero_default("continuous_frame_leases", "count"),
        "ui_element_runtime_continuous_frame_lease_count_max": er_object_array_u64_max_zero_default("continuous_frame_leases", "count"),
        "ui_element_runtime_animation_frame_request_roots_count": er_len_zero_default("animation_frame_request_roots"),
        "ui_element_runtime_retained_keep_alive_roots_len": er_u64("retained_keep_alive_roots_len"),
        "ui_element_runtime_rendered_state_entries": er_u64("rendered_state_entries"),
        "ui_element_runtime_next_state_entries": er_u64("next_state_entries"),
        "ui_element_runtime_lag_state_frames": er_u64("lag_state_frames"),
        "ui_element_runtime_lag_state_entries_total": er_u64("lag_state_entries_total"),
        "ui_element_runtime_state_entries_total": er_u64("state_entries_total"),
        "ui_element_runtime_nodes_count": er_u64("nodes_count"),
        "ui_element_runtime_bounds_entries_total": er_u64("bounds_entries_total"),
        "ui_element_runtime_timer_targets_count": er_u64("timer_targets_count"),
        "ui_element_runtime_transient_events_count": er_u64("transient_events_count"),
        "ui_element_runtime_view_cache_state_key_roots_count": er_u64("view_cache_state_key_roots_count"),
        "ui_element_runtime_view_cache_state_key_entries_total": er_u64("view_cache_state_key_entries_total"),
        "ui_element_runtime_view_cache_element_roots_count": er_u64("view_cache_element_roots_count"),
        "ui_element_runtime_view_cache_element_entries_total": er_u64("view_cache_element_entries_total"),
        "ui_element_runtime_view_cache_key_mismatch_roots_count": er_u64("view_cache_key_mismatch_roots_count"),
        "ui_element_runtime_scratch_element_children_vec_pool_len": er_u64("scratch_element_children_vec_pool_len"),
        "ui_element_runtime_scratch_element_children_vec_pool_capacity_total": er_u64("scratch_element_children_vec_pool_capacity_total"),
        "ui_element_runtime_scratch_element_children_vec_pool_bytes_estimate_total": er_u64("scratch_element_children_vec_pool_bytes_estimate_total"),
        "ui_gallery_nav_scroll_semantics_subtree_nodes": ui_gallery_nav_scroll_semantics_subtree_nodes,
        "ui_gallery_page_overlay_semantics_subtree_nodes": ui_gallery_page_overlay_semantics_subtree_nodes,
        "ui_gallery_command_palette_semantics_subtree_nodes": ui_gallery_command_palette_semantics_subtree_nodes,
        "ui_gallery_settings_open_semantics_subtree_nodes": ui_gallery_settings_open_semantics_subtree_nodes,
        "ui_gallery_workspace_frame_semantics_subtree_nodes": ui_gallery_workspace_frame_semantics_subtree_nodes,
        "ui_gallery_top_bar_semantics_subtree_nodes": ui_gallery_top_bar_semantics_subtree_nodes,
        "ui_gallery_workspace_tabstrip_semantics_subtree_nodes": ui_gallery_workspace_tabstrip_semantics_subtree_nodes,
        "ui_gallery_status_bar_semantics_subtree_nodes": ui_gallery_status_bar_semantics_subtree_nodes,
        "ui_gallery_content_shell_semantics_subtree_nodes": ui_gallery_content_shell_semantics_subtree_nodes,
        "ui_gallery_content_header_semantics_subtree_nodes": ui_gallery_content_header_semantics_subtree_nodes,
        "ui_gallery_content_scroll_semantics_subtree_nodes": ui_gallery_content_scroll_semantics_subtree_nodes,
        "ui_gallery_page_preview_semantics_subtree_nodes": ui_gallery_page_preview_semantics_subtree_nodes,
        "ui_gallery_current_page_semantics_subtree_nodes": ui_gallery_current_page_semantics_subtree_nodes,
        "ui_gallery_card_section_demo_semantics_subtree_nodes": ui_gallery_card_section_demo_semantics_subtree_nodes,
        "ui_gallery_card_section_usage_semantics_subtree_nodes": ui_gallery_card_section_usage_semantics_subtree_nodes,
        "ui_gallery_card_section_size_semantics_subtree_nodes": ui_gallery_card_section_size_semantics_subtree_nodes,
        "ui_gallery_card_section_card_content_semantics_subtree_nodes": ui_gallery_card_section_card_content_semantics_subtree_nodes,
        "ui_gallery_card_section_meeting_notes_semantics_subtree_nodes": ui_gallery_card_section_meeting_notes_semantics_subtree_nodes,
        "ui_gallery_card_section_image_semantics_subtree_nodes": ui_gallery_card_section_image_semantics_subtree_nodes,
        "ui_gallery_card_section_rtl_semantics_subtree_nodes": ui_gallery_card_section_rtl_semantics_subtree_nodes,
        "ui_gallery_card_section_compositions_semantics_subtree_nodes": ui_gallery_card_section_compositions_semantics_subtree_nodes,
        "ui_gallery_card_section_notes_semantics_subtree_nodes": ui_gallery_card_section_notes_semantics_subtree_nodes,
        "ui_layers_in_paint_order": layers_in_paint_order,
        "ui_scroll_nodes": scroll_nodes,
        "ui_gallery_nav_visible_groups_count": shell_u64("nav_visible_groups_count"),
        "ui_gallery_nav_visible_items_count": shell_u64("nav_visible_items_count"),
        "ui_gallery_nav_visible_ai_items_count": shell_u64("nav_visible_ai_items_count"),
        "ui_gallery_nav_visible_tags_count": shell_u64("nav_visible_tags_count"),
        "ui_gallery_nav_max_group_items_count": shell_u64("nav_max_group_items_count"),
        "ui_gallery_nav_visible_string_bytes_estimate_total": shell_u64("nav_visible_string_bytes_estimate_total"),
        "ui_gallery_card_doc_section_slots_total": shell_u64("card_doc_section_slots_total"),
        "ui_gallery_card_doc_visible_sections_count": shell_u64("card_doc_visible_sections_count"),
        "ui_gallery_card_doc_visible_sections_with_code_count": shell_u64("card_doc_visible_sections_with_code_count"),
        "ui_gallery_card_doc_visible_sections_with_shell_count": shell_u64("card_doc_visible_sections_with_shell_count"),
        "ui_gallery_card_doc_intro_len_bytes": shell_u64("card_doc_intro_len_bytes"),
        "ui_gallery_card_doc_visible_static_text_bytes_estimate_total": shell_u64("card_doc_visible_static_text_bytes_estimate_total"),
        "ui_gallery_card_doc_visible_code_bytes_estimate_total": shell_u64("card_doc_visible_code_bytes_estimate_total"),
        "ui_gallery_card_doc_visible_code_lines_estimate_total": shell_u64("card_doc_visible_code_lines_estimate_total"),
        "renderer_gpu_images_bytes_estimate": get_u64("renderer_gpu_images_bytes_estimate"),
        "renderer_gpu_render_targets_bytes_estimate": get_u64("renderer_gpu_render_targets_bytes_estimate"),

        "render_text_present": render_text.is_some(),
        "render_text_blobs_live": rt_u64("blobs_live"),
        "render_text_blob_cache_entries": rt_u64("blob_cache_entries"),
        "render_text_shape_cache_entries": rt_u64("shape_cache_entries"),
        "render_text_measure_cache_buckets": rt_u64("measure_cache_buckets"),
        "render_text_shape_cache_bytes_estimate_total": rt_u64("shape_cache_bytes_estimate_total"),
        "render_text_blob_paint_palette_bytes_estimate_total": rt_u64("blob_paint_palette_bytes_estimate_total"),
        "render_text_blob_decorations_bytes_estimate_total": rt_u64("blob_decorations_bytes_estimate_total"),
        "render_text_registered_font_blobs_total_bytes": rt_u64("registered_font_blobs_total_bytes"),
        "render_text_registered_font_blobs_count": rt_u64("registered_font_blobs_count"),

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
    add_file(
        "resource.macos_footprint.steady",
        "resource.macos_footprint.steady.json",
    );
    add_file("resource.vmmap_summary", "resource.vmmap_summary.txt");
    add_file(
        "resource.vmmap_summary.steady",
        "resource.vmmap_summary.steady.txt",
    );
    add_file(
        "resource.vmmap_regions_sorted.steady",
        "resource.vmmap_regions_sorted.steady.txt",
    );
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
    add_file(
        "check.hello_world_compare_idle_present",
        "check.hello_world_compare_idle_present.json",
    );
    add_file("check.pixels_changed", "check.pixels_changed.json");
    add_file("check.perf_thresholds", "check.perf_thresholds.json");
    add_file("check.perf_hints", "check.perf_hints.json");
    add_file("check.redraw_hitches", "check.redraw_hitches.json");
    add_file(
        "check.wgpu_metal_allocated_size",
        "check.wgpu_metal_allocated_size.json",
    );
    add_file("check.wgpu_hub_counts", "check.wgpu_hub_counts.json");
    add_file(
        "check.render_text_atlas_bytes",
        "check.render_text_atlas_bytes.json",
    );
    add_file(
        "check.render_text_font_db",
        "check.render_text_font_db.json",
    );
    add_file(
        "check.renderer_gpu_budgets",
        "check.renderer_gpu_budgets.json",
    );
    add_file("check.code_editor_memory", "check.code_editor_memory.json");
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
