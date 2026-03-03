//! Builtin post-run checks for UI gallery scripts.
//!
//! These are mostly “policy-level” regression gates for the UI gallery demos. Keep them isolated
//! from engine-level checks so the registry wiring remains stable as the demo evolves.

use std::sync::OnceLock;

use super::{PostRunCheckContext, PostRunCheckEntry};

mod code_editor;
mod markdown_editor;
mod semantics;
mod text;
mod web;

pub(super) fn entries() -> &'static [PostRunCheckEntry] {
    static ENTRIES: OnceLock<Vec<PostRunCheckEntry>> = OnceLock::new();
    ENTRIES
        .get_or_init(|| {
            let mut out = Vec::new();
            out.extend_from_slice(code_editor::EARLY_ENTRIES);
            out.extend_from_slice(markdown_editor::EARLY_ENTRIES);
            out.extend_from_slice(web::ENTRIES);
            out.extend_from_slice(markdown_editor::MAIN_ENTRIES);
            out.extend_from_slice(code_editor::MAIN_ENTRIES);
            out.extend_from_slice(markdown_editor::TAIL_ENTRIES);
            out.extend_from_slice(semantics::ENTRIES);
            out.extend_from_slice(text::ENTRIES);
            out
        })
        .as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_preserve_legacy_order() {
        let ids: Vec<&str> = entries().iter().map(|entry| entry.id).collect();
        let expected = vec![
            "ui_gallery_code_editor_torture_marker_present",
            "ui_gallery_code_editor_torture_undo_redo",
            "ui_gallery_code_editor_torture_geom_fallbacks_low",
            "ui_gallery_markdown_editor_source_soft_wrap_toggle_stable",
            "ui_gallery_markdown_editor_source_word_boundary",
            "ui_gallery_web_ime_bridge_enabled",
            "ui_gallery_markdown_editor_source_line_boundary_triple_click",
            "ui_gallery_markdown_editor_source_a11y_composition",
            "ui_gallery_markdown_editor_source_a11y_composition_soft_wrap",
            "ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable",
            "ui_gallery_markdown_editor_source_folds_toggle_stable",
            "ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds",
            "ui_gallery_markdown_editor_source_folds_placeholder_present",
            "ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap",
            "ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit",
            "ui_gallery_markdown_editor_source_inlays_toggle_stable",
            "ui_gallery_markdown_editor_source_inlays_caret_navigation_stable",
            "ui_gallery_markdown_editor_source_inlays_present",
            "ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap",
            "ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit",
            "ui_gallery_code_editor_torture_folds_placeholder_present",
            "ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap",
            "ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit",
            "ui_gallery_code_editor_torture_inlays_present",
            "ui_gallery_code_editor_torture_inlays_present_under_soft_wrap",
            "ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit",
            "ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed",
            "ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed",
            "ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped",
            "ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations",
            "ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed",
            "ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped",
            "ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations",
            "ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed",
            "ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll",
            "ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection",
            "ui_gallery_code_editor_word_boundary",
            "ui_gallery_code_editor_a11y_selection",
            "ui_gallery_code_editor_a11y_composition",
            "ui_gallery_code_editor_a11y_selection_wrap",
            "ui_gallery_code_editor_a11y_composition_wrap",
            "ui_gallery_code_editor_a11y_composition_wrap_scroll",
            "ui_gallery_code_editor_a11y_composition_drag",
            "ui_gallery_code_editor_torture_read_only_blocks_edits",
            "ui_gallery_markdown_editor_source_read_only_blocks_edits",
            "ui_gallery_markdown_editor_source_disabled_blocks_edits",
            "semantics_changed_repainted",
            "ui_gallery_text_rescan_system_fonts_font_stack_key_bumps",
            "ui_gallery_text_fallback_policy_key_bumps_on_settings_change",
            "ui_gallery_text_fallback_policy_key_bumps_on_locale_change",
            "ui_gallery_text_mixed_script_bundled_fallback_conformance",
        ];

        assert_eq!(ids, expected);
    }
}
