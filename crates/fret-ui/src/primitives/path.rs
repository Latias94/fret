use fret_core::{
    Color, DrawOrder, PathCommand, PathConstraints, PathId, PathMetrics, PathStyle, Point, SceneOp,
    Size,
};

use crate::{LayoutCx, PaintCx, UiHost, Widget};

#[derive(Debug, Clone)]
pub struct Path {
    commands: Vec<PathCommand>,
    style: PathStyle,
    color: Color,
    prepared: Option<PathId>,
    metrics: Option<PathMetrics>,
    prepared_scale_factor_bits: Option<u32>,
}

impl Path {
    pub fn new(commands: Vec<PathCommand>) -> Self {
        Self {
            commands,
            style: PathStyle::Stroke(fret_core::StrokeStyle::default()),
            color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 1.0,
            },
            prepared: None,
            metrics: None,
            prepared_scale_factor_bits: None,
        }
    }

    pub fn style(mut self, style: PathStyle) -> Self {
        self.style = style;
        self.prepared_scale_factor_bits = None;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl<H: UiHost> Widget<H> for Path {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        if let Some(path) = self.prepared.take() {
            services.path().release(path);
        }
        self.prepared_scale_factor_bits = None;
        self.metrics = None;
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };
        let metrics = cx
            .services
            .path()
            .measure(&self.commands, self.style, constraints);
        self.metrics = Some(metrics);
        metrics.bounds.size
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        if self.color.a <= 0.0 {
            return;
        }
        let constraints = PathConstraints {
            scale_factor: cx.scale_factor,
        };
        let scale_bits = cx.scale_factor.to_bits();
        let needs_prepare =
            self.prepared.is_none() || self.prepared_scale_factor_bits != Some(scale_bits);
        if needs_prepare {
            if let Some(path) = self.prepared.take() {
                cx.services.path().release(path);
            }
            let (path, metrics) =
                cx.services
                    .path()
                    .prepare(&self.commands, self.style, constraints);
            self.prepared = Some(path);
            self.metrics = Some(metrics);
            self.prepared_scale_factor_bits = Some(scale_bits);
        }

        let Some(path) = self.prepared else { return };
        let Some(metrics) = self.metrics else { return };

        let origin = Point::new(
            cx.bounds.origin.x - metrics.bounds.origin.x,
            cx.bounds.origin.y - metrics.bounds.origin.y,
        );
        cx.scene.push(SceneOp::Path {
            order: DrawOrder(0),
            origin,
            path,
            color: self.color,
        });
    }
}
