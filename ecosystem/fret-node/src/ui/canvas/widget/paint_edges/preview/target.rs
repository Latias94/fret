use super::*;

#[derive(Clone, Copy)]
pub(super) struct PreviewResolvedState {
    pub(super) to: Point,
    pub(super) style: PreviewWireStyle,
}

pub(super) fn resolve_preview_target_and_style<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    geom: &CanvasGeometry,
    fallback: Point,
    interaction_hint: crate::ui::InteractionChromeHint,
) -> PreviewResolvedState {
    let target_state = PreviewTargetState::from_widget(canvas);
    let to = target_state.resolve_target(geom, fallback);
    let style = target_state.resolve_style(interaction_hint, canvas.style.paint.wire_color_preview);
    PreviewResolvedState { to, style }
}
