use fret_app::App;
use fret_core::{AppWindowId, FrameId, KeyCode, Point, Px, Rect, SemanticsRole, Size as CoreSize};
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::element::{AnyElement, SemanticsProps};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::facade as shadcn;
use fret_ui_shadcn::tabs::TabsActivationMode;
use std::sync::Arc;

#[path = "support/fake_services.rs"]
mod fake_services;
use fake_services::FakeServices;

#[path = "support/input_events.rs"]
mod input_events;
use input_events::{click_at, dispatch_key_press};

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    request_semantics: bool,
    root: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "tabs-gates", root);
    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

fn find_by_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    id: &str,
) -> &'a fret_core::SemanticsNode {
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(id))
        .unwrap_or_else(|| panic!("missing semantics node with test_id={id:?}"))
}

fn is_missing_or_hidden(snap: &fret_core::SemanticsSnapshot, id: &str) -> bool {
    snap.nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some(id))
        .is_none_or(|n| n.flags.hidden)
}

fn has_test_id(snap: &fret_core::SemanticsSnapshot, id: &str) -> bool {
    snap.nodes.iter().any(|n| n.test_id.as_deref() == Some(id))
}

fn rect_center(rect: Rect) -> Point {
    Point::new(
        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
    )
}

fn build_tabs(
    cx: &mut ElementContext<'_, App>,
    selected: Model<Option<Arc<str>>>,
    activation_mode: TabsActivationMode,
) -> Vec<AnyElement> {
    let alpha_panel = cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Generic,
            test_id: Some(Arc::from("tabs-panel-alpha")),
            ..Default::default()
        },
        |_cx| Vec::new(),
    );
    let beta_panel = cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Generic,
            test_id: Some(Arc::from("tabs-panel-beta")),
            ..Default::default()
        },
        |_cx| Vec::new(),
    );
    let gamma_panel = cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Generic,
            test_id: Some(Arc::from("tabs-panel-gamma")),
            ..Default::default()
        },
        |_cx| Vec::new(),
    );

    let tabs = shadcn::Tabs::new(selected)
        .activation_mode(activation_mode)
        .item(
            shadcn::TabsItem::new("alpha", "Alpha", [alpha_panel])
                .trigger_test_id("tabs-trigger-alpha"),
        )
        .item(
            shadcn::TabsItem::new("beta", "Beta", [beta_panel])
                .trigger_test_id("tabs-trigger-beta"),
        )
        .item(
            shadcn::TabsItem::new("gamma", "Gamma", [gamma_panel])
                .trigger_test_id("tabs-trigger-gamma"),
        );

    vec![tabs.into_element(cx)]
}

#[test]
fn tabs_automatic_activation_arrow_keys_update_selection_and_panels() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let selected: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("alpha")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    // Frame 1: mount and click the first trigger to ensure focus is inside the tab list.
    let selected_frame_1 = selected.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| build_tabs(cx, selected_frame_1, TabsActivationMode::Automatic),
    );
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let alpha = find_by_test_id(&snap, "tabs-trigger-alpha");
    click_at(&mut ui, &mut app, &mut services, rect_center(alpha.bounds));

    let selected_frame_2 = selected.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| build_tabs(cx, selected_frame_2, TabsActivationMode::Automatic),
    );

    // ArrowRight moves focus and activates in automatic mode.
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);
    let selected_frame_3 = selected.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| build_tabs(cx, selected_frame_3, TabsActivationMode::Automatic),
    );

    assert_eq!(
        app.models().get_cloned(&selected).flatten().as_deref(),
        Some("beta"),
        "expected ArrowRight to select the next tab in automatic activation mode"
    );
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let beta = find_by_test_id(&snap, "tabs-trigger-beta");
    assert!(beta.flags.focused, "expected focus to move to the next tab");
    assert!(beta.flags.selected, "expected next tab to be selected");
    assert!(
        has_test_id(&snap, "tabs-panel-beta"),
        "expected the active panel marker to match the selected tab"
    );
    assert!(
        is_missing_or_hidden(&snap, "tabs-panel-alpha"),
        "expected inactive panel marker to be absent or hidden"
    );

    // Home/End should jump to first/last and activate in automatic mode.
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::End);
    let selected_frame_4 = selected.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| build_tabs(cx, selected_frame_4, TabsActivationMode::Automatic),
    );
    assert_eq!(
        app.models().get_cloned(&selected).flatten().as_deref(),
        Some("gamma"),
        "expected End to activate the last tab"
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Home);
    let selected_frame_5 = selected.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| build_tabs(cx, selected_frame_5, TabsActivationMode::Automatic),
    );
    assert_eq!(
        app.models().get_cloned(&selected).flatten().as_deref(),
        Some("alpha"),
        "expected Home to activate the first tab"
    );
}

#[test]
fn tabs_manual_activation_moves_focus_without_selecting_until_enter() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let mut app = App::new();
    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
    let selected: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("alpha")));

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let selected_frame_1 = selected.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| build_tabs(cx, selected_frame_1, TabsActivationMode::Manual),
    );
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let alpha = find_by_test_id(&snap, "tabs-trigger-alpha");
    click_at(&mut ui, &mut app, &mut services, rect_center(alpha.bounds));

    // ArrowRight moves focus but should not update selection in manual mode.
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);
    let selected_frame_2 = selected.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| build_tabs(cx, selected_frame_2, TabsActivationMode::Manual),
    );

    assert_eq!(
        app.models().get_cloned(&selected).flatten().as_deref(),
        Some("alpha"),
        "expected ArrowRight to keep selection unchanged in manual activation mode"
    );
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    let beta = find_by_test_id(&snap, "tabs-trigger-beta");
    let alpha = find_by_test_id(&snap, "tabs-trigger-alpha");
    assert!(beta.flags.focused, "expected focus to move to the next tab");
    assert!(
        alpha.flags.selected,
        "expected the previously selected tab to remain selected"
    );
    assert!(
        has_test_id(&snap, "tabs-panel-alpha"),
        "expected active panel marker to remain for the selected tab"
    );
    assert!(
        is_missing_or_hidden(&snap, "tabs-panel-beta"),
        "expected inactive panel marker to be absent or hidden before activation"
    );

    // Enter activates the focused tab.
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::Enter);
    let selected_frame_3 = selected.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        move |cx| build_tabs(cx, selected_frame_3, TabsActivationMode::Manual),
    );
    assert_eq!(
        app.models().get_cloned(&selected).flatten().as_deref(),
        Some("beta"),
        "expected Enter to activate the focused tab in manual activation mode"
    );
    let snap = ui
        .semantics_snapshot()
        .cloned()
        .expect("expected semantics snapshot");
    assert!(
        has_test_id(&snap, "tabs-panel-beta"),
        "expected the active panel marker to update after activation"
    );
}
