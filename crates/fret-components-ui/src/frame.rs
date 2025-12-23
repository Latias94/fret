use fret_core::{Color, Corners, DrawOrder, Edges, Px, Rect, SceneOp, Size};
use fret_ui::{LayoutCx, PaintCx, UiHost, Widget};

use crate::style::{ColorFallback, ColorRef, MetricFallback, MetricRef, StyleRefinement};

pub struct Frame {
    pub style: StyleRefinement,
    last_theme_revision: Option<u64>,
    resolved: ResolvedFrameStyle,
}

#[derive(Debug, Clone)]
struct ResolvedFrameStyle {
    padding_x: Px,
    padding_y: Px,
    radius: Px,
    border_width: Px,
    background: Color,
    border_color: Color,
}

impl Default for ResolvedFrameStyle {
    fn default() -> Self {
        Self {
            padding_x: Px(0.0),
            padding_y: Px(0.0),
            radius: Px(0.0),
            border_width: Px(0.0),
            background: Color::TRANSPARENT,
            border_color: Color::TRANSPARENT,
        }
    }
}

impl Frame {
    pub fn new(style: StyleRefinement) -> Self {
        Self {
            style,
            last_theme_revision: None,
            resolved: ResolvedFrameStyle::default(),
        }
    }

    fn sync_style_from_theme(&mut self, theme: &fret_ui::Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let px = self
            .style
            .padding_x
            .clone()
            .unwrap_or(MetricRef::Token {
                key: "component.frame.padding_x",
                fallback: MetricFallback::ThemePaddingSm,
            })
            .resolve(theme);
        let py = self
            .style
            .padding_y
            .clone()
            .unwrap_or(MetricRef::Token {
                key: "component.frame.padding_y",
                fallback: MetricFallback::ThemePaddingSm,
            })
            .resolve(theme);
        let radius = self
            .style
            .radius
            .clone()
            .unwrap_or(MetricRef::Token {
                key: "component.frame.radius",
                fallback: MetricFallback::ThemeRadiusSm,
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
                key: "component.frame.bg",
                fallback: ColorFallback::ThemePanelBackground,
            })
            .resolve(theme);
        let border_color = self
            .style
            .border_color
            .clone()
            .unwrap_or(ColorRef::Token {
                key: "component.frame.border",
                fallback: ColorFallback::ThemePanelBorder,
            })
            .resolve(theme);

        self.resolved = ResolvedFrameStyle {
            padding_x: px,
            padding_y: py,
            radius,
            border_width,
            background: bg,
            border_color,
        };
    }
}

impl<H: UiHost> Widget<H> for Frame {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());

        let pad_x = self.resolved.padding_x.0.max(0.0);
        let pad_y = self.resolved.padding_y.0.max(0.0);

        let Some(&child) = cx.children.first() else {
            return Size::new(cx.available.width, Px(0.0));
        };

        let inner_origin = fret_core::Point::new(
            Px(cx.bounds.origin.x.0 + pad_x),
            Px(cx.bounds.origin.y.0 + pad_y),
        );
        let inner_size = Size::new(
            Px((cx.available.width.0 - pad_x * 2.0).max(0.0)),
            Px((cx.available.height.0 - pad_y * 2.0).max(0.0)),
        );
        let inner = Rect::new(inner_origin, inner_size);

        let child_size = cx.layout_in(child, inner);
        let final_h = Px((child_size.height.0 + pad_y * 2.0)
            .min(cx.available.height.0)
            .max(0.0));
        Size::new(cx.available.width, final_h)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());

        let border_w = Px(self.resolved.border_width.0.max(0.0));
        let border = Edges::all(border_w);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.resolved.background,
            border,
            border_color: self.resolved.border_color,
            corner_radii: Corners::all(self.resolved.radius),
        });

        let Some(&child) = cx.children.first() else {
            return;
        };
        if let Some(bounds) = cx.child_bounds(child) {
            cx.paint(child, bounds);
        } else {
            cx.paint(child, cx.bounds);
        }
    }
}
