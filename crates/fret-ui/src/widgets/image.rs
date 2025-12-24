use fret_core::{DrawOrder, Event, ImageId, SceneOp, SemanticsRole, Size, UvRect};

use crate::{LayoutCx, PaintCx, UiHost, Widget};

pub struct Image {
    image: ImageId,
    opacity: f32,
    desired_size: Option<Size>,
    uv: Option<UvRect>,
}

impl Image {
    pub fn new(image: ImageId) -> Self {
        Self {
            image,
            opacity: 1.0,
            desired_size: None,
            uv: None,
        }
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
        self.uv = Some(uv);
        self
    }
}

impl<H: UiHost> Widget<H> for Image {
    fn semantics(&mut self, cx: &mut crate::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Generic);
    }

    fn event(&mut self, _cx: &mut crate::EventCx<'_, H>, _event: &Event) {}

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let desired = self.desired_size.unwrap_or(cx.available);
        Size::new(
            fret_core::Px(desired.width.0.min(cx.available.width.0).max(0.0)),
            fret_core::Px(desired.height.0.min(cx.available.height.0).max(0.0)),
        )
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        let opacity = self.opacity.clamp(0.0, 1.0);
        if let Some(uv) = self.uv {
            cx.scene.push(SceneOp::ImageRegion {
                order: DrawOrder(0),
                rect: cx.bounds,
                image: self.image,
                uv,
                opacity,
            });
        } else {
            cx.scene.push(SceneOp::Image {
                order: DrawOrder(0),
                rect: cx.bounds,
                image: self.image,
                opacity,
            });
        }
    }
}
