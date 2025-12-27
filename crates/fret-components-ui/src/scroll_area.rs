use crate::style::{
    ChromeRefinement, ColorFallback, MetricFallback, component_color, component_metric,
};
use fret_core::{Color, Corners, DrawOrder, Edges, Event, Px, Rect, SceneOp, Size};
use fret_ui::{EventCx, LayoutCx, PaintCx, Theme, UiHost, Widget};

#[derive(Debug, Clone, Copy)]
struct ResolvedScrollAreaStyle {
    background: Color,
    border_width: Px,
    border_color: Color,
    corner_radius: Px,
}

impl Default for ResolvedScrollAreaStyle {
    fn default() -> Self {
        Self {
            background: Color::TRANSPARENT,
            border_width: Px(0.0),
            border_color: Color::TRANSPARENT,
            corner_radius: Px(0.0),
        }
    }
}

/// A shadcn-inspired scroll area primitive.
///
/// This widget delegates scrolling behavior to `fret_ui_widgets::primitives::Scroll`, but provides a stable
/// component-level styling surface (tokens + optional chrome).
///
/// Performance notes:
/// - `ScrollArea` does not virtualize its children. For large/unknown-length lists, prefer
///   `fret_ui_widgets::primitives::VirtualList` (or a future `components-ui` list wrapper) to avoid O(N)
///   layout work
///   during scrolling.
pub struct ScrollArea {
    inner: fret_ui_widgets::primitives::Scroll,
    style: ChromeRefinement,
    last_theme_revision: Option<u64>,
    resolved: ResolvedScrollAreaStyle,
    last_bounds: Rect,
}

impl ScrollArea {
    pub fn new() -> Self {
        Self {
            inner: fret_ui_widgets::primitives::Scroll::new().overlay_scrollbar(true),
            style: ChromeRefinement::default(),
            last_theme_revision: None,
            resolved: ResolvedScrollAreaStyle::default(),
            last_bounds: Rect::default(),
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.style = style;
        self
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let default_radius = component_metric(
            "component.scroll_area.radius",
            MetricFallback::ThemeRadiusMd,
        )
        .resolve(theme);
        let default_border_width = component_metric(
            "component.scroll_area.border_width",
            MetricFallback::Px(Px(0.0)),
        )
        .resolve(theme);
        let default_bg = component_color(
            "component.scroll_area.background",
            ColorFallback::Color(Color::TRANSPARENT),
        )
        .resolve(theme);
        let default_border_color = component_color(
            "component.scroll_area.border_color",
            ColorFallback::ThemePanelBorder,
        )
        .resolve(theme);

        self.resolved.corner_radius = self
            .style
            .radius
            .as_ref()
            .map(|v| v.resolve(theme))
            .unwrap_or(default_radius);
        self.resolved.border_width = self
            .style
            .border_width
            .as_ref()
            .map(|v| v.resolve(theme))
            .unwrap_or(default_border_width);
        self.resolved.background = self
            .style
            .background
            .as_ref()
            .map(|v| v.resolve(theme))
            .unwrap_or(default_bg);
        self.resolved.border_color = self
            .style
            .border_color
            .as_ref()
            .map(|v| v.resolve(theme))
            .unwrap_or(default_border_color);
    }
}

impl Default for ScrollArea {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for ScrollArea {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;
        <fret_ui_widgets::primitives::Scroll as Widget<H>>::event(&mut self.inner, cx, event);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;
        <fret_ui_widgets::primitives::Scroll as Widget<H>>::layout(&mut self.inner, cx)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        if self.resolved.background.a > 0.0 {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: cx.bounds,
                background: self.resolved.background,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(self.resolved.corner_radius),
            });
        }

        <fret_ui_widgets::primitives::Scroll as Widget<H>>::paint(&mut self.inner, cx);

        if self.resolved.border_width.0 > 0.0 && self.resolved.border_color.a > 0.0 {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(99),
                rect: cx.bounds,
                background: Color::TRANSPARENT,
                border: Edges::all(self.resolved.border_width),
                border_color: self.resolved.border_color,
                corner_radii: Corners::all(self.resolved.corner_radius),
            });
        }
    }
}
