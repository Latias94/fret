use fret_ui::element::{AnyElement, PointerRegionProps};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::OnFloatingAreaLeftDoubleClick;
use crate::imui::{FloatingAreaContext, ImUiFacade};

mod behavior;
mod content;

pub(in crate::imui) fn floating_area_drag_surface_element<H: UiHost, Setup, Build>(
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
        super::float_layer_bring_to_front_if_activated(cx, area.id);

        content::prepare_drag_surface_region(cx, region_id, &mut setup);
        behavior::install_drag_surface_pointer_behavior(
            cx,
            area,
            on_left_double_click_for_down.clone(),
            enable_drag,
            enable_activation,
        );
        content::mount_drag_surface_content(cx, &mut build)
    })
}
