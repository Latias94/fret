use super::super::*;
use super::state::EdgeAnchorInteractionState;

#[derive(Debug, Clone, Copy)]
pub(super) struct EdgeAnchorBaseStyle {
    pub(super) border_base: Px,
    pub(super) border_hover: Px,
    pub(super) border_active: Px,
    pub(super) anchor_color: Color,
    pub(super) fill_color: Color,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct EdgeAnchorPaintStyle {
    pub(super) border: Px,
    pub(super) anchor_color: Color,
    pub(super) background: Color,
}

pub(super) fn base_anchor_style<M: NodeGraphCanvasMiddleware>(
    color: Color,
    zoom: f32,
) -> EdgeAnchorBaseStyle {
    let z = zoom.max(1.0e-6);
    EdgeAnchorBaseStyle {
        border_base: Px(NodeGraphCanvasWith::<M>::EDGE_FOCUS_ANCHOR_BORDER_SCREEN / z),
        border_hover: Px((NodeGraphCanvasWith::<M>::EDGE_FOCUS_ANCHOR_BORDER_SCREEN + 0.5) / z),
        border_active: Px((NodeGraphCanvasWith::<M>::EDGE_FOCUS_ANCHOR_BORDER_SCREEN + 1.0) / z),
        anchor_color: Color {
            r: color.r,
            g: color.g,
            b: color.b,
            a: 0.95,
        },
        fill_color: Color {
            r: color.r,
            g: color.g,
            b: color.b,
            a: 0.15,
        },
    }
}

pub(super) fn edge_anchor_paint_style(
    base: EdgeAnchorBaseStyle,
    interaction_state: EdgeAnchorInteractionState,
) -> EdgeAnchorPaintStyle {
    let border = if interaction_state.active {
        base.border_active
    } else if interaction_state.hovered {
        base.border_hover
    } else {
        base.border_base
    };

    let background = if interaction_state.active {
        Color {
            a: (base.fill_color.a + 0.20).min(1.0),
            ..base.fill_color
        }
    } else if interaction_state.hovered {
        Color {
            a: (base.fill_color.a + 0.10).min(1.0),
            ..base.fill_color
        }
    } else {
        base.fill_color
    };

    EdgeAnchorPaintStyle {
        border,
        anchor_color: base.anchor_color,
        background,
    }
}

#[cfg(test)]
mod tests;
