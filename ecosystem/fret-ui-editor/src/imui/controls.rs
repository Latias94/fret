use fret_authoring::UiWriter;

use crate::controls::{
    AssetRefField, AxisDragValue, Checkbox, ColorEdit, DragValue, EditorThemePresetPicker,
    EnumSelect, FieldStatusBadge, IconButton, MiniSearchBox, NumericInput, Slider, TextAssistField,
    TextField, TransformEdit, Vec2Edit, Vec3Edit, Vec4Edit,
};
use crate::primitives::DragValueScalar;

use super::add_editor_element;
use fret_ui::UiHost;

/// Adds a `TextField` control to an immediate-style authoring surface.
#[track_caller]
pub fn text_field<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: TextField) {
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds a `Checkbox` control to an immediate-style authoring surface.
#[track_caller]
pub fn checkbox<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: Checkbox) {
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds a `ColorEdit` control to an immediate-style authoring surface.
#[track_caller]
pub fn color_edit<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: ColorEdit) {
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds a `DragValue` control to an immediate-style authoring surface.
#[track_caller]
pub fn drag_value<H, T>(ui: &mut impl UiWriter<H>, control: DragValue<T>)
where
    H: UiHost + 'static,
    T: DragValueScalar + Default,
{
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds an `AxisDragValue` control to an immediate-style authoring surface.
#[track_caller]
pub fn axis_drag_value<H, T>(ui: &mut impl UiWriter<H>, control: AxisDragValue<T>)
where
    H: UiHost + 'static,
    T: DragValueScalar + Default,
{
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds a `NumericInput` control to an immediate-style authoring surface.
#[track_caller]
pub fn numeric_input<H, T>(ui: &mut impl UiWriter<H>, control: NumericInput<T>)
where
    H: UiHost + 'static,
    T: Copy + Default + 'static,
{
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds a `Slider` control to an immediate-style authoring surface.
#[track_caller]
pub fn slider<H, T>(ui: &mut impl UiWriter<H>, control: Slider<T>)
where
    H: UiHost + 'static,
    T: DragValueScalar + Default,
{
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds an `EnumSelect` control to an immediate-style authoring surface.
#[track_caller]
pub fn enum_select<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: EnumSelect) {
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds an `AssetRefField` control to an immediate-style authoring surface.
#[track_caller]
pub fn asset_ref_field<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: AssetRefField) {
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds an `EditorThemePresetPicker` control to an immediate-style authoring surface.
#[track_caller]
pub fn editor_theme_preset_picker<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    control: EditorThemePresetPicker,
) {
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds a `MiniSearchBox` control to an immediate-style authoring surface.
#[track_caller]
pub fn mini_search_box<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: MiniSearchBox) {
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds a `TextAssistField` control to an immediate-style authoring surface.
#[track_caller]
pub fn text_assist_field<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: TextAssistField) {
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds an `IconButton` control to an immediate-style authoring surface.
#[track_caller]
pub fn icon_button<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: IconButton) {
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds a `FieldStatusBadge` control to an immediate-style authoring surface.
#[track_caller]
pub fn field_status_badge<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    control: FieldStatusBadge,
) {
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds a `Vec2Edit` control to an immediate-style authoring surface.
#[track_caller]
pub fn vec2_edit<H, T>(ui: &mut impl UiWriter<H>, control: Vec2Edit<T>)
where
    H: UiHost + 'static,
    T: DragValueScalar + Default,
{
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds a `Vec3Edit` control to an immediate-style authoring surface.
#[track_caller]
pub fn vec3_edit<H, T>(ui: &mut impl UiWriter<H>, control: Vec3Edit<T>)
where
    H: UiHost + 'static,
    T: DragValueScalar + Default,
{
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds a `Vec4Edit` control to an immediate-style authoring surface.
#[track_caller]
pub fn vec4_edit<H, T>(ui: &mut impl UiWriter<H>, control: Vec4Edit<T>)
where
    H: UiHost + 'static,
    T: DragValueScalar + Default,
{
    add_editor_element(ui, move |cx| control.into_element(cx));
}

/// Adds a `TransformEdit` control to an immediate-style authoring surface.
#[track_caller]
pub fn transform_edit<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: TransformEdit) {
    add_editor_element(ui, move |cx| control.into_element(cx));
}
