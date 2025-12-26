use crate::inspector_edit::InspectorEditService;
use fret_core::{AppWindowId, Color, DrawOrder, Px, SceneOp, Size};
use fret_ui::{LayoutCx, PaintCx, UiHost, Widget};

#[derive(Debug)]
pub struct InspectorEditLayout {
    pub width: Px,
    pub height: Px,
    pub margin: Px,
    pub gap: Px,
}

impl InspectorEditLayout {
    pub fn new(width: Px, height: Px) -> Self {
        Self {
            width,
            height,
            margin: Px(8.0),
            gap: Px(6.0),
        }
    }
}

impl<H: UiHost> Widget<H> for InspectorEditLayout {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let Some((&backdrop, rest)) = cx.children.split_first() else {
            return cx.available;
        };

        let backdrop_bounds = cx.bounds;
        let _ = cx.layout_in(backdrop, backdrop_bounds);

        let Some(&panel) = rest.first() else {
            return cx.available;
        };

        let Some(window) = cx.window else {
            let _ = cx.layout_in(panel, cx.bounds);
            return cx.available;
        };

        let mut panel_bounds = cx.bounds;

        let w = self
            .width
            .0
            .min(cx.available.width.0 - self.margin.0 * 2.0)
            .max(0.0);
        let h = self
            .height
            .0
            .min(cx.available.height.0 - self.margin.0 * 2.0)
            .max(0.0);

        let req = cx
            .app
            .global::<InspectorEditService>()
            .and_then(|s| s.get(window));

        let (anchor, preferred_w) = match req {
            Some(r) => (r.anchor, r.preferred_width),
            None => (None, None),
        };

        let w = preferred_w
            .map(|px| px.0)
            .unwrap_or(w)
            .min(cx.available.width.0 - self.margin.0 * 2.0)
            .max(0.0_f32);

        let min_x = cx.bounds.origin.x.0 + self.margin.0;
        let max_x = cx.bounds.origin.x.0 + cx.available.width.0 - self.margin.0 - w;

        let min_y = cx.bounds.origin.y.0 + self.margin.0;
        let max_y = cx.bounds.origin.y.0 + cx.available.height.0 - self.margin.0 - h;

        let (x, y) = match anchor {
            Some(a) => {
                let ax = a.origin.x.0;
                let ay = a.origin.y.0;
                let ah = a.size.height.0;

                let x = if max_x >= min_x {
                    ax.clamp(min_x, max_x)
                } else {
                    cx.bounds.origin.x.0
                };

                let below = ay + ah + self.gap.0;
                let above = ay - h - self.gap.0;
                let y = if below <= max_y {
                    below.clamp(min_y, max_y)
                } else {
                    above.clamp(min_y, max_y)
                };

                (x, y)
            }
            None => {
                let cx_x = cx.bounds.origin.x.0 + (cx.available.width.0 - w) * 0.5;
                let cx_y = cx.bounds.origin.y.0 + (cx.available.height.0 - h) * 0.5;
                let x = if max_x >= min_x {
                    cx_x.clamp(min_x, max_x)
                } else {
                    cx.bounds.origin.x.0
                };
                let y = if max_y >= min_y {
                    cx_y.clamp(min_y, max_y)
                } else {
                    cx.bounds.origin.y.0
                };
                (x, y)
            }
        };

        panel_bounds.origin.x = Px(x);
        panel_bounds.origin.y = Px(y);
        panel_bounds.size = Size::new(Px(w), Px(h));

        let _ = cx.layout_in(panel, panel_bounds);
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}

#[derive(Debug)]
pub struct InspectorEditHint {
    pub window: AppWindowId,
}

impl InspectorEditHint {
    pub fn new(window: AppWindowId) -> Self {
        Self { window }
    }
}

impl<H: UiHost> Widget<H> for InspectorEditHint {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let Some(service) = cx.app.global::<InspectorEditService>() else {
            return Size::new(cx.available.width, Px(0.0));
        };
        if service.error(self.window).is_some() {
            Size::new(cx.available.width, Px(16.0))
        } else {
            Size::new(cx.available.width, Px(0.0))
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        let Some(service) = cx.app.global::<InspectorEditService>() else {
            return;
        };
        let Some(msg) = service.error(self.window) else {
            return;
        };

        let style = fret_core::TextStyle {
            font: fret_core::FontId::default(),
            size: Px(12.0),
            ..Default::default()
        };
        let (blob, metrics) = cx.services.text().prepare(
            msg,
            style,
            fret_core::TextConstraints {
                max_width: Some(cx.bounds.size.width),
                wrap: fret_core::TextWrap::None,
                overflow: fret_core::TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            },
        );

        let y = cx.bounds.origin.y.0 + metrics.baseline.0.max(0.0);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(1),
            origin: fret_core::Point::new(cx.bounds.origin.x, Px(y)),
            text: blob,
            color: Color {
                r: 0.98,
                g: 0.36,
                b: 0.38,
                a: 0.95,
            },
        });
        cx.services.text().release(blob);
    }
}
