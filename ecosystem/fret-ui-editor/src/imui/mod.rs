//! Optional immediate-mode authoring facade adapters.
//!
//! Invariants:
//! - This must remain a thin adapter over the declarative, single source-of-truth implementation.
//! - Do not introduce a parallel widget implementation here.

use fret_authoring::UiWriter;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

mod composites;
mod controls;

pub use composites::*;
pub use controls::*;

pub(super) fn add_editor_element<H: UiHost + 'static>(
    ui: &mut impl UiWriter<H>,
    render: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
) {
    let element = ui.with_cx_mut(render);
    ui.add(element);
}
