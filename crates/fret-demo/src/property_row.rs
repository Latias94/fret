use fret_core::{Color, Corners, DrawOrder, Edges, Px, SceneOp, Size, TextConstraints, TextMetrics, TextStyle, TextWrap};
use fret_ui::{LayoutCx, PaintCx, Widget};

#[derive(Debug)]
pub struct PropertyRow {
    pub label: String,
    pub height: Px,
    pub background: Color,
    blob: Option<fret_core::TextBlobId>,
    metrics: Option<TextMetrics>,
}

impl PropertyRow {
    pub fn new(label: impl Into<String>, height: Px, background: Color) -> Self {
        Self {
            label: label.into(),
            height,
            background,
            blob: None,
            metrics: None,
        }
    }
}

impl Widget for PropertyRow {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        let constraints = TextConstraints {
            max_width: Some(cx.available.width - Px(24.0)),
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };
        let style = TextStyle {
            font: fret_core::FontId::default(),
            size: Px(13.0),
        };
        let (blob, metrics) = cx.text.prepare(&self.label, style, constraints);
        self.blob = Some(blob);
        self.metrics = Some(metrics);

        Size::new(cx.available.width, self.height)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.background,
            border: Edges::all(Px(1.0)),
            border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.25,
            },
            corner_radii: Corners::all(Px(8.0)),
        });

        let (Some(blob), Some(metrics)) = (self.blob, self.metrics) else {
            return;
        };

        let padding_x = Px(12.0);
        let padding_y = ((cx.bounds.size.height.0 - metrics.size.height.0) * 0.5).max(0.0);
        let origin = fret_core::geometry::Point::new(
            cx.bounds.origin.x + padding_x,
            cx.bounds.origin.y + Px(padding_y) + metrics.baseline,
        );

        cx.scene.push(SceneOp::Text {
            order: DrawOrder(0),
            origin,
            text: blob,
            color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 1.0,
            },
        });
    }
}
