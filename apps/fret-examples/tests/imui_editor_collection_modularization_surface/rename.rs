pub(super) fn assert_rename_owner_split(
    rename_source: &str,
    rename_tests_source: &str,
    rename_tests_fixtures_source: &str,
    rename_commit_source: &str,
    rename_commit_tests_source: &str,
    rename_commit_tests_fixtures_source: &str,
    rename_focus_source: &str,
) {
    for needle in [
        "mod commit;",
        "mod focus;",
        "pub(super) use commit::{",
        "ProofCollectionRenameCommit",
        "proof_collection_commit_rename",
        "pub(super) use focus::{",
        "proof_collection_inline_rename_focus_state",
        "proof_collection_restore_focus_after_inline_rename",
        "proof_collection_sync_inline_rename_focus",
        "pub(super) struct ProofCollectionRenameSession",
        "pub(super) fn proof_collection_begin_rename_session(",
        "pub(super) fn proof_collection_begin_inline_rename_in_app(",
        "proof_collection_rename_ready_status(",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            rename_source.contains(needle),
            "the demo-local collection rename hub should keep pure rename workflow state and focus re-exports explicit; missing `{needle}`"
        );
    }

    for needle in [
        "mod fixtures;",
        "use fixtures::selection_state;",
        "authoring_parity_collection_assets()",
        "proof_collection_begin_rename_session(",
        "proof_collection_begin_rename_session_prefers_active_visible_asset",
        "proof_collection_begin_rename_session_falls_back_to_first_visible_asset",
        "proof_collection_rename_shortcut_matches_plain_f2_only",
    ] {
        assert!(
            rename_tests_source.contains(needle),
            "the demo-local collection rename tests owner should keep session and shortcut coverage explicit; missing `{needle}`"
        );
    }

    for needle in [
        "proof_collection_begin_rename_session_prefers_active_visible_asset",
        "proof_collection_begin_rename_session_falls_back_to_first_visible_asset",
        "proof_collection_rename_shortcut_matches_plain_f2_only",
    ] {
        assert!(
            !rename_source.contains(needle),
            "the demo-local collection rename hub should not take root rename tests; unexpected `{needle}`"
        );
    }
    for needle in [
        "fn selection_state(selected: &[&str], anchor: Option<&str>)",
        "pub(super) struct ProofCollectionRenameCommit",
        "pub(in super::super) fn proof_collection_commit_rename(",
        "struct ProofCollectionInlineRenameFocusState",
        "render_collection_first_asset_browser_proof",
        "TextField::new(",
        "TextFieldOptions {",
        "DragPreviewGhostOptions",
        "drag_preview_ghost",
        "kit::ButtonOptions",
        "kit::ChildRegionOptions",
        "kit::GridOptions",
        "kit::MenuItemOptions",
    ] {
        assert!(
            !rename_tests_source.contains(needle),
            "the demo-local collection rename tests owner should not take fixture helpers, commit/focus implementation, render, or UI policy; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn selection_state(",
        "ImUiMultiSelectState::new(",
    ] {
        assert!(
            rename_tests_fixtures_source.contains(needle),
            "the demo-local collection rename tests fixture owner should keep selection fixtures explicit; missing `{needle}`"
        );
    }
    for needle in [
        "proof_collection_begin_rename_session_prefers_active_visible_asset",
        "proof_collection_begin_rename_session_falls_back_to_first_visible_asset",
        "proof_collection_rename_shortcut_matches_plain_f2_only",
        "proof_collection_begin_rename_session(",
        "proof_collection_rename_shortcut_matches(",
        "proof_collection_begin_inline_rename_in_app(",
        "proof_collection_rename_ready_status(",
        "pub(super) struct ProofCollectionRenameSession",
        "pub(super) struct ProofCollectionRenameCommit",
        "struct ProofCollectionInlineRenameFocusState",
        "render_collection_first_asset_browser_proof",
        "TextField",
        "DragPreviewGhostOptions",
        "drag_preview_ghost",
        "kit::ButtonOptions",
        "kit::ChildRegionOptions",
        "kit::GridOptions",
        "kit::MenuItemOptions",
    ] {
        assert!(
            !rename_tests_fixtures_source.contains(needle),
            "the demo-local collection rename tests fixture owner should not take behavior tests, rename implementation, render, or UI policy; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super) struct ProofCollectionRenameCommit",
        "pub(in super::super) fn proof_collection_commit_rename(",
        "draft.trim()",
        "asset.label = next_label.clone();",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            rename_commit_source.contains(needle),
            "the demo-local collection rename commit owner should keep commit mutation explicit; missing `{needle}`"
        );
    }

    for needle in [
        "mod fixtures;",
        "use fixtures::{rename_session, stored_assets};",
        "proof_collection_commit_rename(",
        "proof_collection_commit_rename_updates_label_without_touching_order_or_ids",
        "proof_collection_commit_rename_rejects_empty_trimmed_label",
    ] {
        assert!(
            rename_commit_tests_source.contains(needle),
            "the demo-local collection rename commit tests owner should keep commit behavior coverage explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn stored_assets() -> Vec<ProofCollectionAsset>",
        "authoring_parity_collection_assets()",
        "pub(super) fn rename_session() -> ProofCollectionRenameSession",
        "ProofCollectionRenameSession {",
        "target_id: Arc::from(\"stone-normal\")",
        "original_label: Arc::from(\"Stone Normal\")",
    ] {
        assert!(
            rename_commit_tests_fixtures_source.contains(needle),
            "the demo-local collection rename commit tests fixture owner should keep commit setup explicit; missing `{needle}`"
        );
    }

    for needle in [
        "authoring_parity_collection_assets()",
        "ProofCollectionRenameSession {",
    ] {
        assert!(
            !rename_commit_tests_source.contains(needle),
            "the demo-local collection rename commit tests owner should import fixtures instead of defining setup; unexpected `{needle}`"
        );
    }

    for needle in [
        "proof_collection_commit_rename(",
        "proof_collection_commit_rename_updates_label_without_touching_order_or_ids",
        "proof_collection_commit_rename_rejects_empty_trimmed_label",
        "pub(in super::super) struct ProofCollectionRenameCommit",
        "draft.trim()",
        "asset.label = next_label.clone();",
    ] {
        assert!(
            !rename_commit_tests_fixtures_source.contains(needle),
            "the demo-local collection rename commit tests fixture owner should not take commit behavior or mutation; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super) struct ProofCollectionRenameCommit",
        "pub(in super::super) fn proof_collection_commit_rename(",
        "draft.trim()",
        "asset.label = next_label.clone();",
        "proof_collection_commit_rename_updates_label_without_touching_order_or_ids",
        "proof_collection_commit_rename_rejects_empty_trimmed_label",
    ] {
        assert!(
            !rename_source.contains(needle),
            "the demo-local collection rename hub should route commit mutation through rename/commit.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "proof_collection_commit_rename_updates_label_without_touching_order_or_ids",
        "proof_collection_commit_rename_rejects_empty_trimmed_label",
    ] {
        assert!(
            !rename_commit_source.contains(needle),
            "the demo-local collection rename commit owner should not take commit tests; unexpected `{needle}`"
        );
    }

    for needle in [
        "proof_collection_rename_shortcut_matches(",
        "proof_collection_begin_rename_session(",
        "proof_collection_begin_inline_rename_in_app(",
        "proof_collection_rename_ready_status(",
        "ImUiMultiSelectState",
        "struct ProofCollectionInlineRenameFocusState",
        "timer_add_on_timer_for(",
        "host.request_focus(input_id);",
    ] {
        assert!(
            !rename_commit_source.contains(needle),
            "the demo-local collection rename commit owner should not take shortcut/session/app-model/focus policy; unexpected `{needle}`"
        );
    }

    for needle in [
        "struct ProofCollectionInlineRenameFocusState",
        "fn proof_collection_inline_rename_focus_state<",
        "fn proof_collection_sync_inline_rename_focus<",
        "fn proof_collection_restore_focus_after_inline_rename(",
        "timer_add_on_timer_for(",
        "host.request_focus(input_id);",
    ] {
        assert!(
            !rename_source.contains(needle),
            "the demo-local collection rename hub should route focus runtime through rename/focus.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super) struct ProofCollectionInlineRenameFocusState",
        "timer: Option<TimerToken>",
        "pub(in super::super) fn proof_collection_inline_rename_focus_state<",
        "pub(in super::super) fn proof_collection_sync_inline_rename_focus<",
        "pub(in super::super) fn proof_collection_restore_focus_after_inline_rename(",
        "cx.timer_add_on_timer_for(",
        "host.request_focus(input_id);",
        "host.request_redraw(action_cx.window);",
        "Duration::ZERO",
    ] {
        assert!(
            rename_focus_source.contains(needle),
            "the demo-local collection rename focus owner should keep focus handoff runtime explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionRenameSession",
        "pub(super) struct ProofCollectionRenameCommit",
        "pub(in super::super) struct ProofCollectionRenameCommit",
        "proof_collection_begin_rename_session(",
        "proof_collection_begin_inline_rename_in_app(",
        "proof_collection_commit_rename(",
        "proof_collection_rename_ready_status(",
        "ImUiMultiSelectState",
    ] {
        assert!(
            !rename_focus_source.contains(needle),
            "the demo-local collection rename focus owner should not take rename state/commit policy; unexpected `{needle}`"
        );
    }
}
