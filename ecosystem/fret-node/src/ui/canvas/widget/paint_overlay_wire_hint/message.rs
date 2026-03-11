use std::sync::Arc;

use super::super::*;

fn invalid_hover_message(message: Option<&Arc<str>>) -> Arc<str> {
    message
        .cloned()
        .unwrap_or_else(|| Arc::<str>::from("Invalid connection"))
}

fn drag_hint_text(kind: &WireDragKind) -> Option<Arc<str>> {
    match kind {
        WireDragKind::New { bundle, .. } if bundle.len() > 1 => {
            Some(Arc::<str>::from(format!("Bundle: {}", bundle.len())))
        }
        WireDragKind::ReconnectMany { edges } if edges.len() > 1 => {
            Some(Arc::<str>::from(format!("Yank: {}", edges.len())))
        }
        _ => None,
    }
}

pub(in super::super) fn hint_text<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    wire_drag: &WireDrag,
    invalid_hover: bool,
) -> Option<Arc<str>> {
    if invalid_hover {
        return Some(invalid_hover_message(
            canvas
                .interaction
                .hover_port_diagnostic
                .as_ref()
                .map(|(_severity, message)| message),
        ));
    }

    drag_hint_text(&wire_drag.kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_hover_message_prefers_hover_diagnostic() {
        assert_eq!(
            invalid_hover_message(Some(&Arc::<str>::from("warn"))),
            Arc::<str>::from("warn")
        );
        assert_eq!(
            invalid_hover_message(None),
            Arc::<str>::from("Invalid connection")
        );
    }

    #[test]
    fn hint_text_reports_bundle_and_yank_counts() {
        let bundle_kind = WireDragKind::New {
            from: PortId::new(),
            bundle: vec![PortId::new(), PortId::new()],
        };
        let yank_kind = WireDragKind::ReconnectMany {
            edges: vec![
                (
                    EdgeId::new(),
                    crate::rules::EdgeEndpoint::From,
                    PortId::new(),
                ),
                (EdgeId::new(), crate::rules::EdgeEndpoint::To, PortId::new()),
            ],
        };

        assert_eq!(
            drag_hint_text(&bundle_kind),
            Some(Arc::<str>::from("Bundle: 2"))
        );
        assert_eq!(
            drag_hint_text(&yank_kind),
            Some(Arc::<str>::from("Yank: 2"))
        );
    }
}
