use std::sync::Arc;

use fret_core::{AppWindowId, KeyCode, NodeId, Point, Px, Rect, Size, UiServices};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod support;

use support::events::{key_down, key_up};
use support::goldens::run_overlay_frame;
use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

// NavigationBar, NavigationRail, and NavigationDrawer interaction regressions.

fn semantics_node_exists(ui: &UiTree<TestHost>, test_id: &str) -> bool {
    semantics_node_id_by_test_id(ui, test_id).is_some()
}

fn semantics_node_selected(ui: &UiTree<TestHost>, test_id: &str) -> bool {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(test_id))
                .map(|node| node.flags.selected)
        })
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"))
}

fn assert_route_panel_for(
    ui: &UiTree<TestHost>,
    root_test_id: &str,
    item_prefix: &str,
    active: &str,
    inactive: &[&str],
) {
    assert!(
        semantics_node_exists(ui, root_test_id),
        "expected routed content panel root {root_test_id}"
    );
    assert!(
        semantics_node_exists(ui, &format!("{item_prefix}-{active}")),
        "expected routed content panel for {active}"
    );
    for route in inactive {
        assert!(
            !semantics_node_exists(ui, &format!("{item_prefix}-{route}")),
            "expected inactive route panel {route} to be unmounted"
        );
    }
}

fn assert_route_panel(ui: &UiTree<TestHost>, active: &str, inactive: &[&str]) {
    assert_route_panel_for(ui, "nav-routes-panel", "nav-routes-panel", active, inactive);
}

#[test]
fn navigation_bar_roving_skips_disabled_and_updates_model() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationBar, NavigationBarItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(320.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let bar = NavigationBar::new(value)
                    .a11y_label("Material 3 Navigation Bar")
                    .test_id("nav-bar")
                    .items(vec![
                        NavigationBarItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-bar-search"),
                        NavigationBarItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("nav-bar-disabled"),
                        NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .a11y_label("Destination Settings")
                            .test_id("nav-bar-settings"),
                        NavigationBarItem::new("more", "More", ids::ui::MORE_HORIZONTAL)
                            .a11y_label("Destination More")
                            .test_id("nav-bar-more"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), bar)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-search")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-search in semantics snapshot");
    let disabled_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-disabled")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-disabled in semantics snapshot");
    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-settings")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-settings in semantics snapshot");
    let more_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-more")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-more in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowRight to skip disabled destinations"
    );
    assert_ne!(
        ui.focus(),
        Some(disabled_node),
        "expected disabled destination to never receive focus"
    );

    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "settings",
        "expected selection to follow roving focus"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::End));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::End));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(more_node),
        "expected End to rove to the last enabled destination"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(search_node),
        "expected Home to rove to the first enabled destination"
    );
}

#[test]
fn navigation_bar_roving_wraps_and_skips_disabled_on_reverse() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationBar, NavigationBarItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(320.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let bar = NavigationBar::new(value)
                    .a11y_label("Material 3 Navigation Bar")
                    .test_id("nav-bar")
                    .items(vec![
                        NavigationBarItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-bar-search"),
                        NavigationBarItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("nav-bar-disabled"),
                        NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .a11y_label("Destination Settings")
                            .test_id("nav-bar-settings"),
                        NavigationBarItem::new("more", "More", ids::ui::MORE_HORIZONTAL)
                            .a11y_label("Destination More")
                            .test_id("nav-bar-more"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), bar)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-search")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-search in semantics snapshot");
    let disabled_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-disabled")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-disabled in semantics snapshot");
    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-settings")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-settings in semantics snapshot");
    let more_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-more")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-more in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowLeft));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(more_node),
        "expected ArrowLeft to wrap to the last enabled destination when loop_navigation=true"
    );
    assert_ne!(
        ui.focus(),
        Some(disabled_node),
        "expected disabled destination to never receive focus"
    );
    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "more",
        "expected selection to follow roving focus after reverse wrap"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowLeft));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowLeft to rove to the previous enabled destination"
    );

    // Now verify loop_navigation=false clamps at the first enabled item (no wrap).
    let value2: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value2_for_render = value2.clone();
    let render_no_loop =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value2 = value2_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root2", |cx| {
                let bar = NavigationBar::new(value2)
                    .loop_navigation(false)
                    .a11y_label("Material 3 Navigation Bar (no loop)")
                    .test_id("nav-bar-no-loop")
                    .items(vec![
                        NavigationBarItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-bar-no-loop-search"),
                        NavigationBarItem::new("more", "More", ids::ui::MORE_HORIZONTAL)
                            .a11y_label("Destination More")
                            .test_id("nav-bar-no-loop-more"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), bar)]
            })
        };

    let root = render_no_loop(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node2: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-bar-no-loop-search")).then_some(node.id)
            })
        })
        .expect("expected nav-bar-no-loop-search in semantics snapshot");

    ui.set_focus(Some(search_node2));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowLeft));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowLeft));

    let root = render_no_loop(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(search_node2),
        "expected ArrowLeft at the first item to not wrap when loop_navigation=false"
    );
}

#[test]
fn navigation_rail_roving_skips_disabled_and_updates_model() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationRail, NavigationRailItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let rail = NavigationRail::new(value)
                    .a11y_label("Material 3 Navigation Rail")
                    .test_id("nav-rail")
                    .items(vec![
                        NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-rail-search"),
                        NavigationRailItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("nav-rail-disabled"),
                        NavigationRailItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .a11y_label("Destination Settings")
                            .test_id("nav-rail-settings"),
                        NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                            .a11y_label("Destination Play")
                            .test_id("nav-rail-play"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), rail)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-search")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-search in semantics snapshot");
    let disabled_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-disabled")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-disabled in semantics snapshot");
    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-settings")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-settings in semantics snapshot");
    let play_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-play")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-play in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowDown to skip disabled destinations"
    );
    assert_ne!(
        ui.focus(),
        Some(disabled_node),
        "expected disabled destination to never receive focus"
    );

    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "settings",
        "expected selection to follow roving focus"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::End));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::End));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(play_node),
        "expected End to rove to the last enabled destination"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(search_node),
        "expected Home to rove to the first enabled destination"
    );
}

#[test]
fn navigation_rail_roving_wraps_and_skips_disabled_on_reverse() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationRail, NavigationRailItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let rail = NavigationRail::new(value)
                    .a11y_label("Material 3 Navigation Rail")
                    .test_id("nav-rail")
                    .items(vec![
                        NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-rail-search"),
                        NavigationRailItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("nav-rail-disabled"),
                        NavigationRailItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .a11y_label("Destination Settings")
                            .test_id("nav-rail-settings"),
                        NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                            .a11y_label("Destination Play")
                            .test_id("nav-rail-play"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), rail)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-search")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-search in semantics snapshot");
    let disabled_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-disabled")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-disabled in semantics snapshot");
    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-settings")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-settings in semantics snapshot");
    let play_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-play")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-play in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(play_node),
        "expected ArrowUp to wrap to the last enabled destination when loop_navigation=true"
    );
    assert_ne!(
        ui.focus(),
        Some(disabled_node),
        "expected disabled destination to never receive focus"
    );
    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "play",
        "expected selection to follow roving focus after reverse wrap"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowUp to rove to the previous enabled destination"
    );
}

#[test]
fn navigation_rail_roving_does_not_wrap_when_loop_navigation_false() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationRail, NavigationRailItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let rail = NavigationRail::new(value)
                    .loop_navigation(false)
                    .a11y_label("Material 3 Navigation Rail (no loop)")
                    .test_id("nav-rail-no-loop")
                    .items(vec![
                        NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-rail-no-loop-search"),
                        NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                            .a11y_label("Destination Play")
                            .test_id("nav-rail-no-loop-play"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), rail)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-no-loop-search")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-no-loop-search in semantics snapshot");
    let play_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-no-loop-play")).then_some(node.id)
            })
        })
        .expect("expected nav-rail-no-loop-play in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(search_node),
        "expected ArrowUp at the first item to not wrap when loop_navigation=false",
    );

    ui.set_focus(Some(play_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(play_node),
        "expected ArrowDown at the last item to not wrap when loop_navigation=false",
    );
}

#[test]
fn navigation_rail_roving_single_enabled_item_does_not_move_under_no_loop() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationRail, NavigationRailItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("settings"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let rail = NavigationRail::new(value)
                    .loop_navigation(false)
                    .a11y_label("Material 3 Navigation Rail (single enabled, no loop)")
                    .test_id("nav-rail-single-enabled")
                    .items(vec![
                        NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                            .disabled(true)
                            .a11y_label("Destination Search (disabled)")
                            .test_id("nav-rail-single-enabled-search-disabled"),
                        NavigationRailItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .a11y_label("Destination Settings")
                            .test_id("nav-rail-single-enabled-settings"),
                        NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                            .disabled(true)
                            .a11y_label("Destination Play (disabled)")
                            .test_id("nav-rail-single-enabled-play-disabled"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), rail)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-rail-single-enabled-settings"))
                    .then_some(node.id)
            })
        })
        .expect("expected nav-rail-single-enabled-settings in semantics snapshot");

    ui.set_focus(Some(settings_node));

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowUp to keep focus when only one destination is enabled",
    );
    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "settings",
        "expected selection to remain on the only enabled destination",
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowDown to keep focus when only one destination is enabled",
    );
    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "settings",
        "expected selection to remain on the only enabled destination",
    );
}

#[test]
fn navigation_drawer_roving_skips_disabled_and_updates_model() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationDrawer, NavigationDrawerItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let drawer = NavigationDrawer::new(value)
                    .a11y_label("Material 3 Navigation Drawer")
                    .test_id("nav-drawer")
                    .items(vec![
                        NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-drawer-search"),
                        NavigationDrawerItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("nav-drawer-disabled"),
                        NavigationDrawerItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .badge_label("2")
                            .a11y_label("Destination Settings")
                            .test_id("nav-drawer-settings"),
                        NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                            .badge_label("99+")
                            .a11y_label("Destination Play")
                            .test_id("nav-drawer-play"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), drawer)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-search")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-search in semantics snapshot");
    let disabled_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-disabled")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-disabled in semantics snapshot");
    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-settings")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-settings in semantics snapshot");
    let play_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-play")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-play in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowDown to skip disabled destinations"
    );
    assert_ne!(
        ui.focus(),
        Some(disabled_node),
        "expected disabled destination to never receive focus"
    );

    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "settings",
        "expected selection to follow roving focus"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::End));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::End));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(play_node),
        "expected End to rove to the last enabled destination"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Home));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Home));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(search_node),
        "expected Home to rove to the first enabled destination"
    );
}

#[test]
fn navigation_drawer_roving_wraps_and_skips_disabled_on_reverse() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationDrawer, NavigationDrawerItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let drawer = NavigationDrawer::new(value)
                    .a11y_label("Material 3 Navigation Drawer")
                    .test_id("nav-drawer")
                    .items(vec![
                        NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-drawer-search"),
                        NavigationDrawerItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("nav-drawer-disabled"),
                        NavigationDrawerItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .badge_label("2")
                            .a11y_label("Destination Settings")
                            .test_id("nav-drawer-settings"),
                        NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                            .badge_label("99+")
                            .a11y_label("Destination Play")
                            .test_id("nav-drawer-play"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), drawer)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-search")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-search in semantics snapshot");
    let disabled_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-disabled")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-disabled in semantics snapshot");
    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-settings")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-settings in semantics snapshot");
    let play_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-play")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-play in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(play_node),
        "expected ArrowUp to wrap to the last enabled destination when loop_navigation=true"
    );
    assert_ne!(
        ui.focus(),
        Some(disabled_node),
        "expected disabled destination to never receive focus"
    );
    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "play",
        "expected selection to follow roving focus after reverse wrap"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowUp to rove to the previous enabled destination"
    );
}

#[test]
fn navigation_drawer_roving_does_not_wrap_when_loop_navigation_false() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationDrawer, NavigationDrawerItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let drawer = NavigationDrawer::new(value)
                    .loop_navigation(false)
                    .a11y_label("Material 3 Navigation Drawer (no loop)")
                    .test_id("nav-drawer-no-loop")
                    .items(vec![
                        NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("nav-drawer-no-loop-search"),
                        NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                            .a11y_label("Destination Play")
                            .test_id("nav-drawer-no-loop-play"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), drawer)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let search_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-no-loop-search")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-no-loop-search in semantics snapshot");
    let play_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-no-loop-play")).then_some(node.id)
            })
        })
        .expect("expected nav-drawer-no-loop-play in semantics snapshot");

    ui.set_focus(Some(search_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(search_node),
        "expected ArrowUp at the first item to not wrap when loop_navigation=false",
    );

    ui.set_focus(Some(play_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(play_node),
        "expected ArrowDown at the last item to not wrap when loop_navigation=false",
    );
}

#[test]
fn navigation_drawer_roving_single_enabled_item_does_not_move_under_no_loop() {
    use fret_icons::ids;
    use fret_ui_material3::{NavigationDrawer, NavigationDrawerItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(520.0)),
    );

    let value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("settings"));
    let value_for_render = value.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let value = value_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let drawer = NavigationDrawer::new(value)
                    .loop_navigation(false)
                    .a11y_label("Material 3 Navigation Drawer (single enabled, no loop)")
                    .test_id("nav-drawer-single-enabled")
                    .items(vec![
                        NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                            .disabled(true)
                            .a11y_label("Destination Search (disabled)")
                            .test_id("nav-drawer-single-enabled-search-disabled"),
                        NavigationDrawerItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .a11y_label("Destination Settings")
                            .test_id("nav-drawer-single-enabled-settings"),
                        NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                            .disabled(true)
                            .a11y_label("Destination Play (disabled)")
                            .test_id("nav-drawer-single-enabled-play-disabled"),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), drawer)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let settings_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("nav-drawer-single-enabled-settings"))
                    .then_some(node.id)
            })
        })
        .expect("expected nav-drawer-single-enabled-settings in semantics snapshot");

    ui.set_focus(Some(settings_node));

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowUp to keep focus when only one destination is enabled",
    );
    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "settings",
        "expected selection to remain on the only enabled destination",
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(settings_node),
        "expected ArrowDown to keep focus when only one destination is enabled",
    );
    let selected = app.models().get_cloned(&value).expect("value model exists");
    assert_eq!(
        selected.as_ref(),
        "settings",
        "expected selection to remain on the only enabled destination",
    );
}

#[test]
fn navigation_surfaces_drive_routed_panel_content() {
    use fret_core::Axis;
    use fret_icons::ids;
    use fret_ui::Invalidation;
    use fret_ui::element::{ContainerProps, FlexProps, Length, SpacingLength};
    use fret_ui_material3::{
        NavigationBar, NavigationBarItem, NavigationDrawer, NavigationDrawerItem, NavigationRail,
        NavigationRailItem,
    };

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(900.0), Px(520.0)),
    );

    let route: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("search"));
    let route_for_render = route.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let route = route_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let selected_route = cx
                    .get_model_cloned(&route, Invalidation::Layout)
                    .unwrap_or_else(|| Arc::<str>::from("search"));

                let bar = NavigationBar::new(route.clone())
                    .a11y_label("Material routed navigation bar")
                    .test_id("nav-routes-bar")
                    .items(vec![
                        NavigationBarItem::new("search", "Search", ids::ui::SEARCH)
                            .test_id("nav-routes-bar-search"),
                        NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .test_id("nav-routes-bar-settings"),
                        NavigationBarItem::new("play", "Play", ids::ui::PLAY)
                            .test_id("nav-routes-bar-play"),
                    ])
                    .into_element(cx);

                let rail = NavigationRail::new(route.clone())
                    .a11y_label("Material routed navigation rail")
                    .test_id("nav-routes-rail")
                    .items(vec![
                        NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                            .test_id("nav-routes-rail-search"),
                        NavigationRailItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .test_id("nav-routes-rail-settings"),
                        NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                            .test_id("nav-routes-rail-play"),
                    ])
                    .into_element(cx);

                let drawer = NavigationDrawer::new(route.clone())
                    .a11y_label("Material routed navigation drawer")
                    .test_id("nav-routes-drawer")
                    .items(vec![
                        NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                            .test_id("nav-routes-drawer-search"),
                        NavigationDrawerItem::new("settings", "Settings", ids::ui::SETTINGS)
                            .badge_label("2")
                            .test_id("nav-routes-drawer-settings"),
                        NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                            .badge_label("99+")
                            .test_id("nav-routes-drawer-play"),
                    ])
                    .into_element(cx);

                let route_id = match selected_route.as_ref() {
                    "settings" => "settings",
                    "play" => "play",
                    _ => "search",
                };
                let route_text = match route_id {
                    "settings" => "Settings route content",
                    "play" => "Play route content",
                    _ => "Search route content",
                };
                let route_leaf = cx
                    .text(route_text)
                    .test_id(format!("nav-routes-panel-{route_id}"));
                let panel = cx
                    .container(
                        ContainerProps {
                            layout: {
                                let mut layout = fret_ui::element::LayoutStyle::default();
                                layout.size.width = Length::Px(Px(260.0));
                                layout.size.height = Length::Px(Px(180.0));
                                layout
                            },
                            ..Default::default()
                        },
                        move |_cx| vec![route_leaf],
                    )
                    .test_id("nav-routes-panel");

                let row = {
                    let mut props = FlexProps::default();
                    props.direction = Axis::Horizontal;
                    props.gap = SpacingLength::Px(Px(16.0));
                    props.layout.size.width = Length::Fill;
                    props.layout.size.height = Length::Px(Px(280.0));
                    cx.flex(props, move |_cx| vec![rail, drawer, panel])
                };

                let stack = {
                    let mut props = FlexProps::default();
                    props.direction = Axis::Vertical;
                    props.gap = SpacingLength::Px(Px(16.0));
                    props.layout.size.width = Length::Fill;
                    cx.flex(props, move |_cx| vec![bar, row])
                };

                vec![with_padding(cx, Px(24.0), stack)]
            })
        };

    let render_frame =
        |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let root = render(ui, app, services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(app, services, bounds, 1.0);
        };

    render_frame(&mut ui, &mut app, &mut services);

    assert_route_panel(&ui, "search", &["settings", "play"]);
    assert!(semantics_node_selected(&ui, "nav-routes-bar-search"));
    assert!(semantics_node_selected(&ui, "nav-routes-rail-search"));
    assert!(semantics_node_selected(&ui, "nav-routes-drawer-search"));

    let bar_search = semantics_node_id_by_test_id(&ui, "nav-routes-bar-search")
        .expect("expected routed NavigationBar search item");
    ui.set_focus(Some(bar_search));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowRight));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowRight));
    render_frame(&mut ui, &mut app, &mut services);

    assert_eq!(
        app.models().get_cloned(&route).as_deref(),
        Some("settings"),
        "expected NavigationBar focus movement to update the shared route model"
    );
    assert_route_panel(&ui, "settings", &["search", "play"]);
    assert!(semantics_node_selected(&ui, "nav-routes-bar-settings"));
    assert!(semantics_node_selected(&ui, "nav-routes-rail-settings"));
    assert!(semantics_node_selected(&ui, "nav-routes-drawer-settings"));
    assert_eq!(
        ui.focus(),
        semantics_node_id_by_test_id(&ui, "nav-routes-bar-settings"),
        "expected focus to stay on the active NavigationBar destination"
    );

    let rail_settings = semantics_node_id_by_test_id(&ui, "nav-routes-rail-settings")
        .expect("expected routed NavigationRail settings item");
    ui.set_focus(Some(rail_settings));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));
    render_frame(&mut ui, &mut app, &mut services);

    assert_eq!(
        app.models().get_cloned(&route).as_deref(),
        Some("play"),
        "expected NavigationRail focus movement to update the shared route model"
    );
    assert_route_panel(&ui, "play", &["search", "settings"]);
    assert!(semantics_node_selected(&ui, "nav-routes-bar-play"));
    assert!(semantics_node_selected(&ui, "nav-routes-rail-play"));
    assert!(semantics_node_selected(&ui, "nav-routes-drawer-play"));
    assert_eq!(
        ui.focus(),
        semantics_node_id_by_test_id(&ui, "nav-routes-rail-play"),
        "expected focus to stay on the active NavigationRail destination"
    );

    let drawer_play = semantics_node_id_by_test_id(&ui, "nav-routes-drawer-play")
        .expect("expected routed NavigationDrawer play item");
    ui.set_focus(Some(drawer_play));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));
    render_frame(&mut ui, &mut app, &mut services);

    assert_eq!(
        app.models().get_cloned(&route).as_deref(),
        Some("settings"),
        "expected NavigationDrawer focus movement to update the shared route model"
    );
    assert_route_panel(&ui, "settings", &["search", "play"]);
    assert!(semantics_node_selected(&ui, "nav-routes-bar-settings"));
    assert!(semantics_node_selected(&ui, "nav-routes-rail-settings"));
    assert!(semantics_node_selected(&ui, "nav-routes-drawer-settings"));
    assert_eq!(
        ui.focus(),
        semantics_node_id_by_test_id(&ui, "nav-routes-drawer-settings"),
        "expected focus to stay on the active NavigationDrawer destination"
    );
}

#[test]
fn modal_navigation_drawer_drives_routed_content_and_closes_on_destination_activation() {
    use fret_core::Axis;
    use fret_icons::ids;
    use fret_ui::Invalidation;
    use fret_ui::action::OnActivate;
    use fret_ui::element::{ContainerProps, FlexProps, Length, SpacingLength};
    use fret_ui_material3::{
        Button, ModalNavigationDrawer, NavigationDrawer, NavigationDrawerItem,
        NavigationDrawerVariant,
    };

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(640.0), Px(420.0)),
    );

    let open = app.models_mut().insert(false);
    let route = app.models_mut().insert(Arc::<str>::from("search"));

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let selected_route = cx
                .get_model_cloned(&route, Invalidation::Layout)
                .unwrap_or_else(|| Arc::<str>::from("search"));
            let route_id = match selected_route.as_ref() {
                "settings" => "settings",
                "play" => "play",
                _ => "search",
            };
            let route_text = match route_id {
                "settings" => "Settings route content",
                "play" => "Play route content",
                _ => "Search route content",
            };
            let route_leaf = cx
                .text(route_text)
                .test_id(format!("modal-routes-panel-{route_id}"));
            let route_panel = cx
                .container(
                    ContainerProps {
                        layout: {
                            let mut layout = fret_ui::element::LayoutStyle::default();
                            layout.size.width = Length::Px(Px(260.0));
                            layout.size.height = Length::Px(Px(180.0));
                            layout
                        },
                        ..Default::default()
                    },
                    move |_cx| vec![route_leaf],
                )
                .test_id("modal-routes-panel");

            let close_on_select: OnActivate = {
                let open = open.clone();
                Arc::new(move |host, action_cx, _reason| {
                    let _ = host.models_mut().update(&open, |v| *v = false);
                    host.request_redraw(action_cx.window);
                })
            };

            let modal = ModalNavigationDrawer::new(open.clone())
                .test_id("modal-routes-drawer")
                .into_element(
                    cx,
                    {
                        let route = route.clone();
                        let close_on_select = close_on_select.clone();
                        move |cx| {
                            NavigationDrawer::new(route)
                                .variant(NavigationDrawerVariant::Modal)
                                .a11y_label("Material routed modal navigation drawer")
                                .test_id("modal-routes-drawer-content")
                                .items(vec![
                                    NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                                        .on_select(close_on_select.clone())
                                        .test_id("modal-routes-drawer-search"),
                                    NavigationDrawerItem::new(
                                        "settings",
                                        "Settings",
                                        ids::ui::SETTINGS,
                                    )
                                    .badge_label("2")
                                    .on_select(close_on_select.clone())
                                    .test_id("modal-routes-drawer-settings"),
                                    NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                                        .badge_label("99+")
                                        .on_select(close_on_select.clone())
                                        .test_id("modal-routes-drawer-play"),
                                ])
                                .into_element(cx)
                        }
                    },
                    {
                        let open = open.clone();
                        move |cx| {
                            let open_drawer: OnActivate = {
                                let open = open.clone();
                                Arc::new(move |host, action_cx, _reason| {
                                    let _ = host.models_mut().update(&open, |v| *v = true);
                                    host.request_redraw(action_cx.window);
                                })
                            };
                            let trigger = Button::new("Open drawer")
                                .on_activate(open_drawer)
                                .test_id("modal-routes-trigger")
                                .into_element(cx);

                            let stack = {
                                let mut props = FlexProps::default();
                                props.direction = Axis::Vertical;
                                props.gap = SpacingLength::Px(Px(16.0));
                                props.layout.size.width = Length::Fill;
                                props.layout.size.height = Length::Fill;
                                cx.flex(props, move |_cx| vec![trigger, route_panel])
                            };
                            with_padding(cx, Px(24.0), stack)
                        }
                    },
                );
            vec![modal]
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
    assert_route_panel_for(
        &ui,
        "modal-routes-panel",
        "modal-routes-panel",
        "search",
        &["settings", "play"],
    );

    let trigger_node = semantics_node_id_by_test_id(&ui, "modal-routes-trigger")
        .expect("expected modal routed drawer trigger");
    ui.set_focus(Some(trigger_node));
    let _ = app.models_mut().update(&open, |v| *v = true);
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot with modal drawer open");
    assert!(
        snapshot.barrier_root.is_some(),
        "expected modal barrier root while routed drawer is open"
    );
    assert!(semantics_node_selected(&ui, "modal-routes-drawer-search"));

    let search = semantics_node_id_by_test_id(&ui, "modal-routes-drawer-search")
        .expect("expected modal routed drawer search item");
    ui.set_focus(Some(search));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    assert_eq!(
        app.models().get_cloned(&route).as_deref(),
        Some("settings"),
        "expected modal NavigationDrawer roving to update the caller-owned route model"
    );
    assert_eq!(
        app.models().get_copied(&open),
        Some(true),
        "expected roving selection to keep the modal drawer open"
    );
    assert!(semantics_node_selected(&ui, "modal-routes-drawer-settings"));
    assert_eq!(
        ui.focus(),
        semantics_node_id_by_test_id(&ui, "modal-routes-drawer-settings"),
        "expected focus to stay on the routed modal drawer destination after roving"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Enter));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Enter));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );
    assert_eq!(
        app.models().get_copied(&open),
        Some(false),
        "expected activating the selected destination to close the modal drawer"
    );

    let mut saw_barrier_cleared = false;
    for _ in 0..60 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let snapshot = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after modal drawer close");
        if snapshot.barrier_root.is_none() {
            saw_barrier_cleared = true;
            break;
        }
    }
    assert!(
        saw_barrier_cleared,
        "expected modal routed drawer barrier to unmount after destination activation"
    );
    assert_eq!(
        ui.focus(),
        semantics_node_id_by_test_id(&ui, "modal-routes-trigger"),
        "expected focus to restore to the drawer trigger after destination activation closes it"
    );
    assert_route_panel_for(
        &ui,
        "modal-routes-panel",
        "modal-routes-panel",
        "settings",
        &["search", "play"],
    );
}

#[test]
fn modal_navigation_drawer_focus_is_contained_and_restored_across_schemes() {
    use fret_ui_material3::{Button, ModalNavigationDrawer};

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];

    for (mode, variant, label) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, mode, variant);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(480.0), Px(360.0)),
        );

        let open = app.models_mut().insert(false);

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let drawer = ModalNavigationDrawer::new(open.clone())
                        .test_id("drawer")
                        .into_element(
                            cx,
                            |cx| {
                                Button::new("Drawer item")
                                    .test_id("drawer-item")
                                    .into_element(cx)
                            },
                            |cx| {
                                let trigger = Button::new("Open drawer")
                                    .test_id("drawer-trigger")
                                    .into_element(cx);
                                with_padding(cx, Px(24.0), trigger)
                            },
                        );
                    vec![drawer]
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

        let trigger_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                assert_eq!(snapshot.barrier_root, None);
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("drawer-trigger") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected drawer-trigger in semantics snapshot ({label})"));
        ui.set_focus(Some(trigger_node));
        assert_eq!(ui.focus(), Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let snapshot = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot");
        assert!(
            snapshot.barrier_root.is_some(),
            "expected modal barrier root while drawer is open ({label})"
        );
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|node| node.test_id.as_deref() == Some("drawer.scrim")),
            "expected drawer scrim node while drawer is open ({label})"
        );
        assert_ne!(
            ui.focus(),
            Some(trigger_node),
            "expected focus to move into drawer layer while open ({label})"
        );

        ui.set_focus(Some(trigger_node));
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        assert_ne!(
            ui.focus(),
            Some(trigger_node),
            "expected modal barrier to enforce focus containment ({label})"
        );

        let _ = app.models_mut().update(&open, |v| *v = false);
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected focus to restore to trigger on close ({label})"
        );

        let mut saw_barrier_cleared = false;
        for _ in 0..60 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );

            let snapshot = ui
                .semantics_snapshot()
                .expect("expected semantics snapshot");
            if snapshot.barrier_root.is_none() {
                saw_barrier_cleared = true;
                break;
            }
        }
        assert!(
            saw_barrier_cleared,
            "expected drawer barrier to unmount after close transition ({label})"
        );
    }
}
