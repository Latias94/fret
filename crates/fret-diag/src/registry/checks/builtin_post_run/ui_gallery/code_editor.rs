use super::{PostRunCheckContext, PostRunCheckEntry};
use crate::diag_run::RunChecks;

pub(super) const EARLY_ENTRIES: &[PostRunCheckEntry] = &[
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
];

pub(super) const MAIN_ENTRIES: &[PostRunCheckEntry] = &[
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
];

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
    checks.check_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed
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
    checks.check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped
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
    checks.check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations
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
    checks.check_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed
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
    checks.check_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed
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
