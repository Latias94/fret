use std::sync::Arc;

use fret_core::{Event, Size};
use fret_ui::{EventCx, LayoutCx, PaintCx, TooltipRequest, TooltipService, UiHost, Widget};

pub struct TooltipArea {
    text: Arc<str>,
}

impl TooltipArea {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }
}

impl<H: UiHost> Widget<H> for TooltipArea {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };

        let Event::Pointer(pe) = event else {
            return;
        };

        match pe {
            fret_core::PointerEvent::Move { position, .. } => {
                if !cx.bounds.contains(*position) {
                    return;
                }

                cx.app
                    .with_global_mut(TooltipService::default, |service, app| {
                        service.set_request(
                            app,
                            window,
                            TooltipRequest {
                                owner: cx.node,
                                anchor: cx.bounds,
                                text: self.text.clone(),
                            },
                        );
                    });
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let Some(&child) = cx.children.first() else {
            return cx.available;
        };
        cx.layout_in(child, cx.bounds)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        let Some(&child) = cx.children.first() else {
            return;
        };
        let bounds = cx.child_bounds(child).unwrap_or(cx.bounds);
        cx.paint(child, bounds);
    }
}
