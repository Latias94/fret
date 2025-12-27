use fret_core::{
    Color, DrawOrder, FontId, Px, SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics,
    TextOverflow, TextStyle, TextWrap,
};

use fret_ui::widget::SemanticsCx;
use fret_ui::{LayoutCx, PaintCx, Theme, UiHost, Widget};

#[derive(Debug, Clone)]
pub struct Text {
    text: String,
    style: TextStyle,
    color: Color,
    use_theme_defaults: bool,
    last_theme_revision: Option<u64>,
    blob: Option<fret_core::TextBlobId>,
    metrics: Option<TextMetrics>,
    prepared_scale_factor_bits: Option<u32>,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TextStyle {
                font: FontId::default(),
                size: Px(13.0),
                ..Default::default()
            },
            color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 1.0,
            },
            use_theme_defaults: true,
            last_theme_revision: None,
            blob: None,
            metrics: None,
            prepared_scale_factor_bits: None,
        }
    }

    pub fn with_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self.use_theme_defaults = false;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self.use_theme_defaults = false;
        self
    }

    fn sync_from_theme(&mut self, theme: &Theme) {
        if !self.use_theme_defaults {
            return;
        }
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        self.style.size = theme
            .metric_by_key("font.size")
            .unwrap_or(theme.metrics.font_size);
        self.style.line_height = Some(
            theme
                .metric_by_key("font.line_height")
                .unwrap_or(theme.metrics.font_line_height),
        );
        self.color = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);
        self.prepared_scale_factor_bits = None;
    }
}

impl<H: UiHost> Widget<H> for Text {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        if let Some(blob) = self.blob.take() {
            services.text().release(blob);
        }
        self.prepared_scale_factor_bits = None;
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Text);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_from_theme(cx.theme());
        let constraints = TextConstraints {
            max_width: Some(cx.available.width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let metrics = cx
            .services
            .text()
            .measure(&self.text, self.style, constraints);
        self.metrics = Some(metrics);
        metrics.size
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_from_theme(cx.theme());
        let constraints = TextConstraints {
            max_width: Some(cx.bounds.size.width),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        let scale_bits = cx.scale_factor.to_bits();
        let needs_prepare =
            self.blob.is_none() || self.prepared_scale_factor_bits != Some(scale_bits);
        if needs_prepare {
            if let Some(blob) = self.blob.take() {
                cx.services.text().release(blob);
            }
            let (blob, metrics) = cx
                .services
                .text()
                .prepare(&self.text, self.style, constraints);
            self.blob = Some(blob);
            self.metrics = Some(metrics);
            self.prepared_scale_factor_bits = Some(scale_bits);
        }

        let Some(blob) = self.blob else { return };
        let Some(metrics) = self.metrics else { return };

        let origin = fret_core::geometry::Point::new(
            cx.bounds.origin.x,
            cx.bounds.origin.y + metrics.baseline,
        );
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(0),
            origin,
            text: blob,
            color: self.color,
        });
    }
}
