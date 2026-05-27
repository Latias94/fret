use fret_ui::UiHost;
use fret_ui::element::{AnyElement, ScrollAxis, ScrollProps};
use fret_ui::{ElementContext, GlobalElementId};

mod behavior;

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
        let scroll_layout =
            super::floating_window_content_props::content_scroll_layout(resizable_layout);

        cx.scroll(
            ScrollProps {
                layout: scroll_layout,
                axis: ScrollAxis::Y,
                scroll_handle: Some(handle),
                ..Default::default()
            },
            move |cx| {
                vec![cx.container(
                    super::floating_window_content_props::content_container_props(resizable_layout),
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

    behavior::floating_window_content_surface(
        cx,
        window_id,
        resizable_layout,
        options,
        content_container,
    )
}
