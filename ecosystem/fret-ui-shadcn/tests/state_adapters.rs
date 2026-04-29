#![cfg(feature = "state")]

use std::cell::Cell;
use std::rc::Rc;

use fret_app::App;
use fret_core::{AppWindowId, FrameId, Point, Px, Rect, Size};
use fret_query::{QueryError, QueryState, QueryStatus};
use fret_runtime::{ActionId, CommandId, TypedAction};
use fret_ui::ElementContext;
use fret_ui::declarative::render_root;
use fret_ui::element::AnyElement;
use fret_ui::tree::UiTree;
use fret_ui_shadcn::facade as shadcn;

#[path = "support/style_aware_services.rs"]
mod style_aware_services;
use style_aware_services::StyleAwareServices;

#[derive(Clone, Copy)]
struct OpenResultRow;

impl TypedAction for OpenResultRow {
    fn action_id() -> ActionId {
        ActionId::from("test.state_adapters.open_result_row.v1")
    }
}

impl From<OpenResultRow> for CommandId {
    fn from(_: OpenResultRow) -> CommandId {
        OpenResultRow::action_id()
    }
}

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    )
}

fn render_state_root(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut StyleAwareServices,
    frame: u64,
    render: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) {
    app.set_frame_id(FrameId(frame));
    let window = AppWindowId::default();
    ui.set_window(window);
    let root = render_root(
        ui,
        app,
        services,
        window,
        bounds(),
        "shadcn-state-adapters",
        render,
    );
    ui.set_root(root);
    ui.layout_all(app, services, bounds(), 1.0);
}

#[test]
fn selector_badge_adapter_runs_inside_render_context_and_reuses_stable_deps() {
    let mut app = App::new();
    let mut ui = UiTree::<App>::new();
    let mut services = StyleAwareServices::default();
    let compute_calls = Rc::new(Cell::new(0usize));

    for frame in 1..=2 {
        let compute_calls = compute_calls.clone();
        render_state_root(&mut ui, &mut app, &mut services, frame, move |cx| {
            let badge = shadcn::use_selector_badge(
                cx,
                shadcn::BadgeVariant::Secondary,
                |_cx| 7u32,
                |_cx| {
                    compute_calls.set(compute_calls.get().saturating_add(1));
                    42u32
                },
            );
            let debug = format!("{badge:?}");
            assert!(
                debug.contains("42") && debug.contains("Secondary"),
                "selector badge should preserve computed label and requested variant; debug={debug}"
            );
            vec![badge.into_element(cx)]
        });
    }

    assert_eq!(
        compute_calls.get(),
        1,
        "stable selector dependencies should not recompute across retained frames"
    );
}

#[test]
fn query_badge_adapter_maps_status_and_error_alert_without_state_stack_leakage() {
    let mut app = App::new();
    let mut ui = UiTree::<App>::new();
    let mut services = StyleAwareServices::default();

    render_state_root(&mut ui, &mut app, &mut services, 1, |cx| {
        let idle = QueryState::<u32>::default();
        let idle_badge = shadcn::query_status_badge(cx, &idle);
        let idle_debug = format!("{idle_badge:?}");
        assert!(
            idle_debug.contains("Idle") && idle_debug.contains("Secondary"),
            "idle query badge should map to secondary Idle badge; debug={idle_debug}"
        );

        let ready = QueryState {
            status: QueryStatus::Success,
            data: Some(5u32.into()),
            ..QueryState::default()
        };
        let ready_badge = shadcn::query_status_badge(cx, &ready);
        let ready_debug = format!("{ready_badge:?}");
        assert!(
            ready_debug.contains("Ready") && ready_debug.contains("Default"),
            "success query badge should map to default Ready badge; debug={ready_debug}"
        );

        let failed = QueryState::<u32> {
            status: QueryStatus::Error,
            error: Some(QueryError::permanent("offline")),
            ..QueryState::default()
        };
        let failed_badge = shadcn::query_status_badge(cx, &failed);
        let failed_debug = format!("{failed_badge:?}");
        assert!(
            failed_debug.contains("Error") && failed_debug.contains("Destructive"),
            "error query badge should map to destructive Error badge; debug={failed_debug}"
        );

        let alert = shadcn::query_error_alert(cx, &failed).expect("expected query error alert");
        let alert_debug = format!("{alert:?}");
        assert!(
            alert_debug.contains("Destructive"),
            "query error alert should preserve destructive variant; debug={alert_debug}"
        );
        assert!(
            shadcn::query_error_alert(cx, &ready).is_none(),
            "non-error query states should not produce an alert"
        );

        vec![failed_badge.into_element(cx), alert.into_element(cx)]
    });
}

#[test]
fn state_adapters_preserve_typed_payload_routing_for_dynamic_items() {
    let mut app = App::new();
    let mut ui = UiTree::<App>::new();
    let mut services = StyleAwareServices::default();

    render_state_root(&mut ui, &mut app, &mut services, 1, |cx| {
        let ready = QueryState {
            status: QueryStatus::Success,
            data: Some(5u32.into()),
            ..QueryState::default()
        };

        let row_badge = shadcn::query_status_badge(cx, &ready)
            .action(OpenResultRow)
            .action_payload(5u32);
        let debug = format!("{row_badge:?}");
        assert!(
            debug.contains("test.state_adapters.open_result_row.v1"),
            "adapted recipes should keep dynamic row/item intents on typed action IDs; debug={debug}"
        );
        assert!(
            debug.contains("action_payload: true"),
            "adapted recipes should preserve typed payload routing for dynamic row/item intents; debug={debug}"
        );

        vec![row_badge.into_element(cx)]
    });
}
