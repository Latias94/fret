use std::sync::Arc;

use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_runtime::TimerToken;
use fret_ui::Invalidation;

use crate::core::Graph;
use crate::rules::DiagnosticSeverity;
use crate::ui::canvas::state::ToastState;

use super::{NullServices, TestUiHostImpl, event_cx, insert_graph_view_editor_config};

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

#[test]
fn matching_toast_timer_clears_toast_and_invalidates_paint() {
    let mut host = TestUiHostImpl::default();
    let (graph, view, editor_config) = insert_graph_view_editor_config(&mut host, Graph::default());
    let mut canvas = new_canvas!(host, graph, view, editor_config);
    let token = TimerToken(7);
    let window = AppWindowId::default();
    canvas.interaction.toast = Some(ToastState {
        timer: token,
        severity: DiagnosticSeverity::Warning,
        message: Arc::<str>::from("expired"),
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

    assert!(super::super::event_timer_toast::clear_expired_toast(
        &mut canvas,
        &mut cx,
        token
    ));

    assert!(canvas.interaction.toast.is_none());
    assert!(
        cx.invalidations
            .iter()
            .any(|(_, kind)| *kind == Invalidation::Paint)
    );
    assert!(cx.app.redraw.contains(&window));
}

#[test]
fn stale_toast_timer_keeps_toast_without_feedback_side_effects() {
    let mut host = TestUiHostImpl::default();
    let (graph, view, editor_config) = insert_graph_view_editor_config(&mut host, Graph::default());
    let mut canvas = new_canvas!(host, graph, view, editor_config);
    let token = TimerToken(7);
    let stale_token = TimerToken(8);
    let window = AppWindowId::default();
    canvas.interaction.toast = Some(ToastState {
        timer: token,
        severity: DiagnosticSeverity::Warning,
        message: Arc::<str>::from("still active"),
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

    assert!(!super::super::event_timer_toast::clear_expired_toast(
        &mut canvas,
        &mut cx,
        stale_token
    ));

    let toast = canvas.interaction.toast.as_ref().expect("toast remains");
    assert_eq!(toast.timer, token);
    assert_eq!(&*toast.message, "still active");
    assert!(cx.invalidations.is_empty());
    assert!(!cx.app.redraw.contains(&window));
}
