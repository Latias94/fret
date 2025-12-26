use fret_core::{Color, DrawOrder, Event, SceneOp, SemanticsRole, Size, SvgFit};

use crate::{LayoutCx, PaintCx, UiHost, Widget};

use super::SvgSource;

pub struct SvgIcon {
    svg: SvgSource,
    tint: Color,
    opacity: f32,
    desired_size: Option<Size>,
    fit: SvgFit,
}

impl SvgIcon {
    pub fn new(svg: SvgSource) -> Self {
        Self {
            svg,
            tint: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            opacity: 1.0,
            desired_size: None,
            fit: SvgFit::Contain,
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

    pub fn fit(mut self, fit: SvgFit) -> Self {
        self.fit = fit;
        self
    }
}

impl<H: UiHost> Widget<H> for SvgIcon {
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
        let tint = self.tint;
        if opacity <= 0.0 || tint.a <= 0.0 {
            return;
        }

        let svg = self.svg.resolve(cx.services);
        cx.scene.push(SceneOp::SvgMaskIcon {
            order: DrawOrder(0),
            rect: cx.bounds,
            svg,
            fit: self.fit,
            color: tint,
            opacity,
        });
    }
}
