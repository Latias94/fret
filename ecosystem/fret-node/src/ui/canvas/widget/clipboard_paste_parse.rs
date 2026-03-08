use super::*;

pub(super) fn parse_clipboard_fragment<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    text: &str,
) -> Option<GraphFragment> {
    match GraphFragment::from_clipboard_text(text) {
        Ok(fragment) => Some(fragment),
        Err(_) => {
            canvas.show_toast(
                host,
                window,
                DiagnosticSeverity::Info,
                "clipboard does not contain a fret-node fragment",
            );
            None
        }
    }
}

pub(super) fn paste_offset_for_fragment(
    fragment: &GraphFragment,
    at: CanvasPoint,
) -> Option<CanvasPoint> {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    for node in fragment.nodes.values() {
        min_x = min_x.min(node.pos.x);
        min_y = min_y.min(node.pos.y);
    }
    for group in fragment.groups.values() {
        min_x = min_x.min(group.rect.origin.x);
        min_y = min_y.min(group.rect.origin.y);
    }
    if !min_x.is_finite() || !min_y.is_finite() {
        return None;
    }

    Some(CanvasPoint {
        x: at.x - min_x,
        y: at.y - min_y,
    })
}
