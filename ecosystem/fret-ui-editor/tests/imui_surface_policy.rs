#![cfg(feature = "imui")]

const IMUI_RS: &str = include_str!("../src/imui.rs");
const COLOR_EDIT_RS: &str = include_str!("../src/controls/color_edit.rs");
const COLOR_EDIT_ELEMENT_RS: &str = include_str!("../src/controls/color_edit/element.rs");
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
const COLOR_EDIT_DRAG_DROP_SOURCE_RS: &str =
    include_str!("../src/controls/color_edit/drag_drop/source.rs");
const COLOR_EDIT_LAYOUT_RS: &str = include_str!("../src/controls/color_edit/layout.rs");
const COLOR_EDIT_MODEL_RS: &str = include_str!("../src/controls/color_edit/model.rs");
const COLOR_EDIT_MODEL_NUMERIC_RS: &str =
    include_str!("../src/controls/color_edit/model/numeric.rs");
const COLOR_EDIT_OPTIONS_RS: &str = include_str!("../src/controls/color_edit/options.rs");
const COLOR_EDIT_OPTIONS_POPUP_RS: &str =
    include_str!("../src/controls/color_edit/options/popup.rs");
const COLOR_EDIT_RECORDS_RS: &str = include_str!("../src/controls/color_edit/records.rs");
const COLOR_EDIT_STATE_RS: &str = include_str!("../src/controls/color_edit/state.rs");
const COLOR_EDIT_SWATCH_RS: &str = include_str!("../src/controls/color_edit/swatch.rs");
const COLOR_EDIT_SWATCH_CONTEXT_MENU_RS: &str =
    include_str!("../src/controls/color_edit/swatch/context_menu.rs");
const COLOR_EDIT_POPUP_RS: &str = include_str!("../src/controls/color_edit/popup.rs");
const COLOR_EDIT_POPUP_BODY_RS: &str = include_str!("../src/controls/color_edit/popup/body.rs");
const COLOR_EDIT_POPUP_BODY_LAYOUT_RS: &str =
    include_str!("../src/controls/color_edit/popup/body/layout.rs");
const COLOR_EDIT_POPUP_NUMERIC_RS: &str =
    include_str!("../src/controls/color_edit/popup/numeric.rs");
const COLOR_EDIT_POPUP_NUMERIC_FIELD_RS: &str =
    include_str!("../src/controls/color_edit/popup/numeric/field.rs");
const COLOR_EDIT_POPUP_OPTIONS_RS: &str =
    include_str!("../src/controls/color_edit/popup/options.rs");
const COLOR_EDIT_POPUP_OPTIONS_PICKER_RS: &str =
    include_str!("../src/controls/color_edit/popup/options/picker.rs");
const COLOR_EDIT_POPUP_OPTIONS_THUMBNAIL_RS: &str =
    include_str!("../src/controls/color_edit/popup/options/thumbnail.rs");
const COLOR_EDIT_POPUP_PICKER_RS: &str = include_str!("../src/controls/color_edit/popup/picker.rs");
const COLOR_EDIT_POPUP_PICKER_ALPHA_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/alpha.rs");
const COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/alpha/bar.rs");
const COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/alpha/preview.rs");
const COLOR_EDIT_POPUP_PICKER_HUE_BAR_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/hue_bar.rs");
const COLOR_EDIT_POPUP_PICKER_HUE_BAR_BAR_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/hue_bar/bar.rs");
const COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/hue_bar/preview.rs");
const COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/hue_wheel.rs");
const COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/hue_wheel_picker.rs");
const COLOR_EDIT_POPUP_PICKER_SV_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/sv.rs");
const COLOR_EDIT_POPUP_PICKER_SV_BAR_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/sv/bar.rs");
const COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_RS: &str =
    include_str!("../src/controls/color_edit/popup/picker/sv/preview.rs");
const COLOR_EDIT_POPUP_PREVIEW_RS: &str =
    include_str!("../src/controls/color_edit/popup/preview.rs");
const COLOR_EDIT_POPUP_PREVIEW_FILL_RS: &str =
    include_str!("../src/controls/color_edit/popup/preview/fill.rs");
const COLOR_EDIT_POPUP_PREVIEW_SIDE_RS: &str =
    include_str!("../src/controls/color_edit/popup/preview/side.rs");
const COLOR_EDIT_POPUP_SWATCHES_RS: &str =
    include_str!("../src/controls/color_edit/popup/swatches.rs");
const COLOR_EDIT_POPUP_SWATCHES_SLOT_RS: &str =
    include_str!("../src/controls/color_edit/popup/swatches/slot.rs");
const COLOR_EDIT_POPUP_TOOLTIP_RS: &str =
    include_str!("../src/controls/color_edit/popup/tooltip.rs");
const COLOR_EDIT_POPUP_TOOLTIP_PANEL_RS: &str =
    include_str!("../src/controls/color_edit/popup/tooltip/panel.rs");

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
    assert!(COLOR_EDIT_ELEMENT_RS.contains("color_hex_input("));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("color_swatch("));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("request_popup_overlay("));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("color_edit_root_layout("));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub struct ColorEditOptions"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("mod popup;"));
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
    assert!(COLOR_EDIT_SWATCH_RS.contains("pub(super) fn color_swatch<"));
    assert!(COLOR_EDIT_SWATCH_RS.contains("PressableProps"));
    assert!(COLOR_EDIT_SWATCH_RS.contains("pressable_add_on_activate"));
    assert!(COLOR_EDIT_SWATCH_RS.contains("mod context_menu;"));
    assert!(COLOR_EDIT_SWATCH_RS.contains("install_context_menu_pointer_handler"));
    assert!(COLOR_EDIT_SWATCH_RS.contains("install_context_menu_keyboard_handler"));
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
    assert!(COLOR_EDIT_SWATCH_RS.contains("ColorEditDragDropPayload::from_color"));
    assert!(COLOR_EDIT_SWATCH_RS.contains("install_color_drag_source"));
    assert!(COLOR_EDIT_SWATCH_RS.contains("update_color_drop_target"));
    assert!(COLOR_EDIT_SWATCH_RS.contains("EditorWidgetVisuals"));
    assert!(COLOR_EDIT_SWATCH_CONTEXT_MENU_RS.contains("key_on_key_down_for"));
    assert!(COLOR_EDIT_SWATCH_CONTEXT_MENU_RS.contains("KeyCode::ContextMenu"));
    assert!(COLOR_EDIT_SWATCH_RS.contains("color_preview_stack"));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("ColorEditDeliveredDropArgs"));
    assert!(COLOR_EDIT_ELEMENT_RS.contains("apply_delivered_color_drop("));
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
    assert!(COLOR_EDIT_POPUP_PREVIEW_FILL_RS.contains("fn checkerboard_grid<"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("mod hue_wheel;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("pub(in crate::controls::color_edit) mod alpha;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("pub(super) use alpha::alpha_bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("mod hue_bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("use hue_bar::hue_bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("pub(super) use hue_bar::hue_bar_preview_stack;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("mod hue_wheel_picker;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("use hue_wheel_picker::hue_wheel_picker;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("mod sv;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("use sv::sv_picker;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("pub(super) use sv::sv_picker_preview_stack;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("pub(super) use hue_wheel::hue_wheel_canvas;"));
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("fn hsv_picker<"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_RS.contains("mod bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_RS.contains("mod interaction;"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_RS.contains("mod preview;"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_RS.contains("pub(super) use bar::sv_picker;"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_RS.contains(
        "pub(in crate::controls::color_edit::popup) use preview::sv_picker_preview_stack;"
    ));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_BAR_RS.contains("fn sv_picker<"));
    assert!(COLOR_EDIT_POPUP_PICKER_SV_PREVIEW_RS.contains("fn sv_picker_grid<"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_RS.contains("mod bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_RS.contains("mod interaction;"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_RS.contains("mod preview;"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_RS.contains("pub(super) use bar::hue_bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_RS.contains(
        "pub(in crate::controls::color_edit::popup) use preview::hue_bar_preview_stack;"
    ));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_BAR_BAR_RS.contains("fn hue_bar<"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_HUE_BAR_PREVIEW_RS.contains("fn vertical_hue_gradient_overlay<")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_RS.contains("mod bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_RS.contains("mod interaction;"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_RS.contains("mod preview;"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_RS
            .contains("pub(in crate::controls::color_edit::popup) use bar::alpha_bar;")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_RS.contains("pub(super) use bar::vertical_alpha_bar;"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS.contains("fn vertical_alpha_bar<"));
    assert!(
        COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_RS.contains("fn vertical_alpha_gradient_overlay<")
    );
    assert!(COLOR_EDIT_POPUP_NUMERIC_RS.contains("fn color_numeric_inputs<"));
    assert!(COLOR_EDIT_POPUP_NUMERIC_RS.contains("mod field;"));
    assert!(COLOR_EDIT_POPUP_NUMERIC_RS.contains("fn color_numeric_error_line<"));
    assert!(COLOR_EDIT_POPUP_NUMERIC_FIELD_RS.contains("fn color_numeric_input_field<"));
    assert!(COLOR_EDIT_POPUP_NUMERIC_FIELD_RS.contains("parse_color_numeric_input"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_RS.contains("fn color_picker_options<"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_PICKER_RS.contains("fn picker_options_row<"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_RS.contains("fn alpha_bar_option<"));
    assert!(COLOR_EDIT_MODEL_RS.contains("mod numeric;"));
    assert!(COLOR_EDIT_MODEL_RS.contains("pub(super) use numeric::{"));
    assert!(COLOR_EDIT_MODEL_NUMERIC_RS.contains("fn rgb_numeric_text("));
    assert!(COLOR_EDIT_MODEL_NUMERIC_RS.contains("fn hsv_numeric_text("));
    assert!(COLOR_EDIT_MODEL_NUMERIC_RS.contains("fn parse_color_numeric_input("));
    assert!(COLOR_EDIT_MODEL_RS.contains("fn rgb_to_hsv("));
    assert!(COLOR_EDIT_MODEL_RS.contains("fn hsv_to_rgb("));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_BAR_RS.contains("fn alpha_bar<"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_PREVIEW_RS.contains("fn alpha_gradient_overlay<"));
    assert!(COLOR_EDIT_POPUP_PICKER_ALPHA_RS.contains("fn alpha_from_local_x("));
    assert!(COLOR_EDIT_OPTIONS_POPUP_RS.contains("pub struct ColorEditPopupOptions"));
    assert!(COLOR_EDIT_RECORDS_RS.contains("pub struct ColorEditPaletteEntry"));
    assert!(COLOR_EDIT_RECORDS_RS.contains("pub struct ColorEditPaletteSlotDrop"));
    assert!(COLOR_EDIT_RECORDS_RS.contains("pub type OnColorEditPaletteSlotDrop"));
    assert!(COLOR_EDIT_RECORDS_RS.contains("pub fn default_color_edit_palette()"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub history: Arc<[ColorEditPaletteEntry]>"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub enum ColorEditAlphaPreview"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub struct ColorEditDragDropOptions"));
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
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub struct ColorEditTooltipOptions"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub tooltip: ColorEditTooltipOptions"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub struct ColorEditCopyOptions"));
    assert!(COLOR_EDIT_OPTIONS_RS.contains("pub copy: ColorEditCopyOptions"));
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
    assert!(
        COLOR_EDIT_POPUP_RS.contains("use self::body::{ColorPopupBodyArgs, color_popup_body};")
    );
    assert!(COLOR_EDIT_POPUP_RS.contains("request_popup_overlay<"));
    assert!(COLOR_EDIT_POPUP_RS.contains("OverlayRequest::dismissible_menu"));
    assert!(COLOR_EDIT_POPUP_RS.contains("on_close_auto_focus"));
    assert!(COLOR_EDIT_POPUP_RS.contains("color_popup_body("));
    assert!(!COLOR_EDIT_POPUP_RS.contains("color_eyedropper_action("));
    assert!(!COLOR_EDIT_POPUP_RS.contains("picker_side_preview_row("));
    assert!(COLOR_EDIT_POPUP_EYEDROPPER_RS.contains("fn color_eyedropper_action<"));
    assert!(COLOR_EDIT_POPUP_EYEDROPPER_RS.contains("ColorEditEyedropperRequest::new("));
    assert!(!COLOR_EDIT_POPUP_EYEDROPPER_RS.contains("Effect::"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("mod source;"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("use source::install_color_drag_source;"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("use source::resolve_color_drag_threshold;"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("fn update_color_drop_target<"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("pub(super) struct ColorEditDeliveredDropArgs"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("fn apply_delivered_color_drop<"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("take_delivered_color_drop(cx, &args.store"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("format_hex(next, args.show_alpha)"));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("fn apply_color_drop_payload("));
    assert!(COLOR_EDIT_DRAG_DROP_RS.contains("fn palette_slot_drop_from_payload("));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("pressable_add_on_pointer_move"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("PressablePointerDownResult"));
    assert!(!COLOR_EDIT_DRAG_DROP_RS.contains("COMPONENT_IMUI_DRAG_THRESHOLD_PX"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("fn resolve_color_drag_threshold<"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("COMPONENT_IMUI_DRAG_THRESHOLD_PX"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("fn install_color_drag_source<"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("pressable_add_on_pointer_down"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("pressable_add_on_pointer_move"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("pressable_add_on_pointer_up"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("begin_cross_window_drag_with_kind"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("begin_drag_with_kind"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("DragPhase::Dragging"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("PressablePointerUpResult::SkipActivate"));
    assert!(COLOR_EDIT_DRAG_DROP_SOURCE_RS.contains("fn color_drag_threshold_exceeded("));
    assert!(COLOR_EDIT_POPUP_PREVIEW_FILL_RS.contains("ColorEditAlphaPreview::Half"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("fn color_side_preview<"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("SIDE_PREVIEW_SWATCH_WIDTH"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("SIDE_PREVIEW_SWATCH_HEIGHT"));
    assert!(COLOR_EDIT_POPUP_PREVIEW_SIDE_RS.contains("fn restore_reference_color("));
    assert!(COLOR_EDIT_POPUP_PREVIEW_FILL_RS.contains("fn preview_color_for_alpha_visibility("));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("struct ColorPopupBodyArgs"));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("fn color_popup_body<"));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("mod layout;"));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("ColorPopupContentArgs"));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("color_popup_content("));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("color_popup_width("));
    assert!(!COLOR_EDIT_POPUP_BODY_RS.contains("fn picker_side_preview_row<"));
    assert!(!COLOR_EDIT_POPUP_BODY_RS.contains("COLOR_POPUP_WITH_SIDE_PREVIEW_WIDTH"));
    assert!(COLOR_EDIT_POPUP_BODY_LAYOUT_RS.contains("fn picker_side_preview_row<"));
    assert!(COLOR_EDIT_POPUP_BODY_LAYOUT_RS.contains("COLOR_POPUP_WITH_SIDE_PREVIEW_WIDTH"));
    assert!(COLOR_EDIT_POPUP_BODY_LAYOUT_RS.contains("ColorEditPopupPicker::Hidden"));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("ColorEditPopupPicker::HsvHueBar"));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("ColorEditPopupPicker::HsvHueWheel"));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("color_picker_options("));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("color_eyedropper_action("));
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
    assert!(COLOR_EDIT_POPUP_PICKER_RS.contains("fn hsv_hue_wheel_picker<"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_RS.contains("fn hue_wheel_picker<"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_RS.contains("fn apply_hue_wheel_position("));
    assert!(
        COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_PICKER_RS
            .contains("hue_wheel_target_from_local_position")
    );
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_RS.contains("fn hue_wheel_canvas<"));
    assert!(COLOR_EDIT_POPUP_PICKER_HUE_WHEEL_RS.contains("fn paint_hue_wheel_canvas("));
    assert!(COLOR_EDIT_POPUP_OPTIONS_THUMBNAIL_RS.contains("fn picker_option_thumbnail<"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_THUMBNAIL_RS.contains("fn hue_bar_picker_thumbnail<"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_THUMBNAIL_RS.contains("fn hue_wheel_picker_thumbnail<"));
    assert!(COLOR_EDIT_POPUP_OPTIONS_THUMBNAIL_RS.contains("sv_picker_preview_stack("));
    assert!(COLOR_EDIT_POPUP_OPTIONS_THUMBNAIL_RS.contains("hue_wheel_canvas("));
    assert!(COLOR_EDIT_POPUP_BODY_RS.contains("popup_options.side_preview"));
    assert!(COLOR_EDIT_MODEL_NUMERIC_RS.contains("ColorEditPopupNumericInputs::RgbAndHsv"));
    assert!(COLOR_EDIT_MODEL_NUMERIC_RS.contains("fn color_numeric_input_modes("));
    assert!(COLOR_EDIT_OPTIONS_POPUP_RS.contains("fn has_visible_content_with_swatches("));
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
