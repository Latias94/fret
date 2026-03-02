use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::stats::{BundleStatsReport, BundleStatsSort};
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
            .saturating_add(worst.renderer_image_upload_bytes);
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
                    "min_budget_for_two_full_targets_bytes": min_budget_for_two_full_targets_bytes,
                    "renderer_intermediate_peak_in_use_bytes": worst.renderer_intermediate_peak_in_use_bytes,
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
                    "min_budget_for_two_full_targets_bytes": min_budget_for_two_full_targets_bytes,
                    "renderer_intermediate_peak_in_use_bytes": worst.renderer_intermediate_peak_in_use_bytes,
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
                        "contained_layout": r.contained_layout,
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
	            "layout_observation_record_time_us": row.layout_observation_record_time_us,
	            "layout_observation_record_models_items": row.layout_observation_record_models_items,
	            "layout_observation_record_globals_items": row.layout_observation_record_globals_items,
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
                    "contained_layout": r.contained_layout,
                    "paint_replayed_ops": r.paint_replayed_ops,
                    "reuse_reason": r.reuse_reason,
                    "root_role": r.root_role,
                    "root_test_id": r.root_test_id,
                })
            }).collect::<Vec<_>>(),
            "top_layout_engine_solves": row.top_layout_engine_solves.iter().take(4).map(|s| {
                json!({
                    "root_node": s.root_node,
                    "solve_time_us": s.solve_time_us,
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

    let stats_json = report.to_json();
    json!({
        "schema_version": 1,
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
