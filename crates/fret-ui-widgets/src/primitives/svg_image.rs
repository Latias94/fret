use fret_core::{DrawOrder, Event, SceneOp, SemanticsRole, Size, SvgFit};

use fret_ui::widget::SemanticsCx;
use fret_ui::{EventCx, LayoutCx, PaintCx, UiHost, Widget};

use super::SvgSource;

pub struct SvgImage {
    svg: SvgSource,
    opacity: f32,
    desired_size: Option<Size>,
    fit: SvgFit,
}

impl SvgImage {
    pub fn new(svg: SvgSource) -> Self {
        Self {
            svg,
            opacity: 1.0,
            desired_size: None,
            fit: SvgFit::Contain,
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

    pub fn fit(mut self, fit: SvgFit) -> Self {
        self.fit = fit;
        self
    }
}

impl<H: UiHost> Widget<H> for SvgImage {
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
        if opacity <= 0.0 {
            return;
        }

        let svg = self.svg.resolve(cx.services);
        cx.scene.push(SceneOp::SvgImage {
            order: DrawOrder(0),
            rect: cx.bounds,
            svg,
            fit: self.fit,
            opacity,
        });
    }
}
