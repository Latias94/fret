// Policy helpers for selecting diagnostics gates per script.
use std::path::Path;

pub(crate) fn bundle_paint_cache_hit_test_only_replay_maxes(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(u64, u64), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok((0, 0));
    }

    let mut allowed_max: u64 = 0;
    let mut rejected_key_mismatch_max: u64 = 0;

    for w in windows {
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }

            let stats = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.as_object());
            let Some(stats) = stats else {
                continue;
            };

            let allowed = stats
                .get("paint_cache_hit_test_only_replay_allowed")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let rejected = stats
                .get("paint_cache_hit_test_only_replay_rejected_key_mismatch")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            allowed_max = allowed_max.max(allowed);
            rejected_key_mismatch_max = rejected_key_mismatch_max.max(rejected);
        }
    }

    Ok((allowed_max, rejected_key_mismatch_max))
}

pub(crate) fn docking_arbitration_script_default_gates(
    script: &Path,
) -> (Option<u64>, Option<u64>, Option<u64>) {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return (None, None, None);
    };

    match name {
        "docking-arbitration-demo-split-viewports.json" => (Some(1), None, None),
        "docking-arbitration-demo-modal-dock-drag-viewport-capture.json" => {
            (Some(1), Some(1), Some(1))
        }
        _ => (None, None, None),
    }
}

pub(crate) fn ui_gallery_script_requires_retained_vlist_reconcile_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-ai-transcript-torture-scroll.json"
            | "ui-gallery-virtual-list-window-boundary-scroll-retained.json"
            | "ui-gallery-tree-window-boundary-scroll-retained.json"
            | "ui-gallery-data-table-window-boundary-scroll-retained.json"
            | "ui-gallery-table-retained-window-boundary-scroll.json"
            | "components-gallery-file-tree-window-boundary-scroll.json"
            | "components-gallery-file-tree-window-boundary-bounce.json"
            | "components-gallery-table-window-boundary-scroll.json"
            | "components-gallery-table-window-boundary-bounce.json"
    )
}

pub(crate) fn ui_gallery_script_requires_retained_vlist_keep_alive_reuse_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "components-gallery-file-tree-window-boundary-bounce.json"
            | "components-gallery-table-window-boundary-bounce.json"
            | "ui-gallery-data-table-window-boundary-bounce-keep-alive.json"
            | "ui-gallery-inspector-torture-bounce-keep-alive.json"
            | "workspace-shell-demo-file-tree-bounce-keep-alive.json"
    )
}

pub(crate) fn ui_gallery_script_requires_overlay_synthesis_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    // These scripts are expected to exercise the cached overlay synthesis seam when view-cache
    // shell reuse is enabled.
    matches!(
        name,
        "ui-gallery-overlay-torture.json"
            | "ui-gallery-modal-barrier-underlay-block.json"
            | "ui-gallery-popover-dialog-escape-underlay.json"
            | "ui-gallery-portal-geometry-scroll-clamp.json"
            | "ui-gallery-dropdown-open-select.json"
            | "ui-gallery-dropdown-submenu-underlay-dismiss.json"
            | "ui-gallery-context-menu-right-click.json"
            | "ui-gallery-dialog-escape-focus-restore.json"
            | "ui-gallery-menubar-keyboard-nav.json"
    )
}

pub(crate) fn ui_gallery_script_requires_viewport_input_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    // Viewport input forwarding is only expected in scripts that explicitly exercise viewport
    // panels / docking viewport tooling scenarios.
    name.contains("viewport") || name.contains("dock")
}

pub(crate) fn ui_gallery_script_requires_windowed_rows_offset_changes_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-scroll-stability.json"
            | "ui-gallery-code-editor-torture-soft-wrap-editing-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_windowed_rows_visible_start_repaint_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(name, "ui-gallery-code-editor-torture-scroll-stability.json")
}

pub(crate) fn ui_gallery_script_pixels_changed_test_id(script: &Path) -> Option<&'static str> {
    let name = script.file_name().and_then(|v| v.to_str())?;

    match name {
        "ui-gallery-alert-tabs-shared-indicator-pixels-changed-fixed-frame-delta.json" => {
            Some("ui-gallery-alert-tabs-shared-indicator")
        }
        "ui-gallery-motion-presets-fluid-tabs-pixels-changed-fixed-frame-delta.json" => {
            Some("ui-gallery-motion-presets-fluid-tabs-content-stage")
        }
        "ui-gallery-code-editor-torture-soft-wrap-editing-baseline.json" => {
            Some("ui-gallery-code-editor-torture-root")
        }
        "ui-gallery-code-view-scroll-refresh-pixels-changed.json" => {
            Some("ui-gallery-code-view-root")
        }
        "ui-gallery-carousel-demo-inertia-pixels-changed.json" => Some("ui-gallery-carousel-demo"),
        _ => None,
    }
}

pub(crate) fn ui_gallery_script_wheel_scroll_hit_changes_test_id(
    script: &Path,
) -> Option<&'static str> {
    let name = script.file_name().and_then(|v| v.to_str())?;

    match name {
        "ui-gallery-select-wheel-scroll.json" => Some("select-scroll-viewport"),
        "ui-gallery-select-wheel-up-from-bottom.json" => Some("select-scroll-viewport"),
        "ui-gallery-code-view-torture-wheel-scroll-hit-changes.json" => {
            Some("ui-gallery-code-view-root")
        }
        _ => None,
    }
}

pub(crate) fn ui_gallery_script_requires_wheel_events_max_per_frame_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(name, "ui-gallery-wheel-burst-coalescing.json")
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_marker_present_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-soft-wrap-editing-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_undo_redo_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-soft-wrap-editing-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_geom_fallbacks_low_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-soft-wrap-geom-fallback-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_read_only_blocks_edits_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-read-only-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_read_only_blocks_edits_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-read-only-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_disabled_blocks_edits_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-disabled-baseline.json"
            | "ui-gallery-markdown-editor-source-disabled-inject-preedit-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_soft_wrap_toggle_stable_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-soft-wrap-toggle-stability-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_word_boundary_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-word-boundary-baseline.json"
            | "ui-gallery-markdown-editor-source-word-boundary-double-click-baseline.json"
            | "ui-gallery-markdown-editor-source-word-boundary-inlays-baseline.json"
            | "ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-baseline.json"
            | "ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-soft-wrap-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_web_ime_bridge_enabled_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-web-markdown-editor-source-ime-bridge-attach-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_text_rescan_system_fonts_font_stack_key_bumps_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-text-rescan-system-fonts-font-stack-key-bumps.json"
    )
}

pub(crate) fn ui_gallery_script_requires_text_fallback_policy_key_bumps_on_settings_change_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-text-fallback-policy-key-bumps-on-settings-change.json"
    )
}

pub(crate) fn ui_gallery_script_requires_text_fallback_policy_key_bumps_on_locale_change_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-text-fallback-policy-key-bumps-on-locale-change.json"
    )
}

pub(crate) fn ui_gallery_script_requires_text_mixed_script_bundled_fallback_conformance_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-text-mixed-script-bundled-fallback-conformance.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_line_boundary_triple_click_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-line-boundary-triple-click-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_a11y_composition_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-a11y-composition-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_a11y_composition_soft_wrap_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-a11y-composition-soft-wrap-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_soft_wrap_editing_selection_wrap_stable_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-soft-wrap-editing-selection-wrap-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_folds_toggle_stable_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-folds-placeholder-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_folds_clamp_selection_out_of_folds_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-folds-clamp-selection-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_folds_placeholder_present_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-folds-placeholder-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_folds_placeholder_present_under_soft_wrap_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-folds-soft-wrap-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_folds_placeholder_absent_under_inline_preedit_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-folds-soft-wrap-inline-preedit-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_inlays_present_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-inlays-baseline.json"
            | "ui-gallery-markdown-editor-source-word-boundary-inlays-baseline.json"
            | "ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_inlays_toggle_stable_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-inlays-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_inlays_caret_navigation_stable_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-inlays-caret-navigation-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_inlays_present_under_soft_wrap_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-inlays-soft-wrap-baseline.json"
            | "ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-soft-wrap-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_markdown_editor_source_inlays_absent_under_inline_preedit_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-markdown-editor-source-inlays-soft-wrap-inline-preedit-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-folds-placeholder-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_soft_wrap_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-folds-soft-wrap-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_folds_placeholder_absent_under_inline_preedit_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-folds-soft-wrap-inline-preedit-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-folds-inline-preedit-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-folds-soft-wrap-inline-preedit-with-decorations-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-folds-soft-wrap-inline-preedit-with-decorations-composed-baseline.json"
            | "ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_composed_preedit_stable_after_wheel_scroll_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_composed_preedit_cancels_on_drag_selection_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-drag-select-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_inlays_present_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(name, "ui-gallery-code-editor-torture-inlays-baseline.json")
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_inlays_absent_under_inline_preedit_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-inlays-soft-wrap-inline-preedit-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-inlays-inline-preedit-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-inlays-soft-wrap-inline-preedit-with-decorations-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-inlays-soft-wrap-inline-preedit-with-decorations-composed-baseline.json"
            | "ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_torture_inlays_present_under_soft_wrap_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-torture-inlays-soft-wrap-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_word_boundary_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-word-boundary-baseline.json"
            | "ui-gallery-code-editor-word-boundary-soft-wrap-baseline.json"
            | "ui-gallery-code-editor-word-boundary-soft-wrap-double-click-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_a11y_selection_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-a11y-selection-baseline.json"
            | "ui-gallery-code-editor-a11y-selection-soft-wrap-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_a11y_composition_gate(script: &Path) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-a11y-composition-baseline.json"
            | "ui-gallery-code-editor-a11y-composition-soft-wrap-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_a11y_selection_wrap_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-a11y-selection-wrap-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_a11y_composition_wrap_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-a11y-composition-wrap-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_a11y_composition_wrap_scroll_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-a11y-composition-wrap-scroll-baseline.json"
    )
}

pub(crate) fn ui_gallery_script_requires_code_editor_a11y_composition_drag_gate(
    script: &Path,
) -> bool {
    let Some(name) = script.file_name().and_then(|v| v.to_str()) else {
        return false;
    };

    matches!(
        name,
        "ui-gallery-code-editor-a11y-composition-drag-baseline.json"
    )
}
