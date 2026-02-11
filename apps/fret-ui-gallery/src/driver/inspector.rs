use fret_app::{App, Model};
use fret_core::Point;
use fret_ui::action::{ActionCx, PointerDownCx, PointerMoveCx, UiPointerActionHost};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation};
use std::sync::Arc;

pub(super) fn wrap_content_if_enabled(
    cx: &mut ElementContext<'_, App>,
    inspector_enabled: &Model<bool>,
    inspector_last_pointer: &Model<Option<Point>>,
    content: Vec<AnyElement>,
) -> Vec<AnyElement> {
    if !cx
        .get_model_copied(inspector_enabled, Invalidation::Layout)
        .unwrap_or(false)
    {
        return content;
    }

    cx.observe_model(inspector_last_pointer, Invalidation::Paint);

    let mut props = fret_ui::element::PointerRegionProps::default();
    props.layout.size.width = fret_ui::element::Length::Fill;
    props.layout.size.height = fret_ui::element::Length::Fill;

    let on_pointer_move = {
        let inspector_last_pointer = inspector_last_pointer.clone();
        Arc::new(
            move |host: &mut dyn UiPointerActionHost, cx: ActionCx, mv: PointerMoveCx| {
                let _ = host.models_mut().update(&inspector_last_pointer, |v| {
                    *v = Some(mv.position);
                });
                host.request_redraw(cx.window);
                false
            },
        )
    };
    let on_pointer_down = {
        let inspector_last_pointer = inspector_last_pointer.clone();
        Arc::new(
            move |host: &mut dyn UiPointerActionHost, cx: ActionCx, down: PointerDownCx| {
                let _ = host.models_mut().update(&inspector_last_pointer, |v| {
                    *v = Some(down.position);
                });
                host.request_redraw(cx.window);
                false
            },
        )
    };

    vec![cx.pointer_region(props, move |cx| {
        cx.pointer_region_on_pointer_move(on_pointer_move);
        cx.pointer_region_on_pointer_down(on_pointer_down);
        content
    })]
}
