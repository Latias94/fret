#![cfg(feature = "imui")]

const IMUI_RS: &str = include_str!("../src/imui.rs");
const COLOR_EDIT_RS: &str = include_str!("../src/controls/color_edit.rs");
const COLOR_EDIT_ELEMENT_RS: &str = include_str!("../src/controls/color_edit/element.rs");
const COLOR_EDIT_ELEMENT_AFFORDANCE_RS: &str =
    include_str!("../src/controls/color_edit/element/affordance.rs");
const COLOR_EDIT_ELEMENT_FRAME_RS: &str =
    include_str!("../src/controls/color_edit/element/frame.rs");
const COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS: &str =
    include_str!("../src/controls/color_edit/element/frame/overlays.rs");
const COLOR_EDIT_ELEMENT_KEYING_RS: &str =
    include_str!("../src/controls/color_edit/element/keying.rs");
const COLOR_EDIT_ELEMENT_TEST_IDS_RS: &str =
    include_str!("../src/controls/color_edit/element/test_ids.rs");
const COLOR_EDIT_INPUT_RS: &str = include_str!("../src/controls/color_edit/input.rs");
const COLOR_EDIT_POPUP_COPY_RS: &str = include_str!("../src/controls/color_edit/popup/copy.rs");
const COLOR_EDIT_POPUP_COPY_ENTRIES_RS: &str =
    include_str!("../src/controls/color_edit/popup/copy/entries.rs");
const COLOR_EDIT_POPUP_COPY_PANEL_RS: &str =
    include_str!("../src/controls/color_edit/popup/copy/panel.rs");
const COLOR_EDIT_POPUP_COPY_ROW_RS: &str =
    include_str!("../src/controls/color_edit/popup/copy/row.rs");
const COLOR_EDIT_POPUP_EYEDROPPER_RS: &str =
    include_str!("../src/controls/color_edit/popup/eyedropper.rs");
const COLOR_EDIT_DRAG_DROP_RS: &str = include_str!("../src/controls/color_edit/drag_drop.rs");
const COLOR_EDIT_DRAG_DROP_DELIVERY_RS: &str =
    include_str!("../src/controls/color_edit/drag_drop/delivery.rs");
const COLOR_EDIT_DRAG_DROP_SOURCE_RS: &str =
    include_str!("../src/controls/color_edit/drag_drop/source.rs");
const COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS: &str =
    include_str!("../src/controls/color_edit/drag_drop/source/handlers.rs");
const COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_DOWN_RS: &str =
    include_str!("../src/controls/color_edit/drag_drop/source/handlers/down.rs");
const COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_MOVE_RS: &str =
    include_str!("../src/controls/color_edit/drag_drop/source/handlers/move_phase.rs");
const COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_UP_RS: &str =
    include_str!("../src/controls/color_edit/drag_drop/source/handlers/up.rs");
const COLOR_EDIT_DRAG_DROP_STORE_RS: &str =
    include_str!("../src/controls/color_edit/drag_drop/store.rs");
const COLOR_EDIT_LAYOUT_RS: &str = include_str!("../src/controls/color_edit/layout.rs");
const COLOR_EDIT_MODEL_RS: &str = include_str!("../src/controls/color_edit/model.rs");
const COLOR_EDIT_MODEL_HSV_RS: &str = include_str!("../src/controls/color_edit/model/hsv.rs");
const COLOR_EDIT_MODEL_NUMERIC_RS: &str =
    include_str!("../src/controls/color_edit/model/numeric.rs");
const COLOR_EDIT_MODEL_NUMERIC_MODE_RS: &str =
    include_str!("../src/controls/color_edit/model/numeric/mode.rs");
const COLOR_EDIT_MODEL_NUMERIC_PARSE_RS: &str =
    include_str!("../src/controls/color_edit/model/numeric/parse.rs");
const COLOR_EDIT_MODEL_NUMERIC_TEXT_RS: &str =
    include_str!("../src/controls/color_edit/model/numeric/text.rs");
const COLOR_EDIT_OPTIONS_RS: &str = include_str!("../src/controls/color_edit/options.rs");
const COLOR_EDIT_OPTIONS_POLICIES_RS: &str =
    include_str!("../src/controls/color_edit/options/policies.rs");
const COLOR_EDIT_OPTIONS_POPUP_RS: &str =
    include_str!("../src/controls/color_edit/options/popup.rs");
const COLOR_EDIT_RECORDS_RS: &str = include_str!("../src/controls/color_edit/records.rs");
const COLOR_EDIT_STATE_RS: &str = include_str!("../src/controls/color_edit/state.rs");
const COLOR_EDIT_SWATCH_RS: &str = include_str!("../src/controls/color_edit/swatch.rs");
const COLOR_EDIT_SWATCH_CONTEXT_MENU_RS: &str =
    include_str!("../src/controls/color_edit/swatch/context_menu.rs");
const COLOR_EDIT_SWATCH_ELEMENT_RS: &str =
    include_str!("../src/controls/color_edit/swatch/element.rs");
const COLOR_EDIT_SWATCH_VISUAL_RS: &str =
    include_str!("../src/controls/color_edit/swatch/visual.rs");
const COLOR_EDIT_POPUP_RS: &str = include_str!("../src/controls/color_edit/popup.rs");
const COLOR_EDIT_POPUP_REQUEST_RS: &str =
    include_str!("../src/controls/color_edit/popup/request.rs");
const COLOR_EDIT_POPUP_BODY_RS: &str = include_str!("../src/controls/color_edit/popup/body.rs");
const COLOR_EDIT_POPUP_BODY_LAYOUT_RS: &str =
    include_str!("../src/controls/color_edit/popup/body/layout.rs");
const COLOR_EDIT_POPUP_BODY_SECTIONS_RS: &str =
    include_str!("../src/controls/color_edit/popup/body/sections.rs");
const COLOR_EDIT_POPUP_BODY_SECTIONS_ASSEMBLY_RS: &str =
    include_str!("../src/controls/color_edit/popup/body/sections/assembly.rs");
const COLOR_EDIT_POPUP_BODY_SECTIONS_ACTIONS_RS: &str =
    include_str!("../src/controls/color_edit/popup/body/sections/actions.rs");
const COLOR_EDIT_POPUP_BODY_SECTIONS_PICKER_RS: &str =
    include_str!("../src/controls/color_edit/popup/body/sections/picker.rs");
const COLOR_EDIT_POPUP_BODY_SECTIONS_PREVIEW_RS: &str =
    include_str!("../src/controls/color_edit/popup/body/sections/preview.rs");
const COLOR_EDIT_POPUP_BODY_SECTIONS_SWATCHES_RS: &str =
    include_str!("../src/controls/color_edit/popup/body/sections/swatches.rs");
const COLOR_EDIT_POPUP_NUMERIC_RS: &str =
    include_str!("../src/controls/color_edit/popup/numeric.rs");
const COLOR_EDIT_POPUP_NUMERIC_FIELD_RS: &str =
    include_str!("../src/controls/color_edit/popup/numeric/field.rs");
const COLOR_EDIT_POPUP_OPTIONS_RS: &str =
    include_str!("../src/controls/color_edit/popup/options.rs");
const COLOR_EDIT_POPUP_OPTIONS_PICKER_RS: &str =
    include_str!("../src/controls/color_edit/popup/options/picker.rs");
const COLOR_EDIT_POPUP_OPTIONS_PICKER_CARD_RS: &str =
    include_str!("../src/controls/color_edit/popup/options/picker/card.rs");
const COLOR_EDIT_POPUP_OPTIONS_THUMBNAIL_RS: &str =
    include_str!("../src/controls/color_edit/popup/options/thumbnail.rs");
const COLOR_EDIT_POPUP_PICKER_RS: &str = include_str!("../src/controls/color_edit/popup/picker.rs");
const COLOR_EDIT_POPUP_PICKER_LAYOUT_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/layout.rs");
const COLOR_EDIT_POPUP_PICKER_ALPHA_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/alpha.rs");
const COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/alpha/bar.rs");
const COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_POINTER_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/alpha/bar/pointer.rs");
const COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_SURFACE_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/alpha/bar/surface.rs");
const COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/alpha/preview.rs");
const COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_GRADIENT_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/alpha/preview/gradient.rs");
const COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_THUMB_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/alpha/preview/thumb.rs");
const COLOR_EDIT_POPUP_PICKER_HUE_BAR_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/hue_bar.rs");
const COLOR_EDIT_POPUP_PICKER_HUE_BAR_BAR_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/hue_bar/bar.rs");
const COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/hue_bar/preview.rs");
const COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_GRADIENT_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/hue_bar/preview/gradient.rs");
const COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_THUMB_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/hue_bar/preview/thumb.rs");
const COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/hue_wheel.rs");
const COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/hue_wheel_picker.rs");
const COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_POINTER_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/hue_wheel_picker/pointer.rs");
const COLOR_EDIT_POPUP_PICKER_SV_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/sv.rs");
const COLOR_EDIT_POPUP_PICKER_SV_BAR_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/sv/bar.rs");
const COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/sv/preview.rs");
const COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_GRID_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/sv/preview/grid.rs");
const COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_THUMB_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/sv/preview/thumb.rs");
const COLOR_EDIT_POPUP_PREVIEW_RS: &str =
    include_str!("../src/controls/color_edit/popup/preview.rs");
const COLOR_EDIT_POPUP_PREVIEW_FILL_RS: &str =
    include_str!("../src/controls/color_edit/popup/preview/fill.rs");
const COLOR_EDIT_POPUP_PREVIEW_FILL_CHECKERBOARD_RS: &str =
    include_str!("../src/controls/color_edit/popup/preview/fill/checkerboard.rs");
const COLOR_EDIT_POPUP_PREVIEW_SIDE_RS: &str =
    include_str!("../src/controls/color_edit/popup/preview/side.rs");
const COLOR_EDIT_POPUP_PREVIEW_SIDE_CELL_RS: &str =
    include_str!("../src/controls/color_edit/popup/preview/side/cell.rs");
const COLOR_EDIT_POPUP_PREVIEW_SIDE_ORIGINAL_RS: &str =
    include_str!("../src/controls/color_edit/popup/preview/side/original.rs");
const COLOR_EDIT_POPUP_SWATCHES_RS: &str =
    include_str!("../src/controls/color_edit/popup/swatches.rs");
const COLOR_EDIT_POPUP_SWATCHES_SLOT_RS: &str =
    include_str!("../src/controls/color_edit/popup/swatches/slot.rs");
const COLOR_EDIT_POPUP_TOOLTIP_RS: &str =
    include_str!("../src/controls/color_edit/popup/tooltip.rs");
const COLOR_EDIT_POPUP_TOOLTIP_PANEL_RS: &str =
    include_str!("../src/controls/color_edit/popup/tooltip/panel.rs");
const COLOR_EDIT_TESTS_PICKER_RS: &str = include_str!("../src/controls/color_edit/tests/picker.rs");
const COLOR_EDIT_TESTS_PICKER_BARS_RS: &str =
    include_str!("../src/controls/color_edit/tests/picker/bars.rs");
const COLOR_EDIT_TESTS_PICKER_HUE_WHEEL_RS: &str =
    include_str!("../src/controls/color_edit/tests/picker/hue_wheel.rs");
const COLOR_EDIT_TESTS_PICKER_HUE_WHEEL_TRIANGLE_RS: &str =
    include_str!("../src/controls/color_edit/tests/picker/hue_wheel_triangle.rs");
const COLOR_EDIT_TESTS_PICKER_PREVIEW_ALPHA_RS: &str =
    include_str!("../src/controls/color_edit/tests/picker/preview_alpha.rs");
const COLOR_EDIT_TESTS_POPUP_POLICY_RS: &str =
    include_str!("../src/controls/color_edit/tests/popup_policy.rs");
const COLOR_EDIT_TESTS_POPUP_POLICY_DEFAULTS_RS: &str =
    include_str!("../src/controls/color_edit/tests/popup_policy/defaults.rs");
const COLOR_EDIT_TESTS_POPUP_POLICY_RUNTIME_RS: &str =
    include_str!("../src/controls/color_edit/tests/popup_policy/runtime.rs");
const COLOR_EDIT_TESTS_POPUP_POLICY_VISIBILITY_RS: &str =
    include_str!("../src/controls/color_edit/tests/popup_policy/visibility.rs");
const COLOR_EDIT_TESTS_NUMERIC_RS: &str =
    include_str!("../src/controls/color_edit/tests/numeric.rs");
const COLOR_EDIT_TESTS_NUMERIC_CONVERSION_RS: &str =
    include_str!("../src/controls/color_edit/tests/numeric/conversion.rs");
const COLOR_EDIT_TESTS_NUMERIC_HEX_RS: &str =
    include_str!("../src/controls/color_edit/tests/numeric/hex.rs");
const COLOR_EDIT_TESTS_NUMERIC_INPUT_RS: &str =
    include_str!("../src/controls/color_edit/tests/numeric/input.rs");
const COLOR_EDIT_TESTS_NUMERIC_MODES_RS: &str =
    include_str!("../src/controls/color_edit/tests/numeric/modes.rs");

fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

#[test]
fn color_edit_popup_is_a_real_preset_palette_not_a_stub() {
    assert!(COLOR_EDIT_RS.contains("mod element;"));
    assert!(COLOR_EDIT_RS.contains("pub use self::element::ColorEdit;"));
    assert!(COLOR_EDIT_RS.contains("mod input;"));
    assert!(COLOR_EDIT_RS.contains("mod layout;"));
    assert!(COLOR_EDIT_RS.contains("mod options;"));
    assert!(COLOR_EDIT_RS.contains("pub use self::options::{"));
    assert!(COLOR_EDIT_RS.contains("mod records;"));
    assert!(COLOR_EDIT_RS.contains("pub use self::records::{"));
    assert!(COLOR_EDIT_RS.contains("mod state;"));
    assert!(COLOR_EDIT_RS.contains("mod swatch;"));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("pub struct ColorEdit"));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("fn into_element_keyed"));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("mod affordance;"));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("mod frame;"));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("use self::frame::color_edit_into_element_keyed;"));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("mod keying;"));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("use self::keying::color_edit_into_element;"));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("color_edit_into_element(self, cx)"));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("color_edit_into_element_keyed(self, cx)"));
    assert!(!COLOR_EDIT_ELEMENT_RS.contains("Location::caller"));
    assert!(!COLOR_EDIT_ELEMENT_RS.contains("cx.keyed("));
    assert!(!COLOR_EDIT_ELEMENT_RS.contains("color_hex_input("));
    assert!(!COLOR_EDIT_ELEMENT_RS.contains("color_swatch("));
    assert!(!COLOR_EDIT_ELEMENT_RS.contains("request_popup_overlay("));
    assert!(!COLOR_EDIT_ELEMENT_RS.contains("apply_delivered_color_drop("));
    assert!(COLOR_EDIT_ELEMENT_KEYING_RS.contains("ColorEdit caller-keyed element routing owner."));
    assert!(COLOR_EDIT_ELEMENT_KEYING_RS.contains("fn color_edit_into_element<"));
    assert!(COLOR_EDIT_ELEMENT_KEYING_RS.contains("type Callsite = (&'static str, u32, u32);"));
    assert!(COLOR_EDIT_ELEMENT_KEYING_RS.contains("Location::caller"));
    assert!(COLOR_EDIT_ELEMENT_KEYING_RS.contains("id_source.as_deref()"));
    assert!(COLOR_EDIT_ELEMENT_KEYING_RS.contains("\"fret-ui-editor.color_edit\""));
    assert!(COLOR_EDIT_ELEMENT_KEYING_RS.contains("control.into_element_keyed(cx)"));
    assert!(COLOR_EDIT_ELEMENT_KEYING_RS.contains("fn current_callsite() -> Callsite"));
    assert!(!COLOR_EDIT_ELEMENT_KEYING_RS.contains("color_hex_input("));
    assert!(!COLOR_EDIT_ELEMENT_KEYING_RS.contains("color_swatch("));
    assert!(!COLOR_EDIT_ELEMENT_KEYING_RS.contains("request_popup_overlay("));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("mod test_ids;"));
    assert!(!COLOR_EDIT_ELEMENT_RS.contains("use self::test_ids::color_edit_element_test_ids;"));
    assert!(!COLOR_EDIT_ELEMENT_RS.contains("color_edit_element_test_ids(&"));
    assert!(!COLOR_EDIT_ELEMENT_RS.contains("derived_test_id("));
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("use super::ColorEdit;"));
    assert!(
        COLOR_EDIT_ELEMENT_FRAME_RS
            .contains("use super::affordance::color_edit_frame_affordances;")
    );
    assert!(
        COLOR_EDIT_ELEMENT_FRAME_RS.contains("use super::test_ids::color_edit_element_test_ids;")
    );
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("mod overlays;"));
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("fn color_edit_into_element_keyed<"));
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("color_edit_element_test_ids(&control.options)"));
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("color_edit_frame_affordances("));
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("color_hex_input("));
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("color_swatch("));
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("ColorEditFrameOverlayArgs"));
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("request_color_edit_frame_overlays("));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_RS.contains("request_popup_overlay("));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_RS.contains("request_color_tooltip_overlay("));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_RS.contains("request_color_copy_menu_overlay("));
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("color_edit_root_layout("));
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("ColorEditDeliveredDropArgs"));
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("apply_delivered_color_drop("));
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("sync_popup_runtime_options"));
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("test_id: test_ids.input.clone()"));
    assert!(COLOR_EDIT_ELEMENT_FRAME_RS.contains("test_id: test_ids.swatch.clone()"));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_RS.contains("test_ids.popup.clone()"));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_RS.contains("test_ids.tooltip.clone()"));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_RS.contains("test_ids.copy_menu.clone()"));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_RS.contains("test_ids.eyedropper.clone()"));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_RS.contains("has_visible_content_with_swatches"));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_RS.contains("tooltip_enabled"));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_RS.contains("Location::caller"));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_RS.contains("cx.keyed("));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_RS.contains("derived_test_id("));
    assert!(COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("struct ColorEditFrameOverlayArgs"));
    assert!(COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("fn request_color_edit_frame_overlays<"));
    assert!(COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("request_popup_overlay("));
    assert!(COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("request_color_tooltip_overlay("));
    assert!(COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("request_color_copy_menu_overlay("));
    assert!(COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("test_ids.popup.clone()"));
    assert!(COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("test_ids.tooltip.clone()"));
    assert!(COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("test_ids.copy_menu.clone()"));
    assert!(COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("test_ids.eyedropper.clone()"));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("color_hex_input("));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("color_swatch("));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("apply_delivered_color_drop("));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("color_edit_root_layout("));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("Location::caller"));
    assert!(!COLOR_EDIT_ELEMENT_FRAME_OVERLAYS_RS.contains("cx.keyed("));
    assert!(COLOR_EDIT_ELEMENT_AFFORDANCE_RS.contains("struct ColorEditFrameAffordances"));
    assert!(COLOR_EDIT_ELEMENT_AFFORDANCE_RS.contains("fn color_edit_frame_affordances("));
    assert!(COLOR_EDIT_ELEMENT_AFFORDANCE_RS.contains("has_visible_content_with_swatches"));
    assert!(COLOR_EDIT_ELEMENT_AFFORDANCE_RS.contains("drag_drop_enabled"));
    assert!(COLOR_EDIT_ELEMENT_AFFORDANCE_RS.contains("tooltip_enabled"));
    assert!(COLOR_EDIT_ELEMENT_AFFORDANCE_RS.contains("copy_enabled"));
    assert!(COLOR_EDIT_ELEMENT_AFFORDANCE_RS.contains("eyedropper_enabled"));
    assert!(COLOR_EDIT_ELEMENT_AFFORDANCE_RS.contains("swatch_enabled"));
    assert!(COLOR_EDIT_ELEMENT_AFFORDANCE_RS.contains("swatch_focusable"));
    assert!(!COLOR_EDIT_ELEMENT_AFFORDANCE_RS.contains("color_hex_input("));
    assert!(!COLOR_EDIT_ELEMENT_AFFORDANCE_RS.contains("color_swatch("));
    assert!(!COLOR_EDIT_ELEMENT_AFFORDANCE_RS.contains("request_popup_overlay("));
    assert!(COLOR_EDIT_ELEMENT_TEST_IDS_RS.contains("struct ColorEditElementTestIds"));
    assert!(COLOR_EDIT_ELEMENT_TEST_IDS_RS.contains("fn color_edit_element_test_ids("));
    assert!(COLOR_EDIT_ELEMENT_TEST_IDS_RS.contains("ColorEditOptions"));
    assert!(COLOR_EDIT_ELEMENT_TEST_IDS_RS.contains("derived_test_id"));
    assert!(COLOR_EDIT_ELEMENT_TEST_IDS_RS.contains("\"input\""));
    assert!(COLOR_EDIT_ELEMENT_TEST_IDS_RS.contains("\"swatch\""));
    assert!(COLOR_EDIT_ELEMENT_TEST_IDS_RS.contains("\"popup\""));
    assert!(COLOR_EDIT_ELEMENT_TEST_IDS_RS.contains("\"tooltip\""));
    assert!(COLOR_EDIT_ELEMENT_TEST_IDS_RS.contains("\"copy-menu\""));
    assert!(COLOR_EDIT_ELEMENT_TEST_IDS_RS.contains("\"eyedropper\""));
    assert!(!COLOR_EDIT_ELEMENT_TEST_IDS_RS.contains("color_hex_input("));
    assert!(!COLOR_EDIT_ELEMENT_TEST_IDS_RS.contains("color_swatch("));
    assert!(!COLOR_EDIT_ELEMENT_TEST_IDS_RS.contains("request_popup_overlay("));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub struct ColorEditOptions"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("mod policies;"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("mod popup;"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub use policies::{"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub use popup::{"));
    assert!(
        COLOR_EDIT_OPTIONS_RS.contains(
            "pub(in crate::controls::color_edit) use popup::ColorEditPopupRuntimeOptions"
        )
    );
    assert!(COLOR_EDIT_INPUT_RS.contains("pub(super) struct ColorEditInputArgs"));
    assert!(COLOR_EDIT_INPUT_RS.contains("pub(super) fn color_hex_input<"));
    assert!(COLOR_EDIT_INPUT_RS.contains("TextInputProps::new"));
    assert!(COLOR_EDIT_INPUT_RS.contains("key_add_on_key_down_capture_for"));
    assert!(COLOR_EDIT_INPUT_RS.contains("KeyCode::Enter"));
    assert!(COLOR_EDIT_INPUT_RS.contains("KeyCode::Escape"));
    assert!(COLOR_EDIT_INPUT_RS.contains("PointerRegionProps"));
    assert!(COLOR_EDIT_INPUT_RS.contains("parse_hex("));
    assert!(COLOR_EDIT_LAYOUT_RS.contains("pub(super) struct ColorEditRootLayoutArgs"));
    assert!(COLOR_EDIT_LAYOUT_RS.contains("pub(super) fn color_edit_root_layout<"));
    assert!(COLOR_EDIT_LAYOUT_RS.contains("editor_inline_error_text_props"));
    assert!(COLOR_EDIT_LAYOUT_RS.contains("Axis::Vertical"));
    assert!(COLOR_EDIT_LAYOUT_RS.contains("Axis::Horizontal"));
    assert!(COLOR_EDIT_LAYOUT_RS.contains("FlexProps"));
    assert!(COLOR_EDIT_LAYOUT_RS.contains("el.test_id(test_id.clone())"));
    assert!(COLOR_EDIT_SWATCH_RS.contains("pub(super) struct ColorEditSwatchArgs"));
    assert!(COLOR_EDIT_SWATCH_RS.contains("mod element;"));
    assert!(COLOR_EDIT_SWATCH_RS.contains("pub(super) use element::color_swatch;"));
    assert!(!COLOR_EDIT_SWATCH_RS.contains("pub(super) fn color_swatch<"));
    assert!(!COLOR_EDIT_SWATCH_RS.contains("PressableProps"));
    assert!(!COLOR_EDIT_SWATCH_RS.contains("pressable_add_on_activate"));
    assert!(
        COLOR_EDIT_SWATCH_ELEMENT_RS
            .contains("pub(in crate::controls::color_edit) fn color_swatch<")
    );
    assert!(COLOR_EDIT_SWATCH_ELEMENT_RS.contains("PressableProps"));
    assert!(COLOR_EDIT_SWATCH_ELEMENT_RS.contains("pressable_add_on_activate"));
    assert!(COLOR_EDIT_SWATCH_RS.contains("mod context_menu;"));
    assert!(!COLOR_EDIT_SWATCH_RS.contains("install_context_menu_pointer_handler"));
    assert!(!COLOR_EDIT_SWATCH_RS.contains("install_context_menu_keyboard_handler"));
    assert!(COLOR_EDIT_SWATCH_ELEMENT_RS.contains("install_context_menu_pointer_handler"));
    assert!(COLOR_EDIT_SWATCH_ELEMENT_RS.contains("install_context_menu_keyboard_handler"));
    assert!(!COLOR_EDIT_SWATCH_RS.contains("MouseButton::Right"));
    assert!(!COLOR_EDIT_SWATCH_RS.contains("KeyCode::ContextMenu"));
    assert!(!COLOR_EDIT_SWATCH_RS.contains("PressablePointerDownResult"));
    assert!(COLOR_EDIT_SWATCH_CONTEXT_MENU_RS.contains("pressable_add_on_pointer_down"));
    assert!(COLOR_EDIT_SWATCH_CONTEXT_MENU_RS.contains("MouseButton::Right"));
    assert!(COLOR_EDIT_SWATCH_CONTEXT_MENU_RS.contains("KeyCode::F10"));
    assert!(COLOR_EDIT_SWATCH_CONTEXT_MENU_RS.contains("KeyCode::ContextMenu"));
    assert!(
        COLOR_EDIT_SWATCH_CONTEXT_MENU_RS
            .contains("PressablePointerDownResult::SkipDefaultAndStopPropagation")
    );
    assert!(COLOR_EDIT_SWATCH_CONTEXT_MENU_RS.contains("key_on_key_down_for"));
    assert!(!COLOR_EDIT_SWATCH_RS.contains("ColorEditDragDropPayload::from_color"));
    assert!(!COLOR_EDIT_SWATCH_RS.contains("install_color_drag_source"));
    assert!(!COLOR_EDIT_SWATCH_RS.contains("update_color_drop_target"));
    assert!(COLOR_EDIT_SWATCH_ELEMENT_RS.contains("ColorEditDragDropPayload::from_color"));
    assert!(COLOR_EDIT_SWATCH_ELEMENT_RS.contains("install_color_drag_source"));
    assert!(COLOR_EDIT_SWATCH_ELEMENT_RS.contains("update_color_drop_target"));
    assert!(!COLOR_EDIT_SWATCH_RS.contains("EditorWidgetVisuals"));
    assert!(COLOR_EDIT_SWATCH_ELEMENT_RS.contains("color_swatch_visual("));
    assert!(COLOR_EDIT_SWATCH_ELEMENT_RS.contains("ColorSwatchVisualArgs {"));
    assert!(COLOR_EDIT_SWATCH_ELEMENT_RS.contains("COLOR_SWATCH_SIZE"));
    assert!(COLOR_EDIT_SWATCH_VISUAL_RS.contains("EditorWidgetVisuals"));
    assert!(COLOR_EDIT_SWATCH_CONTEXT_MENU_RS.contains("key_on_key_down_for"));
    assert!(COLOR_EDIT_SWATCH_CONTEXT_MENU_RS.contains("KeyCode::ContextMenu"));
    assert!(!COLOR_EDIT_SWATCH_RS.contains("color_preview_stack"));
    assert!(COLOR_EDIT_SWATCH_VISUAL_RS.contains("color_preview_stack"));
    assert!(COLOR_EDIT_RECORDS_RS.contains("const COLOR_PRESETS:"));
    assert!(COLOR_EDIT_POPUP_SWATCHES_RS.contains("mod slot;"));
    assert!(COLOR_EDIT_POPUP_SWATCHES_RS.contains("use self::slot::preset_swatch;"));
    assert!(COLOR_EDIT_POPUP_SWATCHES_RS.contains("fn swatch_row<"));
    assert!(!COLOR_EDIT_POPUP_SWATCHES_RS.contains("take_delivered_color_drop("));
    assert!(!COLOR_EDIT_POPUP_SWATCHES_RS.contains("ColorEditPaletteSlotDrop::new("));
    assert!(COLOR_EDIT_POPUP_SWATCHES_SLOT_RS.contains("fn preset_swatch<"));
    assert!(COLOR_EDIT_POPUP_SWATCHES_SLOT_RS.contains("install_color_drag_source("));
    assert!(COLOR_EDIT_POPUP_SWATCHES_SLOT_RS.contains("take_delivered_color_drop("));
    assert!(COLOR_EDIT_POPUP_SWATCHES_SLOT_RS.contains("ColorEditPaletteSlotDrop::new("));
    assert!(COLOR_EDIT_POPUP_PREVIEW_RS.contains("use fill::{"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_RS.contains("mod side;"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_RS.contains("use side::{"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_FILL_RS.contains("fn color_preview_stack<"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_RS.contains(
        "pub(in crate::controls::color_edit) use fill::checkerboard::checkerboard_cell_color;"
    ));
    assert!(
        COLOR_EDIT_POPUP_PREVIEW_FILL_RS
            .contains("pub(in crate::controls::color_edit) mod checkerboard;")
    );
    assert!(COLOR_EDIT_POPUP_PREVIEW_FILL_RS.contains(
        "pub(in crate::controls::color_edit::popup) use checkerboard::checkerboard_grid"
    ));
    assert!(
        !COLOR_EDIT_POPUP_PREVIEW_FILL_RS.contains(
            "pub(in crate::controls::color_edit) use checkerboard::checkerboard_cell_color"
        )
    );
    assert!(!COLOR_EDIT_POPUP_PREVIEW_FILL_RS.contains("fn checkerboard_grid<"));
    assert!(!COLOR_EDIT_POPUP_PREVIEW_FILL_RS.contains("fn checkerboard_cell_color("));
    assert!(COLOR_EDIT_POPUP_PREVIEW_FILL_CHECKERBOARD_RS.contains("fn checkerboard_grid<"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_FILL_CHECKERBOARD_RS.contains("fn checkerboard_cell_color("));
    assert!(COLOR_EDIT_POPUP_PREVIEW_FILL_CHECKERBOARD_RS.contains("CHECKERBOARD_LIGHT_RGB"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_FILL_CHECKERBOARD_RS.contains("GridProps"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("mod cell;"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("mod original;"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("use cell::current_preview_cell;"));
    assert!(
        COLOR_EDIT_POPUP_PREVIEW_SIDE_RS
            .contains("pub(in crate::controls::color_edit) use cell::{")
    );
    assert!(
        COLOR_EDIT_POPUP_PREVIEW_SIDE_RS
            .contains("pub(in crate::controls::color_edit) use original::restore_reference_color;")
    );
    assert!(
        COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("original::original_reference_preview_cell(")
    );
    assert!(!COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("OnActivate"));
    assert!(!COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("PressableProps"));
    assert!(!COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("fn current_preview_cell<"));
    assert!(!COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("fn preview_cell_content<"));
    assert!(!COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("fn preview_cell_layout()"));
    assert!(!COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("editor_preview_caption_text_props"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_CELL_RS.contains("fn current_preview_cell<"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_CELL_RS.contains("fn preview_cell_content<"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_CELL_RS.contains("fn preview_cell_layout()"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_CELL_RS.contains("SIDE_PREVIEW_SWATCH_WIDTH"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_CELL_RS.contains("SIDE_PREVIEW_SWATCH_HEIGHT"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_CELL_RS.contains("editor_preview_caption_text_props"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_CELL_RS.contains("color_preview_stack"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_CELL_RS.contains("format_hex"));
    assert!(
        COLOR_EDIT_POPUP_PREVIEW_SIDE_ORIGINAL_RS.contains("fn original_reference_preview_cell<")
    );
    assert!(
        COLOR_EDIT_POPUP_PREVIEW_SIDE_ORIGINAL_RS
            .contains("use super::cell::{preview_cell_content, preview_cell_layout};")
    );
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_ORIGINAL_RS.contains("fn restore_reference_color("));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_ORIGINAL_RS.contains("OnActivate"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_ORIGINAL_RS.contains("PressableProps"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("mod hue_wheel;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("pub(in crate::controls::color_edit) mod alpha;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("pub(super) use alpha::alpha_bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("mod hue_bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("pub(super) use hue_bar::hue_bar_preview_stack;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("mod hue_wheel_picker;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("mod layout;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("mod sv;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("pub(super) use sv::sv_picker_preview_stack;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("pub(super) use hue_wheel::hue_wheel_canvas;"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_RS
            .contains("pub(super) use layout::{hsv_hue_wheel_picker, hsv_picker};")
    );
    assert!(!COLOR_EDIT_POPUP_PICKER_RS.contains("fn hsv_picker<"));
    assert!(!COLOR_EDIT_POPUP_PICKER_RS.contains("fn hsv_hue_wheel_picker<"));
    assert!(!COLOR_EDIT_POPUP_PICKER_RS.contains("derived_test_id("));
    assert!(!COLOR_EDIT_POPUP_PICKER_RS.contains("cx.flex("));
    assert!(COLOR_EDIT_POPUP_PICKER_LAYOUT_RS.contains("fn hsv_picker<"));
    assert!(COLOR_EDIT_POPUP_PICKER_LAYOUT_RS.contains("fn hsv_hue_wheel_picker<"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_LAYOUT_RS.contains("derived_test_id(test_id.as_ref(), \"sv\")")
    );
    assert!(
        COLOR_EDIT_POPUP_PICKER_LAYOUT_RS.contains("derived_test_id(test_id.as_ref(), \"wheel\")")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_LAYOUT_RS.contains("let sv = sv_picker("));
    assert!(COLOR_EDIT_POPUP_PICKER_LAYOUT_RS.contains("let hue = hue_bar("));
    assert!(COLOR_EDIT_POPUP_PICKER_LAYOUT_RS.contains("let wheel = hue_wheel_picker("));
    assert!(COLOR_EDIT_POPUP_PICKER_LAYOUT_RS.contains("vertical_alpha_bar(cx, current,"));
    assert!(COLOR_EDIT_POPUP_PICKER_LAYOUT_RS.contains("FlexProps"));
    assert!(COLOR_EDIT_POPUP_PICKER_LAYOUT_RS.contains("direction: Axis::Horizontal"));
    assert!(COLOR_EDIT_POPUP_PICKER_LAYOUT_RS.contains("gap: SpacingLength::Px(Px(6.0))"));
    assert!(!COLOR_EDIT_POPUP_PICKER_LAYOUT_RS.contains("apply_hsv_color("));
    assert!(!COLOR_EDIT_POPUP_PICKER_LAYOUT_RS.contains("horizontal_bar_thumb_spacer"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_RS.contains("mod bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_RS.contains("mod interaction;"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_RS.contains("mod preview;"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_RS.contains("pub(super) use bar::sv_picker;"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_RS.contains(
        "pub(in crate::controls::color_edit::popup) use preview::sv_picker_preview_stack;"
    ));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_BAR_RS.contains("fn sv_picker<"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_RS.contains("mod grid;"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_RS.contains("mod thumb;"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_RS.contains("fn sv_picker_preview_stack<"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_RS.contains("sv_picker_grid(cx, hsv.hue)"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_RS
            .contains("sv_picker_thumb_overlay(cx, hsv.saturation, hsv.value)")
    );
    assert!(!COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_RS.contains("fn sv_picker_grid<"));
    assert!(!COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_RS.contains("fn sv_picker_thumb_overlay<"));
    assert!(!COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_RS.contains("SV_PICKER_STEPS"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_GRID_RS.contains("fn sv_picker_grid<"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_GRID_RS.contains("SV_PICKER_STEPS"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_GRID_RS.contains("unit_from_step"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_GRID_RS.contains("GridProps"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_THUMB_RS.contains("fn sv_picker_thumb_overlay<"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_THUMB_RS.contains("fn sv_thumb_vertical_spacer<"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_THUMB_RS.contains("Axis::Vertical"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_THUMB_RS.contains("horizontal_bar_thumb_spacer"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_RS.contains("mod bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_RS.contains("mod interaction;"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_RS.contains("mod preview;"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_RS.contains("pub(super) use bar::hue_bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_RS.contains(
        "pub(in crate::controls::color_edit::popup) use preview::hue_bar_preview_stack;"
    ));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_BAR_RS.contains("fn hue_bar<"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_RS.contains("mod gradient;"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_RS.contains("mod thumb;"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_RS.contains("fn hue_bar_preview_stack<"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_RS.contains("vertical_hue_gradient_overlay(cx)")
    );
    assert!(
        COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_RS.contains("vertical_bar_thumb_overlay(cx, hue)")
    );
    assert!(
        !COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_RS.contains("fn vertical_hue_gradient_overlay<")
    );
    assert!(!COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_RS.contains("fn vertical_bar_thumb_overlay<"));
    assert!(!COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_RS.contains("HUE_BAR_STEPS"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_GRADIENT_RS
            .contains("fn vertical_hue_gradient_overlay<")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_GRADIENT_RS.contains("HUE_BAR_STEPS"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_GRADIENT_RS.contains("GridProps"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_THUMB_RS.contains("fn vertical_bar_thumb_overlay<")
    );
    assert!(
        COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_THUMB_RS.contains("fn vertical_bar_thumb_spacer<")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_THUMB_RS.contains("Axis::Vertical"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_RS.contains("mod bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_RS.contains("mod interaction;"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_RS.contains("mod preview;"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_RS
            .contains("pub(in crate::controls::color_edit::popup) use bar::alpha_bar;")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_RS.contains("pub(super) use bar::vertical_alpha_bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS.contains("mod pointer;"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS.contains("mod surface;"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS.contains("fn vertical_alpha_bar<"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS
            .contains("install_vertical_alpha_bar_pointer_handlers(")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS.contains("install_alpha_bar_pointer_handlers("));
    assert!(!COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS.contains("pressable_add_on_pointer_down"));
    assert!(!COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS.contains("MouseButton::Left"));
    assert!(!COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS.contains("apply_alpha_bar_position("));
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_POINTER_RS
            .contains("fn install_vertical_alpha_bar_pointer_handlers<")
    );
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_POINTER_RS
            .contains("fn install_alpha_bar_pointer_handlers<")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_POINTER_RS.contains("pressable_add_on_pointer_down"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_POINTER_RS.contains("pressable_add_on_pointer_move"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_POINTER_RS.contains("pressable_add_on_pointer_up"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_POINTER_RS.contains("MouseButton::Left"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_POINTER_RS.contains("apply_alpha_bar_position("));
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_POINTER_RS.contains("apply_vertical_alpha_bar_position(")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_POINTER_RS.contains("host.capture_pointer()"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_POINTER_RS.contains("host.release_pointer_capture()")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS.contains("vertical_alpha_bar_surface("));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS.contains("alpha_bar_surface("));
    assert!(!COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS.contains("picker_border_and_ring"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_SURFACE_RS.contains("fn vertical_alpha_bar_surface<")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_SURFACE_RS.contains("fn alpha_bar_surface<"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_SURFACE_RS.contains("picker_border_and_ring"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_SURFACE_RS.contains("vertical_alpha_bar_preview_stack")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_SURFACE_RS.contains("alpha_bar_preview_stack"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_RS.contains("mod gradient;"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_RS.contains("mod thumb;"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_RS.contains("vertical_alpha_gradient_overlay"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_RS.contains("horizontal_bar_thumb_overlay"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_GRADIENT_RS
            .contains("fn vertical_alpha_gradient_overlay<")
    );
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_GRADIENT_RS.contains("fn alpha_gradient_overlay<")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_GRADIENT_RS.contains("ALPHA_BAR_STEPS"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_THUMB_RS.contains("fn horizontal_bar_thumb_overlay<")
    );
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_THUMB_RS.contains("fn vertical_bar_thumb_overlay<")
    );
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_THUMB_RS.contains("fn vertical_bar_thumb_spacer<")
    );
    assert!(COLOR_EDIT_POPUP_NUMERIC_RS.contains("fn color_numeric_inputs<"));
    assert!(COLOR_EDIT_POPUP_NUMERIC_RS.contains("mod field;"));
    assert!(COLOR_EDIT_POPUP_NUMERIC_RS.contains("fn color_numeric_error_line<"));
    assert!(COLOR_EDIT_POPUP_NUMERIC_FIELD_RS.contains("fn color_numeric_input_field<"));
    assert!(COLOR_EDIT_POPUP_NUMERIC_FIELD_RS.contains("parse_color_numeric_input"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_RS.contains("fn color_picker_options<"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_PICKER_RS.contains("mod card;"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_PICKER_RS.contains("use card::picker_option_button;"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_PICKER_RS.contains("fn picker_options_row<"));
    assert!(!COLOR_EDIT_POPUP_OPTIONS_PICKER_RS.contains("fn picker_option_button<"));
    assert!(!COLOR_EDIT_POPUP_OPTIONS_PICKER_RS.contains("PressableProps"));
    assert!(!COLOR_EDIT_POPUP_OPTIONS_PICKER_RS.contains("runtime.picker = picker"));
    assert!(!COLOR_EDIT_POPUP_OPTIONS_PICKER_RS.contains("picker_option_thumbnail"));
    assert!(
        !COLOR_EDIT_POPUP_OPTIONS_PICKER_RS.contains("editor_popup_list_option_caption_text_props")
    );
    assert!(!COLOR_EDIT_POPUP_OPTIONS_PICKER_RS.contains("hsv_from_color"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_PICKER_CARD_RS.contains("fn picker_option_button<"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_PICKER_CARD_RS.contains("PressableProps"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_PICKER_CARD_RS.contains("runtime.picker = picker"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_PICKER_CARD_RS.contains("picker_option_thumbnail"));
    assert!(
        COLOR_EDIT_POPUP_OPTIONS_PICKER_CARD_RS
            .contains("editor_popup_list_option_caption_text_props")
    );
    assert!(COLOR_EDIT_POPUP_OPTIONS_PICKER_CARD_RS.contains("hsv_from_color"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_RS.contains("fn alpha_bar_option<"));
    assert!(COLOR_EDIT_MODEL_RS.contains("mod hsv;"));
    assert!(COLOR_EDIT_MODEL_RS.contains("pub(super) use hsv::{"));
    assert!(COLOR_EDIT_MODEL_RS.contains("mod numeric;"));
    assert!(COLOR_EDIT_MODEL_RS.contains("pub(super) use numeric::{"));
    assert!(COLOR_EDIT_MODEL_HSV_RS.contains("struct HsvColor"));
    assert!(COLOR_EDIT_MODEL_HSV_RS.contains("fn hsv_from_color("));
    assert!(COLOR_EDIT_MODEL_HSV_RS.contains("fn rgb_to_hsv("));
    assert!(COLOR_EDIT_MODEL_HSV_RS.contains("fn hsv_to_rgb("));
    assert!(COLOR_EDIT_MODEL_HSV_RS.contains("fn hsv_with_sv_from_local_position("));
    assert!(COLOR_EDIT_MODEL_HSV_RS.contains("fn hue_from_local_y("));
    assert!(COLOR_EDIT_MODEL_HSV_RS.contains("fn unit_from_step("));
    assert!(COLOR_EDIT_MODEL_HSV_RS.contains("fn sv_picker_a11y_text("));
    assert!(COLOR_EDIT_MODEL_HSV_RS.contains("fn hue_percent_text("));
    assert!(!COLOR_EDIT_MODEL_RS.contains("fn rgb_to_hsv("));
    assert!(!COLOR_EDIT_MODEL_RS.contains("fn hsv_to_rgb("));
    assert!(COLOR_EDIT_MODEL_NUMERIC_RS.contains("mod mode;"));
    assert!(COLOR_EDIT_MODEL_NUMERIC_RS.contains("mod parse;"));
    assert!(COLOR_EDIT_MODEL_NUMERIC_RS.contains("mod text;"));
    assert!(COLOR_EDIT_MODEL_NUMERIC_RS.contains("pub(in crate::controls::color_edit) use"));
    assert!(!COLOR_EDIT_MODEL_NUMERIC_RS.contains("fn rgb_numeric_text("));
    assert!(!COLOR_EDIT_MODEL_NUMERIC_RS.contains("fn parse_color_numeric_input("));
    assert!(COLOR_EDIT_MODEL_NUMERIC_MODE_RS.contains("enum ColorNumericInputMode"));
    assert!(COLOR_EDIT_MODEL_NUMERIC_MODE_RS.contains("fn color_numeric_input_modes("));
    assert!(COLOR_EDIT_MODEL_NUMERIC_MODE_RS.contains("ColorEditPopupNumericInputs::RgbAndHsv"));
    assert!(COLOR_EDIT_MODEL_NUMERIC_TEXT_RS.contains("fn rgb_numeric_text("));
    assert!(COLOR_EDIT_MODEL_NUMERIC_TEXT_RS.contains("fn hsv_numeric_text("));
    assert!(COLOR_EDIT_MODEL_NUMERIC_TEXT_RS.contains("fn color_numeric_text("));
    assert!(COLOR_EDIT_MODEL_NUMERIC_PARSE_RS.contains("fn parse_color_numeric_input("));
    assert!(COLOR_EDIT_MODEL_NUMERIC_PARSE_RS.contains("fn parse_rgb_numeric_input("));
    assert!(COLOR_EDIT_MODEL_NUMERIC_PARSE_RS.contains("fn parse_hsv_numeric_input("));
    assert!(COLOR_EDIT_MODEL_HSV_RS.contains("fn rgb_to_hsv("));
    assert!(COLOR_EDIT_MODEL_HSV_RS.contains("fn hsv_to_rgb("));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS.contains("fn alpha_bar<"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_SURFACE_RS.contains("alpha_bar_preview_stack"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_GRADIENT_RS.contains("fn alpha_gradient_overlay<")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_RS.contains("fn alpha_from_local_x("));
    assert!(COLOR_EDIT_OPTIONS_POPUP_RS.contains("pub struct ColorEditPopupOptions"));
    assert!(COLOR_EDIT_RECORDS_RS.contains("pub struct ColorEditPaletteEntry"));
    assert!(COLOR_EDIT_RECORDS_RS.contains("pub struct ColorEditPaletteSlotDrop"));
    assert!(COLOR_EDIT_RECORDS_RS.contains("pub type OnColorEditPaletteSlotDrop"));
    assert!(COLOR_EDIT_RECORDS_RS.contains("pub fn default_color_edit_palette()"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub history: Arc<[ColorEditPaletteEntry]>"));
    assert!(!COLOR_EDIT_OPTIONS_RS.contains("pub enum ColorEditAlphaPreview"));
    assert!(!COLOR_EDIT_OPTIONS_RS.contains("pub struct ColorEditDragDropOptions"));
    assert!(COLOR_EDIT_OPTIONS_POLICIES_RS.contains("pub enum ColorEditAlphaPreview"));
    assert!(COLOR_EDIT_OPTIONS_POLICIES_RS.contains("impl Default for ColorEditAlphaPreview"));
    assert!(COLOR_EDIT_OPTIONS_POLICIES_RS.contains("pub struct ColorEditDragDropOptions"));
    assert!(COLOR_EDIT_OPTIONS_POLICIES_RS.contains("impl Default for ColorEditDragDropOptions"));
    assert!(COLOR_EDIT_RECORDS_RS.contains("pub enum ColorEditDragDropComponents"));
    assert!(COLOR_EDIT_RECORDS_RS.contains("pub struct ColorEditDragDropPayload"));
    assert!(COLOR_EDIT_OPTIONS_POPUP_RS.contains("pub enum ColorEditPopupPicker"));
    assert!(COLOR_EDIT_OPTIONS_POPUP_RS.contains("ColorEditPopupPicker::HsvHueWheel"));
    assert!(COLOR_EDIT_OPTIONS_POPUP_RS.contains("struct ColorEditPopupRuntimeOptions"));
    assert!(COLOR_EDIT_STATE_RS.contains("fn popup_open_model<"));
    assert!(COLOR_EDIT_STATE_RS.contains("fn tooltip_open_model<"));
    assert!(COLOR_EDIT_STATE_RS.contains("fn copy_menu_open_model<"));
    assert!(COLOR_EDIT_STATE_RS.contains("fn reference_model<"));
    assert!(COLOR_EDIT_STATE_RS.contains("fn draft_model<"));
    assert!(COLOR_EDIT_STATE_RS.contains("fn error_model<"));
    assert!(COLOR_EDIT_STATE_RS.contains("fn popup_runtime_options_model<"));
    assert!(COLOR_EDIT_STATE_RS.contains("fn sync_popup_runtime_options<"));
    assert!(COLOR_EDIT_OPTIONS_POPUP_RS.contains("pub enum ColorEditPopupNumericInputs"));
    assert!(COLOR_EDIT_OPTIONS_POPUP_RS.contains("pub enum ColorEditPopupSidePreview"));
    assert!(!COLOR_EDIT_OPTIONS_RS.contains("pub struct ColorEditTooltipOptions"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub tooltip: ColorEditTooltipOptions"));
    assert!(COLOR_EDIT_OPTIONS_POLICIES_RS.contains("pub struct ColorEditTooltipOptions"));
    assert!(COLOR_EDIT_OPTIONS_POLICIES_RS.contains("impl Default for ColorEditTooltipOptions"));
    assert!(!COLOR_EDIT_OPTIONS_RS.contains("pub struct ColorEditCopyOptions"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub copy: ColorEditCopyOptions"));
    assert!(COLOR_EDIT_OPTIONS_POLICIES_RS.contains("pub struct ColorEditCopyOptions"));
    assert!(COLOR_EDIT_OPTIONS_POLICIES_RS.contains("impl Default for ColorEditCopyOptions"));
    assert!(COLOR_EDIT_RECORDS_RS.contains("pub struct ColorEditEyedropperRequest"));
    assert!(COLOR_EDIT_RECORDS_RS.contains("pub type OnColorEditEyedropper"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub on_eyedropper: Option<OnColorEditEyedropper>"));
    assert!(COLOR_EDIT_POPUP_COPY_RS.contains("fn request_color_copy_menu_overlay<"));
    assert!(COLOR_EDIT_POPUP_COPY_RS.contains("mod entries;"));
    assert!(COLOR_EDIT_POPUP_COPY_RS.contains("mod panel;"));
    assert!(COLOR_EDIT_POPUP_COPY_RS.contains("mod row;"));
    assert!(COLOR_EDIT_POPUP_COPY_ENTRIES_RS.contains("fn color_copy_entries("));
    assert!(COLOR_EDIT_POPUP_COPY_PANEL_RS.contains("fn color_copy_menu_panel<"));
    assert!(COLOR_EDIT_POPUP_COPY_ROW_RS.contains("Effect::ClipboardWriteText"));
    assert!(!COLOR_EDIT_POPUP_COPY_RS.contains("Effect::ClipboardWriteText"));
    assert!(COLOR_EDIT_POPUP_RS.contains("mod body;"));
    assert!(COLOR_EDIT_POPUP_RS.contains("pub(super) use self::request::request_popup_overlay;"));
    assert!(COLOR_EDIT_POPUP_RS.contains("mod request;"));
    assert!(!COLOR_EDIT_POPUP_RS.contains("OverlayRequest::dismissible_menu"));
    assert!(!COLOR_EDIT_POPUP_RS.contains("on_close_auto_focus"));
    assert!(!COLOR_EDIT_POPUP_RS.contains("color_popup_body("));
    assert!(!COLOR_EDIT_POPUP_RS.contains("color_eyedropper_action("));
    assert!(!COLOR_EDIT_POPUP_RS.contains("picker_side_preview_row("));
    assert!(
        COLOR_EDIT_POPUP_REQUEST_RS
            .contains("use super::body::{ColorPopupBodyArgs, color_popup_body};")
    );
    assert!(COLOR_EDIT_POPUP_REQUEST_RS.contains("request_popup_overlay<"));
    assert!(COLOR_EDIT_POPUP_REQUEST_RS.contains("OverlayRequest::dismissible_menu"));
    assert!(COLOR_EDIT_POPUP_REQUEST_RS.contains("on_close_auto_focus"));
    assert!(COLOR_EDIT_POPUP_REQUEST_RS.contains("color_popup_body("));
    assert!(!COLOR_EDIT_POPUP_REQUEST_RS.contains("color_eyedropper_action("));
    assert!(!COLOR_EDIT_POPUP_REQUEST_RS.contains("picker_side_preview_row("));
    assert!(COLOR_EDIT_POPUP_EYEDROPPER_RS.contains("fn color_eyedropper_action<"));
    assert!(COLOR_EDIT_POPUP_EYEDROPPER_RS.contains("ColorEditEyedropperRequest::new("));
    assert!(!COLOR_EDIT_POPUP_EYEDROPPER_RS.contains("Effect::"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("mod delivery;"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("mod source;"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("mod store;"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("pub(super) use delivery::apply_color_drop_payload;"));
    assert!(
        COLOR_EDIT_DRAG_DROP_RS.contains(
            "pub(in crate::controls::color_edit) use delivery::take_delivered_color_drop;"
        )
    );
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("pub(super) use delivery::{"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains(
        "ColorEditDeliveredDropArgs, apply_delivered_color_drop, palette_slot_drop_from_payload,"
    ));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("use source::install_color_drag_source;"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("use source::resolve_color_drag_threshold;"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("pub(in crate::controls::color_edit) use store::{"));
    assert!(
        COLOR_EDIT_DRAG_DROP_RS
            .contains("ActiveColorDrag, ColorDragDropStore, DeliveredColorDrop,")
    );
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains(
        "pub(super) use store::{color_drag_drop_store_for, prune_color_drag_drop_store};"
    ));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("fn update_color_drop_target<"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("pressable_add_on_pointer_move"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("PressablePointerDownResult"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("COMPONENT_IMUI_DRAG_THRESHOLD_PX"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("std::collections::HashMap"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("struct ColorDragDropStoreGlobal"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("struct ActiveColorDrag"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("struct DeliveredColorDrop"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("fn color_drag_drop_store_for<"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("fn prune_color_drag_drop_store<"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("fn take_delivered_color_drop<"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("struct ColorEditDeliveredDropArgs"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("fn apply_delivered_color_drop<"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("take_delivered_color_drop(cx, &args.store"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("format_hex(next, args.show_alpha)"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("fn apply_color_drop_payload("));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("fn palette_slot_drop_from_payload("));
    assert!(COLOR_EDIT_DRAG_DROP_DELIVERY_RS.contains("fn take_delivered_color_drop<"));
    assert!(COLOR_EDIT_DRAG_DROP_DELIVERY_RS.contains("st.delivered.remove(&target_id)?"));
    assert!(
        COLOR_EDIT_DRAG_DROP_DELIVERY_RS
            .contains("current_tick.0 > delivered.tick_id.0.saturating_add(1)")
    );
    assert!(COLOR_EDIT_DRAG_DROP_DELIVERY_RS.contains("struct ColorEditDeliveredDropArgs"));
    assert!(COLOR_EDIT_DRAG_DROP_DELIVERY_RS.contains("fn apply_delivered_color_drop<"));
    assert!(COLOR_EDIT_DRAG_DROP_DELIVERY_RS.contains("take_delivered_color_drop(cx, &args.store"));
    assert!(COLOR_EDIT_DRAG_DROP_DELIVERY_RS.contains("format_hex(next, args.show_alpha)"));
    assert!(COLOR_EDIT_DRAG_DROP_DELIVERY_RS.contains("fn apply_color_drop_payload("));
    assert!(
        COLOR_EDIT_DRAG_DROP_DELIVERY_RS
            .contains("ColorEditDragDropComponents::Rgb || !target_show_alpha")
    );
    assert!(COLOR_EDIT_DRAG_DROP_DELIVERY_RS.contains("fn palette_slot_drop_from_payload("));
    assert!(
        COLOR_EDIT_DRAG_DROP_DELIVERY_RS
            .contains("fret_ui_kit::colors::hex_rgb_from_linear(payload.color())")
    );
    assert!(!COLOR_EDIT_DRAG_DROP_DELIVERY_RS.contains("update_color_drop_target"));
    assert!(!COLOR_EDIT_DRAG_DROP_DELIVERY_RS.contains("install_color_drag_source"));
    assert!(!COLOR_EDIT_DRAG_DROP_DELIVERY_RS.contains("pressable_add_on_pointer_move"));
    assert!(!COLOR_EDIT_DRAG_DROP_DELIVERY_RS.contains("std::collections::HashMap"));
    assert!(COLOR_EDIT_DRAG_DROP_STORE_RS.contains("std::collections::HashMap"));
    assert!(
        COLOR_EDIT_DRAG_DROP_STORE_RS
            .contains("pub(in crate::controls::color_edit) struct ColorDragDropStore")
    );
    assert!(COLOR_EDIT_DRAG_DROP_STORE_RS.contains("pub(super) active: HashMap"));
    assert!(COLOR_EDIT_DRAG_DROP_STORE_RS.contains("pub(super) delivered: HashMap"));
    assert!(
        COLOR_EDIT_DRAG_DROP_STORE_RS
            .contains("pub(in crate::controls::color_edit) struct ActiveColorDrag")
    );
    assert!(
        COLOR_EDIT_DRAG_DROP_STORE_RS
            .contains("pub(in crate::controls::color_edit) struct DeliveredColorDrop")
    );
    assert!(COLOR_EDIT_DRAG_DROP_STORE_RS.contains("fn color_drag_drop_store_for<H: UiHost>"));
    assert!(COLOR_EDIT_DRAG_DROP_STORE_RS.contains("with_global_mut_untracked"));
    assert!(COLOR_EDIT_DRAG_DROP_STORE_RS.contains("fn prune_color_drag_drop_store<H: UiHost>"));
    assert!(COLOR_EDIT_DRAG_DROP_STORE_RS.contains("st.active.remove(session_id)"));
    assert!(
        COLOR_EDIT_DRAG_DROP_STORE_RS
            .contains("current_tick.0 <= drop.tick_id.0.saturating_add(1)")
    );
    assert!(!COLOR_EDIT_DRAG_DROP_STORE_RS.contains("apply_delivered_color_drop"));
    assert!(!COLOR_EDIT_DRAG_DROP_STORE_RS.contains("apply_color_drop_payload"));
    assert!(!COLOR_EDIT_DRAG_DROP_STORE_RS.contains("update_color_drop_target"));
    assert!(!COLOR_EDIT_DRAG_DROP_STORE_RS.contains("install_color_drag_source"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("mod handlers;"));
    assert!(
        COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains(
            "pub(in crate::controls::color_edit) use handlers::install_color_drag_source;"
        )
    );
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("fn resolve_color_drag_threshold<"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("COMPONENT_IMUI_DRAG_THRESHOLD_PX"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("pressable_add_on_pointer_down"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("pressable_add_on_pointer_move"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("pressable_add_on_pointer_up"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("begin_cross_window_drag_with_kind"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("DragPhase::Dragging"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("fn color_drag_threshold_exceeded("));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("fn install_color_drag_source<"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("mod down;"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("mod move_phase;"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("mod up;"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("install_color_drag_pointer_down"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("install_color_drag_pointer_move"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("install_color_drag_pointer_up"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("fn color_drag_threshold_exceeded("));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("COLOR_DRAG_KIND_MASK"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("fn color_drag_kind_for_element"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("MouseButton::Left"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("pressable_add_on_pointer_down"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("pressable_add_on_pointer_move"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("pressable_add_on_pointer_up"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("begin_cross_window_drag_with_kind"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("DragPhase::Dragging"));
    assert!(
        !COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("PressablePointerUpResult::SkipActivate")
    );
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_RS.contains("DeliveredColorDrop"));
    assert!(
        COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_DOWN_RS
            .contains("fn install_color_drag_pointer_down<")
    );
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_DOWN_RS.contains("pressable_add_on_pointer_down"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_DOWN_RS.contains("MouseButton::Left"));
    assert!(
        COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_DOWN_RS.contains("begin_cross_window_drag_with_kind")
    );
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_DOWN_RS.contains("begin_drag_with_kind"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_DOWN_RS.contains("DragPhase::Dragging"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_DOWN_RS.contains("DeliveredColorDrop"));
    assert!(
        COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_MOVE_RS
            .contains("fn install_color_drag_pointer_move<")
    );
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_MOVE_RS.contains("pressable_add_on_pointer_move"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_MOVE_RS.contains("DragPhase::Dragging"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_MOVE_RS.contains("ActiveColorDrag"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_MOVE_RS.contains("color_drag_threshold_exceeded"));
    assert!(
        !COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_MOVE_RS.contains("begin_cross_window_drag_with_kind")
    );
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_MOVE_RS.contains("DeliveredColorDrop"));
    assert!(
        COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_UP_RS.contains("fn install_color_drag_pointer_up<")
    );
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_UP_RS.contains("pressable_add_on_pointer_up"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_UP_RS.contains("DeliveredColorDrop"));
    assert!(
        COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_UP_RS
            .contains("PressablePointerUpResult::SkipActivate")
    );
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_UP_RS.contains("MouseButton::Left"));
    assert!(!COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_UP_RS.contains("DragPhase::Dragging"));
    assert!(
        !COLOR_EDIT_DRAG_DROP_SOURCE_HANDLERS_UP_RS.contains("begin_cross_window_drag_with_kind")
    );
    assert!(COLOR_EDIT_POPUP_PREVIEW_FILL_RS.contains("ColorEditAlphaPreview::Half"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("fn color_side_preview<"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_CELL_RS.contains("SIDE_PREVIEW_SWATCH_WIDTH"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_CELL_RS.contains("SIDE_PREVIEW_SWATCH_HEIGHT"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_ORIGINAL_RS.contains("fn restore_reference_color("));
    assert!(COLOR_EDIT_POPUP_PREVIEW_FILL_RS.contains("fn preview_color_for_alpha_visibility("));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("struct ColorPopupBodyArgs"));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("fn color_popup_body<"));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("mod layout;"));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("mod sections;"));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("ColorPopupContentArgs"));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("color_popup_content("));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("color_popup_width("));
    assert!(!COLOR_EDIT_POPUP_BODY_RS.contains("fn picker_side_preview_row<"));
    assert!(!COLOR_EDIT_POPUP_BODY_RS.contains("COLOR_POPUP_WITH_SIDE_PREVIEW_WIDTH"));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("mod assembly;"));
    assert!(
        COLOR_EDIT_POPUP_BODY_SECTIONS_RS
            .contains("pub(super) use assembly::color_popup_body_sections;")
    );
    assert!(COLOR_EDIT_POPUP_BODY_LAYOUT_RS.contains("fn picker_side_preview_row<"));
    assert!(COLOR_EDIT_POPUP_BODY_LAYOUT_RS.contains("COLOR_POPUP_WITH_SIDE_PREVIEW_WIDTH"));
    assert!(COLOR_EDIT_POPUP_BODY_LAYOUT_RS.contains("ColorEditPopupPicker::Hidden"));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("mod actions;"));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("mod picker;"));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("mod preview;"));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("mod swatches;"));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("color_popup_picker_options_section("));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("color_popup_side_preview_section("));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("color_popup_eyedropper_section("));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("color_popup_numeric_section("));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("color_popup_history_swatches_section("));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("color_popup_preset_swatches_section("));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("color_popup_picker_section("));
    assert!(
        !COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("color_popup_standalone_alpha_bar_section(")
    );
    assert!(
        COLOR_EDIT_POPUP_BODY_SECTIONS_ASSEMBLY_RS.contains("color_popup_picker_options_section(")
    );
    assert!(
        COLOR_EDIT_POPUP_BODY_SECTIONS_ASSEMBLY_RS.contains("color_popup_side_preview_section(")
    );
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_ASSEMBLY_RS.contains("color_popup_eyedropper_section("));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_ASSEMBLY_RS.contains("color_popup_numeric_section("));
    assert!(
        COLOR_EDIT_POPUP_BODY_SECTIONS_ASSEMBLY_RS
            .contains("color_popup_history_swatches_section(")
    );
    assert!(
        COLOR_EDIT_POPUP_BODY_SECTIONS_ASSEMBLY_RS.contains("color_popup_preset_swatches_section(")
    );
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_ASSEMBLY_RS.contains("color_popup_picker_section("));
    assert!(
        COLOR_EDIT_POPUP_BODY_SECTIONS_ASSEMBLY_RS
            .contains("color_popup_standalone_alpha_bar_section(")
    );
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_ASSEMBLY_RS.contains("ColorPopupContentArgs {"));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_ASSEMBLY_RS.contains("has_side_preview"));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("ColorEditPopupPicker::HsvHueBar"));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("ColorEditPopupPicker::HsvHueWheel"));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("color_picker_options("));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("color_side_preview("));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("color_eyedropper_action("));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("color_numeric_inputs("));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("history_swatches("));
    assert!(!COLOR_EDIT_POPUP_BODY_SECTIONS_RS.contains("preset_swatches("));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_PICKER_RS.contains("ColorEditPopupPicker::HsvHueBar"));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_PICKER_RS.contains("ColorEditPopupPicker::HsvHueWheel"));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_PICKER_RS.contains("alpha_bar("));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_ACTIONS_RS.contains("color_picker_options("));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_ACTIONS_RS.contains("color_eyedropper_action("));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_ACTIONS_RS.contains("color_numeric_inputs("));
    assert!(
        COLOR_EDIT_POPUP_BODY_SECTIONS_ACTIONS_RS.contains("ColorEditPopupNumericInputs::Hidden")
    );
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_PREVIEW_RS.contains("color_side_preview("));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_SWATCHES_RS.contains("history_swatches("));
    assert!(COLOR_EDIT_POPUP_BODY_SECTIONS_SWATCHES_RS.contains("preset_swatches("));
    assert!(!COLOR_EDIT_POPUP_BODY_RS.contains("color_picker_options("));
    assert!(!COLOR_EDIT_POPUP_BODY_RS.contains("color_eyedropper_action("));
    assert!(COLOR_EDIT_POPUP_SWATCHES_RS.contains("ColorEditPaletteEntry"));
    assert!(COLOR_EDIT_POPUP_SWATCHES_RS.contains("fn history_swatches<"));
    assert!(COLOR_EDIT_POPUP_SWATCHES_SLOT_RS.contains("install_color_drag_source("));
    assert!(COLOR_EDIT_POPUP_SWATCHES_SLOT_RS.contains("take_delivered_color_drop("));
    assert!(COLOR_EDIT_POPUP_SWATCHES_SLOT_RS.contains("ColorEditPaletteSlotDrop::new("));
    assert!(COLOR_EDIT_POPUP_SWATCHES_SLOT_RS.contains("color_preview_stack("));
    assert!(COLOR_EDIT_POPUP_TOOLTIP_RS.contains("mod panel;"));
    assert!(COLOR_EDIT_POPUP_TOOLTIP_RS.contains("fn request_color_tooltip_overlay<"));
    assert!(COLOR_EDIT_POPUP_TOOLTIP_RS.contains("fn color_tooltip_lines("));
    assert!(COLOR_EDIT_POPUP_TOOLTIP_RS.contains("radix_tooltip::tooltip_request("));
    assert!(COLOR_EDIT_POPUP_TOOLTIP_PANEL_RS.contains("fn color_tooltip_panel<"));
    assert!(COLOR_EDIT_POPUP_TOOLTIP_PANEL_RS.contains("editor_tooltip_readout_text_props("));
    assert!(COLOR_EDIT_POPUP_TOOLTIP_PANEL_RS.contains("SemanticsRole::Tooltip"));
    assert!(!COLOR_EDIT_POPUP_TOOLTIP_RS.contains("editor_tooltip_readout_text_props("));
    assert!(COLOR_EDIT_POPUP_PICKER_LAYOUT_RS.contains("fn hsv_hue_wheel_picker<"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_RS.contains("mod pointer;"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_RS.contains("fn hue_wheel_picker<"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_RS.contains("install_hue_wheel_pointer_handlers(")
    );
    assert!(!COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_RS.contains("fn apply_hue_wheel_position("));
    assert!(
        !COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_RS
            .contains("hue_wheel_target_from_local_position")
    );
    assert!(!COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_RS.contains("MouseButton::Left"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_POINTER_RS
            .contains("fn apply_hue_wheel_position(")
    );
    assert!(
        COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_POINTER_RS
            .contains("hue_wheel_target_from_local_position")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_POINTER_RS.contains("MouseButton::Left"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_POINTER_RS.contains("host.capture_pointer()"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_POINTER_RS
            .contains("host.release_pointer_capture()")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_RS.contains("fn hue_wheel_canvas<"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_RS.contains("fn paint_hue_wheel_canvas("));
    assert!(COLOR_EDIT_POPUP_OPTIONS_THUMBNAIL_RS.contains("fn picker_option_thumbnail<"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_THUMBNAIL_RS.contains("fn hue_bar_picker_thumbnail<"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_THUMBNAIL_RS.contains("fn hue_wheel_picker_thumbnail<"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_THUMBNAIL_RS.contains("sv_picker_preview_stack("));
    assert!(COLOR_EDIT_POPUP_OPTIONS_THUMBNAIL_RS.contains("hue_wheel_canvas("));
    assert!(
        COLOR_EDIT_POPUP_BODY_SECTIONS_PREVIEW_RS.contains("effective_popup_options.side_preview")
    );
    assert!(COLOR_EDIT_MODEL_NUMERIC_MODE_RS.contains("ColorEditPopupNumericInputs::RgbAndHsv"));
    assert!(COLOR_EDIT_MODEL_NUMERIC_MODE_RS.contains("fn color_numeric_input_modes("));
    assert!(COLOR_EDIT_OPTIONS_POPUP_RS.contains("fn has_visible_content_with_swatches("));
    assert!(COLOR_EDIT_TESTS_PICKER_RS.contains("mod bars;"));
    assert!(COLOR_EDIT_TESTS_PICKER_RS.contains("mod hue_wheel;"));
    assert!(COLOR_EDIT_TESTS_PICKER_RS.contains("mod hue_wheel_triangle;"));
    assert!(COLOR_EDIT_TESTS_PICKER_RS.contains("mod preview_alpha;"));
    assert!(!COLOR_EDIT_TESTS_PICKER_RS.contains("#[test]"));
    assert!(COLOR_EDIT_TESTS_PICKER_BARS_RS.contains("sv_picker_position_preserves_hue"));
    assert!(COLOR_EDIT_TESTS_PICKER_BARS_RS.contains("alpha_bar_position_maps_local_x"));
    assert!(COLOR_EDIT_TESTS_PICKER_HUE_WHEEL_RS.contains("hue_wheel_ring_maps_screen_angle"));
    assert!(COLOR_EDIT_TESTS_PICKER_HUE_WHEEL_RS.contains("hue_wheel_target_rejects_outside"));
    assert!(
        COLOR_EDIT_TESTS_PICKER_HUE_WHEEL_TRIANGLE_RS
            .contains("hue_wheel_triangle_maps_imgui_barycentric_sv")
    );
    assert!(
        COLOR_EDIT_TESTS_PICKER_HUE_WHEEL_TRIANGLE_RS
            .contains("hue_wheel_triangle_rotates_with_hue")
    );
    assert!(COLOR_EDIT_TESTS_PICKER_PREVIEW_ALPHA_RS.contains("hsv_color_edits_preserve"));
    assert!(COLOR_EDIT_TESTS_PICKER_PREVIEW_ALPHA_RS.contains("popup_original_restore_matches"));
    assert!(COLOR_EDIT_TESTS_POPUP_POLICY_RS.contains("mod defaults;"));
    assert!(COLOR_EDIT_TESTS_POPUP_POLICY_RS.contains("mod runtime;"));
    assert!(COLOR_EDIT_TESTS_POPUP_POLICY_RS.contains("mod visibility;"));
    assert!(!COLOR_EDIT_TESTS_POPUP_POLICY_RS.contains("#[test]"));
    assert!(
        COLOR_EDIT_TESTS_POPUP_POLICY_DEFAULTS_RS
            .contains("popup_options_default_to_imgui_like_hue_bar_surface")
    );
    assert!(
        COLOR_EDIT_TESTS_POPUP_POLICY_DEFAULTS_RS
            .contains("copy_options_default_to_imgui_context_copy_enabled")
    );
    assert!(
        COLOR_EDIT_TESTS_POPUP_POLICY_RUNTIME_RS
            .contains("popup_runtime_options_are_local_overrides_until_defaults_change")
    );
    assert!(
        COLOR_EDIT_TESTS_POPUP_POLICY_RUNTIME_RS
            .contains("popup_runtime_options_are_ignored_when_options_surface_is_disabled")
    );
    assert!(
        COLOR_EDIT_TESTS_POPUP_POLICY_VISIBILITY_RS
            .contains("popup_options_can_hide_every_popup_affordance")
    );
    assert!(
        COLOR_EDIT_TESTS_POPUP_POLICY_VISIBILITY_RS
            .contains("non_empty_history_counts_as_visible_popup_content_without_palette")
    );
    assert!(COLOR_EDIT_TESTS_NUMERIC_RS.contains("mod conversion;"));
    assert!(COLOR_EDIT_TESTS_NUMERIC_RS.contains("mod hex;"));
    assert!(COLOR_EDIT_TESTS_NUMERIC_RS.contains("mod input;"));
    assert!(COLOR_EDIT_TESTS_NUMERIC_RS.contains("mod modes;"));
    assert!(!COLOR_EDIT_TESTS_NUMERIC_RS.contains("#[test]"));
    assert!(
        COLOR_EDIT_TESTS_NUMERIC_MODES_RS
            .contains("popup_numeric_input_modes_are_explicit_and_ordered")
    );
    assert!(
        COLOR_EDIT_TESTS_NUMERIC_HEX_RS
            .contains("rgb_hex_parse_preserves_alpha_when_alpha_is_not_explicit")
    );
    assert!(
        COLOR_EDIT_TESTS_NUMERIC_HEX_RS
            .contains("numeric_readout_formats_rgb_hsv_and_optional_alpha")
    );
    assert!(
        COLOR_EDIT_TESTS_NUMERIC_INPUT_RS
            .contains("rgb_numeric_input_parses_channels_and_optional_alpha_percent")
    );
    assert!(
        COLOR_EDIT_TESTS_NUMERIC_INPUT_RS
            .contains("numeric_input_rejects_out_of_range_or_incomplete_values")
    );
    assert!(
        COLOR_EDIT_TESTS_NUMERIC_CONVERSION_RS.contains("hsv_conversion_matches_primary_colors")
    );
    assert!(
        COLOR_EDIT_TESTS_NUMERIC_CONVERSION_RS.contains("hsv_conversion_roundtrips_color_presets")
    );
    assert!(!COLOR_EDIT_RS.contains("Color picker (stub)"));
    assert!(!COLOR_EDIT_RS.contains("picker TBD"));
    assert!(!COLOR_EDIT_POPUP_RS.contains("Color picker (stub)"));
    assert!(!COLOR_EDIT_POPUP_RS.contains("picker TBD"));
}

#[test]
fn imui_module_stays_a_thin_into_element_adapter_layer() {
    let normalized = normalize_ws(IMUI_RS);

    let required_markers = [
        "Optional immediate-mode authoring facade adapters.",
        "This must remain a thin adapter over the declarative, single source-of-truth implementation.",
        "Do not introduce a parallel widget implementation here.",
        "fn add_editor_element<H: UiHost + 'static>(",
        "pub fn text_field<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: TextField) {",
        "pub fn checkbox<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: Checkbox) {",
        "pub fn color_edit<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: ColorEdit) {",
        "pub fn drag_value<H, T>(ui: &mut impl UiWriter<H>, control: DragValue<T>)",
        "pub fn axis_drag_value<H, T>(ui: &mut impl UiWriter<H>, control: AxisDragValue<T>)",
        "pub fn numeric_input<H, T>(ui: &mut impl UiWriter<H>, control: NumericInput<T>)",
        "pub fn slider<H, T>(ui: &mut impl UiWriter<H>, control: Slider<T>)",
        "pub fn enum_select<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: EnumSelect) {",
        "pub fn editor_theme_preset_picker<H: UiHost + 'static>(",
        "pub fn mini_search_box<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: MiniSearchBox) {",
        "pub fn text_assist_field<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: TextAssistField) {",
        "pub fn icon_button<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: IconButton) {",
        "pub fn field_status_badge<H: UiHost + 'static>(",
        "pub fn vec2_edit<H, T>(ui: &mut impl UiWriter<H>, control: Vec2Edit<T>)",
        "pub fn vec3_edit<H, T>(ui: &mut impl UiWriter<H>, control: Vec3Edit<T>)",
        "pub fn vec4_edit<H, T>(ui: &mut impl UiWriter<H>, control: Vec4Edit<T>)",
        "pub fn transform_edit<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: TransformEdit) {",
        "pub fn property_group<H: UiHost + 'static>(",
        "pub fn property_grid<H: UiHost + 'static>(",
        "pub fn gradient_editor<H: UiHost + 'static>(",
        "pub fn property_grid_virtualized<H: UiHost + 'static>(",
        "pub fn inspector_panel<H: UiHost + 'static>(",
    ];
    let forbidden_markers = [
        "pub struct ",
        "pub enum ",
        "Model<",
        "LocalState",
        "ActionCx",
        "OnActivate",
        "fret_ui_kit",
        "fret_ui_shadcn",
        "selector_model",
        "watch(",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "imui.rs should keep the promoted editor adapter surface explicit and auditable"
        );
    }

    for marker in forbidden_markers {
        assert!(
            !IMUI_RS.contains(marker),
            "imui.rs should stay free of declarative control internals and adapter-local state/policy"
        );
    }

    assert_eq!(
        count_occurrences(
            IMUI_RS,
            "add_editor_element(ui, move |cx| control.into_element(cx));",
        ),
        17,
        "imui.rs should keep each promoted control adapter as a one-hop `into_element` forwarder",
    );

    assert_eq!(
        count_occurrences(
            &normalized,
            &normalize_ws(
                "add_editor_element(ui, move |cx| {
                    composite.into_element(cx, header_actions, contents)
                });",
            ),
        ),
        1,
        "property_group should stay a one-hop `into_element` forwarder",
    );

    assert_eq!(
        count_occurrences(
            &normalized,
            &normalize_ws("add_editor_element(ui, move |cx| composite.into_element(cx, rows));"),
        ),
        1,
        "property_grid should stay a one-hop `into_element` forwarder",
    );

    assert_eq!(
        count_occurrences(
            &normalized,
            &normalize_ws("add_editor_element(ui, move |cx| composite.into_element(cx));"),
        ),
        1,
        "gradient_editor should stay a one-hop `into_element` forwarder",
    );

    assert_eq!(
        count_occurrences(
            &normalized,
            &normalize_ws(
                "add_editor_element(ui, move |cx| {
                    composite.into_element(cx, len, key_at, row_at)
                });",
            ),
        ),
        1,
        "property_grid_virtualized should stay a one-hop `into_element` forwarder",
    );

    assert_eq!(
        count_occurrences(
            &normalized,
            &normalize_ws(
                "add_editor_element(ui, move |cx| composite.into_element(cx, toolbar, contents));",
            ),
        ),
        1,
        "inspector_panel should stay a one-hop `into_element` forwarder",
    );
}
