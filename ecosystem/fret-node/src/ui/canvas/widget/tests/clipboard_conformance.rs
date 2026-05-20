use std::time::Duration;

use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_runtime::{ClipboardToken, Effect, TimerToken};
use fret_ui::Invalidation;

use crate::core::{CanvasPoint, Graph};
use crate::rules::DiagnosticSeverity;
use crate::ui::canvas::state::PendingPaste;

use super::{NullServices, TestUiHostImpl, event_cx, insert_graph_view_editor_config};

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

#[test]
fn clipboard_unavailable_with_matching_token_shows_toast_and_invalidates_paint() {
    let mut host = TestUiHostImpl::default();
    let (graph, view, editor_config) = insert_graph_view_editor_config(&mut host, Graph::default());
    let mut canvas = new_canvas!(host, graph, view, editor_config);
    let token = ClipboardToken(7);
    let window = AppWindowId::default();
    canvas.interaction.pending_paste = Some(PendingPaste {
        token,
        at: CanvasPoint { x: 10.0, y: 20.0 },
    });

    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        test_bounds(),
        &mut prevented_default_actions,
    );
    cx.window = Some(window);

    canvas.handle_clipboard_text_unavailable(&mut cx, token);

    assert!(canvas.interaction.pending_paste.is_none());
    let toast = canvas.interaction.toast.as_ref().expect("toast shown");
    assert_eq!(toast.severity, DiagnosticSeverity::Info);
    assert_eq!(&*toast.message, "clipboard text unavailable");
    assert_eq!(toast.timer, TimerToken(1));
    assert!(
        cx.invalidations
            .iter()
            .any(|(_, kind)| *kind == Invalidation::Paint)
    );
    assert!(cx.app.redraw.contains(&window));
    assert!(matches!(
        cx.app.effects.as_slice(),
        [Effect::SetTimer {
            window: Some(effect_window),
            after,
            repeat: None,
            ..
        }] if *effect_window == window && *after == Duration::from_millis(2400)
    ));
}

#[test]
fn clipboard_unavailable_with_stale_token_has_no_feedback_side_effects() {
    let mut host = TestUiHostImpl::default();
    let (graph, view, editor_config) = insert_graph_view_editor_config(&mut host, Graph::default());
    let mut canvas = new_canvas!(host, graph, view, editor_config);
    let pending_token = ClipboardToken(7);
    let stale_token = ClipboardToken(8);
    let window = AppWindowId::default();
    canvas.interaction.pending_paste = Some(PendingPaste {
        token: pending_token,
        at: CanvasPoint { x: 10.0, y: 20.0 },
    });

    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        test_bounds(),
        &mut prevented_default_actions,
    );
    cx.window = Some(window);

    canvas.handle_clipboard_text_unavailable(&mut cx, stale_token);

    assert!(canvas.interaction.pending_paste.is_some());
    assert!(canvas.interaction.toast.is_none());
    assert!(cx.invalidations.is_empty());
    assert!(cx.app.effects.is_empty());
    assert!(!cx.app.redraw.contains(&window));
}
