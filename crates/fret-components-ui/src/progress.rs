use fret_core::{Color, Corners, DrawOrder, Edges, Px, Rect, SceneOp, SemanticsRole, Size};
use fret_runtime::Model;
use fret_ui::{Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget};

use crate::style::{ColorFallback, MetricFallback, component_color, component_metric};

#[derive(Debug, Clone)]
struct ResolvedProgressStyle {
    height: Px,
    radius: Px,
    border_width: Px,
    track_bg: Color,
    fill: Color,
    border: Color,
}

impl Default for ResolvedProgressStyle {
    fn default() -> Self {
        Self {
            height: Px(10.0),
            radius: Px(999.0),
            border_width: Px(1.0),
            track_bg: Color::TRANSPARENT,
            fill: Color::TRANSPARENT,
            border: Color::TRANSPARENT,
        }
    }
}

pub struct ProgressBar {
    model: Model<f32>,
    min: f32,
    max: f32,
    last_theme_revision: Option<u64>,
    resolved: ResolvedProgressStyle,
}

impl ProgressBar {
    pub fn new(model: Model<f32>) -> Self {
        Self {
            model,
            min: 0.0,
            max: 1.0,
            last_theme_revision: None,
            resolved: ResolvedProgressStyle::default(),
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    fn sync_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let height = component_metric("component.progress.height", MetricFallback::Px(Px(10.0)))
            .resolve(theme);
        let radius = component_metric("component.progress.radius", MetricFallback::Px(Px(999.0)))
            .resolve(theme);
        let border_width = component_metric(
            "component.progress.border_width",
            MetricFallback::Px(Px(1.0)),
        )
        .resolve(theme);

        let track_bg = component_color(
            "component.progress.track_bg",
            // Match `Slider`'s baseline: a visible overlay color on dark themes.
            ColorFallback::ThemeHoverBackground,
        )
        .resolve(theme);
        let fill =
            component_color("component.progress.fill", ColorFallback::ThemeAccent).resolve(theme);
        let border = component_color("component.progress.border", ColorFallback::ThemePanelBorder)
            .resolve(theme);

        self.resolved = ResolvedProgressStyle {
            height,
            radius,
            border_width,
            track_bg,
            fill,
            border,
        };
    }

    fn value<H: UiHost>(&self, app: &H) -> f32 {
        app.models().get(self.model).copied().unwrap_or(self.min)
    }

    fn normalized(&self, v: f32) -> f32 {
        let span = self.max - self.min;
        if !span.is_finite() || span.abs() <= f32::EPSILON {
            return 0.0;
        }
        ((v - self.min) / span).clamp(0.0, 1.0)
    }
}

impl<H: UiHost> Widget<H> for ProgressBar {
    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Generic);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_from_theme(cx.theme());
        cx.observe_model(self.model, Invalidation::Paint);
        let h = self.resolved.height.0.max(0.0).min(cx.available.height.0);
        Size::new(cx.available.width, Px(h))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_from_theme(cx.theme());
        cx.observe_model(self.model, Invalidation::Paint);

        let t = self.normalized(self.value(cx.app));
        let border_w = Px(self.resolved.border_width.0.max(0.0));
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.resolved.track_bg,
            border: Edges::all(border_w),
            border_color: self.resolved.border,
            corner_radii: Corners::all(self.resolved.radius),
        });

        let fill_w = Px((cx.bounds.size.width.0 * t).max(0.0));
        let fill = Rect::new(cx.bounds.origin, Size::new(fill_w, cx.bounds.size.height));
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect: fill,
            background: self.resolved.fill,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(self.resolved.radius),
        });
    }
}
