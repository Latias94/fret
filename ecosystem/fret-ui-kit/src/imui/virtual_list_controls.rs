use std::cell::Cell;
use std::rc::Rc;
mod row;
mod runtime;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, SemanticsProps};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::containers::build_imui_children_with_focus;
use super::{ImUiFacade, VirtualListOptions, VirtualListResponse};
use row::{pack_row_children, row_height_for_index, row_test_id, wrap_row};
use runtime::{list_layout, resolved_measure_mode, runtime_options};

pub(super) fn virtual_list_element<H: UiHost, K, R>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    len: usize,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: VirtualListOptions,
    mut key_at: K,
    mut row: R,
) -> (AnyElement, VirtualListResponse)
where
    K: FnMut(usize) -> fret_ui::ItemKey,
    R: for<'cx2, 'a2> FnMut(&mut ImUiFacade<'cx2, 'a2, H>, usize),
{
    cx.keyed(("fret-ui-kit.imui.virtual_list", id), |cx| {
        let handle = options
            .handle
            .clone()
            .unwrap_or_else(|| cx.slot_state(VirtualListScrollHandle::new, |h| h.clone()));
        let root_test_id = options.test_id.clone();
        let row_height_fn = options.known_row_height_at.clone();
        let resolved_measure_mode = resolved_measure_mode(&options);
        let first_rendered = Rc::new(Cell::new(None::<usize>));
        let last_rendered = Rc::new(Cell::new(None::<usize>));
        let first_rendered_out = first_rendered.clone();
        let last_rendered_out = last_rendered.clone();

        let list = cx.virtual_list_keyed_with_layout(
            list_layout(&options),
            len,
            runtime_options(&options, resolved_measure_mode),
            &handle,
            &mut key_at,
            move |cx, index| {
                if first_rendered.get().is_none() {
                    first_rendered.set(Some(index));
                }
                last_rendered.set(Some(index));

                let mut out = Vec::new();
                build_imui_children_with_focus(cx, &mut out, build_focus.clone(), |ui| {
                    row(ui, index);
                });
                let content = pack_row_children(cx, out);
                wrap_row(
                    cx,
                    index,
                    content,
                    row_test_id(root_test_id.as_ref(), index),
                    row_height_for_index(
                        index,
                        resolved_measure_mode,
                        options.estimate_row_height,
                        row_height_fn.as_ref(),
                    ),
                )
            },
        );

        let list = if let Some(test_id) = options.test_id {
            let mut semantics = SemanticsProps::default();
            semantics.role = SemanticsRole::List;
            semantics.test_id = Some(test_id);
            cx.semantics(semantics, move |_cx| vec![list])
        } else {
            list
        };

        (
            list,
            VirtualListResponse {
                handle,
                rendered_range: first_rendered_out.get().zip(last_rendered_out.get()),
            },
        )
    })
}

#[cfg(test)]
mod tests;
