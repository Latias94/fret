use fret_core::{
    Color, Edges, Scene, SceneOp,
    geometry::{Point, Px, Rect, Size},
};

use super::super::consts::{DOCK_TAB_CLOSE_GAP, DOCK_TAB_CLOSE_SIZE, DOCK_TAB_H};
use super::super::tab_bar_geometry::dock_tab_width_for_title;
use super::super::types::PreparedTabTitle;

pub(in crate::dock) struct DockDragGhostPaint {
    pub(in crate::dock) position: Point,
    pub(in crate::dock) grab_offset: Point,
    pub(in crate::dock) title: PreparedTabTitle,
}

fn drag_ghost_title_clip_rect(
    theme: fret_ui::ThemeSnapshot,
    tab_rect: Rect,
    close_glyph_present: bool,
) -> Rect {
    let pad_x = theme.metric_token("metric.padding.md").0.max(0.0);
    let pad_sm = theme.metric_token("metric.padding.sm").0.max(0.0);
    let reserve = if close_glyph_present {
        DOCK_TAB_CLOSE_SIZE.0 + DOCK_TAB_CLOSE_GAP.0 + pad_sm
    } else {
        0.0
    };

    let max_pad = (tab_rect.size.width.0 - reserve - 1.0).max(0.0);
    let pad_x = pad_x.clamp(0.0, max_pad);

    Rect {
        origin: Point::new(Px(tab_rect.origin.x.0 + pad_x), tab_rect.origin.y),
        size: Size::new(
            Px((tab_rect.size.width.0 - pad_x - reserve).max(1.0)),
            tab_rect.size.height,
        ),
    }
}

pub(in crate::dock) fn paint_drag_payload_ghost(
    theme: fret_ui::ThemeSnapshot,
    ghost: Option<&DockDragGhostPaint>,
    close_glyph_present: bool,
    scene: &mut Scene,
) {
    let Some(ghost) = ghost else {
        return;
    };

    let width = dock_tab_width_for_title(
        theme.clone(),
        ghost.title.metrics.size.width,
        close_glyph_present,
    );
    let rect = Rect::new(
        Point::new(
            Px(ghost.position.x.0 - ghost.grab_offset.x.0),
            Px(ghost.position.y.0 - ghost.grab_offset.y.0),
        ),
        Size::new(width, DOCK_TAB_H),
    );

    let card = theme.color_token("card");
    let border = theme.color_token("border");
    let fg = theme.color_token("foreground");
    let radius_sm = theme.metric_token("metric.radius.sm");
    let clip = drag_ghost_title_clip_rect(theme.clone(), rect, close_glyph_present);

    scene.push(SceneOp::Quad {
        order: fret_core::DrawOrder(10_020),
        rect,
        background: fret_core::Paint::Solid(Color { a: 0.94, ..card }).into(),
        border: Edges::all(Px(1.0)),
        border_paint: fret_core::Paint::Solid(Color { a: 0.88, ..border }).into(),
        corner_radii: fret_core::Corners::all(Px(radius_sm.0.max(4.0))),
    });

    let inner_y =
        rect.origin.y.0 + ((rect.size.height.0 - ghost.title.metrics.size.height.0) * 0.5);
    let text_y = Px(inner_y + ghost.title.metrics.baseline.0);
    scene.push(SceneOp::PushClipRect { rect: clip });
    scene.push(SceneOp::Text {
        order: fret_core::DrawOrder(10_021),
        origin: Point::new(clip.origin.x, text_y),
        text: ghost.title.blob,
        paint: (Color { a: 0.96, ..fg }).into(),
        outline: None,
        shadow: None,
    });
    scene.push(SceneOp::PopClip);
}
