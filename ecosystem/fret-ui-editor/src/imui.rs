//! Optional immediate-mode authoring facade adapters.
//!
//! Invariants:
//! - This must remain a thin adapter over the declarative, single source-of-truth implementation.
//! - Do not introduce a parallel widget implementation here.

use fret_authoring::UiWriter;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::controls::{Checkbox, DragValue, EnumSelect, Slider, TextField};
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

/// Adds a `DragValue` control to an immediate-style authoring surface.
#[track_caller]
pub fn drag_value<H, T>(ui: &mut impl UiWriter<H>, control: DragValue<T>)
where
    H: UiHost + 'static,
    T: DragValueScalar + Default,
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
