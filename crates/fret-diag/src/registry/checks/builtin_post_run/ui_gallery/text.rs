use super::{PostRunCheckContext, PostRunCheckEntry};
use crate::diag_run::RunChecks;

pub(super) const ENTRIES: &[PostRunCheckEntry] = &[
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
];

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
