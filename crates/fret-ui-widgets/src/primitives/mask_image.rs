use fret_core::{Color, DrawOrder, Event, ImageId, SceneOp, SemanticsRole, Size, UvRect};

use fret_ui::widget::SemanticsCx;
use fret_ui::{EventCx, LayoutCx, PaintCx, UiHost, Widget};

pub struct MaskImage {
    image: ImageId,
    tint: Color,
    opacity: f32,
    desired_size: Option<Size>,
    uv: UvRect,
}

impl MaskImage {
    pub fn new(image: ImageId) -> Self {
        Self {
            image,
            tint: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            opacity: 1.0,
            desired_size: None,
            uv: UvRect {
                u0: 0.0,
                v0: 0.0,
                u1: 1.0,
                v1: 1.0,
            },
        }
    }

    pub fn tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_size(mut self, size: Size) -> Self {
        self.desired_size = Some(size);
        self
    }

    pub fn with_uv(mut self, uv: UvRect) -> Self {
        self.uv = uv;
        self
    }
}

impl<H: UiHost> Widget<H> for MaskImage {
    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Generic);
    }

    fn event(&mut self, _cx: &mut EventCx<'_, H>, _event: &Event) {}

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let desired = self.desired_size.unwrap_or(cx.available);
        Size::new(
            fret_core::Px(desired.width.0.min(cx.available.width.0).max(0.0)),
            fret_core::Px(desired.height.0.min(cx.available.height.0).max(0.0)),
        )
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        let opacity = self.opacity.clamp(0.0, 1.0);
        let tint = self.tint;
        if opacity <= 0.0 || tint.a <= 0.0 {
            return;
        }
        cx.scene.push(SceneOp::MaskImage {
            order: DrawOrder(0),
            rect: cx.bounds,
            image: self.image,
            uv: self.uv,
            color: tint,
            opacity,
        });
    }
}
