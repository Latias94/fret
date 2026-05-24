use std::sync::Arc;

use fret_ui::UiHost;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, Overflow, PointerRegionProps, ScrollAxis,
    ScrollProps,
};
use fret_ui::{ElementContext, GlobalElementId};

pub(super) fn floating_window_content_element<H: UiHost, Build>(
    cx: &mut ElementContext<'_, H>,
    window_id: GlobalElementId,
    resizable_layout: bool,
    options: super::FloatingWindowOptions,
    build: Build,
) -> AnyElement
where
    Build: for<'cx2, 'a2> FnOnce(&mut super::ImUiFacade<'cx2, 'a2, H>),
{
    let content_container = move |cx: &mut ElementContext<'_, H>| {
        let handle = cx.slot_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());
        let mut scroll_layout = LayoutStyle::default();
        if resizable_layout {
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;
        } else {
            scroll_layout.size.width = Length::Auto;
            scroll_layout.size.height = Length::Auto;
        }
        scroll_layout.overflow = Overflow::Clip;

        cx.scroll(
            ScrollProps {
                layout: scroll_layout,
                axis: ScrollAxis::Y,
                scroll_handle: Some(handle),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    {
                        let mut props = ContainerProps::default();
                        props.layout.size.width = if resizable_layout {
                            Length::Fill
                        } else {
                            Length::Auto
                        };
                        props.padding = fret_core::Edges::all(fret_core::Px(6.0)).into();
                        props
                    },
                    move |cx| {
                        let mut out = Vec::new();
                        let mut ui = super::ImUiFacade {
                            cx,
                            out: &mut out,
                            build_focus: None,
                        };
                        build(&mut ui);
                        out
                    },
                )]
            },
        )
    };

    if options.inputs_enabled && (options.activate_on_click || options.focus_on_click) {
        let layout = {
            let mut layout = LayoutStyle::default();
            if resizable_layout {
                layout.size.width = Length::Fill;
                layout.size.height = Length::Fill;
            } else {
                layout.size.width = Length::Auto;
                layout.size.height = Length::Auto;
            }
            layout
        };
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
                super::floating_surface::float_layer_bring_to_front_if_activated(cx, window_id);
                // Make the surface focusable so `request_focus(...)` is effective even when the
                // click lands on a non-focusable background area.
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
                            super::floating_surface::KEY_FLOAT_WINDOW_ACTIVATE,
                        );
                    }
                    host.notify(acx);
                    false
                }));

                vec![content_container(cx)]
            },
        )
    } else {
        content_container(cx)
    }
}
