//! Optional immediate-mode authoring facade adapters.
//!
//! Invariants:
//! - This must remain a thin adapter over the declarative, single source-of-truth implementation.
//! - Do not introduce a parallel widget implementation here.

use fret_authoring::UiWriter;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::composites::{
    GradientEditor, InspectorPanel, InspectorPanelCx, PropertyGrid, PropertyGridRowCx,
    PropertyGridVirtualized, PropertyGridVirtualizedRowCx, PropertyGroup,
};
use crate::controls::{
    AxisDragValue, Checkbox, ColorEdit, DragValue, EnumSelect, FieldStatusBadge, IconButton,
    MiniSearchBox, NumericInput, Slider, TextAssistField, TextField, TransformEdit, Vec2Edit,
    Vec3Edit, Vec4Edit,
};
use crate::primitives::DragValueScalar;

#[track_caller]
fn add_editor_element<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    render: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) {
    let element = ui.with_cx_mut(render);
    ui.add(element);
}

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

/// Adds a `PropertyGroup` composite to an immediate-style authoring surface.
#[track_caller]
pub fn property_group<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    composite: PropertyGroup,
    header_actions: impl FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>,
    contents: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) {
    add_editor_element(ui, move |cx| {
        composite.into_element(cx, header_actions, contents)
    });
}

/// Adds a `PropertyGrid` composite to an immediate-style authoring surface.
#[track_caller]
pub fn property_grid<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    composite: PropertyGrid,
    rows: impl FnOnce(&mut ElementContext<'_, H>, PropertyGridRowCx) -> Vec<AnyElement>,
) {
    add_editor_element(ui, move |cx| composite.into_element(cx, rows));
}

/// Adds a `GradientEditor` composite to an immediate-style authoring surface.
#[track_caller]
pub fn gradient_editor<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, composite: GradientEditor) {
    add_editor_element(ui, move |cx| composite.into_element(cx));
}

/// Adds a `PropertyGridVirtualized` composite to an immediate-style authoring surface.
#[track_caller]
pub fn property_grid_virtualized<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    composite: PropertyGridVirtualized,
    len: usize,
    key_at: impl FnMut(usize) -> fret_ui::ItemKey + 'static,
    row_at: impl FnMut(&mut ElementContext<'_, H>, usize, PropertyGridVirtualizedRowCx) -> AnyElement
    + 'static,
) {
    add_editor_element(ui, move |cx| {
        composite.into_element(cx, len, key_at, row_at)
    });
}

/// Adds an `InspectorPanel` composite to an immediate-style authoring surface.
#[track_caller]
pub fn inspector_panel<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    composite: InspectorPanel,
    toolbar: impl FnOnce(&mut ElementContext<'_, H>, &InspectorPanelCx) -> Vec<AnyElement>,
    contents: impl FnOnce(&mut ElementContext<'_, H>, &InspectorPanelCx) -> Vec<AnyElement>,
) {
    add_editor_element(ui, move |cx| composite.into_element(cx, toolbar, contents));
}
