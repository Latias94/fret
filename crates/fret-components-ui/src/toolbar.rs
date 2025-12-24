use fret_core::{Color, Corners, DrawOrder, Edges, Px, Rect, SceneOp, Size};
use fret_ui::{LayoutCx, PaintCx, Theme, UiHost, Widget};

use crate::style::{ColorFallback, ColorRef, MetricFallback, MetricRef, StyleRefinement};

#[derive(Debug, Clone)]
struct ResolvedToolbarStyle {
    padding_x: Px,
    padding_y: Px,
    radius: Px,
    border_width: Px,
    background: Color,
    border_color: Color,
    gap: Px,
    height: Px,
}

impl Default for ResolvedToolbarStyle {
    fn default() -> Self {
        Self {
            padding_x: Px(8.0),
            padding_y: Px(6.0),
            radius: Px(8.0),
            border_width: Px(1.0),
            background: Color::TRANSPARENT,
            border_color: Color::TRANSPARENT,
            gap: Px(6.0),
            height: Px(32.0),
        }
    }
}

pub struct Toolbar {
    style: StyleRefinement,
    last_theme_revision: Option<u64>,
    resolved: ResolvedToolbarStyle,
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            style: StyleRefinement::default(),
            last_theme_revision: None,
            resolved: ResolvedToolbarStyle::default(),
        }
    }

    pub fn refine_style(mut self, style: StyleRefinement) -> Self {
        self.style = style;
        self.last_theme_revision = None;
        self
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let px = self
            .style
            .padding_x
            .clone()
            .unwrap_or(MetricRef::Token {
                key: "component.toolbar.padding_x",
                fallback: MetricFallback::ThemePaddingSm,
            })
            .resolve(theme);
        let py = self
            .style
            .padding_y
            .clone()
            .unwrap_or(MetricRef::Token {
                key: "component.toolbar.padding_y",
                fallback: MetricFallback::ThemePaddingSm,
            })
            .resolve(theme);
        let radius = self
            .style
            .radius
            .clone()
            .unwrap_or(MetricRef::Token {
                key: "component.toolbar.radius",
                fallback: MetricFallback::ThemeRadiusMd,
            })
            .resolve(theme);
        let border_width = self
            .style
            .border_width
            .clone()
            .unwrap_or(MetricRef::Px(Px(1.0)))
            .resolve(theme);

        let bg = self
            .style
            .background
            .clone()
            .unwrap_or(ColorRef::Token {
                key: "component.toolbar.bg",
                fallback: ColorFallback::ThemePanelBackground,
            })
            .resolve(theme);
        let border_color = self
            .style
            .border_color
            .clone()
            .unwrap_or(ColorRef::Token {
                key: "component.toolbar.border",
                fallback: ColorFallback::ThemePanelBorder,
            })
            .resolve(theme);

        let gap = theme
            .metric_by_key("component.toolbar.gap")
            .unwrap_or(theme.metrics.padding_sm);
        let height = theme
            .metric_by_key("component.toolbar.height")
            .unwrap_or(Px(32.0));

        self.resolved = ResolvedToolbarStyle {
            padding_x: px,
            padding_y: py,
            radius,
            border_width,
            background: bg,
            border_color,
            gap,
            height,
        };
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for Toolbar {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());

        let pad_x = self.resolved.padding_x.0.max(0.0);
        let pad_y = self.resolved.padding_y.0.max(0.0);
        let gap = self.resolved.gap.0.max(0.0);

        let inner_origin = fret_core::Point::new(
            Px(cx.bounds.origin.x.0 + pad_x),
            Px(cx.bounds.origin.y.0 + pad_y),
        );

        let inner_width = Px((cx.available.width.0 - pad_x * 2.0).max(0.0));
        let inner_height = Px((self.resolved.height.0 - pad_y * 2.0).max(0.0));

        let mut remaining_w = inner_width.0;
        let mut max_h = 0.0f32;
        let mut placements: Vec<(fret_core::NodeId, fret_core::Point, Size)> = Vec::new();
        let mut x = inner_origin.x.0;

        for (i, &child) in cx.children.iter().enumerate() {
            if i > 0 {
                x += gap;
                remaining_w = (remaining_w - gap).max(0.0);
            }

            let is_last = i + 1 == cx.children.len();
            let probe = Rect::new(
                fret_core::Point::new(Px(x), inner_origin.y),
                Size::new(Px(remaining_w), Px(1.0e9)),
            );
            let child_size = cx.layout_in(child, probe);

            let w = if is_last {
                Px(remaining_w)
            } else {
                Px(child_size.width.0.min(remaining_w))
            };
            let h = Px(child_size.height.0.min(inner_height.0).max(0.0));
            placements.push((
                child,
                fret_core::Point::new(Px(x), inner_origin.y),
                Size::new(w, h),
            ));

            x += w.0;
            remaining_w = (remaining_w - w.0).max(0.0);
            max_h = max_h.max(h.0);
        }

        for (child, origin, size) in placements {
            let dy = ((inner_height.0 - size.height.0).max(0.0)) * 0.5;
            let child_origin = fret_core::Point::new(origin.x, Px(origin.y.0 + dy));
            let bounds = Rect::new(child_origin, Size::new(size.width, Px(inner_height.0)));
            let _ = cx.layout_in(child, bounds);
        }

        let h = self.resolved.height.0.max(0.0).min(cx.available.height.0);
        Size::new(cx.available.width, Px(h))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.resolved.background,
            border: Edges::all(Px(self.resolved.border_width.0.max(0.0))),
            border_color: self.resolved.border_color,
            corner_radii: Corners::all(self.resolved.radius),
        });

        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}
