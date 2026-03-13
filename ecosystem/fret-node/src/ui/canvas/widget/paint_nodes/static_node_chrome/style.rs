use super::*;

#[derive(Clone, Copy)]
pub(super) struct StaticNodePaintStyle {
    pub(super) body_background: fret_core::scene::PaintBindingV1,
    pub(super) header_background: Option<fret_core::scene::PaintBindingV1>,
    pub(super) border_paint: fret_core::scene::PaintBindingV1,
    pub(super) title_text: Color,
    pub(super) shadow: Option<crate::ui::NodeShadowHint>,
}

fn resolve_border_color(
    canvas: &NodeGraphCanvasWith<impl NodeGraphCanvasMiddleware>,
    hint: crate::ui::NodeChromeHint,
    is_selected: bool,
) -> Color {
    if is_selected {
        hint.border_selected
            .or(hint.border)
            .unwrap_or(canvas.style.paint.node_border_selected)
    } else {
        hint.border.unwrap_or(canvas.style.paint.node_border)
    }
}

pub(super) fn resolve_static_node_paint_style<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    node: GraphNodeId,
    is_selected: bool,
    hint: crate::ui::NodeChromeHint,
) -> StaticNodePaintStyle {
    let background = hint
        .background
        .unwrap_or(canvas.style.paint.node_background);
    let border = resolve_border_color(canvas, hint, is_selected);

    let paint_override = canvas
        .paint_overrides
        .as_ref()
        .and_then(|overrides| overrides.node_paint_override(node));

    let body_background = paint_override
        .as_ref()
        .and_then(|override_item| override_item.body_background)
        .unwrap_or_else(|| fret_core::Paint::Solid(background).into());

    let header_background = paint_override
        .as_ref()
        .and_then(|override_item| override_item.header_background)
        .or_else(|| {
            hint.header_background
                .map(|color| fret_core::Paint::Solid(color).into())
        });

    let border_paint = paint_override
        .as_ref()
        .and_then(|override_item| override_item.border_paint)
        .unwrap_or_else(|| fret_core::Paint::Solid(border).into());

    let title_text = hint
        .title_text
        .unwrap_or(canvas.style.paint.context_menu_text);

    StaticNodePaintStyle {
        body_background,
        header_background,
        border_paint,
        title_text,
        shadow: hint.shadow,
    }
}
