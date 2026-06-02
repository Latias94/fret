use fret_core::{
    Color, Edges, Scene, SceneOp,
    geometry::{Point, Px, Rect, Size},
};

use super::super::consts::DOCK_FLOATING_BORDER;
use super::super::types::PreparedTabTitle;

#[derive(Debug, Clone)]
pub(in crate::dock) struct FloatingChromePaintInput {
    pub(in crate::dock) outer: Rect,
    pub(in crate::dock) title_bar: Rect,
    pub(in crate::dock) close_button: Rect,
    pub(in crate::dock) title_bar_hovered: bool,
    pub(in crate::dock) close_hovered: bool,
    pub(in crate::dock) close_pressed: bool,
}

fn paint_floating_chrome_input(
    theme: fret_ui::ThemeSnapshot,
    input: &FloatingChromePaintInput,
    tab_close_glyph: Option<PreparedTabTitle>,
    tab_close_svg: Option<fret_core::SvgId>,
    scene: &mut Scene,
) {
    let border = theme.color_token("border");
    let surface = theme.color_token("background");
    let hover_bg = theme.color_token("accent");
    let fg = theme.color_token("foreground");
    let fg_muted = theme.color_token("muted-foreground");
    let radius_md = theme.metric_token("metric.radius.md");
    let radius_sm = theme.metric_token("metric.radius.sm");

    let border_color = Color { a: 0.85, ..border };
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(0),
        rect: input.outer,
        background: fret_core::Paint::Solid(surface).into(),
        border: Edges::all(DOCK_FLOATING_BORDER),
        border_paint: fret_core::Paint::Solid(border_color).into(),
        corner_radii: fret_core::Corners::all(Px(radius_md.0.max(6.0))),
    });
    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(1),
        rect: input.title_bar,
        background: fret_core::Paint::Solid(if input.title_bar_hovered {
            Color {
                a: 0.22,
                ..hover_bg
            }
        } else {
            surface
        })
        .into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: fret_core::Corners::all(Px(0.0)),
    });

    if input.close_hovered || input.close_pressed {
        scene.push(SceneOp::Quad {
            order: fret_core::DrawOrder(2),
            rect: input.close_button,
            background: fret_core::Paint::Solid(hover_bg).into(),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
        });
    }

    let color = if input.close_hovered || input.close_pressed {
        fg
    } else {
        fg_muted
    };
    if let Some(svg) = tab_close_svg {
        let pad = Px(1.0);
        let rect = Rect {
            origin: Point::new(
                Px(input.close_button.origin.x.0 + pad.0),
                Px(input.close_button.origin.y.0 + pad.0),
            ),
            size: Size::new(
                Px((input.close_button.size.width.0 - pad.0 * 2.0).max(1.0)),
                Px((input.close_button.size.height.0 - pad.0 * 2.0).max(1.0)),
            ),
        };
        scene.push(SceneOp::SvgMaskIcon {
            order: fret_core::DrawOrder(3),
            rect,
            svg,
            fit: fret_core::SvgFit::Contain,
            color,
            opacity: 1.0,
        });
    } else if let Some(glyph) = tab_close_glyph {
        let text_x = Px(input.close_button.origin.x.0
            + (input.close_button.size.width.0 - glyph.metrics.size.width.0) * 0.5);
        let inner_y = input.close_button.origin.y.0
            + ((input.close_button.size.height.0 - glyph.metrics.size.height.0) * 0.5);
        let text_y = Px(inner_y + glyph.metrics.baseline.0);
        scene.push(SceneOp::Text {
            order: fret_core::DrawOrder(3),
            origin: Point::new(text_x, text_y),
            text: glyph.blob,
            paint: (color).into(),
            outline: None,
            shadow: None,
        });
    }
}

pub(in crate::dock) fn paint_floating_chrome_inputs(
    theme: fret_ui::ThemeSnapshot,
    inputs: &[FloatingChromePaintInput],
    tab_close_glyph: Option<PreparedTabTitle>,
    tab_close_svg: Option<fret_core::SvgId>,
    scene: &mut Scene,
) {
    for input in inputs {
        paint_floating_chrome_input(theme.clone(), input, tab_close_glyph, tab_close_svg, scene);
    }
}
