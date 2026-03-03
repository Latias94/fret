//! Internal registries for diag checks.
//!
//! This module starts as a small seam. The long-term goal is to move ad-hoc check wiring
//! (lint/post-run gates/perf hint gates) behind explicit registries so adding checks does not
//! require editing a giant central match statement.
//!
//! NOTE: This is tooling-only; it is not a runtime contract.

use std::path::Path;

use crate::diag_run::RunChecks;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CheckKind {
    Lint,
    Triage,
    Perf,
    Hotspots,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PostRunCheckContext<'a> {
    pub bundle_path: &'a Path,
    #[allow(dead_code)]
    pub out_dir: &'a Path,
    pub warmup_frames: u64,
}

#[derive(Clone, Copy)]
struct PostRunCheckEntry {
    #[allow(dead_code)]
    id: &'static str,
    requires_bundle_artifact: bool,
    requires_screenshots: bool,
    should_run: fn(&RunChecks) -> bool,
    run: fn(PostRunCheckContext<'_>, &RunChecks) -> Result<(), String>,
}

pub(crate) struct CheckRegistry {
    post_run_checks: &'static [PostRunCheckEntry],
}

impl CheckRegistry {
    pub(crate) fn builtin() -> Self {
        Self {
            post_run_checks: BUILTIN_POST_RUN_CHECKS,
        }
    }

    pub(crate) fn wants_post_run_checks(&self, checks: &RunChecks) -> bool {
        self.post_run_checks
            .iter()
            .any(|entry| (entry.should_run)(checks))
    }

    pub(crate) fn wants_bundle_artifact(&self, checks: &RunChecks) -> bool {
        self.post_run_checks
            .iter()
            .any(|entry| entry.requires_bundle_artifact && (entry.should_run)(checks))
    }

    pub(crate) fn wants_screenshots(&self, checks: &RunChecks) -> bool {
        self.post_run_checks
            .iter()
            .any(|entry| entry.requires_screenshots && (entry.should_run)(checks))
    }

    pub(crate) fn apply_post_run_checks(
        &self,
        ctx: PostRunCheckContext<'_>,
        checks: &RunChecks,
    ) -> Result<(), String> {
        for entry in self.post_run_checks {
            if (entry.should_run)(checks) {
                (entry.run)(ctx, checks)?;
            }
        }
        Ok(())
    }
}

const BUILTIN_POST_RUN_CHECKS: &[PostRunCheckEntry] = &[
    PostRunCheckEntry {
        id: "gc_sweep_liveness",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_gc_sweep_liveness,
        run: run_gc_sweep_liveness,
    },
    PostRunCheckEntry {
        id: "stale_paint_test_id",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_stale_paint_test_id,
        run: run_stale_paint_test_id,
    },
    PostRunCheckEntry {
        id: "stale_scene_test_id",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_stale_scene_test_id,
        run: run_stale_scene_test_id,
    },
    PostRunCheckEntry {
        id: "idle_no_paint_min",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_idle_no_paint_min,
        run: run_idle_no_paint_min,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_marker_present",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_marker_present,
        run: run_ui_gallery_code_editor_torture_marker_present,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_undo_redo",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_undo_redo,
        run: run_ui_gallery_code_editor_torture_undo_redo,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_geom_fallbacks_low",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_geom_fallbacks_low,
        run: run_ui_gallery_code_editor_torture_geom_fallbacks_low,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_soft_wrap_toggle_stable",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable,
        run: run_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_word_boundary",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_word_boundary,
        run: run_ui_gallery_markdown_editor_source_word_boundary,
    },
    PostRunCheckEntry {
        id: "ui_gallery_web_ime_bridge_enabled",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_web_ime_bridge_enabled,
        run: run_ui_gallery_web_ime_bridge_enabled,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_line_boundary_triple_click",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_line_boundary_triple_click,
        run: run_ui_gallery_markdown_editor_source_line_boundary_triple_click,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_a11y_composition",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_a11y_composition,
        run: run_ui_gallery_markdown_editor_source_a11y_composition,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_a11y_composition_soft_wrap",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap,
        run: run_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable,
        run: run_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_folds_toggle_stable",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_folds_toggle_stable,
        run: run_ui_gallery_markdown_editor_source_folds_toggle_stable,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds,
        run: run_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_folds_placeholder_present",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_folds_placeholder_present,
        run: run_ui_gallery_markdown_editor_source_folds_placeholder_present,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap,
        run: run_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit,
        run: run_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_inlays_toggle_stable",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_inlays_toggle_stable,
        run: run_ui_gallery_markdown_editor_source_inlays_toggle_stable,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_inlays_caret_navigation_stable",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable,
        run: run_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_inlays_present",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_inlays_present,
        run: run_ui_gallery_markdown_editor_source_inlays_present,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap,
        run: run_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit,
        run: run_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_folds_placeholder_present",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_folds_placeholder_present,
        run: run_ui_gallery_code_editor_torture_folds_placeholder_present,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap,
        run: run_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit,
        run: run_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_inlays_present",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_inlays_present,
        run: run_ui_gallery_code_editor_torture_inlays_present,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_inlays_present_under_soft_wrap",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap,
        run: run_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit,
        run: run_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed,
        run: run_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed,
        run: run_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped,
        run: run_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations,
        run: run_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed,
        run: run_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped,
        run: run_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations,
        run: run_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed,
        run: run_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll,
        run: run_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection,
        run: run_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_word_boundary",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_word_boundary,
        run: run_ui_gallery_code_editor_word_boundary,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_a11y_selection",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_a11y_selection,
        run: run_ui_gallery_code_editor_a11y_selection,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_a11y_composition",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_a11y_composition,
        run: run_ui_gallery_code_editor_a11y_composition,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_a11y_selection_wrap",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_a11y_selection_wrap,
        run: run_ui_gallery_code_editor_a11y_selection_wrap,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_a11y_composition_wrap",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_a11y_composition_wrap,
        run: run_ui_gallery_code_editor_a11y_composition_wrap,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_a11y_composition_wrap_scroll",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_a11y_composition_wrap_scroll,
        run: run_ui_gallery_code_editor_a11y_composition_wrap_scroll,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_a11y_composition_drag",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_a11y_composition_drag,
        run: run_ui_gallery_code_editor_a11y_composition_drag,
    },
    PostRunCheckEntry {
        id: "ui_gallery_code_editor_torture_read_only_blocks_edits",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_code_editor_torture_read_only_blocks_edits,
        run: run_ui_gallery_code_editor_torture_read_only_blocks_edits,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_read_only_blocks_edits",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_read_only_blocks_edits,
        run: run_ui_gallery_markdown_editor_source_read_only_blocks_edits,
    },
    PostRunCheckEntry {
        id: "ui_gallery_markdown_editor_source_disabled_blocks_edits",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_markdown_editor_source_disabled_blocks_edits,
        run: run_ui_gallery_markdown_editor_source_disabled_blocks_edits,
    },
    PostRunCheckEntry {
        id: "semantics_changed_repainted",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_semantics_changed_repainted,
        run: run_semantics_changed_repainted,
    },
    PostRunCheckEntry {
        id: "ui_gallery_text_rescan_system_fonts_font_stack_key_bumps",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps,
        run: run_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps,
    },
    PostRunCheckEntry {
        id: "ui_gallery_text_fallback_policy_key_bumps_on_settings_change",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_text_fallback_policy_key_bumps_on_settings_change,
        run: run_ui_gallery_text_fallback_policy_key_bumps_on_settings_change,
    },
    PostRunCheckEntry {
        id: "ui_gallery_text_fallback_policy_key_bumps_on_locale_change",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_text_fallback_policy_key_bumps_on_locale_change,
        run: run_ui_gallery_text_fallback_policy_key_bumps_on_locale_change,
    },
    PostRunCheckEntry {
        id: "ui_gallery_text_mixed_script_bundled_fallback_conformance",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_ui_gallery_text_mixed_script_bundled_fallback_conformance,
        run: run_ui_gallery_text_mixed_script_bundled_fallback_conformance,
    },
    PostRunCheckEntry {
        id: "notify_hotspot_file_max",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_notify_hotspot_file_max,
        run: run_notify_hotspot_file_max,
    },
    PostRunCheckEntry {
        id: "triage_hint_absent_codes",
        requires_bundle_artifact: true,
        requires_screenshots: false,
        should_run: should_run_triage_hint_absent_codes,
        run: run_triage_hint_absent_codes,
    },
    PostRunCheckEntry {
        id: "pixels_changed_test_id",
        requires_bundle_artifact: true,
        requires_screenshots: true,
        should_run: should_run_pixels_changed_test_id,
        run: run_pixels_changed_test_id,
    },
    PostRunCheckEntry {
        id: "pixels_unchanged_test_id",
        requires_bundle_artifact: true,
        requires_screenshots: true,
        should_run: should_run_pixels_unchanged_test_id,
        run: run_pixels_unchanged_test_id,
    },
];

fn should_run_gc_sweep_liveness(checks: &RunChecks) -> bool {
    checks.check_gc_sweep_liveness
}

fn run_gc_sweep_liveness(ctx: PostRunCheckContext<'_>, _checks: &RunChecks) -> Result<(), String> {
    crate::stats::check_bundle_for_gc_sweep_liveness(ctx.bundle_path, ctx.warmup_frames)
}

fn should_run_stale_paint_test_id(checks: &RunChecks) -> bool {
    checks.check_stale_paint_test_id.is_some()
}

fn run_stale_paint_test_id(ctx: PostRunCheckContext<'_>, checks: &RunChecks) -> Result<(), String> {
    let Some(test_id) = checks.check_stale_paint_test_id.as_deref() else {
        return Ok(());
    };
    crate::stats::check_bundle_for_stale_paint(
        ctx.bundle_path,
        test_id,
        checks.check_stale_paint_eps,
    )
}

fn should_run_stale_scene_test_id(checks: &RunChecks) -> bool {
    checks.check_stale_scene_test_id.is_some()
}

fn run_stale_scene_test_id(ctx: PostRunCheckContext<'_>, checks: &RunChecks) -> Result<(), String> {
    let Some(test_id) = checks.check_stale_scene_test_id.as_deref() else {
        return Ok(());
    };
    crate::stats::check_bundle_for_stale_scene(
        ctx.bundle_path,
        test_id,
        checks.check_stale_scene_eps,
    )
}

fn should_run_idle_no_paint_min(checks: &RunChecks) -> bool {
    checks.check_idle_no_paint_min.is_some()
}

fn run_idle_no_paint_min(ctx: PostRunCheckContext<'_>, checks: &RunChecks) -> Result<(), String> {
    let Some(min) = checks.check_idle_no_paint_min else {
        return Ok(());
    };
    crate::stats::check_bundle_for_idle_no_paint_min(
        ctx.bundle_path,
        ctx.out_dir,
        min,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_marker_present(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_code_editor_torture_marker_present
}

fn run_ui_gallery_code_editor_torture_marker_present(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_marker_present(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_undo_redo(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_code_editor_torture_undo_redo
}

fn run_ui_gallery_code_editor_torture_undo_redo(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_geom_fallbacks_low(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_code_editor_torture_geom_fallbacks_low
}

fn run_ui_gallery_code_editor_torture_geom_fallbacks_low(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_geom_fallbacks_low(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable
}

fn run_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_word_boundary(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_markdown_editor_source_word_boundary
}

fn run_ui_gallery_markdown_editor_source_word_boundary(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_word_boundary(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_web_ime_bridge_enabled(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_web_ime_bridge_enabled
}

fn run_ui_gallery_web_ime_bridge_enabled(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_web_ime_bridge_enabled(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_line_boundary_triple_click(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_markdown_editor_source_line_boundary_triple_click
}

fn run_ui_gallery_markdown_editor_source_line_boundary_triple_click(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_line_boundary_triple_click(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_a11y_composition(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_markdown_editor_source_a11y_composition
}

fn run_ui_gallery_markdown_editor_source_a11y_composition(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap
}

fn run_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable
}

fn run_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_folds_toggle_stable(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_markdown_editor_source_folds_toggle_stable
}

fn run_ui_gallery_markdown_editor_source_folds_toggle_stable(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds
}

fn run_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_folds_placeholder_present(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_markdown_editor_source_folds_placeholder_present
}

fn run_ui_gallery_markdown_editor_source_folds_placeholder_present(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap
}

fn run_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit
}

fn run_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_inlays_toggle_stable(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_markdown_editor_source_inlays_toggle_stable
}

fn run_ui_gallery_markdown_editor_source_inlays_toggle_stable(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable
}

fn run_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_inlays_present(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_markdown_editor_source_inlays_present
}

fn run_ui_gallery_markdown_editor_source_inlays_present(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_inlays_present(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap
}

fn run_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit
}

fn run_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_folds_placeholder_present(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_code_editor_torture_folds_placeholder_present
}

fn run_ui_gallery_code_editor_torture_folds_placeholder_present(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap
}

fn run_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit
}

fn run_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_inlays_present(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_code_editor_torture_inlays_present
}

fn run_ui_gallery_code_editor_torture_inlays_present(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_inlays_present(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap
}

fn run_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit
}

fn run_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed(
    checks: &RunChecks,
) -> bool {
    checks
        .check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed
}

fn run_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed
}

fn run_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped(
    checks: &RunChecks,
) -> bool {
    checks
        .check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped
}

fn run_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations(
    checks: &RunChecks,
) -> bool {
    checks
        .check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations
}

fn run_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed(
    checks: &RunChecks,
) -> bool {
    checks
        .check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed
}

fn run_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped
}

fn run_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations
}

fn run_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed(
    checks: &RunChecks,
) -> bool {
    checks
        .check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed
}

fn run_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll
}

fn run_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection
}

fn run_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_word_boundary(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_code_editor_word_boundary
}

fn run_ui_gallery_code_editor_word_boundary(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_word_boundary(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_a11y_selection(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_code_editor_a11y_selection
}

fn run_ui_gallery_code_editor_a11y_selection(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_a11y_selection(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_a11y_composition(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_code_editor_a11y_composition
}

fn run_ui_gallery_code_editor_a11y_composition(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_a11y_composition(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_a11y_selection_wrap(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_code_editor_a11y_selection_wrap
}

fn run_ui_gallery_code_editor_a11y_selection_wrap(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_a11y_selection_wrap(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_a11y_composition_wrap(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_code_editor_a11y_composition_wrap
}

fn run_ui_gallery_code_editor_a11y_composition_wrap(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_a11y_composition_wrap_scroll(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_code_editor_a11y_composition_wrap_scroll
}

fn run_ui_gallery_code_editor_a11y_composition_wrap_scroll(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_scroll(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_a11y_composition_drag(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_code_editor_a11y_composition_drag
}

fn run_ui_gallery_code_editor_a11y_composition_drag(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_a11y_composition_drag(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_code_editor_torture_read_only_blocks_edits(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_code_editor_torture_read_only_blocks_edits
}

fn run_ui_gallery_code_editor_torture_read_only_blocks_edits(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_read_only_blocks_edits(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_markdown_editor_source_read_only_blocks_edits
}

fn run_ui_gallery_markdown_editor_source_read_only_blocks_edits(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_ui_gallery_markdown_editor_source_disabled_blocks_edits(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_markdown_editor_source_disabled_blocks_edits
}

fn run_ui_gallery_markdown_editor_source_disabled_blocks_edits(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits(
        ctx.bundle_path,
        ctx.warmup_frames,
    )
}

fn should_run_semantics_changed_repainted(checks: &RunChecks) -> bool {
    checks.check_semantics_changed_repainted
}

fn run_semantics_changed_repainted(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_bundle_for_semantics_changed_repainted(
        ctx.bundle_path,
        ctx.warmup_frames,
        checks.dump_semantics_changed_repainted_json,
    )
}

fn should_run_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps(checks: &RunChecks) -> bool {
    checks.check_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps
}

fn run_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_out_dir_for_ui_gallery_text_rescan_system_fonts_font_stack_key_bumps(
        ctx.out_dir,
    )
}

fn should_run_ui_gallery_text_fallback_policy_key_bumps_on_settings_change(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_text_fallback_policy_key_bumps_on_settings_change
}

fn run_ui_gallery_text_fallback_policy_key_bumps_on_settings_change(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_settings_change(
        ctx.out_dir,
    )
}

fn should_run_ui_gallery_text_fallback_policy_key_bumps_on_locale_change(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_text_fallback_policy_key_bumps_on_locale_change
}

fn run_ui_gallery_text_fallback_policy_key_bumps_on_locale_change(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_out_dir_for_ui_gallery_text_fallback_policy_key_bumps_on_locale_change(
        ctx.out_dir,
    )
}

fn should_run_ui_gallery_text_mixed_script_bundled_fallback_conformance(
    checks: &RunChecks,
) -> bool {
    checks.check_ui_gallery_text_mixed_script_bundled_fallback_conformance
}

fn run_ui_gallery_text_mixed_script_bundled_fallback_conformance(
    ctx: PostRunCheckContext<'_>,
    _checks: &RunChecks,
) -> Result<(), String> {
    crate::stats::check_out_dir_for_ui_gallery_text_mixed_script_bundled_fallback_conformance(
        ctx.out_dir,
    )
}

fn should_run_notify_hotspot_file_max(checks: &RunChecks) -> bool {
    !checks.check_notify_hotspot_file_max.is_empty()
}

fn run_notify_hotspot_file_max(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    for (file, max) in checks.check_notify_hotspot_file_max.iter() {
        crate::stats::check_bundle_for_notify_hotspot_file_max(
            ctx.bundle_path,
            file.as_str(),
            *max,
            ctx.warmup_frames,
        )?;
    }
    Ok(())
}

fn should_run_triage_hint_absent_codes(checks: &RunChecks) -> bool {
    !checks.check_triage_hint_absent_codes.is_empty()
}

fn run_triage_hint_absent_codes(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let sort = crate::BundleStatsSort::Invalidation;
    let report = crate::bundle_stats_from_path(
        ctx.bundle_path,
        1,
        sort,
        crate::BundleStatsOptions {
            warmup_frames: ctx.warmup_frames,
        },
    )?;
    let triage = crate::triage_json_from_stats(ctx.bundle_path, &report, sort, ctx.warmup_frames);
    let present_codes: Vec<String> = triage
        .get("hints")
        .and_then(|v| v.as_array())
        .map(|hints| {
            hints
                .iter()
                .filter_map(|h| {
                    h.get("code")
                        .and_then(|c| c.as_str())
                        .map(|s| s.to_string())
                })
                .collect()
        })
        .unwrap_or_default();

    let mut violations: Vec<String> = Vec::new();
    for code in checks.check_triage_hint_absent_codes.iter() {
        if present_codes.iter().any(|c| c == code) {
            violations.push(code.clone());
        }
    }
    if !violations.is_empty() {
        return Err(format!(
            "triage hint(s) present but forbidden by --check-triage-hint-absent: {}\n\
 bundle={}\n\
 present_hints={}",
            violations.join(", "),
            ctx.bundle_path.display(),
            present_codes.join(", ")
        ));
    }

    Ok(())
}

fn should_run_pixels_changed_test_id(checks: &RunChecks) -> bool {
    checks.check_pixels_changed_test_id.is_some()
}

fn run_pixels_changed_test_id(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(test_id) = checks.check_pixels_changed_test_id.as_deref() else {
        return Ok(());
    };
    crate::stats::check_out_dir_for_pixels_changed(ctx.out_dir, test_id, ctx.warmup_frames)
}

fn should_run_pixels_unchanged_test_id(checks: &RunChecks) -> bool {
    checks.check_pixels_unchanged_test_id.is_some()
}

fn run_pixels_unchanged_test_id(
    ctx: PostRunCheckContext<'_>,
    checks: &RunChecks,
) -> Result<(), String> {
    let Some(test_id) = checks.check_pixels_unchanged_test_id.as_deref() else {
        return Ok(());
    };
    crate::stats::check_out_dir_for_pixels_unchanged(ctx.out_dir, test_id, ctx.warmup_frames)
}
