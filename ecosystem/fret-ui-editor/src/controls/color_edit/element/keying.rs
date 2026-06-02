//! ColorEdit caller-keyed element routing owner.

use std::panic::Location;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::ColorEdit;

type Callsite = (&'static str, u32, u32);

#[track_caller]
pub(super) fn color_edit_into_element<H: UiHost>(
    control: ColorEdit,
    cx: &mut ElementContext<'_, H>,
) -> AnyElement {
    let model_id = control.model.id();
    let id_source = control.options.id_source.clone();
    let callsite = current_callsite();

    if let Some(id_source) = id_source.as_deref() {
        cx.keyed(("fret-ui-editor.color_edit", id_source, model_id), |cx| {
            control.into_element_keyed(cx)
        })
    } else {
        cx.keyed(("fret-ui-editor.color_edit", callsite, model_id), |cx| {
            control.into_element_keyed(cx)
        })
    }
}

#[track_caller]
fn current_callsite() -> Callsite {
    let loc = Location::caller();
    (loc.file(), loc.line(), loc.column())
}
