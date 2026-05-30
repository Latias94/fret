use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::ImUiFacade;

pub(super) fn prepare_drag_surface_region<H: UiHost, Setup>(
    cx: &mut ElementContext<'_, H>,
    region_id: GlobalElementId,
    setup: &mut Option<Setup>,
) where
    Setup: FnOnce(&mut ElementContext<'_, H>, GlobalElementId),
{
    cx.key_clear_on_key_down_for(region_id);
    if let Some(setup) = setup.take() {
        setup(cx, region_id);
    }
    cx.key_add_on_key_down_for(region_id, Arc::new(|_host, _acx, _down| false));
}

pub(super) fn mount_drag_surface_content<H: UiHost, Build>(
    cx: &mut ElementContext<'_, H>,
    build: &mut Option<Build>,
) -> Vec<AnyElement>
where
    Build: for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
{
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
}
