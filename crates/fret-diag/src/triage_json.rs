use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::stats::{
    BundleStatsReport, BundleStatsSort, clean_geometry_solve_skip_rejection_to_json,
};
use fret_diag_protocol::{FilesystemCapabilitiesV1, UiScriptResultV1};

fn candidate_sidecar_paths(bundle_dir: &Path, file_name: &str) -> [PathBuf; 2] {
    [
        bundle_dir.join(file_name),
        bundle_dir.join("_root").join(file_name),
    ]
}

fn compat_summary_for_bundle_path(bundle_path: &Path) -> serde_json::Value {
    use serde_json::json;

    let bundle_dir = bundle_path.parent();
    let bundle_dir_has_schema2_sibling =
        bundle_dir.is_some_and(|d| d.join("bundle.schema2.json").is_file());
    let bundle_artifact_file_name = bundle_path
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    let bundle_schema_version = crate::compat::bundle::sniff_bundle_schema_version(bundle_path)
        .ok()
        .flatten();

    let mut markers: BTreeSet<String> = BTreeSet::new();

    if bundle_schema_version == Some(1) {
        markers.insert("compat.bundle_schema_v1".to_string());
    }

    if bundle_artifact_file_name
        .as_deref()
        .is_some_and(|n| n.eq_ignore_ascii_case("bundle.json"))
        && bundle_dir_has_schema2_sibling
    {
        markers.insert("compat.bundle_json_view_with_schema2_present".to_string());
    }

    let mut legacy_capabilities_present = false;
    if let Some(bundle_dir) = bundle_dir {
        for path in candidate_sidecar_paths(bundle_dir, "capabilities.json") {
            if !path.is_file() {
                continue;
            }
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let Ok(caps) = serde_json::from_slice::<FilesystemCapabilitiesV1>(&bytes) else {
                continue;
            };
            if caps.capabilities.iter().any(|c| !c.contains('.')) {
                legacy_capabilities_present = true;
                break;
            }
        }
    }
    if legacy_capabilities_present {
        markers.insert("compat.legacy_capabilities_present".to_string());
    }

    let mut script_compat_event_kinds: BTreeSet<String> = BTreeSet::new();
    let mut script_compat_events_total: u64 = 0;
    if let Some(bundle_dir) = bundle_dir {
        for path in candidate_sidecar_paths(bundle_dir, "script.result.json") {
            if !path.is_file() {
                continue;
            }
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let Ok(res) = serde_json::from_slice::<UiScriptResultV1>(&bytes) else {
                continue;
            };
            let Some(evidence) = res.evidence else {
                continue;
            };
            for ev in evidence.event_log {
                if ev.kind.starts_with("compat.") {
                    script_compat_events_total = script_compat_events_total.saturating_add(1);
                    if script_compat_event_kinds.len() < 20 {
                        script_compat_event_kinds.insert(ev.kind);
                    }
                }
            }
        }
    }
    for k in &script_compat_event_kinds {
        markers.insert(k.clone());
    }

    json!({
        "schema_version": 1,
        "bundle_schema_version": bundle_schema_version,
        "bundle_artifact_file_name": bundle_artifact_file_name,
        "bundle_dir_has_schema2_sibling": bundle_dir_has_schema2_sibling,
        "legacy_capabilities_present": legacy_capabilities_present,
        "script_compat_event_kinds": script_compat_event_kinds.into_iter().collect::<Vec<_>>(),
        "script_compat_events_total": script_compat_events_total,
        "markers": markers.into_iter().collect::<Vec<_>>(),
    })
}

pub(crate) fn triage_json_from_stats(
    bundle_path: &Path,
    report: &BundleStatsReport,
    sort: BundleStatsSort,
    warmup_frames: u64,
) -> serde_json::Value {
    use serde_json::json;

    fn ratio_pct(numer: u64, denom: u64) -> f64 {
        if denom == 0 {
            return 0.0;
        }
        (numer as f64) * 100.0 / (denom as f64)
    }

    fn triage_hints(
        stats_json: &serde_json::Value,
        worst: Option<&crate::stats::BundleStatsSnapshotRow>,
    ) -> Vec<serde_json::Value> {
        let mut out: Vec<serde_json::Value> = Vec::new();

        let Some(worst) = worst else {
            return out;
        };

        let sum_layout_observation_record_time_us = stats_json
            .get("sum")
            .and_then(|v| v.get("layout_observation_record_time_us"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let sum_layout_request_build_roots_time_us = stats_json
            .get("sum")
            .and_then(|v| v.get("layout_request_build_roots_time_us"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let sum_layout_roots_time_us = stats_json
            .get("sum")
            .and_then(|v| v.get("layout_roots_time_us"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let sum_layout_view_cache_time_us = stats_json
            .get("sum")
            .and_then(|v| v.get("layout_view_cache_time_us"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let sum_layout_time_us = stats_json
            .get("sum")
            .and_then(|v| v.get("layout_time_us"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let custom_effect_chain_present =
            worst.renderer_render_plan_custom_effect_chain_budget_samples > 0;
        let custom_effect_chain_effective_budget_bytes = custom_effect_chain_present
            .then_some(worst.renderer_render_plan_custom_effect_chain_effective_budget_min_bytes);
        let custom_effect_chain_base_required_bytes = custom_effect_chain_present
            .then_some(worst.renderer_render_plan_custom_effect_chain_base_required_max_bytes);
        let custom_effect_chain_optional_required_bytes = custom_effect_chain_present
            .then_some(worst.renderer_render_plan_custom_effect_chain_optional_required_max_bytes);
        let custom_effect_chain_optional_mask_bytes = custom_effect_chain_present
            .then_some(worst.renderer_render_plan_custom_effect_chain_optional_mask_max_bytes);
        let custom_effect_chain_headroom_before_optional_bytes = custom_effect_chain_present
            .then_some(
                worst
                    .renderer_render_plan_custom_effect_chain_effective_budget_min_bytes
                    .saturating_sub(
                        worst.renderer_render_plan_custom_effect_chain_base_required_max_bytes,
                    ),
            );
        let custom_effect_chain_headroom_after_mask_bytes = custom_effect_chain_present.then_some(
            worst
                .renderer_render_plan_custom_effect_chain_effective_budget_min_bytes
                .saturating_sub(
                    worst.renderer_render_plan_custom_effect_chain_base_required_max_bytes,
                )
                .saturating_sub(
                    worst.renderer_render_plan_custom_effect_chain_optional_mask_max_bytes,
                ),
        );
        let custom_effect_chain_headroom_after_optional_bytes = custom_effect_chain_present
            .then_some(
                worst
                    .renderer_render_plan_custom_effect_chain_effective_budget_min_bytes
                    .saturating_sub(
                        worst.renderer_render_plan_custom_effect_chain_base_required_max_bytes,
                    )
                    .saturating_sub(
                        worst.renderer_render_plan_custom_effect_chain_optional_required_max_bytes,
                    ),
            );

        // Heuristics are intentionally simple, bounded, and explainable.
        // Keep thresholds conservative; they are hints, not gates.

        // layout.observation_heavy
        if worst.layout_observation_record_time_us > 0 && worst.layout_time_us > 0 {
            let pct = ratio_pct(
                worst.layout_observation_record_time_us,
                worst.layout_time_us,
            );
            if worst.layout_observation_record_time_us >= 2_000 || pct >= 20.0 {
                out.push(json!({
                    "code": "layout.observation_heavy",
                    "severity": "warn",
                    "message": "Layout observation recording is a significant slice of layout time in the worst frame.",
                    "evidence": {
                        "layout_observation_record_time_us": worst.layout_observation_record_time_us,
                        "layout_time_us": worst.layout_time_us,
                        "layout_observation_record_pct_of_layout": pct,
                        "sum_layout_observation_record_time_us": sum_layout_observation_record_time_us,
                        "sum_layout_time_us": sum_layout_time_us,
                        "sum_layout_observation_record_pct_of_layout": ratio_pct(sum_layout_observation_record_time_us, sum_layout_time_us),
                    }
                }));
            }
        }

        // layout.solve_heavy
        if worst.layout_engine_solve_time_us > 0 && worst.layout_time_us > 0 {
            let pct = ratio_pct(worst.layout_engine_solve_time_us, worst.layout_time_us);
            let per_solve = if worst.layout_engine_solves == 0 {
                None
            } else {
                Some(worst.layout_engine_solve_time_us / worst.layout_engine_solves)
            };
            if worst.layout_engine_solve_time_us >= 5_000 || pct >= 50.0 {
                out.push(json!({
                    "code": "layout.solve_heavy",
                    "severity": "warn",
                    "message": "Layout engine solve dominates layout time in the worst frame.",
                    "evidence": {
                        "layout_engine_solve_time_us": worst.layout_engine_solve_time_us,
                        "layout_engine_solves": worst.layout_engine_solves,
                        "layout_engine_solve_us_per_solve": per_solve,
                        "layout_time_us": worst.layout_time_us,
                        "layout_engine_solve_pct_of_layout": pct,
                    }
                }));
            }
        }

        // layout.build_roots_heavy
        if worst.layout_request_build_roots_time_us > 0 && worst.layout_time_us > 0 {
            let pct = ratio_pct(
                worst.layout_request_build_roots_time_us,
                worst.layout_time_us,
            );
            if worst.layout_request_build_roots_time_us >= 2_000 || pct >= 20.0 {
                let examples: Vec<serde_json::Value> = worst
                    .layout_request_build_roots
                    .iter()
                    .take(4)
                    .map(|r| {
                        json!({
                            "root_node": r.root_node,
                            "root_kind": r.root_kind,
                            "elapsed_us": r.elapsed_us,
                            "mode": r.mode,
                            "had_layout_engine_node": r.had_layout_engine_node,
                            "layout_invalidated": r.layout_invalidated,
                            "subtree_layout_dirty": r.subtree_layout_dirty,
                            "subtree_layout_dirty_count": r.subtree_layout_dirty_count,
                            "descendant_layout_dirty_count": r.descendant_layout_dirty_count,
                            "needs_layout": r.needs_layout,
                            "is_translation_only": r.is_translation_only,
                            "nodes_marked_seen": r.nodes_marked_seen,
                            "root_role": r.root_role,
                            "root_test_id": r.root_test_id,
                            "dirty_descendants": r.dirty_descendants.iter().take(4).map(|d| {
                                json!({
                                    "node": d.node,
                                    "element": d.element,
                                    "element_kind": d.element_kind,
                                    "element_path": d.element_path,
                                    "subtree_layout_dirty_count": d.subtree_layout_dirty_count,
                                    "source_root_node": d.source_root_node,
                                    "source": d.source,
                                    "detail": d.detail,
                                    "role": d.role,
                                    "test_id": d.test_id,
                                })
                            }).collect::<Vec<_>>(),
                        })
                    })
                    .collect();
                out.push(json!({
                    "code": "layout.build_roots_heavy",
                    "severity": "info",
                    "message": "Layout root-building work is a significant slice of layout time in the worst frame.",
                    "evidence": {
                        "layout_request_build_roots_time_us": worst.layout_request_build_roots_time_us,
                        "layout_time_us": worst.layout_time_us,
                        "layout_request_build_roots_pct_of_layout": pct,
                        "sum_layout_request_build_roots_time_us": sum_layout_request_build_roots_time_us,
                        "sum_layout_time_us": sum_layout_time_us,
                        "sum_layout_request_build_roots_pct_of_layout": ratio_pct(sum_layout_request_build_roots_time_us, sum_layout_time_us),
                        "examples": examples,
                    }
                }));
            }
        }

        // layout.roots_heavy
        if worst.layout_roots_time_us > 0 && worst.layout_time_us > 0 {
            let pct = ratio_pct(worst.layout_roots_time_us, worst.layout_time_us);
            if worst.layout_time_us >= 15_000
                && (worst.layout_roots_time_us >= 10_000 || pct >= 70.0)
            {
                out.push(json!({
                    "code": "layout.roots_heavy",
                    "severity": "info",
                    "message": "Layout root processing dominates layout time in the worst frame.",
                    "evidence": {
                        "layout_roots_time_us": worst.layout_roots_time_us,
                        "layout_time_us": worst.layout_time_us,
                        "layout_roots_pct_of_layout": pct,
                        "sum_layout_roots_time_us": sum_layout_roots_time_us,
                        "sum_layout_time_us": sum_layout_time_us,
                        "sum_layout_roots_pct_of_layout": ratio_pct(sum_layout_roots_time_us, sum_layout_time_us),
                    }
                }));
            }
        }

        // layout.scroll_profile_present
        if !worst.scroll_layout_profiles.is_empty() {
            let examples: Vec<serde_json::Value> = worst
                .scroll_layout_profiles
                .iter()
                .take(4)
                .map(|p| {
                    json!({
                        "node": p.node,
                        "element": p.element,
                        "test_id": p.test_id.as_ref().or(p.semantics_test_id.as_ref()),
                        "axis": p.axis,
                        "pass": p.pass,
                        "total_us": p.total_us,
                        "measure_children_us": p.measure_children_us,
                        "solve_barrier_us": p.solve_barrier_us,
                        "layout_children_us": p.layout_children_us,
                        "layout_children_first_pass_us": p.layout_children_first_pass_us,
                        "layout_child_first_pass_nodes_visited": p.layout_child_first_pass_nodes_visited,
                        "layout_child_first_pass_nodes_performed": p.layout_child_first_pass_nodes_performed,
                        "layout_child_first_pass_max_us": p.layout_child_first_pass_max_us,
                        "layout_child_first_pass_kind_profiles": p.layout_child_first_pass_kind_profiles.iter().take(6).map(|k| {
                            json!({
                                "kind": k.kind,
                                "nodes": k.nodes,
                                "self_us": k.self_us,
                                "total_us": k.total_us,
                                "max_self_us": k.max_self_us,
                                "max_total_us": k.max_total_us,
                            })
                        }).collect::<Vec<_>>(),
                        "corrected_content_relayout": p.corrected_content_relayout,
                        "layout_children_corrected_content_us": p.layout_children_corrected_content_us,
                        "layout_child_corrected_content_nodes_visited": p.layout_child_corrected_content_nodes_visited,
                        "layout_child_corrected_content_nodes_performed": p.layout_child_corrected_content_nodes_performed,
                        "layout_child_corrected_content_max_us": p.layout_child_corrected_content_max_us,
                        "layout_child_corrected_content_kind_profiles": p.layout_child_corrected_content_kind_profiles.iter().take(6).map(|k| {
                            json!({
                                "kind": k.kind,
                                "nodes": k.nodes,
                                "self_us": k.self_us,
                                "total_us": k.total_us,
                                "max_self_us": k.max_self_us,
                                "max_total_us": k.max_total_us,
                            })
                        }).collect::<Vec<_>>(),
                        "layout_child_kind_profiles": p.layout_child_kind_profiles.iter().take(6).map(|k| {
                            json!({
                                "kind": k.kind,
                                "nodes": k.nodes,
                                "self_us": k.self_us,
                                "total_us": k.total_us,
                                "max_self_us": k.max_self_us,
                                "max_total_us": k.max_total_us,
                            })
                        }).collect::<Vec<_>>(),
                        "layout_child_max_us": p.layout_child_max_us,
                        "layout_child_max_node": p.layout_child_max_node,
                        "layout_child_max_invalidated": p.layout_child_max_invalidated,
                        "layout_child_max_subtree_dirty": p.layout_child_max_subtree_dirty,
                        "layout_child_max_subtree_dirty_count": p.layout_child_max_subtree_dirty_count,
                        "layout_child_max_bounds_changed": p.layout_child_max_bounds_changed,
                        "layout_child_max_bounds_size_changed": p.layout_child_max_bounds_size_changed,
                        "layout_child_max_input_matches_before": p.layout_child_max_input_matches_before,
                        "layout_child_max_input_size_matches_before": p.layout_child_max_input_size_matches_before,
                        "interactive_resize": p.interactive_resize,
                        "direct_children_layout_invalidated": p.direct_children_layout_invalidated,
                        "descendant_subtree_layout_dirty": p.descendant_subtree_layout_dirty,
                        "post_layout_extents_mode": p.post_layout_extents_mode,
                        "phase_profiles": p.phase_profiles.iter().map(|phase| {
                            json!({
                                "phase": phase.phase,
                                "us": phase.us,
                            })
                        }).collect::<Vec<_>>(),
                        "element_path": p.element_path,
                    })
                })
                .collect();
            out.push(json!({
                "code": "layout.scroll_profile_present",
                "severity": "info",
                "message": "Scroll layout profiling was captured in the worst frame; inspect child bounds deltas before changing resize/scroll layout behavior.",
                "evidence": {
                    "examples": examples,
                }
            }));
        }

        // view_cache.layout_invalidated
        if worst.view_cache_roots_layout_invalidated > 0 {
            out.push(json!({
                "code": "view_cache.layout_invalidated",
                "severity": "info",
                "message": "One or more view cache roots were layout-invalidated in the worst frame (may cause cache misses and relayout).",
                "evidence": {
                    "view_cache_roots_layout_invalidated": worst.view_cache_roots_layout_invalidated,
                    "view_cache_roots_total": worst.view_cache_roots_total,
                    "view_cache_roots_reused": worst.view_cache_roots_reused,
                    "view_cache_roots_cache_key_mismatch": worst.view_cache_roots_cache_key_mismatch,
                    "view_cache_roots_not_marked_reuse_root": worst.view_cache_roots_not_marked_reuse_root,
                    "layout_view_cache_time_us": worst.layout_view_cache_time_us,
                    "layout_expand_view_cache_invalidations_time_us": worst.layout_expand_view_cache_invalidations_time_us,
                    "sum_layout_view_cache_time_us": sum_layout_view_cache_time_us,
                }
            }));
        }

        // paint.widget_heavy
        if worst.paint_widget_time_us > 0 {
            let pct = ratio_pct(worst.paint_widget_time_us, worst.paint_time_us);
            if worst.paint_widget_time_us >= 2_000 || (worst.paint_time_us > 0 && pct >= 50.0) {
                let examples: Vec<serde_json::Value> = worst
                    .paint_widget_hotspots
                    .iter()
                    .take(4)
                    .map(|h| {
                        json!({
                            "node": h.node,
                            "element": h.element,
                            "element_kind": h.element_kind,
                            "widget_type": h.widget_type,
                            "paint_time_us": h.paint_time_us,
                            "inclusive_time_us": h.inclusive_time_us,
                            "inclusive_scene_ops_delta": h.inclusive_scene_ops_delta,
                            "exclusive_scene_ops_delta": h.exclusive_scene_ops_delta,
                            "role": h.role,
                            "test_id": h.test_id,
                        })
                    })
                    .collect();
                out.push(json!({
                    "code": "paint.widget_heavy",
                    "severity": "warn",
                    "message": "Widget paint work is a significant slice of paint time in the worst frame; inspect paint_widget_hotspots before changing renderer or layout thresholds.",
                    "evidence": {
                        "paint_widget_time_us": worst.paint_widget_time_us,
                        "paint_time_us": worst.paint_time_us,
                        "paint_widget_pct_of_paint": pct,
                        "examples": examples,
                    }
                }));
            }
        }

        // paint.text_prepare_churn
        if worst.paint_text_prepare_time_us > 0 || worst.paint_text_prepare_calls > 0 {
            let per_call = if worst.paint_text_prepare_calls == 0 {
                None
            } else {
                Some(worst.paint_text_prepare_time_us / (worst.paint_text_prepare_calls as u64))
            };
            if worst.paint_text_prepare_time_us >= 2_000
                || (per_call.is_some_and(|v| v >= 200) && worst.paint_text_prepare_calls >= 5)
            {
                out.push(json!({
                    "code": "paint.text_prepare_churn",
                    "severity": "warn",
                    "message": "Text prepare work is non-trivial in the worst frame (may indicate cache churn).",
                    "evidence": {
                        "paint_text_prepare_time_us": worst.paint_text_prepare_time_us,
                        "paint_text_prepare_calls": worst.paint_text_prepare_calls,
                        "paint_text_prepare_us_per_call": per_call,
                        "reasons": {
                            "blob_missing": worst.paint_text_prepare_reason_blob_missing,
                            "scale_changed": worst.paint_text_prepare_reason_scale_changed,
                            "text_changed": worst.paint_text_prepare_reason_text_changed,
                            "rich_changed": worst.paint_text_prepare_reason_rich_changed,
                            "style_changed": worst.paint_text_prepare_reason_style_changed,
                            "wrap_changed": worst.paint_text_prepare_reason_wrap_changed,
                            "overflow_changed": worst.paint_text_prepare_reason_overflow_changed,
                            "width_changed": worst.paint_text_prepare_reason_width_changed,
                            "font_stack_changed": worst.paint_text_prepare_reason_font_stack_changed,
                        },
                    }
                }));
            }
        }

        // renderer.upload_churn
        let upload_bytes = worst
            .renderer_text_atlas_upload_bytes
            .saturating_add(worst.renderer_svg_upload_bytes)
            .saturating_add(worst.renderer_image_upload_bytes)
            .saturating_add(worst.renderer_uniform_bytes)
            .saturating_add(worst.renderer_instance_bytes)
            .saturating_add(worst.renderer_vertex_bytes);
        if upload_bytes >= 1_000_000
            || worst.renderer_text_atlas_evicted_pages > 0
            || worst.renderer_svg_raster_budget_evictions > 0
            || worst.renderer_intermediate_pool_evictions > 0
        {
            out.push(json!({
                "code": "renderer.upload_churn",
                "severity": "info",
                "message": "Renderer uploads/evictions are present in the worst frame (may indicate cache pressure or invalidation churn).",
                "evidence": {
                    "upload_bytes_total": upload_bytes,
                    "renderer_uniform_bytes": worst.renderer_uniform_bytes,
                    "renderer_instance_bytes": worst.renderer_instance_bytes,
                    "renderer_vertex_bytes": worst.renderer_vertex_bytes,
                    "renderer_text_atlas_upload_bytes": worst.renderer_text_atlas_upload_bytes,
                    "renderer_svg_upload_bytes": worst.renderer_svg_upload_bytes,
                    "renderer_image_upload_bytes": worst.renderer_image_upload_bytes,
                    "renderer_text_atlas_evicted_pages": worst.renderer_text_atlas_evicted_pages,
                    "renderer_svg_raster_budget_evictions": worst.renderer_svg_raster_budget_evictions,
                    "renderer_intermediate_pool_evictions": worst.renderer_intermediate_pool_evictions,
                }
            }));
        }

        // renderer.external_import_ingest_fallbacks
        //
        // This is intentionally an info-level hint. Many targets (notably wasm/WebGPU today) will
        // legitimately fall back from a requested zero/low-copy strategy to a copy-based path.
        // The purpose is to make this visible in triage/perf bundles so baselines can be
        // interpreted correctly and regressions can be gated when desired.
        if worst.renderer_render_target_updates_ingest_fallbacks > 0 {
            out.push(json!({
                "code": "renderer.external_import_ingest_fallbacks",
                "severity": "info",
                "message": "Imported render target ingestion fell back from the requested strategy (requested != effective).",
                "evidence": {
                    "render_target_updates_ingest_fallbacks": worst.renderer_render_target_updates_ingest_fallbacks,
                    "render_target_updates_requested": {
                        "unknown": worst.renderer_render_target_updates_requested_ingest_unknown,
                        "owned": worst.renderer_render_target_updates_requested_ingest_owned,
                        "external_zero_copy": worst.renderer_render_target_updates_requested_ingest_external_zero_copy,
                        "gpu_copy": worst.renderer_render_target_updates_requested_ingest_gpu_copy,
                        "cpu_upload": worst.renderer_render_target_updates_requested_ingest_cpu_upload,
                    },
                    "render_target_updates_effective": {
                        "unknown": worst.renderer_render_target_updates_ingest_unknown,
                        "owned": worst.renderer_render_target_updates_ingest_owned,
                        "external_zero_copy": worst.renderer_render_target_updates_ingest_external_zero_copy,
                        "gpu_copy": worst.renderer_render_target_updates_ingest_gpu_copy,
                        "cpu_upload": worst.renderer_render_target_updates_ingest_cpu_upload,
                    },
                    "viewport_draw_calls": worst.renderer_viewport_draw_calls,
                    "viewport_draw_calls_by_ingest": {
                        "unknown": worst.renderer_viewport_draw_calls_ingest_unknown,
                        "owned": worst.renderer_viewport_draw_calls_ingest_owned,
                        "external_zero_copy": worst.renderer_viewport_draw_calls_ingest_external_zero_copy,
                        "gpu_copy": worst.renderer_viewport_draw_calls_ingest_gpu_copy,
                        "cpu_upload": worst.renderer_viewport_draw_calls_ingest_cpu_upload,
                    },
                }
            }));
        }

        // renderer.custom_effect_v1_requested_but_skipped
        if worst.renderer_custom_effect_v1_steps_requested > 0
            && worst.renderer_custom_effect_v1_passes_emitted == 0
        {
            let min_budget_for_two_full_targets_bytes = worst
                .renderer_intermediate_full_target_bytes
                .saturating_mul(2);
            out.push(json!({
                "code": "renderer.custom_effect_v1_requested_but_skipped",
                "severity": "warn",
                "message": "CustomEffectV1 was requested but no CustomEffect passes were emitted in the worst frame (likely skipped due to intermediate budget / target constraints).",
                "evidence": {
                    "custom_effect_v1_steps_requested": worst.renderer_custom_effect_v1_steps_requested,
                    "custom_effect_v1_passes_emitted": worst.renderer_custom_effect_v1_passes_emitted,
                    "renderer_intermediate_budget_bytes": worst.renderer_intermediate_budget_bytes,
                    "renderer_intermediate_full_target_bytes": worst.renderer_intermediate_full_target_bytes,
                    "renderer_render_plan_effect_chain_budget_samples": worst.renderer_render_plan_effect_chain_budget_samples,
                    "renderer_render_plan_effect_chain_effective_budget_min_bytes": worst.renderer_render_plan_effect_chain_effective_budget_min_bytes,
                    "renderer_render_plan_effect_chain_effective_budget_max_bytes": worst.renderer_render_plan_effect_chain_effective_budget_max_bytes,
                    "renderer_render_plan_effect_chain_other_live_max_bytes": worst.renderer_render_plan_effect_chain_other_live_max_bytes,
                    "renderer_render_plan_custom_effect_chain_budget_samples": worst.renderer_render_plan_custom_effect_chain_budget_samples,
                    "renderer_render_plan_custom_effect_chain_effective_budget_min_bytes": worst.renderer_render_plan_custom_effect_chain_effective_budget_min_bytes,
                    "renderer_render_plan_custom_effect_chain_effective_budget_max_bytes": worst.renderer_render_plan_custom_effect_chain_effective_budget_max_bytes,
                    "renderer_render_plan_custom_effect_chain_other_live_max_bytes": worst.renderer_render_plan_custom_effect_chain_other_live_max_bytes,
                    "renderer_render_plan_custom_effect_chain_base_required_max_bytes": worst.renderer_render_plan_custom_effect_chain_base_required_max_bytes,
                    "renderer_render_plan_custom_effect_chain_optional_required_max_bytes": worst.renderer_render_plan_custom_effect_chain_optional_required_max_bytes,
                    "renderer_render_plan_custom_effect_chain_base_required_full_targets_max": worst.renderer_render_plan_custom_effect_chain_base_required_full_targets_max,
                    "renderer_render_plan_custom_effect_chain_optional_mask_max_bytes": worst.renderer_render_plan_custom_effect_chain_optional_mask_max_bytes,
                    "renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes": worst.renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes,
                    "custom_effect_chain_effective_budget_bytes": custom_effect_chain_effective_budget_bytes,
                    "custom_effect_chain_base_required_bytes": custom_effect_chain_base_required_bytes,
                    "custom_effect_chain_optional_required_bytes": custom_effect_chain_optional_required_bytes,
                    "custom_effect_chain_optional_mask_bytes": custom_effect_chain_optional_mask_bytes,
                    "custom_effect_chain_headroom_before_optional_bytes": custom_effect_chain_headroom_before_optional_bytes,
                    "custom_effect_chain_headroom_after_mask_bytes": custom_effect_chain_headroom_after_mask_bytes,
                    "custom_effect_chain_headroom_after_optional_bytes": custom_effect_chain_headroom_after_optional_bytes,
                    "min_budget_for_two_full_targets_bytes": min_budget_for_two_full_targets_bytes,
                    "renderer_intermediate_peak_in_use_bytes": worst.renderer_intermediate_peak_in_use_bytes,
                }
            }));
        }

        // renderer.custom_effect_v2_requested_but_skipped
        if worst.renderer_custom_effect_v2_steps_requested > 0
            && worst.renderer_custom_effect_v2_passes_emitted == 0
        {
            let min_budget_for_two_full_targets_bytes = worst
                .renderer_intermediate_full_target_bytes
                .saturating_mul(2);
            out.push(json!({
                "code": "renderer.custom_effect_v2_requested_but_skipped",
                "severity": "warn",
                "message": "CustomEffectV2 was requested but no CustomEffectV2 passes were emitted in the worst frame (likely skipped due to intermediate budget / target constraints).",
                "evidence": {
                    "custom_effect_v2_steps_requested": worst.renderer_custom_effect_v2_steps_requested,
                    "custom_effect_v2_passes_emitted": worst.renderer_custom_effect_v2_passes_emitted,
                    "renderer_intermediate_budget_bytes": worst.renderer_intermediate_budget_bytes,
                    "renderer_intermediate_full_target_bytes": worst.renderer_intermediate_full_target_bytes,
                    "renderer_render_plan_effect_chain_budget_samples": worst.renderer_render_plan_effect_chain_budget_samples,
                    "renderer_render_plan_effect_chain_effective_budget_min_bytes": worst.renderer_render_plan_effect_chain_effective_budget_min_bytes,
                    "renderer_render_plan_effect_chain_effective_budget_max_bytes": worst.renderer_render_plan_effect_chain_effective_budget_max_bytes,
                    "renderer_render_plan_effect_chain_other_live_max_bytes": worst.renderer_render_plan_effect_chain_other_live_max_bytes,
                    "renderer_render_plan_custom_effect_chain_budget_samples": worst.renderer_render_plan_custom_effect_chain_budget_samples,
                    "renderer_render_plan_custom_effect_chain_effective_budget_min_bytes": worst.renderer_render_plan_custom_effect_chain_effective_budget_min_bytes,
                    "renderer_render_plan_custom_effect_chain_effective_budget_max_bytes": worst.renderer_render_plan_custom_effect_chain_effective_budget_max_bytes,
                    "renderer_render_plan_custom_effect_chain_other_live_max_bytes": worst.renderer_render_plan_custom_effect_chain_other_live_max_bytes,
                    "renderer_render_plan_custom_effect_chain_base_required_max_bytes": worst.renderer_render_plan_custom_effect_chain_base_required_max_bytes,
                    "renderer_render_plan_custom_effect_chain_optional_required_max_bytes": worst.renderer_render_plan_custom_effect_chain_optional_required_max_bytes,
                    "renderer_render_plan_custom_effect_chain_base_required_full_targets_max": worst.renderer_render_plan_custom_effect_chain_base_required_full_targets_max,
                    "renderer_render_plan_custom_effect_chain_optional_mask_max_bytes": worst.renderer_render_plan_custom_effect_chain_optional_mask_max_bytes,
                    "renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes": worst.renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes,
                    "custom_effect_chain_effective_budget_bytes": custom_effect_chain_effective_budget_bytes,
                    "custom_effect_chain_base_required_bytes": custom_effect_chain_base_required_bytes,
                    "custom_effect_chain_optional_required_bytes": custom_effect_chain_optional_required_bytes,
                    "custom_effect_chain_optional_mask_bytes": custom_effect_chain_optional_mask_bytes,
                    "custom_effect_chain_headroom_before_optional_bytes": custom_effect_chain_headroom_before_optional_bytes,
                    "custom_effect_chain_headroom_after_mask_bytes": custom_effect_chain_headroom_after_mask_bytes,
                    "custom_effect_chain_headroom_after_optional_bytes": custom_effect_chain_headroom_after_optional_bytes,
                    "min_budget_for_two_full_targets_bytes": min_budget_for_two_full_targets_bytes,
                    "renderer_intermediate_peak_in_use_bytes": worst.renderer_intermediate_peak_in_use_bytes,
                }
            }));
        }

        // renderer.custom_effect_v2_user_image_incompatible_fallbacks
        //
        // CustomEffectV2's ABI requires filterable sampled textures for the user image. When a
        // non-filterable (or otherwise incompatible) image is provided, the renderer binds a
        // deterministic fallback (1x1 transparent) to avoid wgpu validation errors.
        if worst.renderer_custom_effect_v2_user_image_incompatible_fallbacks > 0 {
            out.push(json!({
                "code": "renderer.custom_effect_v2_user_image_incompatible_fallbacks",
                "severity": "warn",
                "message": "CustomEffectV2 bound the fallback user image due to incompatible input formats in the worst frame.",
                "evidence": {
                    "custom_effect_v2_steps_requested": worst.renderer_custom_effect_v2_steps_requested,
                    "custom_effect_v2_passes_emitted": worst.renderer_custom_effect_v2_passes_emitted,
                    "custom_effect_v2_user_image_incompatible_fallbacks": worst.renderer_custom_effect_v2_user_image_incompatible_fallbacks,
                }
            }));
        }

        // renderer.custom_effect_v3_requested_but_skipped
        //
        // This catches the case where the UI requested CustomEffectV3 (effect chains include a
        // CustomV3 step) but the render plan compiler did not emit any CustomEffectV3 passes for
        // the frame. This is usually explained by intermediate budget pressure or target
        // exhaustion preventing the pass from being scheduled at all (so downstream source-level
        // degradation counters remain at 0).
        if worst.renderer_custom_effect_v3_steps_requested > 0
            && worst.renderer_custom_effect_v3_passes_emitted == 0
        {
            let min_budget_for_two_full_targets_bytes = worst
                .renderer_intermediate_full_target_bytes
                .saturating_mul(2);
            out.push(json!({
                "code": "renderer.custom_effect_v3_requested_but_skipped",
                "severity": "warn",
                "message": "CustomEffectV3 was requested but no CustomEffectV3 passes were emitted in the worst frame (likely skipped due to intermediate budget / target constraints).",
                "evidence": {
                    "custom_effect_v3_steps_requested": worst.renderer_custom_effect_v3_steps_requested,
                    "custom_effect_v3_passes_emitted": worst.renderer_custom_effect_v3_passes_emitted,
                    "renderer_intermediate_budget_bytes": worst.renderer_intermediate_budget_bytes,
                    "renderer_intermediate_full_target_bytes": worst.renderer_intermediate_full_target_bytes,
                    "renderer_render_plan_effect_chain_budget_samples": worst.renderer_render_plan_effect_chain_budget_samples,
                    "renderer_render_plan_effect_chain_effective_budget_min_bytes": worst.renderer_render_plan_effect_chain_effective_budget_min_bytes,
                    "renderer_render_plan_effect_chain_effective_budget_max_bytes": worst.renderer_render_plan_effect_chain_effective_budget_max_bytes,
                    "renderer_render_plan_effect_chain_other_live_max_bytes": worst.renderer_render_plan_effect_chain_other_live_max_bytes,
                    "renderer_render_plan_custom_effect_chain_budget_samples": worst.renderer_render_plan_custom_effect_chain_budget_samples,
                    "renderer_render_plan_custom_effect_chain_effective_budget_min_bytes": worst.renderer_render_plan_custom_effect_chain_effective_budget_min_bytes,
                    "renderer_render_plan_custom_effect_chain_effective_budget_max_bytes": worst.renderer_render_plan_custom_effect_chain_effective_budget_max_bytes,
                    "renderer_render_plan_custom_effect_chain_other_live_max_bytes": worst.renderer_render_plan_custom_effect_chain_other_live_max_bytes,
                    "renderer_render_plan_custom_effect_chain_base_required_max_bytes": worst.renderer_render_plan_custom_effect_chain_base_required_max_bytes,
                    "renderer_render_plan_custom_effect_chain_optional_required_max_bytes": worst.renderer_render_plan_custom_effect_chain_optional_required_max_bytes,
                    "renderer_render_plan_custom_effect_chain_base_required_full_targets_max": worst.renderer_render_plan_custom_effect_chain_base_required_full_targets_max,
                    "renderer_render_plan_custom_effect_chain_optional_mask_max_bytes": worst.renderer_render_plan_custom_effect_chain_optional_mask_max_bytes,
                    "renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes": worst.renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes,
                    "custom_effect_chain_effective_budget_bytes": custom_effect_chain_effective_budget_bytes,
                    "custom_effect_chain_base_required_bytes": custom_effect_chain_base_required_bytes,
                    "custom_effect_chain_optional_required_bytes": custom_effect_chain_optional_required_bytes,
                    "custom_effect_chain_optional_mask_bytes": custom_effect_chain_optional_mask_bytes,
                    "custom_effect_chain_headroom_before_optional_bytes": custom_effect_chain_headroom_before_optional_bytes,
                    "custom_effect_chain_headroom_after_mask_bytes": custom_effect_chain_headroom_after_mask_bytes,
                    "custom_effect_chain_headroom_after_optional_bytes": custom_effect_chain_headroom_after_optional_bytes,
                    "min_budget_for_two_full_targets_bytes": min_budget_for_two_full_targets_bytes,
                    "renderer_intermediate_peak_in_use_bytes": worst.renderer_intermediate_peak_in_use_bytes,
                }
            }));
        }

        // renderer.custom_effect_v3_user_image_incompatible_fallbacks
        //
        // CustomEffectV3's ABI requires filterable sampled textures for `user0` / `user1`. When
        // incompatible images are provided, the renderer binds a deterministic fallback (1x1
        // transparent) to keep bind group creation valid and deterministic.
        if worst.renderer_custom_effect_v3_user0_image_incompatible_fallbacks > 0
            || worst.renderer_custom_effect_v3_user1_image_incompatible_fallbacks > 0
        {
            out.push(json!({
                "code": "renderer.custom_effect_v3_user_image_incompatible_fallbacks",
                "severity": "warn",
                "message": "CustomEffectV3 bound fallback user images due to incompatible input formats in the worst frame.",
                "evidence": {
                    "custom_effect_v3_steps_requested": worst.renderer_custom_effect_v3_steps_requested,
                    "custom_effect_v3_passes_emitted": worst.renderer_custom_effect_v3_passes_emitted,
                    "custom_effect_v3_user0_image_incompatible_fallbacks": worst.renderer_custom_effect_v3_user0_image_incompatible_fallbacks,
                    "custom_effect_v3_user1_image_incompatible_fallbacks": worst.renderer_custom_effect_v3_user1_image_incompatible_fallbacks,
                }
            }));
        }

        // renderer.custom_effect_v3_sources_degraded
        //
        // These are correctness/ceiling signals: for liquid-glass-like looks, losing `src_raw` or
        // degrading the pyramid to 1 level can materially change the appearance.
        let worst_v3_pyr_degraded = worst
            .renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_zero
            .saturating_add(
                worst.renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_insufficient,
            );
        if worst_v3_pyr_degraded > 0 {
            let custom_effect_v3_pyramid_required_bytes_levels2_est = worst
                .renderer_intermediate_full_target_bytes
                .saturating_add(worst.renderer_intermediate_full_target_bytes / 4);
            let custom_effect_v3_pyramid_would_fit_levels2_est =
                custom_effect_chain_headroom_after_mask_bytes
                    .map(|h| h >= custom_effect_v3_pyramid_required_bytes_levels2_est);
            out.push(json!({
                "code": "renderer.custom_effect_v3_pyramid_degraded_to_one",
                "severity": "warn",
                "message": "CustomEffectV3 pyramid was degraded to 1 level in the worst frame (budget pressure).",
                "evidence": {
                    "custom_effect_v3_sources_pyramid_requested": worst.renderer_custom_effect_v3_sources_pyramid_requested,
                    "custom_effect_v3_sources_pyramid_applied_levels_ge2": worst.renderer_custom_effect_v3_sources_pyramid_applied_levels_ge2,
                    "custom_effect_v3_sources_pyramid_degraded_to_one_budget_zero": worst.renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_zero,
                    "custom_effect_v3_sources_pyramid_degraded_to_one_budget_insufficient": worst.renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_insufficient,
                    "renderer_intermediate_budget_bytes": worst.renderer_intermediate_budget_bytes,
                    "renderer_intermediate_full_target_bytes": worst.renderer_intermediate_full_target_bytes,
                    "custom_effect_v3_pyramid_required_bytes_levels2_est": custom_effect_v3_pyramid_required_bytes_levels2_est,
                    "custom_effect_v3_pyramid_would_fit_levels2_est": custom_effect_v3_pyramid_would_fit_levels2_est,
                    "renderer_render_plan_effect_chain_budget_samples": worst.renderer_render_plan_effect_chain_budget_samples,
                    "renderer_render_plan_effect_chain_effective_budget_min_bytes": worst.renderer_render_plan_effect_chain_effective_budget_min_bytes,
                    "renderer_render_plan_effect_chain_effective_budget_max_bytes": worst.renderer_render_plan_effect_chain_effective_budget_max_bytes,
                    "renderer_render_plan_effect_chain_other_live_max_bytes": worst.renderer_render_plan_effect_chain_other_live_max_bytes,
                    "renderer_render_plan_custom_effect_chain_budget_samples": worst.renderer_render_plan_custom_effect_chain_budget_samples,
                    "renderer_render_plan_custom_effect_chain_effective_budget_min_bytes": worst.renderer_render_plan_custom_effect_chain_effective_budget_min_bytes,
                    "renderer_render_plan_custom_effect_chain_effective_budget_max_bytes": worst.renderer_render_plan_custom_effect_chain_effective_budget_max_bytes,
                    "renderer_render_plan_custom_effect_chain_other_live_max_bytes": worst.renderer_render_plan_custom_effect_chain_other_live_max_bytes,
                    "renderer_render_plan_custom_effect_chain_base_required_max_bytes": worst.renderer_render_plan_custom_effect_chain_base_required_max_bytes,
                    "renderer_render_plan_custom_effect_chain_optional_required_max_bytes": worst.renderer_render_plan_custom_effect_chain_optional_required_max_bytes,
                    "renderer_render_plan_custom_effect_chain_base_required_full_targets_max": worst.renderer_render_plan_custom_effect_chain_base_required_full_targets_max,
                    "renderer_render_plan_custom_effect_chain_optional_mask_max_bytes": worst.renderer_render_plan_custom_effect_chain_optional_mask_max_bytes,
                    "renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes": worst.renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes,
                    "custom_effect_chain_effective_budget_bytes": custom_effect_chain_effective_budget_bytes,
                    "custom_effect_chain_base_required_bytes": custom_effect_chain_base_required_bytes,
                    "custom_effect_chain_optional_required_bytes": custom_effect_chain_optional_required_bytes,
                    "custom_effect_chain_optional_mask_bytes": custom_effect_chain_optional_mask_bytes,
                    "custom_effect_chain_headroom_before_optional_bytes": custom_effect_chain_headroom_before_optional_bytes,
                    "custom_effect_chain_headroom_after_mask_bytes": custom_effect_chain_headroom_after_mask_bytes,
                    "custom_effect_chain_headroom_after_optional_bytes": custom_effect_chain_headroom_after_optional_bytes,
                    "renderer_intermediate_peak_in_use_bytes": worst.renderer_intermediate_peak_in_use_bytes,
                }
            }));
        }

        if worst.renderer_custom_effect_v3_sources_raw_requested > 0
            && worst.renderer_custom_effect_v3_sources_raw_aliased_to_src > 0
        {
            out.push(json!({
                "code": "renderer.custom_effect_v3_raw_aliased_to_src",
                "severity": "info",
                "message": "CustomEffectV3 `src_raw` was aliased to `src` in the worst frame (raw snapshot unavailable).",
                "evidence": {
                    "custom_effect_v3_sources_raw_requested": worst.renderer_custom_effect_v3_sources_raw_requested,
                    "custom_effect_v3_sources_raw_distinct": worst.renderer_custom_effect_v3_sources_raw_distinct,
                    "custom_effect_v3_sources_raw_aliased_to_src": worst.renderer_custom_effect_v3_sources_raw_aliased_to_src,
                }
            }));
        }

        // renderer.custom_effect_v3_pyramid_cache_miss_heavy
        //
        // This is a perf/efficiency signal: when multiple CustomV3 passes in the same frame
        // request a pyramid (levels >= 2), the renderer has a frame-local reuse cache keyed by
        // (src_raw target, size, format, levels, src_raw epoch). A high miss ratio suggests that
        // `src_raw` is not stable across the passes, or that the pyramid cannot be reused due to
        // intervening writes.
        let custom_effect_v3_pyramid_cache_hits =
            worst.renderer_custom_effect_v3_pyramid_cache_hits;
        let custom_effect_v3_pyramid_cache_misses =
            worst.renderer_custom_effect_v3_pyramid_cache_misses;
        let custom_effect_v3_pyramid_cache_total = custom_effect_v3_pyramid_cache_hits
            .saturating_add(custom_effect_v3_pyramid_cache_misses);
        if worst.renderer_custom_effect_v3_sources_pyramid_applied_levels_ge2 > 0
            && custom_effect_v3_pyramid_cache_total >= 2
            && custom_effect_v3_pyramid_cache_misses > custom_effect_v3_pyramid_cache_hits
        {
            let miss_pct = ratio_pct(
                custom_effect_v3_pyramid_cache_misses,
                custom_effect_v3_pyramid_cache_total,
            );
            let severity = if miss_pct >= 75.0 { "warn" } else { "info" };
            out.push(json!({
                "code": "renderer.custom_effect_v3_pyramid_cache_miss_heavy",
                "severity": severity,
                "message": "CustomEffectV3 pyramid cache misses dominate in the worst frame (pyramid rebuilds likely).",
                "evidence": {
                    "custom_effect_v3_sources_pyramid_requested": worst.renderer_custom_effect_v3_sources_pyramid_requested,
                    "custom_effect_v3_sources_pyramid_applied_levels_ge2": worst.renderer_custom_effect_v3_sources_pyramid_applied_levels_ge2,
                    "custom_effect_v3_pyramid_cache_hits": custom_effect_v3_pyramid_cache_hits,
                    "custom_effect_v3_pyramid_cache_misses": custom_effect_v3_pyramid_cache_misses,
                    "custom_effect_v3_pyramid_cache_total": custom_effect_v3_pyramid_cache_total,
                    "custom_effect_v3_pyramid_cache_miss_pct": miss_pct,
                }
            }));
        }

        // renderer.backdrop_source_group_degraded
        let worst_bsg_raw_degraded = worst
            .renderer_backdrop_source_groups_raw_degraded_budget_zero
            .saturating_add(worst.renderer_backdrop_source_groups_raw_degraded_budget_insufficient)
            .saturating_add(worst.renderer_backdrop_source_groups_raw_degraded_target_exhausted);
        if worst.renderer_backdrop_source_groups_requested > 0 && worst_bsg_raw_degraded > 0 {
            out.push(json!({
                "code": "renderer.backdrop_source_groups_raw_degraded",
                "severity": "warn",
                "message": "Backdrop source group raw snapshot was degraded in the worst frame (sharing ceiling reduced).",
                "evidence": {
                    "backdrop_source_groups_requested": worst.renderer_backdrop_source_groups_requested,
                    "backdrop_source_groups_applied_raw": worst.renderer_backdrop_source_groups_applied_raw,
                    "backdrop_source_groups_raw_degraded_budget_zero": worst.renderer_backdrop_source_groups_raw_degraded_budget_zero,
                    "backdrop_source_groups_raw_degraded_budget_insufficient": worst.renderer_backdrop_source_groups_raw_degraded_budget_insufficient,
                    "backdrop_source_groups_raw_degraded_target_exhausted": worst.renderer_backdrop_source_groups_raw_degraded_target_exhausted,
                }
            }));
        }

        let worst_bsg_pyr_degraded = worst
            .renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_zero
            .saturating_add(
                worst.renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_insufficient,
            )
            .saturating_add(worst.renderer_backdrop_source_groups_pyramid_skipped_raw_unavailable);
        if worst.renderer_backdrop_source_groups_pyramid_requested > 0 && worst_bsg_pyr_degraded > 0
        {
            out.push(json!({
                "code": "renderer.backdrop_source_groups_pyramid_degraded",
                "severity": "info",
                "message": "Backdrop source group pyramid sharing was degraded in the worst frame.",
                "evidence": {
                    "backdrop_source_groups_pyramid_requested": worst.renderer_backdrop_source_groups_pyramid_requested,
                    "backdrop_source_groups_pyramid_applied_levels_ge2": worst.renderer_backdrop_source_groups_pyramid_applied_levels_ge2,
                    "backdrop_source_groups_pyramid_degraded_to_one_budget_zero": worst.renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_zero,
                    "backdrop_source_groups_pyramid_degraded_to_one_budget_insufficient": worst.renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_insufficient,
                    "backdrop_source_groups_pyramid_skipped_raw_unavailable": worst.renderer_backdrop_source_groups_pyramid_skipped_raw_unavailable,
                }
            }));
        }

        // view_cache.cache_key_mismatch
        if worst.view_cache_roots_cache_key_mismatch > 0 {
            let examples: Vec<serde_json::Value> = worst
                .top_cache_roots
                .iter()
                .filter(|r| r.reuse_reason.as_deref() == Some("cache_key_mismatch"))
                .take(3)
                .map(|r| {
                    json!({
                        "root_node": r.root_node,
                        "element": r.element,
                        "element_path": r.element_path.clone(),
                        "reused": r.reused,
                        "layout_dependency": r.layout_dependency.clone(),
                        "paint_replayed_ops": r.paint_replayed_ops,
                        "reuse_reason": r.reuse_reason.clone(),
                        "root_role": r.root_role.clone(),
                        "root_test_id": r.root_test_id.clone(),
                    })
                })
                .collect();

            out.push(json!({
                "code": "view_cache.cache_key_mismatch",
                "severity": "warn",
                "message": "View-cache roots were not reused due to cache key mismatches in the worst frame.",
                "evidence": {
                    "view_cache_roots_cache_key_mismatch": worst.view_cache_roots_cache_key_mismatch,
                    "view_cache_roots_total": worst.view_cache_roots_total,
                    "view_cache_roots_reused": worst.view_cache_roots_reused,
                    "examples": examples,
                }
            }));
        }

        // phase.timeline_hotspots
        if worst.layout_time_us > 0
            || worst.paint_time_us > 0
            || worst.renderer_encode_scene_us > 0
            || worst.renderer_upload_us > 0
            || worst.renderer_record_passes_us > 0
            || worst.renderer_encoder_finish_us > 0
        {
            let layout_examples: Vec<serde_json::Value> = worst
                .layout_hotspots
                .iter()
                .take(3)
                .map(|h| {
                    json!({
                        "node": h.node,
                        "element": h.element,
                        "element_kind": h.element_kind,
                        "widget_type": h.widget_type,
                        "layout_time_us": h.layout_time_us,
                        "inclusive_time_us": h.inclusive_time_us,
                        "role": h.role,
                        "test_id": h.test_id,
                        "element_path": h.element_path,
                    })
                })
                .collect();
            let layout_request_build_roots_examples: Vec<serde_json::Value> = worst
                .layout_request_build_roots
                .iter()
                .take(3)
                .map(|r| {
                    json!({
                        "root_node": r.root_node,
                        "root_kind": r.root_kind,
                        "elapsed_us": r.elapsed_us,
                        "mode": r.mode,
                        "layout_invalidated": r.layout_invalidated,
                        "subtree_layout_dirty": r.subtree_layout_dirty,
                        "subtree_layout_dirty_count": r.subtree_layout_dirty_count,
                        "descendant_layout_dirty_count": r.descendant_layout_dirty_count,
                        "needs_layout": r.needs_layout,
                        "is_translation_only": r.is_translation_only,
                        "root_role": r.root_role,
                        "root_test_id": r.root_test_id,
                    })
                })
                .collect();
            let scroll_phase_examples: Vec<serde_json::Value> = worst
                .scroll_layout_profiles
                .iter()
                .take(3)
                .map(|p| {
                    json!({
                        "node": p.node,
                        "element": p.element,
                        "axis": p.axis,
                        "pass": p.pass,
                        "total_us": p.total_us,
                        "measure_children_us": p.measure_children_us,
                        "solve_barrier_us": p.solve_barrier_us,
                        "layout_children_us": p.layout_children_us,
                        "layout_children_first_pass_us": p.layout_children_first_pass_us,
                        "phase_profiles": p.phase_profiles.iter().take(4).map(|phase| {
                            json!({
                                "phase": phase.phase,
                                "us": phase.us,
                            })
                        }).collect::<Vec<_>>(),
                        "element_path": p.element_path,
                    })
                })
                .collect();
            let paint_widget_examples: Vec<serde_json::Value> = worst
                .paint_widget_hotspots
                .iter()
                .take(3)
                .map(|h| {
                    json!({
                        "node": h.node,
                        "element": h.element,
                        "element_kind": h.element_kind,
                        "widget_type": h.widget_type,
                        "paint_time_us": h.paint_time_us,
                        "inclusive_time_us": h.inclusive_time_us,
                        "inclusive_scene_ops_delta": h.inclusive_scene_ops_delta,
                        "exclusive_scene_ops_delta": h.exclusive_scene_ops_delta,
                        "role": h.role,
                        "test_id": h.test_id,
                    })
                })
                .collect();
            let upload_bytes = worst
                .renderer_text_atlas_upload_bytes
                .saturating_add(worst.renderer_svg_upload_bytes)
                .saturating_add(worst.renderer_image_upload_bytes)
                .saturating_add(worst.renderer_uniform_bytes)
                .saturating_add(worst.renderer_instance_bytes)
                .saturating_add(worst.renderer_vertex_bytes);

            out.push(json!({
                "code": "phase.timeline_hotspots",
                "severity": "info",
                "message": "Phase timing and hotspot evidence are linked in the same summary for the worst frame.",
                "evidence": {
                    "phase_times_us": {
                        "layout": worst.layout_time_us,
                        "prepaint": worst.prepaint_time_us,
                        "paint": worst.paint_time_us,
                        "renderer_encode_scene": worst.renderer_encode_scene_us,
                        "renderer_upload": worst.renderer_upload_us,
                        "renderer_record_passes": worst.renderer_record_passes_us,
                        "renderer_encoder_finish": worst.renderer_encoder_finish_us,
                    },
                    "layout_hotspots": layout_examples,
                    "layout_request_build_roots": layout_request_build_roots_examples,
                    "scroll_phase_profiles": scroll_phase_examples,
                    "paint_widget_hotspots": paint_widget_examples,
                    "renderer_hotspots": [
                        {
                            "kind": "renderer.upload_churn",
                            "upload_bytes_total": upload_bytes,
                            "renderer_uniform_bytes": worst.renderer_uniform_bytes,
                            "renderer_instance_bytes": worst.renderer_instance_bytes,
                            "renderer_vertex_bytes": worst.renderer_vertex_bytes,
                        },
                        {
                            "kind": "renderer.record_passes",
                            "time_us": worst.renderer_record_passes_us,
                        },
                        {
                            "kind": "renderer.encoder_finish",
                            "time_us": worst.renderer_encoder_finish_us,
                        },
                        {
                            "kind": "renderer.encode_scene_text",
                            "time_us": worst.renderer_encode_scene_text_us,
                        },
                    ],
                }
            }));
        }

        out
    }

    fn triage_unit_costs(
        worst: Option<&crate::stats::BundleStatsSnapshotRow>,
    ) -> serde_json::Value {
        let Some(worst) = worst else {
            return json!({});
        };
        json!({
            "layout_engine_solve_us_per_solve": if worst.layout_engine_solves == 0 { None } else { Some(worst.layout_engine_solve_time_us / worst.layout_engine_solves) },
            "layout_engine_child_rect_us_per_query": if worst.layout_engine_child_rect_queries == 0 { None } else { Some(worst.layout_engine_child_rect_time_us / worst.layout_engine_child_rect_queries) },
            "paint_text_prepare_us_per_call": if worst.paint_text_prepare_calls == 0 { None } else { Some(worst.paint_text_prepare_time_us / (worst.paint_text_prepare_calls as u64)) },
            "layout_obs_record_us_per_model_item": if worst.layout_observation_record_models_items == 0 { None } else { Some(worst.layout_observation_record_time_us / (worst.layout_observation_record_models_items as u64)) },
            "layout_obs_record_us_per_global_item": if worst.layout_observation_record_globals_items == 0 { None } else { Some(worst.layout_observation_record_time_us / (worst.layout_observation_record_globals_items as u64)) },
        })
    }

    let generated_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64);

    let file_size_bytes = std::fs::metadata(bundle_path).ok().map(|m| m.len());

    let worst_row = report.top.first();
    let worst = worst_row.map(|row| {
	        json!({
	            "window": row.window,
	            "tick_id": row.tick_id,
	            "frame_id": row.frame_id,
	            "timestamp_unix_ms": row.timestamp_unix_ms,
	            "total_time_us": row.total_time_us,
	            "layout_time_us": row.layout_time_us,
	            "prepaint_time_us": row.prepaint_time_us,
	            "paint_time_us": row.paint_time_us,
	            "renderer_encode_scene_us": row.renderer_encode_scene_us,
	            "renderer_upload_us": row.renderer_upload_us,
            "renderer_record_passes_us": row.renderer_record_passes_us,
            "renderer_encoder_finish_us": row.renderer_encoder_finish_us,
            "renderer_prepare_text_us": row.renderer_prepare_text_us,
            "renderer_prepare_svg_us": row.renderer_prepare_svg_us,
            "renderer_uniform_bytes": row.renderer_uniform_bytes,
            "renderer_instance_bytes": row.renderer_instance_bytes,
            "renderer_vertex_bytes": row.renderer_vertex_bytes,
            "renderer_encode_scene_stack_us": row.renderer_encode_scene_stack_us,
            "renderer_encode_scene_clip_us": row.renderer_encode_scene_clip_us,
            "renderer_encode_scene_mask_us": row.renderer_encode_scene_mask_us,
            "renderer_encode_scene_effect_us": row.renderer_encode_scene_effect_us,
            "renderer_encode_scene_quad_us": row.renderer_encode_scene_quad_us,
            "renderer_encode_scene_image_us": row.renderer_encode_scene_image_us,
            "renderer_encode_scene_text_us": row.renderer_encode_scene_text_us,
            "renderer_encode_scene_path_us": row.renderer_encode_scene_path_us,
            "renderer_encode_scene_viewport_us": row.renderer_encode_scene_viewport_us,
            "renderer_encode_scene_flush_us": row.renderer_encode_scene_flush_us,
            "renderer_encode_scene_text_shadow_us": row.renderer_encode_scene_text_shadow_us,
            "renderer_encode_scene_text_setup_us": row.renderer_encode_scene_text_setup_us,
            "renderer_encode_scene_text_glyphs_us": row.renderer_encode_scene_text_glyphs_us,
            "renderer_encode_scene_text_glyph_transform_us": row
                .renderer_encode_scene_text_glyph_transform_us,
            "renderer_encode_scene_text_glyph_emit_us": row.renderer_encode_scene_text_glyph_emit_us,
            "renderer_encode_scene_text_group_flush_us": row
                .renderer_encode_scene_text_group_flush_us,
            "renderer_encode_scene_text_vertex_grow_events": row
                .renderer_encode_scene_text_vertex_grow_events,
            "renderer_encode_scene_text_transform_fast_path_glyphs": row
                .renderer_encode_scene_text_transform_fast_path_glyphs,
            "renderer_encode_scene_text_transform_generic_glyphs": row
                .renderer_encode_scene_text_transform_generic_glyphs,
            "renderer_encode_scene_stack_ops": row.renderer_encode_scene_stack_ops,
            "renderer_encode_scene_clip_ops": row.renderer_encode_scene_clip_ops,
            "renderer_encode_scene_mask_ops": row.renderer_encode_scene_mask_ops,
            "renderer_encode_scene_effect_ops": row.renderer_encode_scene_effect_ops,
            "renderer_encode_scene_quad_ops": row.renderer_encode_scene_quad_ops,
            "renderer_encode_scene_image_ops": row.renderer_encode_scene_image_ops,
            "renderer_encode_scene_text_ops": row.renderer_encode_scene_text_ops,
            "renderer_encode_scene_path_ops": row.renderer_encode_scene_path_ops,
            "renderer_encode_scene_viewport_ops": row.renderer_encode_scene_viewport_ops,
            "renderer_encode_scene_flushes": row.renderer_encode_scene_flushes,
            "layout_observation_record_time_us": row.layout_observation_record_time_us,
            "layout_observation_record_models_items": row.layout_observation_record_models_items,
            "layout_observation_record_globals_items": row.layout_observation_record_globals_items,
            "layout_engine_child_rect_queries": row.layout_engine_child_rect_queries,
            "layout_engine_child_rect_time_us": row.layout_engine_child_rect_time_us,
            "layout_engine_widget_fallback_solves": row.layout_engine_widget_fallback_solves,
            "paint_observation_record_time_us": row.paint_observation_record_time_us,
            "paint_text_prepare_time_us": row.paint_text_prepare_time_us,
            "paint_text_prepare_calls": row.paint_text_prepare_calls,
            "invalidation_walk_calls": row.invalidation_walk_calls,
            "invalidation_walk_nodes": row.invalidation_walk_nodes,
            "cache_roots": row.cache_roots,
            "cache_roots_reused": row.cache_roots_reused,
            "cache_replayed_ops": row.cache_replayed_ops,
            "top_invalidation_walks": row.top_invalidation_walks.iter().take(10).map(|w| {
                json!({
                    "root_node": w.root_node,
                    "root_element": w.root_element,
                    "walked_nodes": w.walked_nodes,
                    "kind": w.kind,
                    "source": w.source,
                    "detail": w.detail,
                    "truncated_at": w.truncated_at,
                    "root_role": w.root_role,
                    "root_test_id": w.root_test_id,
                })
            }).collect::<Vec<_>>(),
            "top_cache_roots": row.top_cache_roots.iter().take(10).map(|r| {
                json!({
                    "root_node": r.root_node,
                    "element": r.element,
                    "reused": r.reused,
                    "layout_dependency": r.layout_dependency.clone(),
                    "paint_replayed_ops": r.paint_replayed_ops,
                    "reuse_reason": r.reuse_reason,
                    "root_role": r.root_role,
                    "root_test_id": r.root_test_id,
                })
            }).collect::<Vec<_>>(),
            "layout_request_build_roots": row.layout_request_build_roots.iter().take(10).map(|r| {
                json!({
                    "root_node": r.root_node,
                    "root_kind": r.root_kind,
                    "root_element": r.root_element,
                    "root_element_kind": r.root_element_kind,
                    "elapsed_us": r.elapsed_us,
                    "mode": r.mode,
                    "had_layout_engine_node": r.had_layout_engine_node,
                    "layout_invalidated": r.layout_invalidated,
                    "subtree_layout_dirty": r.subtree_layout_dirty,
                    "subtree_layout_dirty_count": r.subtree_layout_dirty_count,
                    "descendant_layout_dirty_count": r.descendant_layout_dirty_count,
                    "needs_layout": r.needs_layout,
                    "is_translation_only": r.is_translation_only,
                    "nodes_marked_seen": r.nodes_marked_seen,
                    "root_role": r.root_role,
                    "root_test_id": r.root_test_id,
                    "dirty_descendants": r.dirty_descendants.iter().take(4).map(|d| {
                        json!({
                            "node": d.node,
                            "element": d.element,
                            "element_kind": d.element_kind,
                            "element_path": d.element_path,
                            "subtree_layout_dirty_count": d.subtree_layout_dirty_count,
                            "source_root_node": d.source_root_node,
                            "source": d.source,
                            "detail": d.detail,
                            "role": d.role,
                            "test_id": d.test_id,
                        })
                    }).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
            "scroll_layout_profiles": row.scroll_layout_profiles.iter().take(10).map(|p| {
                json!({
                    "node": p.node,
                    "element": p.element,
                    "test_id": p.test_id.as_ref().or(p.semantics_test_id.as_ref()),
                    "axis": p.axis,
                    "pass": p.pass,
                    "probe_unbounded": p.probe_unbounded,
                    "children": p.children,
                    "available_w": p.available_w,
                    "available_h": p.available_h,
                    "desired_w": p.desired_w,
                    "desired_h": p.desired_h,
                    "content_w": p.content_w,
                    "content_h": p.content_h,
                    "post_layout_extents_mode": p.post_layout_extents_mode,
                    "interactive_resize": p.interactive_resize,
                    "direct_children_layout_invalidated": p.direct_children_layout_invalidated,
                    "descendant_subtree_layout_dirty": p.descendant_subtree_layout_dirty,
                    "force_barrier_child_root_relayout": p.force_barrier_child_root_relayout,
                    "phase_profiles": p.phase_profiles.iter().map(|phase| {
                        json!({
                            "phase": phase.phase,
                            "us": phase.us,
                        })
                    }).collect::<Vec<_>>(),
                    "measure_children_us": p.measure_children_us,
                    "solve_barrier_us": p.solve_barrier_us,
                    "layout_children_us": p.layout_children_us,
                    "layout_children_first_pass_us": p.layout_children_first_pass_us,
                    "layout_child_first_pass_nodes_visited": p.layout_child_first_pass_nodes_visited,
                    "layout_child_first_pass_nodes_performed": p.layout_child_first_pass_nodes_performed,
                    "layout_child_first_pass_max_us": p.layout_child_first_pass_max_us,
                    "layout_child_first_pass_kind_profiles": p.layout_child_first_pass_kind_profiles.iter().take(6).map(|k| {
                        json!({
                            "kind": k.kind,
                            "nodes": k.nodes,
                            "self_us": k.self_us,
                            "total_us": k.total_us,
                            "max_self_us": k.max_self_us,
                            "max_total_us": k.max_total_us,
                        })
                    }).collect::<Vec<_>>(),
                    "corrected_content_relayout": p.corrected_content_relayout,
                    "layout_children_corrected_content_us": p.layout_children_corrected_content_us,
                    "layout_child_corrected_content_nodes_visited": p.layout_child_corrected_content_nodes_visited,
                    "layout_child_corrected_content_nodes_performed": p.layout_child_corrected_content_nodes_performed,
                    "layout_child_corrected_content_max_us": p.layout_child_corrected_content_max_us,
                    "layout_child_corrected_content_kind_profiles": p.layout_child_corrected_content_kind_profiles.iter().take(6).map(|k| {
                        json!({
                            "kind": k.kind,
                            "nodes": k.nodes,
                            "self_us": k.self_us,
                            "total_us": k.total_us,
                            "max_self_us": k.max_self_us,
                            "max_total_us": k.max_total_us,
                        })
                    }).collect::<Vec<_>>(),
                    "layout_child_nodes_visited": p.layout_child_nodes_visited,
                    "layout_child_nodes_performed": p.layout_child_nodes_performed,
                    "layout_child_kind_profiles": p.layout_child_kind_profiles.iter().take(6).map(|k| {
                        json!({
                            "kind": k.kind,
                            "nodes": k.nodes,
                            "self_us": k.self_us,
                            "total_us": k.total_us,
                            "max_self_us": k.max_self_us,
                            "max_total_us": k.max_total_us,
                        })
                    }).collect::<Vec<_>>(),
                    "layout_child_max_us": p.layout_child_max_us,
                    "layout_child_max_node": p.layout_child_max_node,
                    "layout_child_max_invalidated": p.layout_child_max_invalidated,
                    "layout_child_max_subtree_dirty": p.layout_child_max_subtree_dirty,
                    "layout_child_max_subtree_dirty_count": p.layout_child_max_subtree_dirty_count,
                    "layout_child_max_nodes_visited": p.layout_child_max_nodes_visited,
                    "layout_child_max_nodes_performed": p.layout_child_max_nodes_performed,
                    "layout_child_max_bounds_changed": p.layout_child_max_bounds_changed,
                    "layout_child_max_bounds_size_changed": p.layout_child_max_bounds_size_changed,
                    "layout_child_max_input_matches_before": p.layout_child_max_input_matches_before,
                    "layout_child_max_input_size_matches_before": p.layout_child_max_input_size_matches_before,
                    "total_us": p.total_us,
                    "element_path": p.element_path,
                })
            }).collect::<Vec<_>>(),
            "top_layout_engine_solves": row.top_layout_engine_solves.iter().take(4).map(|s| {
                json!({
                    "root_node": s.root_node,
                    "solve_time_us": s.solve_time_us,
                    "solve_profile": s.solve_profile.as_ref().map(|p| json!({
                        "reason": p.reason,
                        "available_w_kind": p.available_w_kind,
                        "available_h_kind": p.available_h_kind,
                        "available_w": p.available_w,
                        "available_h": p.available_h,
                        "previous_available_w_kind": p.previous_available_w_kind,
                        "previous_available_h_kind": p.previous_available_h_kind,
                        "previous_available_w": p.previous_available_w,
                        "previous_available_h": p.previous_available_h,
                        "available_w_delta": p.available_w_delta,
                        "available_h_delta": p.available_h_delta,
                        "scale_factor": p.scale_factor,
                        "previous_scale_factor": p.previous_scale_factor,
                        "scale_factor_delta": p.scale_factor_delta,
                        "previous_frame_delta": p.previous_frame_delta,
                        "batch_roots": p.batch_roots,
                        "subtree_nodes": p.subtree_nodes,
                        "flex_wrap_patch_time_us": p.flex_wrap_patch_time_us,
                        "flex_wrap_patch_visited_nodes": p.flex_wrap_patch_visited_nodes,
                        "flex_wrap_patch_wrap_nodes": p.flex_wrap_patch_wrap_nodes,
                        "flex_wrap_patch_candidate_children": p.flex_wrap_patch_candidate_children,
                        "flex_wrap_patch_probes": p.flex_wrap_patch_probes,
                        "flex_wrap_patch_mutations": p.flex_wrap_patch_mutations,
                        "flex_wrap_patch_skipped_no_wrap_descendant": p.flex_wrap_patch_skipped_no_wrap_descendant,
                    })),
                    "clean_geometry_solve_skip_rejection": s.clean_geometry_solve_skip_rejection.as_ref().map(clean_geometry_solve_skip_rejection_to_json),
                    "measure_calls": s.measure_calls,
                    "measure_cache_hits": s.measure_cache_hits,
                    "measure_time_us": s.measure_time_us,
                    "root_role": s.root_role,
                    "root_test_id": s.root_test_id,
                    "top_measures": s.top_measures.iter().take(10).map(|m| {
                        json!({
                            "node": m.node,
                            "measure_time_us": m.measure_time_us,
                            "calls": m.calls,
                            "cache_hits": m.cache_hits,
                            "element": m.element,
                            "element_kind": m.element_kind,
                            "role": m.role,
                            "test_id": m.test_id,
                        })
                    }).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        })
    });

    let trace_chrome_path = bundle_path
        .parent()
        .map(|p| p.join("trace.chrome.json"))
        .filter(|p| p.is_file())
        .map(|p| p.display().to_string());

    let source_bundle_schema_version =
        crate::compat::bundle::sniff_bundle_schema_version(bundle_path)
            .ok()
            .flatten();
    let stats_json = report.to_json();
    json!({
        "schema_version": crate::perf_schema::PERF_TRIAGE_SCHEMA_VERSION,
        "kind": crate::perf_schema::PERF_TRIAGE_KIND,
        "schema_policy": crate::perf_schema::schema_policy_json(),
        "source_bundle_schema_version": source_bundle_schema_version,
        "stats_schema_version": stats_json.get("schema_version").and_then(|v| v.as_u64()),
        "generated_unix_ms": generated_unix_ms,
        "bundle": {
            "bundle_path": bundle_path.display().to_string(),
            "bundle_dir": bundle_path.parent().map(|p| p.display().to_string()),
            "bundle_file_size_bytes": file_size_bytes,
            "trace_chrome_json_path": trace_chrome_path,
        },
        "compat": compat_summary_for_bundle_path(bundle_path),
        "params": {
            "sort": sort.as_str(),
            "top": report.top.len(),
            "warmup_frames": warmup_frames,
        },
        "stats": stats_json.clone(),
        "unit_costs": triage_unit_costs(worst_row),
        "hints": triage_hints(&stats_json, worst_row),
        "worst": worst,
    })
}
