use fret_core::{Color, DrawOrder, Edges, Px, SceneOp, Size};
use fret_ui::{LayoutCx, PaintCx, Theme, UiHost, Widget};

use crate::style::{ColorFallback, MetricFallback, component_color, component_metric};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparatorAxis {
    Horizontal,
    Vertical,
}

pub struct Separator {
    axis: SeparatorAxis,
    thickness: Option<Px>,
    last_theme_revision: Option<u64>,
    resolved_thickness: Px,
    resolved_color: Color,
}

impl Separator {
    pub fn horizontal() -> Self {
        Self::new(SeparatorAxis::Horizontal)
    }

    pub fn vertical() -> Self {
        Self::new(SeparatorAxis::Vertical)
    }

    pub fn new(axis: SeparatorAxis) -> Self {
        Self {
            axis,
            thickness: None,
            last_theme_revision: None,
            resolved_thickness: Px(1.0),
            resolved_color: Color::TRANSPARENT,
        }
    }

    pub fn thickness(mut self, thickness: Px) -> Self {
        self.thickness = Some(thickness);
        self
    }

    fn sync_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let thickness = self.thickness.unwrap_or_else(|| {
            component_metric("component.separator.thickness", MetricFallback::Px(Px(1.0)))
                .resolve(theme)
        });
        let color = component_color("component.separator.color", ColorFallback::ThemePanelBorder)
            .resolve(theme);

        self.resolved_thickness = thickness;
        self.resolved_color = color;
    }
}

impl Default for Separator {
    fn default() -> Self {
        Self::horizontal()
    }
}

impl<H: UiHost> Widget<H> for Separator {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_from_theme(cx.theme());
        let t = self.resolved_thickness.0.max(0.0);
        match self.axis {
            SeparatorAxis::Horizontal => {
                Size::new(cx.available.width, Px(t.min(cx.available.height.0)))
            }
            SeparatorAxis::Vertical => {
                Size::new(Px(t.min(cx.available.width.0)), cx.available.height)
            }
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_from_theme(cx.theme());
        let mut rect = cx.bounds;
        let t = self.resolved_thickness.0.max(0.0);
        match self.axis {
            SeparatorAxis::Horizontal => rect.size.height = Px(t.min(rect.size.height.0)),
            SeparatorAxis::Vertical => rect.size.width = Px(t.min(rect.size.width.0)),
        }

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect,
            background: self.resolved_color,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });
    }
}
