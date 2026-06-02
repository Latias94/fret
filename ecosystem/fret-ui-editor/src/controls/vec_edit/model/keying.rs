//! VecEdit caller-keyed element routing owner.

use std::hash::Hash;
use std::panic::Location;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::drag_value_core::DragValueScalar;

use super::{Vec2Edit, Vec3Edit, Vec4Edit};

type Callsite = (&'static str, u32, u32);

#[track_caller]
pub(super) fn vec2_edit_into_element<H, T>(
    control: Vec2Edit<T>,
    cx: &mut ElementContext<'_, H>,
) -> AnyElement
where
    H: UiHost,
    T: DragValueScalar + Default,
{
    let model_ids = (control.x.id(), control.y.id());
    let id_source = control.options.id_source.clone();
    let callsite = current_callsite();

    keyed_vec_edit(
        cx,
        "fret-ui-editor.vec2_edit",
        id_source.as_deref(),
        callsite,
        model_ids,
        |cx| control.into_element_keyed(cx),
    )
}

#[track_caller]
pub(super) fn vec3_edit_into_element<H, T>(
    control: Vec3Edit<T>,
    cx: &mut ElementContext<'_, H>,
) -> AnyElement
where
    H: UiHost,
    T: DragValueScalar + Default,
{
    let model_ids = (control.x.id(), control.y.id(), control.z.id());
    let id_source = control.options.id_source.clone();
    let callsite = current_callsite();

    keyed_vec_edit(
        cx,
        "fret-ui-editor.vec3_edit",
        id_source.as_deref(),
        callsite,
        model_ids,
        |cx| control.into_element_keyed(cx),
    )
}

#[track_caller]
pub(super) fn vec4_edit_into_element<H, T>(
    control: Vec4Edit<T>,
    cx: &mut ElementContext<'_, H>,
) -> AnyElement
where
    H: UiHost,
    T: DragValueScalar + Default,
{
    let model_ids = (
        control.x.id(),
        control.y.id(),
        control.z.id(),
        control.w.id(),
    );
    let id_source = control.options.id_source.clone();
    let callsite = current_callsite();

    keyed_vec_edit(
        cx,
        "fret-ui-editor.vec4_edit",
        id_source.as_deref(),
        callsite,
        model_ids,
        |cx| control.into_element_keyed(cx),
    )
}

#[track_caller]
fn current_callsite() -> Callsite {
    let loc = Location::caller();
    (loc.file(), loc.line(), loc.column())
}

fn keyed_vec_edit<H, K, F>(
    cx: &mut ElementContext<'_, H>,
    key_namespace: &'static str,
    id_source: Option<&str>,
    callsite: Callsite,
    model_ids: K,
    mount: F,
) -> AnyElement
where
    H: UiHost,
    K: Hash,
    F: FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
{
    if let Some(id_source) = id_source {
        cx.keyed((key_namespace, id_source, model_ids), mount)
    } else {
        cx.keyed((key_namespace, callsite, model_ids), mount)
    }
}
