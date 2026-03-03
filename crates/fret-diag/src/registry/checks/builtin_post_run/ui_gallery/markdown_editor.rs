use super::{PostRunCheckContext, PostRunCheckEntry};
use crate::diag_run::RunChecks;

pub(super) const EARLY_ENTRIES: &[PostRunCheckEntry] = &[
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
];

pub(super) const MAIN_ENTRIES: &[PostRunCheckEntry] = &[
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
];

pub(super) const TAIL_ENTRIES: &[PostRunCheckEntry] = &[
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
];

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
