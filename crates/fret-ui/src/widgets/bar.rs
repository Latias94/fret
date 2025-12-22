use crate::{
    UiHost,
    widget::{LayoutCx, PaintCx, Widget},
};
use fret_core::{Color, Corners, DrawOrder, Edges, Px, Rect, SceneOp, Size};

use super::panel::PanelThemeBackground;

pub struct Bar {
    pub background: PanelThemeBackground,
    pub alpha: f32,
    last_theme_revision: Option<u64>,
    resolved_bg: Color,
}

impl Bar {
    pub fn new(background: PanelThemeBackground, alpha: f32) -> Self {
        Self {
            background,
            alpha: alpha.clamp(0.0, 1.0),
            last_theme_revision: None,
            resolved_bg: Color::TRANSPARENT,
        }
    }

    fn sync_theme(&mut self, theme: &crate::Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());
        let base = match self.background {
            PanelThemeBackground::Surface => theme.colors.surface_background,
            PanelThemeBackground::Panel => theme.colors.panel_background,
        };
        self.resolved_bg = Color {
            a: self.alpha,
            ..base
        };
    }
}

impl<H: UiHost> Widget<H> for Bar {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let Some(&child) = cx.children.first() else {
            return Size::new(cx.available.width, Px(0.0));
        };

        let probe_bounds = Rect::new(
            cx.bounds.origin,
            Size::new(cx.available.width, cx.available.height),
        );
        let measured = cx.layout_in(child, probe_bounds);
        let height = Px(measured.height.0.max(0.0).min(cx.available.height.0));

        let final_bounds = Rect::new(cx.bounds.origin, Size::new(cx.available.width, height));
        let _ = cx.layout_in(child, final_bounds);

        Size::new(cx.available.width, height)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_theme(cx.theme());
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.resolved_bg,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });

        if let Some(&child) = cx.children.first() {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}
