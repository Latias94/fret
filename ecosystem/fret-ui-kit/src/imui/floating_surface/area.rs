use fret_core::Point;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::super::{FloatingAreaContext, FloatingAreaOptions, FloatingAreaResponse, ImUiFacade};
use super::kinds::float_window_drag_kind_for_element;

mod drag_state;
mod layout;

use drag_state::{final_floating_area_state, prepare_floating_area_state};

pub(in crate::imui) fn floating_area_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    initial_position: Point,
    options: FloatingAreaOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>, FloatingAreaContext),
) -> (AnyElement, FloatingAreaResponse) {
    cx.named(id, |cx| {
        let area_id = cx.root_id();
        super::layer::register_floating_layer_child(cx, area_id);

        let drag_kind = float_window_drag_kind_for_element(area_id);
        let prepared =
            prepare_floating_area_state(cx, area_id, id, initial_position, &options, drag_kind);

        let ctx = FloatingAreaContext {
            id: area_id,
            position: prepared.position,
            drag_kind,
        };

        let mut out: Vec<AnyElement> = Vec::new();
        {
            let mut ui = ImUiFacade {
                cx,
                out: &mut out,
                build_focus: None,
            };
            f(&mut ui, ctx);
        }

        let (final_position, final_test_id) =
            final_floating_area_state(cx, area_id, prepared.position, prepared.test_id.clone());

        let area = layout::floating_area_shell(cx, area_id, final_position, &options, out);
        let area = area.test_id(final_test_id);

        let response = FloatingAreaResponse {
            id: area_id,
            rect: cx.last_bounds_for_element(area_id),
            position: final_position,
            dragging: prepared.dragging,
            drag_kind,
        };

        (area, response)
    })
}
