pub(super) fn assert_browser_input_runtime_owner_split(
    browser_input_runtime_source: &str,
    browser_input_box_select_runtime_source: &str,
    browser_input_box_select_session_source: &str,
    browser_input_box_select_session_tests_source: &str,
    browser_input_box_select_session_tests_fixtures_source: &str,
    browser_input_context_menu_runtime_source: &str,
    browser_input_context_menu_runtime_tests_source: &str,
    browser_input_context_menu_runtime_tests_fixtures_source: &str,
    browser_input_zoom_runtime_source: &str,
) {
    for needle in [
        "mod box_select;",
        "mod context_menu;",
        "mod zoom;",
        "use box_select::{",
        "install_collection_browser_scope_box_select_runtime,",
        "use context_menu::publish_collection_browser_scope_context_menu_anchor;",
        "use zoom::install_collection_browser_scope_zoom_runtime;",
        "pub(super) struct ProofCollectionBrowserScopeInputModels",
        "pub(super) struct ProofCollectionBrowserScopeInputState",
        "pub(super) fn proof_collection_browser_scope_pointer_props()",
        "pub(super) fn install_collection_browser_scope_input_runtime(",
        "install_collection_keyboard_handler(",
        "publish_collection_browser_scope_context_menu_anchor(",
        "install_collection_browser_scope_zoom_runtime(",
        "install_collection_browser_scope_box_select_runtime(",
        "ProofCollectionBrowserScopeBoxSelectRuntimeModels {",
        "ProofCollectionBrowserScopeBoxSelectRuntimeState {",
        "context_menu_anchor_model_for_up",
    ] {
        assert!(
            browser_input_runtime_source.contains(needle),
            "the demo-local collection browser input runtime owner should keep wheel/context/box-select handlers explicit; missing `{needle}`"
        );
    }
    for needle in [
        "cx.pointer_region_on_wheel(",
        "proof_collection_zoom_request(",
        "collection_scroll_handle.set_offset(update.next_scroll_offset);",
        "up.down_hit_pressable_target.is_some()",
        "up.position_window.unwrap_or(up.position)",
        "*state = Some(position);",
        "cx.pointer_region_on_pointer_down(",
        "cx.pointer_region_on_pointer_move(",
        "cx.pointer_region_on_pointer_up(",
        "cx.pointer_region_on_pointer_cancel(",
        "host.request_focus(acx.target);",
        "ProofCollectionBoxSelectSession {",
        "host.capture_pointer();",
        "proof_collection_box_select_selection(",
        "state.clear();",
        "host.release_pointer_capture();",
    ] {
        assert!(
            !browser_input_runtime_source.contains(needle),
            "the demo-local collection browser input runtime owner should route wheel/context/box-select child behavior through child runtime owners; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeModels",
        "pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeState",
        "pub(super) fn install_collection_browser_scope_box_select_runtime(",
        "mod session;",
        "use session::{",
        "cx.pointer_region_on_pointer_down(",
        "host.request_focus(acx.target);",
        "proof_collection_browser_scope_box_select_can_start_from_down(",
        "proof_collection_browser_scope_box_select_session_from_down(",
        "ProofCollectionBrowserScopeBoxSelectModelOwner::new(host.models_mut())",
        ".begin_session(&box_select_model_for_down, session);",
        "host.capture_pointer();",
        "cx.pointer_region_on_pointer_move(",
        ".session_for_move(&box_select_model_for_move, &mv);",
        "publish_collection_browser_scope_box_select_threshold_selection(",
        "cx.pointer_region_on_pointer_up(",
        "before_box_select_pointer_up(host, acx, &up)",
        ".session_for_up(&box_select_model_for_up, &up);",
        "host.release_pointer_capture();",
        "ProofCollectionModelOwner::new(host.models_mut()).apply_navigation(",
        "ImUiMultiSelectState::default()",
        "ProofCollectionKeyboardState::default()",
        "cx.pointer_region_on_pointer_cancel(",
        ".cancel_pointer(&box_select_model_for_cancel, &cancel);",
        "proof_collection_box_select_selection(",
        "active_id: next_selection.first_selected().cloned(),",
    ] {
        assert!(
            browser_input_box_select_runtime_source.contains(needle),
            "the demo-local collection browser input box-select runtime owner should keep pointer event wiring and selection publication explicit; missing `{needle}`"
        );
    }
    for needle in [
        "ProofCollectionBoxSelectSession {",
        "fn proof_collection_browser_scope_box_select_can_start_from_down(",
        "fn proof_collection_browser_scope_box_select_session_from_down(",
        "fn proof_collection_browser_scope_box_select_update_session_position(",
        "fn proof_collection_browser_scope_box_select_session_for_move(",
        "fn proof_collection_browser_scope_box_select_session_for_up(",
        "fn proof_collection_browser_scope_box_select_cancel_pointer(",
        "box_select_down_arms_left_background_session",
        "box_select_move_marks_threshold_for_matching_pointer",
        "box_select_up_restores_mismatched_pointer_and_takes_matching_session",
        "box_select_cancel_clears_matching_pointer_only",
    ] {
        assert!(
            !browser_input_box_select_runtime_source.contains(needle),
            "the demo-local collection browser input box-select runtime owner should route pure pointer session transitions through box_select/session.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_browser_scope_box_select_can_start_from_down(",
        "pub(super) fn proof_collection_browser_scope_box_select_session_from_down(",
        "fn proof_collection_browser_scope_box_select_update_session_position(",
        "pub(super) fn proof_collection_browser_scope_box_select_session_for_move(",
        "pub(super) fn proof_collection_browser_scope_box_select_session_for_up(",
        "pub(super) fn proof_collection_browser_scope_box_select_cancel_pointer(",
        "proof_collection_drag_threshold_met(",
        "ProofCollectionBoxSelectSession {",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            browser_input_box_select_session_source.contains(needle),
            "the demo-local collection browser input box-select session owner should keep pure pointer session transitions explicit; missing `{needle}`"
        );
    }
    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeModels",
        "pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeState",
        "pub(super) fn install_collection_browser_scope_box_select_runtime(",
        "BeforeCollectionBrowserScopeBoxSelectPointerUp",
        "cx.pointer_region_on_pointer_down(",
        "publish_collection_browser_scope_box_select_threshold_selection(",
        "proof_collection_box_select_selection(",
        "state.active_id = next_selection.first_selected().cloned();",
        "fn pointer_down(",
        "fn pointer_move(",
        "fn pointer_up(",
        "fn pointer_cancel(",
        "box_select_down_arms_left_background_session",
        "box_select_move_marks_threshold_for_matching_pointer",
        "box_select_up_restores_mismatched_pointer_and_takes_matching_session",
        "box_select_cancel_clears_matching_pointer_only",
    ] {
        assert!(
            !browser_input_box_select_session_source.contains(needle),
            "the demo-local collection browser input box-select session owner should not take runtime event/model publication; unexpected `{needle}`"
        );
    }
    for needle in [
        "mod fixtures;",
        "use fixtures::{",
        "box_select_down_arms_left_background_session",
        "box_select_down_ignores_non_left_or_pressable_origin",
        "box_select_move_marks_threshold_for_matching_pointer",
        "box_select_move_ignores_released_left_button",
        "box_select_up_restores_mismatched_pointer_and_takes_matching_session",
        "box_select_cancel_clears_matching_pointer_only",
    ] {
        assert!(
            browser_input_box_select_session_tests_source.contains(needle),
            "the demo-local collection browser input box-select session tests owner should keep pointer fixtures and behavior coverage explicit; missing `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn point(",
        "pub(super) fn pointer_down(",
        "pub(super) fn pointer_move(",
        "pub(super) fn pointer_up(",
        "pub(super) fn pointer_cancel(",
        "pub(super) fn session(pointer_id: PointerId) -> ProofCollectionBoxSelectSession",
    ] {
        assert!(
            browser_input_box_select_session_tests_fixtures_source.contains(needle),
            "the demo-local collection browser input box-select session fixture owner should keep pointer/session fixtures explicit; missing `{needle}`"
        );
    }
    for needle in [
        "fn pointer_down(",
        "fn pointer_move(",
        "fn pointer_up(",
        "fn pointer_cancel(",
        "fn session(pointer_id: PointerId) -> ProofCollectionBoxSelectSession",
        "ProofCollectionBoxSelectSession {",
        "pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeModels",
        "pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeState",
        "pub(super) fn install_collection_browser_scope_box_select_runtime(",
        "BeforeCollectionBrowserScopeBoxSelectPointerUp",
        "cx.pointer_region_on_pointer_down(",
        "publish_collection_browser_scope_box_select_threshold_selection(",
        "proof_collection_box_select_selection(",
        "state.active_id = next_selection.first_selected().cloned();",
    ] {
        assert!(
            !browser_input_box_select_session_tests_source.contains(needle),
            "the demo-local collection browser input box-select session tests owner should not take runtime event/model publication; unexpected `{needle}`"
        );
    }
    for needle in [
        "box_select_down_arms_left_background_session",
        "box_select_down_ignores_non_left_or_pressable_origin",
        "box_select_move_marks_threshold_for_matching_pointer",
        "box_select_move_ignores_released_left_button",
        "box_select_up_restores_mismatched_pointer_and_takes_matching_session",
        "box_select_cancel_clears_matching_pointer_only",
        "pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeModels",
        "pub(super) struct ProofCollectionBrowserScopeBoxSelectRuntimeState",
        "pub(super) fn install_collection_browser_scope_box_select_runtime(",
        "BeforeCollectionBrowserScopeBoxSelectPointerUp",
        "cx.pointer_region_on_pointer_down(",
        "publish_collection_browser_scope_box_select_threshold_selection(",
        "proof_collection_box_select_selection(",
        "state.active_id = next_selection.first_selected().cloned();",
    ] {
        assert!(
            !browser_input_box_select_session_tests_fixtures_source.contains(needle),
            "the demo-local collection browser input box-select session fixture owner should not take behavior tests or runtime event/model publication; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeInputModels",
        "pub(super) struct ProofCollectionBrowserScopeInputState",
        "install_collection_keyboard_handler(",
        "install_collection_browser_scope_zoom_runtime(",
        "pub(super) fn publish_collection_browser_scope_context_menu_anchor(",
        "proof_collection_zoom_request(",
        "up.down_hit_pressable_target.is_some()",
        "up.position_window.unwrap_or(up.position)",
        "*state = Some(position);",
        ".publish_context_menu_anchor(context_menu_anchor_model, position);",
    ] {
        assert!(
            !browser_input_box_select_runtime_source.contains(needle),
            "the demo-local collection browser input box-select runtime owner should not take parent keyboard/zoom/context-menu responsibilities; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_browser_scope_context_menu_anchor_from_up(",
        "up.button != MouseButton::Right || !up.is_click",
        "up.down_hit_pressable_target.is_some()",
        "up.down_hit_pressable_target_in_descendant_subtree",
        "Some(up.position_window.unwrap_or(up.position))",
        "pub(super) fn publish_collection_browser_scope_context_menu_anchor(",
        "host.request_focus(acx.target);",
        "ProofCollectionModelOwner::new(host.models_mut())",
        ".publish_context_menu_anchor(context_menu_anchor_model, position);",
        "host.notify(acx);",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            browser_input_context_menu_runtime_source.contains(needle),
            "the demo-local collection browser input context-menu runtime owner should keep right-click anchor publishing explicit; missing `{needle}`"
        );
    }
    for needle in [
        "mod fixtures;",
        "use fixtures::pointer_up;",
        "context_menu_anchor_prefers_window_position",
        "context_menu_anchor_falls_back_to_pointer_position",
        "context_menu_anchor_ignores_non_right_or_non_click_up",
        "context_menu_anchor_ignores_direct_pressable_clicks",
        "context_menu_anchor_ignores_pressable_descendant_clicks",
        "proof_collection_browser_scope_context_menu_anchor_from_up(",
    ] {
        assert!(
            browser_input_context_menu_runtime_tests_source.contains(needle),
            "the demo-local collection browser input context-menu tests owner should keep fixture imports and behavior coverage explicit; missing `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn pointer_up(",
        "PointerUpCx {",
        "PointerId(0)",
        "Modifiers::default()",
        "PointerType::Mouse",
    ] {
        assert!(
            browser_input_context_menu_runtime_tests_fixtures_source.contains(needle),
            "the demo-local collection browser input context-menu tests fixture owner should keep PointerUpCx fixtures explicit; missing `{needle}`"
        );
    }
    for needle in [
        "fn pointer_up(",
        "context_menu_anchor_prefers_window_position",
        "context_menu_anchor_falls_back_to_pointer_position",
        "context_menu_anchor_ignores_non_right_or_non_click_up",
        "context_menu_anchor_ignores_direct_pressable_clicks",
        "context_menu_anchor_ignores_pressable_descendant_clicks",
    ] {
        assert!(
            !browser_input_context_menu_runtime_source.contains(needle),
            "the demo-local collection browser input context-menu runtime owner should route anchor behavior coverage through context_menu/tests.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "fn pointer_up(",
        "PointerUpCx {",
        "PointerId(0)",
        "Modifiers::default()",
        "PointerType::Mouse",
    ] {
        assert!(
            !browser_input_context_menu_runtime_tests_source.contains(needle),
            "the demo-local collection browser input context-menu tests owner should route PointerUpCx fixtures through context_menu/tests/fixtures.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "context_menu_anchor_prefers_window_position",
        "context_menu_anchor_falls_back_to_pointer_position",
        "context_menu_anchor_ignores_non_right_or_non_click_up",
        "context_menu_anchor_ignores_direct_pressable_clicks",
        "context_menu_anchor_ignores_pressable_descendant_clicks",
        "proof_collection_browser_scope_context_menu_anchor_from_up(",
        "pub(super) fn proof_collection_browser_scope_context_menu_anchor_from_up(",
        "pub(super) fn publish_collection_browser_scope_context_menu_anchor(",
        "host.request_focus(acx.target);",
        "ProofCollectionModelOwner::new(host.models_mut())",
        ".publish_context_menu_anchor(context_menu_anchor_model, position);",
        "host.notify(acx);",
    ] {
        assert!(
            !browser_input_context_menu_runtime_tests_fixtures_source.contains(needle),
            "the demo-local collection browser input context-menu tests fixture owner should not take anchor behavior tests or runtime publication ownership; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_browser_scope_context_menu_anchor_from_up(",
        "pub(super) fn publish_collection_browser_scope_context_menu_anchor(",
        "host.request_focus(acx.target);",
        "ProofCollectionModelOwner::new(host.models_mut())",
        ".publish_context_menu_anchor(context_menu_anchor_model, position);",
        "host.notify(acx);",
    ] {
        assert!(
            !browser_input_context_menu_runtime_tests_source.contains(needle),
            "the demo-local collection browser input context-menu tests owner should not take runtime anchor publication ownership; unexpected `{needle}`"
        );
    }
    for needle in [
        "install_collection_keyboard_handler(",
        "cx.pointer_region_on_pointer_down(",
        "cx.pointer_region_on_pointer_move(",
        "cx.pointer_region_on_pointer_up(",
        "cx.pointer_region_on_pointer_cancel(",
        "ProofCollectionBoxSelectSession",
        "proof_collection_box_select_selection(",
        "proof_collection_zoom_request(",
        "collection_scroll_handle.set_offset(update.next_scroll_offset);",
    ] {
        assert!(
            !browser_input_context_menu_runtime_source.contains(needle),
            "the demo-local collection browser input context-menu runtime owner should not take keyboard/box-select/zoom runtime; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn install_collection_browser_scope_zoom_runtime(",
        "cx.pointer_region_on_wheel(",
        "proof_collection_zoom_request(",
        "collection_scroll_handle.offset()",
        "wheel.position_local",
        "wheel.delta",
        "wheel.modifiers",
        "ProofCollectionModelOwner::new(host.models_mut())",
        ".set_zoom_extent(&collection_zoom_model, update.next_tile_extent);",
        "collection_scroll_handle.set_offset(update.next_scroll_offset);",
        "host.notify(acx);",
    ] {
        assert!(
            browser_input_zoom_runtime_source.contains(needle),
            "the demo-local collection browser input zoom runtime owner should keep Primary+Wheel zoom explicit; missing `{needle}`"
        );
    }
    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeInputModels",
        "pub(super) struct ProofCollectionBrowserScopeInputState",
        "install_collection_keyboard_handler(",
        "cx.pointer_region_on_pointer_down(",
        "cx.pointer_region_on_pointer_move(",
        "cx.pointer_region_on_pointer_up(",
        "cx.pointer_region_on_pointer_cancel(",
        "proof_collection_box_select_selection(",
        "context_menu_anchor_model_for_up",
    ] {
        assert!(
            !browser_input_zoom_runtime_source.contains(needle),
            "the demo-local collection browser input zoom runtime owner should not take keyboard/context/box-select runtime; unexpected `{needle}`"
        );
    }
}
