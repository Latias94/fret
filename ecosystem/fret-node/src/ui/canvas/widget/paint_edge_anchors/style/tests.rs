use super::{EdgeAnchorBaseStyle, base_anchor_style, edge_anchor_paint_style};
use crate::ui::canvas::widget::NoopNodeGraphCanvasMiddleware;
use crate::ui::canvas::widget::paint_edge_anchors::state::EdgeAnchorInteractionState;
use fret_core::{Color, Px};

#[test]
fn base_anchor_style_scales_border_by_zoom() {
    let base = base_anchor_style::<NoopNodeGraphCanvasMiddleware>(
        Color {
            r: 1.0,
            g: 0.5,
            b: 0.25,
            a: 1.0,
        },
        2.0,
    );

    assert_eq!(base.border_base, Px(1.0));
    assert_eq!(base.border_hover, Px(1.25));
    assert_eq!(base.border_active, Px(1.5));
    assert_eq!(base.anchor_color.a, 0.95);
    assert_eq!(base.fill_color.a, 0.15);
}

#[test]
fn edge_anchor_paint_style_elevates_hover_and_active_states() {
    let base = EdgeAnchorBaseStyle {
        border_base: Px(2.0),
        border_hover: Px(2.5),
        border_active: Px(3.0),
        anchor_color: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
        fill_color: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.15,
        },
    };

    let hovered = edge_anchor_paint_style(
        base,
        EdgeAnchorInteractionState {
            hovered: true,
            active: false,
        },
    );
    let active = edge_anchor_paint_style(
        base,
        EdgeAnchorInteractionState {
            hovered: false,
            active: true,
        },
    );

    assert_eq!(hovered.border, Px(2.5));
    assert_eq!(active.border, Px(3.0));
    assert!(hovered.background.a > base.fill_color.a);
    assert!(active.background.a > hovered.background.a);
}
