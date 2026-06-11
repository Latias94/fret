use fret_authoring::UiWriter;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::composites::{
    GradientEditor, InspectorPanel, InspectorPanelCx, PropertyGrid, PropertyGridRowCx,
    PropertyGridVirtualized, PropertyGridVirtualizedRowCx, PropertyGroup, PropertyRow,
};

use super::add_editor_element;

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

/// Adds a `PropertyRow` composite to an immediate-style authoring surface.
#[track_caller]
pub fn property_row<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    composite: PropertyRow,
    label: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    value: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    actions: impl FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>,
) {
    add_editor_element(ui, move |cx| {
        composite.into_element(cx, label, value, actions)
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
