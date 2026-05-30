use std::sync::Arc;

use fret_ui::{ElementContext, UiHost};

use super::super::{KEY_FLOAT_WINDOW_ACTIVATE, OnFloatingAreaLeftDoubleClick};
use crate::imui::FloatingAreaContext;

pub(super) fn install_drag_surface_pointer_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    area: FloatingAreaContext,
    on_left_double_click: Option<OnFloatingAreaLeftDoubleClick>,
    enable_drag: bool,
    enable_activation: bool,
) {
    let drag_kind = area.drag_kind;
    cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
        if !super::super::super::prepare_pointer_region_drag_on_left_down(
            host,
            acx,
            down,
            enable_drag.then_some(drag_kind),
            None,
        ) {
            return false;
        }
        if down.click_count == 2
            && let Some(on_left_double_click) = on_left_double_click.as_ref()
        {
            on_left_double_click(
                host,
                fret_ui::action::ActionCx {
                    window: acx.window,
                    target: area.id,
                },
            );
        }
        if enable_activation {
            host.record_transient_event(
                fret_ui::action::ActionCx {
                    window: acx.window,
                    target: area.id,
                },
                KEY_FLOAT_WINDOW_ACTIVATE,
            );
        }
        host.notify(acx);
        false
    }));

    let drag_threshold = super::super::super::drag_threshold_for(cx);
    cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
        if !enable_drag {
            return false;
        }
        super::super::super::handle_pointer_region_drag_move_with_threshold(
            host,
            acx,
            mv,
            drag_kind,
            drag_threshold,
        )
    }));

    cx.pointer_region_on_pointer_up(Arc::new(move |host, acx, up| {
        if !enable_drag {
            return false;
        }
        super::super::super::finish_pointer_region_drag(host, acx, up.pointer_id, drag_kind)
    }));
}
