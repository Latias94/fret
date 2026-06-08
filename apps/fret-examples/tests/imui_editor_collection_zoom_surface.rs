#[test]
fn imui_editor_proof_demo_keeps_collection_zoom_app_owned_and_explicit() {
    let source = concat!(
        include_str!("../src/imui_editor_proof_demo/collection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/actions.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/asset_grid.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/chrome.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/input_runtime.rs"),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select/session.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/context_menu.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/zoom.rs"
        ),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/box_select.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons/actions.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/context_menu.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/context_menu/actions.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/context_menu/chrome.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/drag_drop.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/geometry.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/geometry/zoom.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/keyboard.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/keyboard/actions.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/models.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/readouts.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/readouts/status.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/runtime_state.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/context_menu.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/keyboard.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/keyboard/navigation.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/projection.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/select_all.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/delete.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/duplicate.rs"),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/naming.rs"
        ),
        "\n",
        include_str!(
            "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/selection.rs"
        ),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename/commit.rs"),
        "\n",
        include_str!("../src/imui_editor_proof_demo/collection/rename/focus.rs"),
    );

    for needle in [
        "struct ProofCollectionLayoutMetrics {",
        "struct ProofCollectionZoomUpdate {",
        "fn proof_collection_layout_metrics(",
        "fn proof_collection_zoom_line(",
        "fn proof_collection_zoom_request(",
        "fn authoring_parity_collection_zoom_model<H: UiHost>(",
        "fn authoring_parity_collection_scroll_handle<H: UiHost>(",
        "imui_editor_proof_demo.model.authoring_parity.collection_zoom",
        "imui_editor_proof_demo.state.authoring_parity.collection_scroll_handle",
        "\"Primary+Wheel zoom stays app-owned: {} px target tiles across {} column(s), with hovered rows staying anchored inside the collection proof.\"",
        "models.scroll.viewport_size().width",
        "collection_browser_child_region_options(collection_scroll_handle.clone())",
        "handle: Some(scroll)",
        "proof_collection_zoom_request(",
        "collection_layout.columns",
        "collection_scroll_handle.set_offset(update.next_scroll_offset);",
    ] {
        assert!(
            source.contains(needle),
            "imui_editor_proof_demo should keep the collection zoom proof explicit and app-owned; missing `{needle}`"
        );
    }

    for needle in [
        "fret_ui_kit::imui::collection_zoom",
        "pub fn collection_zoom",
        "pub fn collection_layout_metrics",
        "struct ImUiCollectionZoom",
    ] {
        assert!(
            !source.contains(needle),
            "imui_editor_proof_demo should not pretend the collection zoom slice is already a shared helper; unexpected `{needle}`"
        );
    }
}
