use crate::style::{
    ChromeRefinement, ColorFallback, MetricFallback, component_color, component_metric,
};
use fret_core::{Color, Corners, Edges, Px};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, ScrollProps};
use fret_ui::{ElementCx, Theme, UiHost};

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

/// A shadcn-inspired scroll area primitive (declarative).
///
/// This is intentionally a thin composition helper around the `Scroll` element in `fret-ui`:
///
/// - runtime owns scrolling mechanics and scrollbar behavior (hard-to-change semantics),
/// - components own chrome/tokens (background, border, radius).
///
/// Performance notes:
/// - `ScrollArea` does not virtualize its children. For large/unknown-length lists, prefer
///   `fret-ui`'s `VirtualList` element (or a future component wrapper) to avoid O(N) layout work.
#[derive(Debug, Default, Clone)]
pub struct ScrollArea {
    style: ChromeRefinement,
}

impl ScrollArea {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.style = style;
        self
    }

    fn resolve_style(&self, theme: &Theme) -> ResolvedScrollAreaStyle {
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

        let corner_radius = self
            .style
            .radius
            .as_ref()
            .map(|v| v.resolve(theme))
            .unwrap_or(default_radius);
        let border_width = self
            .style
            .border_width
            .as_ref()
            .map(|v| v.resolve(theme))
            .unwrap_or(default_border_width);
        let background = self
            .style
            .background
            .as_ref()
            .map(|v| v.resolve(theme))
            .unwrap_or(default_bg);
        let border_color = self
            .style
            .border_color
            .as_ref()
            .map(|v| v.resolve(theme))
            .unwrap_or(default_border_color);

        ResolvedScrollAreaStyle {
            background,
            border_width,
            border_color,
            corner_radius,
        }
    }

    pub fn build<H: UiHost>(
        self,
        cx: &mut ElementCx<'_, H>,
        f: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        scroll_area(cx, self, f)
    }
}

pub fn scroll_area<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    area: ScrollArea,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let resolved = area.resolve_style(&theme);

    let mut container_layout = LayoutStyle::default();
    container_layout.size.width = Length::Fill;
    container_layout.size.height = Length::Fill;
    container_layout.overflow = fret_ui::element::Overflow::Clip;

    let mut scroll_layout = LayoutStyle::default();
    scroll_layout.size.width = Length::Fill;
    scroll_layout.size.height = Length::Fill;

    cx.container(
        ContainerProps {
            layout: container_layout,
            background: (resolved.background.a > 0.0).then_some(resolved.background),
            border: Edges::all(resolved.border_width),
            border_color: (resolved.border_color.a > 0.0).then_some(resolved.border_color),
            corner_radii: Corners::all(resolved.corner_radius),
            ..Default::default()
        },
        |cx| {
            vec![cx.scroll(
                ScrollProps {
                    layout: scroll_layout,
                    show_scrollbar: true,
                    scroll_handle: None,
                },
                f,
            )]
        },
    )
}

