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
