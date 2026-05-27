//! Focused interaction regression tests for Material 3 SearchView.

use fret_core::{AppWindowId, KeyCode, NodeId, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{SearchView, SearchViewPresentation};

mod interaction_harness;
mod support;

use support::events::{key_down, key_up};
use support::goldens::run_overlay_frame;
use support::host::{FakeUiServices, TestHost};
use support::theme::apply_material_theme;

#[test]
fn search_view_full_screen_uses_modal_overlay_and_closes_on_escape() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::Expressive);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(320.0)),
    );

    let open = app.models_mut().insert(true);
    let query = app.models_mut().insert(String::from("alpha"));
    let open_model = open.clone();
    let query_model = query.clone();

    let render = move |ui: &mut UiTree<TestHost>,
                       app: &mut TestHost,
                       services: &mut dyn UiServices| {
        let open = open_model.clone();
        let query = query_model.clone();
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", move |cx| {
            vec![
                SearchView::new(open, query)
                    .test_id("m3-search-view")
                    .placeholder("Search")
                    .presentation(SearchViewPresentation::FullScreen)
                    .into_element(cx, |cx| vec![cx.text("Result alpha")]),
            ]
        })
    };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack
            .stack
            .iter()
            .any(|entry| entry.kind == OverlayStackEntryKind::Modal && entry.open && entry.visible),
        "expected full-screen SearchView to use a visible modal overlay"
    );

    let has_overlay_id = ui
        .semantics_snapshot()
        .map(|snapshot| {
            snapshot
                .nodes
                .iter()
                .any(|node| node.test_id.as_deref() == Some("m3-search-view.overlay"))
        })
        .unwrap_or(false);
    assert!(
        has_overlay_id,
        "expected full-screen SearchView overlay to expose m3-search-view.overlay"
    );

    let header_input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("m3-search-view.overlay.header"))
                    .then_some(node.id)
            })
        })
        .expect("expected full-screen SearchView header input test id");
    assert_eq!(
        ui.focus(),
        Some(header_input_node),
        "expected full-screen SearchView to focus the overlay-local header input"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));

    let mut closed = false;
    for _ in 0..16 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );

        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        if !stack
            .stack
            .iter()
            .any(|entry| entry.kind == OverlayStackEntryKind::Modal && entry.visible)
        {
            closed = true;
            break;
        }
    }

    assert!(closed, "expected full-screen SearchView to close on Escape");
    assert_eq!(
        app.models().get_copied(&open),
        Some(false),
        "expected Escape to collapse the SearchView open model"
    );
}
