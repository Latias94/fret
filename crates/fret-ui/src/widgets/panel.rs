use crate::{
    UiHost,
    widget::{LayoutCx, PaintCx, Widget},
};
use fret_core::{Color, Corners, DrawOrder, Edges, Px, SceneOp, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelThemeBackground {
    Surface,
    Panel,
}

pub struct ColoredPanel {
    pub background: Color,
    pub border: Edges,
    pub border_color: Color,
    pub corner_radii: Corners,
    theme_background: Option<(PanelThemeBackground, f32)>,
    last_theme_revision: Option<u64>,
}

impl ColoredPanel {
    pub fn new(background: Color) -> Self {
        Self {
            background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
            theme_background: None,
            last_theme_revision: None,
        }
    }

    pub fn themed(background: PanelThemeBackground, alpha: f32) -> Self {
        let mut out = Self::new(Color::TRANSPARENT);
        out.theme_background = Some((background, alpha.clamp(0.0, 1.0)));
        out
    }
}

impl<H: UiHost> Widget<H> for ColoredPanel {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        if let Some((bg, alpha)) = self.theme_background {
            let theme = cx.theme();
            if self.last_theme_revision != Some(theme.revision()) {
                self.last_theme_revision = Some(theme.revision());
                let base = match bg {
                    PanelThemeBackground::Surface => theme.colors.surface_background,
                    PanelThemeBackground::Panel => theme.colors.panel_background,
                };
                self.background = Color { a: alpha, ..base };
                self.border_color = Color {
                    a: alpha,
                    ..theme.colors.panel_border
                };
            }
        }

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.background,
            border: self.border,
            border_color: self.border_color,
            corner_radii: self.corner_radii,
        });
    }
}
