use std::sync::Arc;

use fret_ui::element::{AnyElement, PointerRegionProps};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

pub(super) fn floating_window_content_surface<H: UiHost, BuildContent>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    resizable_layout: bool,
    options: super::super::FloatingWindowOptions,
    content_container: BuildContent,
) -> AnyElement
where
    BuildContent: FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
{
    if !installs_content_surface_behavior(&options) {
        return content_container(cx);
    }

    let layout =
        super::super::floating_window_content_props::content_surface_layout(resizable_layout);
    let focus_on_click = options.focus_on_click;
    let activate_on_click = options.activate_on_click;
    cx.pointer_region(
        PointerRegionProps {
            layout,
            enabled: true,
            ..Default::default()
        },
        move |cx| {
            let region_id = cx.root_id();
            install_content_surface_behavior(
                cx,
                region_id,
                window_id,
                focus_on_click,
                activate_on_click,
            );
            vec![content_container(cx)]
        },
    )
}

fn installs_content_surface_behavior(options: &super::super::FloatingWindowOptions) -> bool {
    options.inputs_enabled && (options.activate_on_click || options.focus_on_click)
}

fn install_content_surface_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    region_id: GlobalElementId,
    window_id: GlobalElementId,
    focus_on_click: bool,
    activate_on_click: bool,
) {
    super::super::floating_surface::float_layer_bring_to_front_if_activated(cx, window_id);
    // Make the surface focusable so `request_focus(...)` is effective even when the click lands on
    // a non-focusable background area.
    cx.key_on_key_down_for(region_id, Arc::new(|_host, _acx, _down| false));

    cx.pointer_region_clear_on_pointer_down();
    cx.pointer_region_on_pointer_down(Arc::new(move |host, acx, _down| {
        if focus_on_click {
            host.request_focus(acx.target);
        }
        if activate_on_click {
            host.record_transient_event(
                fret_ui::action::ActionCx {
                    window: acx.window,
                    target: window_id,
                },
                super::super::floating_surface::KEY_FLOAT_WINDOW_ACTIVATE,
            );
        }
        host.notify(acx);
        false
    }));
}
