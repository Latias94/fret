use std::sync::Arc;

use fret_ui::element::{AnyElement, PointerRegionProps};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{FloatingAreaContext, ImUiFacade};

mod area;
mod kinds;
mod layer;
mod state;

pub(super) use area::floating_area_element;
pub(super) use kinds::{
    FloatWindowResizeHandle, KEY_FLOAT_WINDOW_ACTIVATE, KEY_FLOAT_WINDOW_TOGGLE_COLLAPSED,
    OnFloatingAreaLeftDoubleClick, float_window_drag_kind_for_element,
    float_window_resize_kind_for_element,
};
pub(super) use layer::{float_layer_bring_to_front_if_activated, floating_layer_element};
pub(super) use state::{FloatWindowState, FloatingAreaState, FloatingWindowChromeResponse};

pub(super) fn floating_area_drag_surface_element<H: UiHost, Setup, Build>(
    cx: &mut ElementContext<'_, H>,
    area: FloatingAreaContext,
    props: PointerRegionProps,
    on_left_double_click: Option<OnFloatingAreaLeftDoubleClick>,
    enable_drag: bool,
    enable_activation: bool,
    setup: Setup,
    build: Build,
) -> AnyElement
where
    Setup: FnOnce(&mut ElementContext<'_, H>, GlobalElementId),
    Build: for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
{
    let mut build = Some(build);
    let mut setup = Some(setup);
    let on_left_double_click_for_down = on_left_double_click.clone();
    cx.pointer_region(props, move |cx| {
        let region_id = cx.root_id();
        float_layer_bring_to_front_if_activated(cx, area.id);

        cx.key_clear_on_key_down_for(region_id);
        if let Some(setup) = setup.take() {
            setup(cx, region_id);
        }
        cx.key_add_on_key_down_for(region_id, Arc::new(|_host, _acx, _down| false));

        let drag_kind = area.drag_kind;
        cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, down| {
            if !super::prepare_pointer_region_drag_on_left_down(
                host,
                acx,
                down,
                enable_drag.then_some(drag_kind),
                None,
            ) {
                return false;
            }
            if down.click_count == 2
                && let Some(on_left_double_click) = on_left_double_click_for_down.as_ref()
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

        let drag_threshold = super::drag_threshold_for(cx);
        cx.pointer_region_on_pointer_move(Arc::new(move |host, acx, mv| {
            if !enable_drag {
                return false;
            }
            super::handle_pointer_region_drag_move_with_threshold(
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
            super::finish_pointer_region_drag(host, acx, up.pointer_id, drag_kind)
        }));

        let mut out = Vec::new();
        if let Some(build) = build.take() {
            let mut ui = ImUiFacade {
                cx,
                out: &mut out,
                build_focus: None,
            };
            build(&mut ui);
        }
        out
    })
}
