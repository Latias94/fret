use std::{collections::HashMap, sync::Arc};

use fret_core::{
    AppWindowId, DrawOrder, Edges, Event, KeyCode, Modifiers, NodeId, Point, PointerId, Px, Rect,
    Scene, SceneOp, SemanticsInvalid, SemanticsLive, SemanticsRole, Size, Transform2D, UiServices,
};
use fret_runtime::{Effect, Model, ModelHost, PlatformCapabilities};
use fret_ui::{Theme, UiTree};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod interaction_harness;
mod support;

use interaction_harness::{QuadGeomSig, SceneSig, scene_quad_geometry_signature, scene_signature};
use support::events::{
    drain_zero_delay_timer_tokens, key_down, key_up, pointer_down, pointer_move,
    pointer_move_touch, pointer_up,
};
use support::goldens::{run_overlay_frame, run_overlay_frame_scaled};
use support::host::{FakeUiServices, TestHost};
use support::layout::{
    find_first_bounds_with_size, paint_alpha, semantics_node_id_by_test_id, with_padding,
};
use support::theme::apply_material_theme;

fn semantics_invalid_by_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Option<SemanticsInvalid> {
    ui.semantics_snapshot().and_then(|snapshot| {
        snapshot
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some(test_id))
            .and_then(|node| node.flags.invalid)
    })
}

fn semantics_label_by_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Option<String> {
    ui.semantics_snapshot().and_then(|snapshot| {
        snapshot
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some(test_id))
            .and_then(|node| node.label.clone())
    })
}

fn semantics_live_by_test_id(
    ui: &UiTree<TestHost>,
    test_id: &str,
) -> Option<(SemanticsLive, bool)> {
    ui.semantics_snapshot().and_then(|snapshot| {
        snapshot
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some(test_id))
            .and_then(|node| node.flags.live.map(|live| (live, node.flags.live_atomic)))
    })
}
#[test]
fn text_input_text_input_event_updates_model() {
    use fret_ui::element::TextInputProps;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(420.0)),
    );

    let model = app.models_mut().insert(String::new());
    let model_for_render = model.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let mut props = TextInputProps::new(model_for_render.clone());
                props.layout.size.width = fret_ui::element::Length::Px(Px(200.0));
                props.layout.size.height = fret_ui::element::Length::Px(Px(40.0));
                props.a11y_label = Some(Arc::<str>::from("input"));
                props.test_id = Some(Arc::<str>::from("plain-text-input"));
                let input = cx.text_input(props);
                vec![with_padding(cx, Px(24.0), input)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("plain-text-input")).then_some(node.id)
            })
        })
        .expect("expected plain-text-input in semantics snapshot");

    ui.set_focus(Some(input_node));
    assert_eq!(
        ui.focus(),
        Some(input_node),
        "expected focus to be set to the input node",
    );

    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("a".to_string()));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let value = app.models().get_cloned(&model).expect("model exists");
    assert_eq!(value, "a", "expected text input event to update the model");
}

#[test]
fn snackbar_action_emits_command_and_dismisses() {
    use fret_runtime::CommandId;
    use fret_ui::action::UiActionHostAdapter;
    use fret_ui_kit::ToastStore;
    use fret_ui_material3::{Snackbar, SnackbarController, SnackbarHost};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let store = app.models_mut().insert(ToastStore::default());
    let controller = SnackbarController::new(store.clone());
    let cmd = CommandId::new("material3_snackbar_action");

    {
        let mut action_host = UiActionHostAdapter { app: &mut app };
        let _id = controller.show(
            &mut action_host,
            window,
            Snackbar::new("Saved").action_id("Undo", cmd.clone()),
        );
    }

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let store = store.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![SnackbarHost::new(store).max_snackbars(1).into_element(cx)]
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

    let toast_root = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find(|node| {
                node.test_id
                    .as_deref()
                    .is_some_and(|id| id.starts_with("toast-entry-"))
            })
        })
        .expect("expected a toast-entry semantics node");

    let toast_root_id = toast_root.id;

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot for toast");
    let by_id: HashMap<NodeId, &fret_core::SemanticsNode> =
        snapshot.nodes.iter().map(|n| (n.id, n)).collect();

    let is_descendant_of = |mut node: NodeId, ancestor: NodeId| -> bool {
        let mut guard = 0usize;
        while guard < 256 {
            if node == ancestor {
                return true;
            }
            guard += 1;
            let Some(parent) = by_id.get(&node).and_then(|n| n.parent) else {
                return false;
            };
            node = parent;
        }
        false
    };

    let action_text = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.label.as_deref() == Some("Undo") && is_descendant_of(node.id, toast_root_id)
        })
        .expect("expected the toast action text (Undo) to appear in semantics");

    let mut action_button_id = action_text.id;
    let mut guard = 0usize;
    while guard < 256 {
        guard += 1;
        let Some(node) = by_id.get(&action_button_id) else {
            break;
        };
        if node.role == fret_core::SemanticsRole::Button {
            break;
        }
        let Some(parent) = node.parent else {
            break;
        };
        action_button_id = parent;
    }

    let action_bounds = ui
        .debug_node_visual_bounds(action_button_id)
        .expect("expected toast action bounds");
    let click_at = Point::new(
        Px(action_bounds.origin.x.0 + action_bounds.size.width.0 * 0.5),
        Px(action_bounds.origin.y.0 + action_bounds.size.height.0 * 0.5),
    );

    app.effects.clear();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    assert!(
        app.effects.iter().any(|effect| matches!(
            effect,
            Effect::Command { command, .. } if *command == cmd
        )),
        "expected clicking snackbar action to emit a command effect"
    );

    let remove_tokens: Vec<fret_runtime::TimerToken> = app
        .effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::SetTimer {
                window: Some(w),
                token,
                after: _,
                repeat: None,
            } if *w == window => Some(*token),
            _ => None,
        })
        .collect();
    assert!(
        !remove_tokens.is_empty(),
        "expected snackbar dismiss to schedule a timer for removal"
    );

    for token in remove_tokens {
        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });
    }

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let has_toast = ui.semantics_snapshot().is_some_and(|snapshot| {
        snapshot.nodes.iter().any(|node| {
            node.test_id
                .as_deref()
                .is_some_and(|id| id.starts_with("toast-entry-"))
        })
    });
    assert!(
        !has_toast,
        "expected snackbar to be removed after dismiss timer fires"
    );
}

#[test]
fn snackbar_dismiss_button_dismisses_without_emitting_command() {
    use fret_ui::action::UiActionHostAdapter;
    use fret_ui_kit::ToastStore;
    use fret_ui_material3::{Snackbar, SnackbarController, SnackbarHost};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let store = app.models_mut().insert(ToastStore::default());
    let controller = SnackbarController::new(store.clone());
    {
        let mut action_host = UiActionHostAdapter { app: &mut app };
        let _id = controller.show(&mut action_host, window, Snackbar::new("Dismiss me"));
    }

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let store = store.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![SnackbarHost::new(store).max_snackbars(1).into_element(cx)]
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

    let toast_root = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find(|node| {
                node.test_id
                    .as_deref()
                    .is_some_and(|id| id.starts_with("toast-entry-"))
            })
        })
        .expect("expected a toast-entry semantics node");

    let toast_root_id = toast_root.id;

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot for toast");
    let by_id: HashMap<NodeId, &fret_core::SemanticsNode> =
        snapshot.nodes.iter().map(|n| (n.id, n)).collect();

    let is_descendant_of = |mut node: NodeId, ancestor: NodeId| -> bool {
        let mut guard = 0usize;
        while guard < 256 {
            if node == ancestor {
                return true;
            }
            guard += 1;
            let Some(parent) = by_id.get(&node).and_then(|n| n.parent) else {
                return false;
            };
            node = parent;
        }
        false
    };

    let close_text = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.label.as_deref() == Some("\u{00D7}") && is_descendant_of(node.id, toast_root_id)
        })
        .expect("expected toast close glyph (×) to appear in semantics");

    let mut close_button_id = close_text.id;
    let mut guard = 0usize;
    while guard < 256 {
        guard += 1;
        let Some(node) = by_id.get(&close_button_id) else {
            break;
        };
        if node.role == fret_core::SemanticsRole::Button {
            break;
        }
        let Some(parent) = node.parent else {
            break;
        };
        close_button_id = parent;
    }

    let close_bounds = ui
        .debug_node_visual_bounds(close_button_id)
        .expect("expected close button bounds");
    let click_at = Point::new(
        Px(close_bounds.origin.x.0 + close_bounds.size.width.0 * 0.5),
        Px(close_bounds.origin.y.0 + close_bounds.size.height.0 * 0.5),
    );

    app.effects.clear();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    assert!(
        !app.effects
            .iter()
            .any(|effect| matches!(effect, Effect::Command { .. })),
        "expected clicking snackbar dismiss button not to emit a command effect",
    );

    let remove_tokens: Vec<fret_runtime::TimerToken> = app
        .effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::SetTimer {
                window: Some(w),
                token,
                after: _,
                repeat: None,
            } if *w == window => Some(*token),
            _ => None,
        })
        .collect();
    assert!(
        !remove_tokens.is_empty(),
        "expected snackbar dismiss to schedule a timer for removal"
    );

    for token in remove_tokens {
        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });
    }

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let has_toast = ui.semantics_snapshot().is_some_and(|snapshot| {
        snapshot.nodes.iter().any(|node| {
            node.test_id
                .as_deref()
                .is_some_and(|id| id.starts_with("toast-entry-"))
        })
    });
    assert!(
        !has_toast,
        "expected snackbar to be removed after dismiss timer fires"
    );
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
fn time_picker_clock_dial_drag_updates_time() {
    use fret_ui_material3::{DockedTimePicker, TimePickerDisplayMode};
    use time::Time;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(420.0)),
    );

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");
    let time = app.models_mut().insert(selected_time);
    let time_for_render = time.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let picker = DockedTimePicker::new(time_for_render.clone())
                    .display_mode(TimePickerDisplayMode::Dial)
                    .test_id("time-picker-docked")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), picker)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let dial: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                if node.test_id.as_deref() == Some("time-picker-docked.clock-dial") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected time picker clock dial node in semantics snapshot");

    let dial_bounds = ui
        .debug_node_visual_bounds(dial)
        .expect("expected dial bounds");

    let center = Point::new(
        Px(dial_bounds.origin.x.0 + dial_bounds.size.width.0 * 0.5),
        Px(dial_bounds.origin.y.0 + dial_bounds.size.height.0 * 0.5),
    );
    let r = dial_bounds.size.width.0.min(dial_bounds.size.height.0) * 0.45;

    let start_at = Point::new(center.x, Px(center.y.0 - r));
    let drag_to = Point::new(Px(center.x.0 + r), center.y);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), start_at),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), drag_to),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), drag_to));

    let after = app.models().get_cloned(&time).unwrap_or(selected_time);
    assert_ne!(
        after, selected_time,
        "expected dial drag to update the time model"
    );
}

#[test]
fn time_picker_selector_keyboard_arrows_step_time() {
    use fret_ui_material3::{DockedTimePicker, TimePickerDisplayMode};
    use time::Time;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(420.0)),
    );

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");
    let time = app.models_mut().insert(selected_time);
    let time_for_render = time.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let picker = DockedTimePicker::new(time_for_render.clone())
                    .display_mode(TimePickerDisplayMode::Dial)
                    .test_id("time-picker-docked")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), picker)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let hour_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                if node.test_id.as_deref() == Some("time-picker-docked.hour-selector") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected hour selector node in semantics snapshot");

    ui.set_focus(Some(hour_node));
    assert_eq!(
        ui.focus(),
        Some(hour_node),
        "expected focus to be set to the hour input node"
    );
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowUp));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowUp));

    let after_hour = app.models().get_cloned(&time).expect("time model exists");
    assert_eq!(
        after_hour,
        Time::from_hms(10, 41, 0).expect("valid time"),
        "expected ArrowUp on hour selector to step +1 hour",
    );

    let minute_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                if node.test_id.as_deref() == Some("time-picker-docked.minute-selector") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected minute selector node in semantics snapshot");

    ui.set_focus(Some(minute_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let after_minute = app.models().get_cloned(&time).expect("time model exists");
    assert_eq!(
        after_minute,
        Time::from_hms(10, 40, 0).expect("valid time"),
        "expected ArrowDown on minute selector to step -1 minute",
    );
}

#[test]
fn time_picker_time_input_replaces_and_auto_advances_hour() {
    use fret_ui_material3::{DockedTimePicker, TimePickerDisplayMode};
    use time::Time;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(420.0)),
    );

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");
    let time = app.models_mut().insert(selected_time);
    let time_for_render = time.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let picker = DockedTimePicker::new(time_for_render.clone())
                    .display_mode(TimePickerDisplayMode::Input)
                    .test_id("time-picker-docked-input")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), picker)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let hour_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("time-picker-docked-input.input.hour"))
                    .then_some(node.id)
            })
        })
        .expect("expected time-picker-docked-input.input.hour in semantics snapshot");
    let minute_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("time-picker-docked-input.input.minute"))
                    .then_some(node.id)
            })
        })
        .expect("expected time-picker-docked-input.input.minute in semantics snapshot");

    ui.set_focus(Some(hour_node));

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Digit1));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Digit1));
    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("1".to_string()));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let after_first = app.models().get_cloned(&time).expect("time model exists");
    assert_eq!(
        after_first,
        Time::from_hms(1, 41, 0).expect("valid time"),
        "expected first digit to replace the existing hour",
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Digit2));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Digit2));
    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("2".to_string()));

    for token in drain_zero_delay_timer_tokens(&mut app, window) {
        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });
    }

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let after_second = app.models().get_cloned(&time).expect("time model exists");
    assert_eq!(
        after_second,
        Time::from_hms(0, 41, 0).expect("valid time"),
        "expected second digit to complete a two-digit hour (12 AM -> 00h in 24h time)",
    );
    assert_eq!(
        ui.focus(),
        Some(minute_node),
        "expected entering a two-digit hour to auto-advance focus to minutes",
    );
}

#[test]
fn time_picker_time_input_rejects_invalid_values_and_recovers() {
    use fret_ui_material3::{DockedTimePicker, TimePickerDisplayMode};
    use time::Time;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(420.0)),
    );

    let selected_time = Time::from_hms(9, 41, 0).expect("valid time");
    let time = app.models_mut().insert(selected_time);
    let time_for_render = time.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let picker = DockedTimePicker::new(time_for_render.clone())
                    .is_24h(true)
                    .display_mode(TimePickerDisplayMode::Input)
                    .test_id("time-picker-docked-input")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), picker)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let hour_node = semantics_node_id_by_test_id(&ui, "time-picker-docked-input.input.hour")
        .expect("expected time input hour field in semantics snapshot");
    ui.set_focus(Some(hour_node));

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Digit2));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Digit2));
    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("2".to_string()));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        app.models().get_cloned(&time).expect("time model exists"),
        Time::from_hms(2, 41, 0).expect("valid time"),
        "first valid hour digit should still update the committed time",
    );
    assert_eq!(
        semantics_invalid_by_test_id(&ui, "time-picker-docked-input.input.hour"),
        None,
        "single valid hour digit should not expose invalid semantics",
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Digit7));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Digit7));
    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("7".to_string()));

    for token in drain_zero_delay_timer_tokens(&mut app, window) {
        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });
    }

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        app.models().get_cloned(&time).expect("time model exists"),
        Time::from_hms(2, 41, 0).expect("valid time"),
        "invalid 24h hour input must not clamp or overwrite the committed time",
    );
    assert_eq!(
        semantics_invalid_by_test_id(&ui, "time-picker-docked-input.input.hour"),
        Some(SemanticsInvalid::True),
        "invalid hour input should expose aria-invalid semantics",
    );
    assert_eq!(
        semantics_label_by_test_id(&ui, "time-picker-docked-input.input.hour.supporting-text"),
        Some(String::from("Hour must be 0-23")),
        "invalid hour input should expose Material supporting error text",
    );
    assert_eq!(
        semantics_live_by_test_id(&ui, "time-picker-docked-input.input.hour.supporting-text"),
        Some((SemanticsLive::Polite, true)),
        "supporting error text should be a polite atomic live region",
    );

    let hour_node = semantics_node_id_by_test_id(&ui, "time-picker-docked-input.input.hour")
        .expect("expected time input hour field after invalid input");
    ui.set_focus(Some(hour_node));
    for _ in 0..2 {
        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Backspace));
        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Backspace));
    }
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Digit1));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Digit1));
    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("1".to_string()));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Digit2));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Digit2));
    ui.dispatch_event(&mut app, &mut services, &Event::TextInput("2".to_string()));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        app.models().get_cloned(&time).expect("time model exists"),
        Time::from_hms(12, 41, 0).expect("valid time"),
        "recovered valid hour input should update the committed time",
    );
    assert_eq!(
        semantics_invalid_by_test_id(&ui, "time-picker-docked-input.input.hour"),
        None,
        "valid recovery should clear invalid semantics",
    );
    assert_eq!(
        semantics_label_by_test_id(&ui, "time-picker-docked-input.input.hour.supporting-text"),
        Some(String::from("Hour")),
        "valid recovery should restore supporting text",
    );
}

#[test]
fn radio_selected_dot_is_centered_in_outline() {
    for scale_factor in [1.0, 1.25, 2.0] {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());

        let cfg = fret_ui_material3::tokens::v30::theme_config_with_colors(
            fret_ui_material3::tokens::v30::TypographyOptions::default(),
            fret_ui_material3::tokens::v30::ColorSchemeOptions::default(),
        );
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(300.0), Px(200.0)),
        );

        let selected = app.models_mut().insert(true);

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let child = fret_ui_material3::Radio::new(selected.clone())
                        .a11y_label("radio")
                        .into_element(cx);
                    vec![with_padding(cx, Px(37.0), child)]
                })
            };

        let mut found = None;
        for _ in 0..12 {
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, scale_factor);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, scale_factor);

            let mut outline: Option<Rect> = None;
            let mut dot: Option<Rect> = None;

            for op in scene.ops() {
                let SceneOp::Quad {
                    rect,
                    background,
                    border,
                    ..
                } = op
                else {
                    continue;
                };

                let border_any =
                    border.top.0 > 0.0 || border.right.0 > 0.0 || border.bottom.0 > 0.0;
                if border_any && paint_alpha(&background.paint) <= 0.01 {
                    if outline.is_none_or(|r| rect.size.width.0 < r.size.width.0 + 1e-3) {
                        outline = Some(*rect);
                    }
                    continue;
                }

                if border == &Edges::all(Px(0.0))
                    && paint_alpha(&background.paint) > 0.5
                    && rect.size.width.0 <= 12.0
                    && rect.size.height.0 <= 12.0
                    && dot.is_none_or(|r| rect.size.width.0 > r.size.width.0 + 1e-3)
                {
                    dot = Some(*rect);
                }
            }

            if let (Some(outline), Some(dot)) = (outline, dot) {
                found = Some((outline, dot));
                if dot.size.width.0 > 1.0 {
                    break;
                }
            }

            app.advance_frame();
        }

        let Some((outline, dot)) = found else {
            panic!("expected radio outline + selected dot quads in the scene");
        };

        let outline_cx = outline.origin.x.0 + outline.size.width.0 * 0.5;
        let outline_cy = outline.origin.y.0 + outline.size.height.0 * 0.5;
        let dot_cx = dot.origin.x.0 + dot.size.width.0 * 0.5;
        let dot_cy = dot.origin.y.0 + dot.size.height.0 * 0.5;

        assert!(
            (outline_cx - dot_cx).abs() < 0.75 && (outline_cy - dot_cy).abs() < 0.75,
            "dot center should match outline center (scale={scale_factor}): outline={outline:?} dot={dot:?}"
        );
    }
}

#[test]
fn radio_ripple_origin_tracks_pointer_down_position() {
    for scale_factor in [1.0, 1.25, 2.0] {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());

        let cfg = fret_ui_material3::tokens::v30::theme_config_with_colors(
            fret_ui_material3::tokens::v30::TypographyOptions::default(),
            fret_ui_material3::tokens::v30::ColorSchemeOptions::default(),
        );
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(300.0), Px(200.0)),
        );

        let selected = app.models_mut().insert(false);

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let child = fret_ui_material3::Radio::new(selected.clone())
                        .a11y_label("radio")
                        .into_element(cx);
                    vec![with_padding(cx, Px(37.0), child)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, scale_factor);

        let radio_bounds = find_first_bounds_with_size(&ui, root, 40.0, 40.0)
            .expect("expected a 40x40 radio chrome bounds");
        let press_at = Point::new(
            Px(radio_bounds.origin.x.0 + radio_bounds.size.width.0 * 0.5),
            Px(radio_bounds.origin.y.0 + radio_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut ripple_center: Option<Point> = None;
        let mut saw_ripple_clip = false;
        for _ in 0..4 {
            app.advance_frame();

            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, scale_factor);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, scale_factor);

            saw_ripple_clip |= scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::PushClipRRect { .. }));

            for op in scene.ops() {
                let SceneOp::Quad {
                    order,
                    rect: circle,
                    background,
                    border,
                    corner_radii,
                    ..
                } = op
                else {
                    continue;
                };

                if order != &DrawOrder(1) {
                    continue;
                }
                if border != &Edges::all(Px(0.0)) || paint_alpha(&background.paint) <= 0.01 {
                    continue;
                }
                if circle.size.width.0 <= 14.0 || circle.size.height.0 <= 14.0 {
                    continue;
                }

                let r = corner_radii.top_left.0;
                let r_ok = (corner_radii.top_right.0 - r).abs() < 1e-3
                    && (corner_radii.bottom_left.0 - r).abs() < 1e-3
                    && (corner_radii.bottom_right.0 - r).abs() < 1e-3;
                if !r_ok {
                    continue;
                }
                if (circle.size.width.0 * 0.5 - r).abs() > 1e-3
                    || (circle.size.height.0 * 0.5 - r).abs() > 1e-3
                {
                    continue;
                }

                ripple_center = Some(Point::new(
                    Px(circle.origin.x.0 + circle.size.width.0 * 0.5),
                    Px(circle.origin.y.0 + circle.size.height.0 * 0.5),
                ));
                break;
            }

            if ripple_center.is_some() {
                break;
            }
        }

        let Some(ripple_center) = ripple_center else {
            panic!("expected a ripple circle quad in the scene");
        };
        assert!(
            saw_ripple_clip,
            "expected ripple to be clipped to its state-layer bounds (scale={scale_factor})"
        );

        assert!(
            (ripple_center.x.0 - press_at.x.0).abs() < 0.75
                && (ripple_center.y.0 - press_at.y.0).abs() < 0.75,
            "expected ripple origin to match pointer down position (scale={scale_factor}): ripple_center={ripple_center:?} press_at={press_at:?}"
        );
    }
}

#[test]
fn switch_ripple_origin_tracks_pointer_down_position() {
    for scale_factor in [1.0, 1.25, 2.0] {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());

        let cfg = fret_ui_material3::tokens::v30::theme_config_with_colors(
            fret_ui_material3::tokens::v30::TypographyOptions::default(),
            fret_ui_material3::tokens::v30::ColorSchemeOptions::default(),
        );
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

        let theme = Theme::global(&app);
        let track_width = theme
            .metric_by_key("md.comp.switch.track.width")
            .unwrap_or(Px(52.0));
        let state_layer = theme
            .metric_by_key("md.comp.switch.state-layer.size")
            .unwrap_or(Px(40.0));

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(300.0), Px(200.0)),
        );

        let selected = app.models_mut().insert(false);

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let child = fret_ui_material3::Switch::new(selected.clone())
                        .a11y_label("switch")
                        .into_element(cx);
                    vec![with_padding(cx, Px(37.0), child)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, scale_factor);

        let switch_bounds = find_first_bounds_with_size(&ui, root, track_width.0, state_layer.0)
            .expect("expected a switch outer bounds");
        let press_at = Point::new(
            Px(switch_bounds.origin.x.0 + switch_bounds.size.width.0 * 0.5),
            Px(switch_bounds.origin.y.0 + switch_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut ripple_center: Option<Point> = None;
        let mut saw_ripple_clip = false;
        for _ in 0..4 {
            app.advance_frame();

            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, scale_factor);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, scale_factor);

            saw_ripple_clip |= scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::PushClipRRect { .. }));

            for op in scene.ops() {
                let SceneOp::Quad {
                    order,
                    rect: circle,
                    background,
                    border,
                    corner_radii,
                    ..
                } = op
                else {
                    continue;
                };

                if order != &DrawOrder(1) {
                    continue;
                }
                if border != &Edges::all(Px(0.0)) || paint_alpha(&background.paint) <= 0.01 {
                    continue;
                }
                if circle.size.width.0 <= 14.0 || circle.size.height.0 <= 14.0 {
                    continue;
                }

                let r = corner_radii.top_left.0;
                let r_ok = (corner_radii.top_right.0 - r).abs() < 1e-3
                    && (corner_radii.bottom_left.0 - r).abs() < 1e-3
                    && (corner_radii.bottom_right.0 - r).abs() < 1e-3;
                if !r_ok {
                    continue;
                }
                if (circle.size.width.0 * 0.5 - r).abs() > 1e-3
                    || (circle.size.height.0 * 0.5 - r).abs() > 1e-3
                {
                    continue;
                }

                ripple_center = Some(Point::new(
                    Px(circle.origin.x.0 + circle.size.width.0 * 0.5),
                    Px(circle.origin.y.0 + circle.size.height.0 * 0.5),
                ));
                break;
            }

            if ripple_center.is_some() {
                break;
            }
        }

        let Some(ripple_center) = ripple_center else {
            panic!("expected a ripple circle quad in the scene");
        };
        assert!(
            saw_ripple_clip,
            "expected ripple to be clipped to its state-layer bounds (scale={scale_factor})"
        );

        assert!(
            (ripple_center.x.0 - press_at.x.0).abs() < 0.75
                && (ripple_center.y.0 - press_at.y.0).abs() < 0.75,
            "expected ripple origin to match pointer down position (scale={scale_factor}): ripple_center={ripple_center:?} press_at={press_at:?}"
        );
    }
}

#[test]
fn switch_keyboard_ripple_origin_ignores_stale_pointer_down() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());

    let cfg = fret_ui_material3::tokens::v30::theme_config_with_colors(
        fret_ui_material3::tokens::v30::TypographyOptions::default(),
        fret_ui_material3::tokens::v30::ColorSchemeOptions::default(),
    );
    Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

    let theme = Theme::global(&app);
    let track_width = theme
        .metric_by_key("md.comp.switch.track.width")
        .unwrap_or(Px(52.0));
    let track_height = theme
        .metric_by_key("md.comp.switch.track.height")
        .unwrap_or(Px(32.0));
    let state_layer = theme
        .metric_by_key("md.comp.switch.state-layer.size")
        .unwrap_or(Px(40.0));

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let selected = app.models_mut().insert(false);

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let child = fret_ui_material3::Switch::new(selected.clone())
                .a11y_label("switch")
                .into_element(cx);
            vec![with_padding(cx, Px(37.0), child)]
        })
    };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let track_bounds = find_first_bounds_with_size(&ui, root, track_width.0, track_height.0)
        .expect("expected switch track bounds");
    let old_press_at = Point::new(
        Px(track_bounds.origin.x.0 + 2.0),
        Px(track_bounds.origin.y.0 + 2.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), old_press_at),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_up(PointerId(1), old_press_at),
    );

    // Let the pointer-started ripple fully finish so we don't confuse it with the keyboard ripple.
    for _ in 0..120 {
        app.advance_frame();
        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    }

    // Ensure keyboard events are delivered by explicitly focusing the switch node via semantics.
    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let focus: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                if node.label.as_deref() == Some("switch") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected switch node in semantics snapshot");
    ui.set_focus(Some(focus));

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Space));
    assert!(
        fret_ui::input_modality::is_keyboard(&mut app, Some(window)),
        "expected keydown to switch input modality to keyboard"
    );

    let mut expected_center: Option<Point> = None;
    let mut ripple_center: Option<Point> = None;
    for attempt in 0..6 {
        if attempt > 0 {
            app.advance_frame();
        }

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        // Scene ops may contain transforms. Always compare centers in the same coordinate space
        // by applying the transform stack while scanning.
        let mut transform = Transform2D::IDENTITY;
        let mut transform_stack: Vec<Transform2D> = Vec::new();
        let mut clip_stack: Vec<Option<Point>> = Vec::new();

        for op in scene.ops() {
            match *op {
                SceneOp::PushTransform { transform: next } => {
                    transform_stack.push(transform);
                    transform = transform.compose(next);
                }
                SceneOp::PopTransform => {
                    transform = transform_stack.pop().unwrap_or(Transform2D::IDENTITY);
                }
                SceneOp::PushClipRect { .. } => {
                    clip_stack.push(None);
                }
                SceneOp::PushClipRRect { rect, .. } => {
                    let is_state_layer = (rect.size.width.0 - state_layer.0).abs() < 0.25
                        && (rect.size.height.0 - state_layer.0).abs() < 0.25;
                    let center = Point::new(
                        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
                        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
                    );
                    clip_stack.push(is_state_layer.then_some(transform.apply_point(center)));
                }
                SceneOp::PopClip => {
                    clip_stack.pop();
                }
                SceneOp::Quad {
                    order,
                    rect,
                    background,
                    border,
                    corner_radii,
                    ..
                } => {
                    let Some(center_expected) = clip_stack.iter().rev().find_map(|center| *center)
                    else {
                        continue;
                    };
                    if order != DrawOrder(1)
                        || border != Edges::all(Px(0.0))
                        || paint_alpha(&background.paint) <= 0.001
                        || paint_alpha(&background.paint) >= 0.9
                        || (rect.size.width.0 - rect.size.height.0).abs() >= 0.25
                    {
                        continue;
                    }

                    let r = corner_radii.top_left.0;
                    let r_ok = (corner_radii.top_right.0 - r).abs() < 0.25
                        && (corner_radii.bottom_left.0 - r).abs() < 0.25
                        && (corner_radii.bottom_right.0 - r).abs() < 0.25;
                    if !r_ok {
                        continue;
                    }
                    if (rect.size.width.0 * 0.5 - r).abs() > 0.25
                        || (rect.size.height.0 * 0.5 - r).abs() > 0.25
                    {
                        continue;
                    }

                    let center_ripple = Point::new(
                        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
                        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
                    );
                    expected_center = Some(center_expected);
                    ripple_center = Some(transform.apply_point(center_ripple));
                    break;
                }
                _ => {}
            }
        }

        if expected_center.is_some() && ripple_center.is_some() {
            break;
        }
    }

    let expected_center = expected_center.expect("expected state-layer bounds quad");
    let ripple_center = ripple_center.expect("expected a ripple quad");

    assert!(
        (ripple_center.x.0 - expected_center.x.0).abs() < 0.75
            && (ripple_center.y.0 - expected_center.y.0).abs() < 0.75,
        "expected keyboard ripple origin to be centered in the state-layer bounds: ripple_center={ripple_center:?} expected_center={expected_center:?}"
    );
    assert!(
        (ripple_center.x.0 - old_press_at.x.0).abs() > 2.0
            || (ripple_center.y.0 - old_press_at.y.0).abs() > 2.0,
        "expected keyboard ripple origin to ignore stale pointer down: ripple_center={ripple_center:?} old_press_at={old_press_at:?}"
    );

    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Space));
}

#[test]
fn switch_ripple_holds_for_minimum_press_duration_before_fade() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());

    let cfg = fret_ui_material3::tokens::v30::theme_config_with_colors(
        fret_ui_material3::tokens::v30::TypographyOptions::default(),
        fret_ui_material3::tokens::v30::ColorSchemeOptions::default(),
    );
    Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

    let theme = Theme::global(&app);
    let min_frames = fret_ui_material3::motion::ms_to_frames(225);
    let track_width = theme
        .metric_by_key("md.comp.switch.track.width")
        .unwrap_or(Px(52.0));
    let track_height = theme
        .metric_by_key("md.comp.switch.track.height")
        .unwrap_or(Px(32.0));

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(200.0)),
    );

    let selected = app.models_mut().insert(false);

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            vec![
                fret_ui_material3::Switch::new(selected.clone())
                    .a11y_label("switch")
                    .into_element(cx),
            ]
        })
    };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Ensure the pressable is focused so it responds to keyboard events.
    let _ = find_first_bounds_with_size(&ui, root, track_width.0, track_height.0)
        .expect("expected switch track bounds");
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let focus: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                if node.label.as_deref() == Some("switch") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected switch node in semantics snapshot");
    ui.set_focus(Some(focus));

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Space));

    // Ensure the ripple has started (pressed rising observed).
    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Space));

    let mut held_alpha: Option<f32> = None;
    let mut saw_fade = false;
    for frame_offset in 0..(min_frames.saturating_add(3)) {
        if frame_offset > 0 {
            app.advance_frame();
        }

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let ripple_alpha = scene
            .ops()
            .iter()
            .filter_map(|op| match op {
                SceneOp::Quad {
                    order,
                    background,
                    border,
                    ..
                } if *order == DrawOrder(1) && *border == Edges::all(Px(0.0)) => {
                    Some(paint_alpha(&background.paint))
                }
                _ => None,
            })
            .next()
            .unwrap_or(0.0);

        if held_alpha.is_none() && ripple_alpha > 0.001 {
            held_alpha = Some(ripple_alpha);
        }
        let Some(held_alpha) = held_alpha else {
            continue;
        };

        if frame_offset < min_frames {
            assert!(
                (ripple_alpha - held_alpha).abs() < 1e-3,
                "expected ripple alpha to hold until min press duration: offset={frame_offset} ripple_alpha={ripple_alpha} held_alpha={held_alpha}"
            );
        }

        if frame_offset >= min_frames {
            assert!(
                ripple_alpha < held_alpha - 1e-4,
                "expected ripple alpha to start fading after min press duration: offset={frame_offset} ripple_alpha={ripple_alpha} held_alpha={held_alpha} min_frames={min_frames}"
            );
            saw_fade = true;
            break;
        }
    }

    assert!(
        held_alpha.is_some(),
        "expected to observe a keyboard ripple"
    );
    assert!(saw_fade, "expected the ripple to start fading");
}

#[test]
fn tabs_pressed_scene_structure_is_stable() {
    use std::sync::Arc;

    use fret_ui_material3::{TabItem, Tabs};

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
            Size::new(Px(360.0), Px(240.0)),
        );

        let selected = app.models_mut().insert(Arc::<str>::from("b"));
        let items = vec![
            TabItem::new("a", "A").test_id("tab-a"),
            TabItem::new("b", "B").test_id("tab-b"),
            TabItem::new("c", "C").test_id("tab-c"),
        ];

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let tabs = Tabs::new(selected.clone())
                        .items(items.clone())
                        .a11y_label("tabs")
                        .into_element(cx);
                    vec![with_padding(cx, Px(24.0), tabs)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let tab_b: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("tab-b") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .expect("expected tab-b in semantics snapshot");
        let tab_b_bounds = ui
            .debug_node_visual_bounds(tab_b)
            .expect("expected tab-b visual bounds");
        let press_at = Point::new(
            Px(tab_b_bounds.origin.x.0 + tab_b_bounds.size.width.0 * 0.5),
            Px(tab_b_bounds.origin.y.0 + tab_b_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut baseline_structure: Option<Vec<SceneSig>> = None;
        let mut prev_quads: Option<Vec<QuadGeomSig>> = None;
        let mut stable_quads_count: usize = 0;
        let settle_probe_start = 12;
        for frame in 0..48 {
            app.advance_frame();
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

            if (2..7).contains(&frame) {
                let sig = scene_signature(&scene);
                if let Some(prev) = baseline_structure.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Tabs to keep a stable scene structure while pressed ({label})"
                    );
                } else {
                    baseline_structure = Some(sig);
                }
            }

            if frame >= settle_probe_start {
                let sig = scene_quad_geometry_signature(&scene);
                match prev_quads.as_ref() {
                    None => {
                        stable_quads_count = 1;
                    }
                    Some(prev) if sig == *prev => {
                        stable_quads_count += 1;
                    }
                    Some(_) => {
                        stable_quads_count = 1;
                    }
                }
                prev_quads = Some(sig);
            }
        }

        assert!(
            stable_quads_count >= 6,
            "expected Tabs quad geometry to stabilize after animations settle ({label})"
        );

        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    }
}

#[test]
fn icon_button_pressed_scene_structure_is_stable() {
    use fret_icons::ids;
    use fret_ui_material3::IconButton;

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
            Size::new(Px(320.0), Px(240.0)),
        );

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let button = IconButton::new(ids::ui::CHECK)
                        .a11y_label("icon button")
                        .test_id("icon-button")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), button)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let button_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("icon-button") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .expect("expected icon-button in semantics snapshot");
        let button_bounds = ui
            .debug_node_visual_bounds(button_node)
            .expect("expected icon-button visual bounds");
        let press_at = Point::new(
            Px(button_bounds.origin.x.0 + button_bounds.size.width.0 * 0.5),
            Px(button_bounds.origin.y.0 + button_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut baseline_structure: Option<Vec<SceneSig>> = None;
        let mut baseline_quads: Option<Vec<QuadGeomSig>> = None;
        for frame in 0..24 {
            app.advance_frame();
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

            if (2..7).contains(&frame) {
                let sig = scene_signature(&scene);
                if let Some(prev) = baseline_structure.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected IconButton to keep a stable scene structure while pressed ({label})"
                    );
                } else {
                    baseline_structure = Some(sig);
                }
            }

            if frame >= 16 {
                let sig = scene_quad_geometry_signature(&scene);
                if let Some(prev) = baseline_quads.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected IconButton to keep stable quad geometry after animations settle ({label})"
                    );
                } else {
                    baseline_quads = Some(sig);
                }
            }
        }

        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    }
}

#[test]
fn icon_toggle_button_semantics_role_and_checked_state_are_stable() {
    use fret_icons::ids;
    use fret_ui_material3::IconToggleButton;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Dark, DynamicVariant::Expressive);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let checked_model = app.models.insert(false);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(240.0)),
    );

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let button = IconToggleButton::new(checked_model.clone(), ids::ui::CHECK)
                .a11y_label("icon toggle button")
                .test_id("icon-toggle-button")
                .into_element(cx);
            vec![with_padding(cx, Px(32.0), button)]
        })
    };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let (button_node, button_bounds, initial_checked) = {
        let snapshot = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot");
        let node = snapshot
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some("icon-toggle-button"))
            .expect("expected icon-toggle-button in semantics snapshot");
        assert_eq!(
            node.role,
            fret_core::SemanticsRole::Checkbox,
            "expected IconToggleButton semantics role=Checkbox"
        );
        assert!(
            !node.flags.selected,
            "expected IconToggleButton not to set `selected`"
        );
        assert_eq!(
            node.flags.checked,
            Some(false),
            "expected IconToggleButton checked=false initially"
        );
        assert_eq!(
            node.flags.checked_state,
            Some(fret_core::SemanticsCheckedState::False),
            "expected IconToggleButton checked_state=false initially"
        );
        let bounds = ui
            .debug_node_visual_bounds(node.id)
            .expect("expected icon-toggle-button visual bounds");
        (node.id, bounds, node.flags.checked)
    };

    let press_at = Point::new(
        Px(button_bounds.origin.x.0 + button_bounds.size.width.0 * 0.5),
        Px(button_bounds.origin.y.0 + button_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), press_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));

    app.advance_frame();
    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot");
    let node = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("icon-toggle-button"))
        .expect("expected icon-toggle-button in semantics snapshot");
    assert_eq!(
        node.role,
        fret_core::SemanticsRole::Checkbox,
        "expected IconToggleButton semantics role=Checkbox after toggle"
    );
    assert!(
        !node.flags.selected,
        "expected IconToggleButton not to set `selected` after toggle"
    );
    assert_eq!(
        initial_checked,
        Some(false),
        "expected initial checked state to be false"
    );
    assert_eq!(
        node.flags.checked,
        Some(true),
        "expected IconToggleButton checked=true after click"
    );
    assert_eq!(
        node.flags.checked_state,
        Some(fret_core::SemanticsCheckedState::True),
        "expected IconToggleButton checked_state=true after click"
    );

    // Sanity: the visual node should still be queryable.
    ui.debug_node_visual_bounds(button_node)
        .expect("expected icon-toggle-button visual bounds after click");
}

#[test]
fn chips_export_checked_state_for_selected_semantics() {
    use fret_ui_material3::{FilterChip, InputChip};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Dark, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let filter_selected = app.models.insert(true);
    let input_unselected = app.models.insert(false);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(180.0)),
    );

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let mut props = fret_ui::element::FlexProps::default();
            props.direction = fret_core::Axis::Vertical;
            props.gap = fret_ui::element::SpacingLength::Px(Px(8.0));
            let chips = cx.flex(props, |cx| {
                vec![
                    FilterChip::new(filter_selected.clone(), "Filter")
                        .test_id("filter-chip-selected")
                        .into_element(cx),
                    InputChip::new(input_unselected.clone(), "Input")
                        .test_id("input-chip-unselected")
                        .into_element(cx),
                ]
            });
            vec![with_padding(cx, Px(24.0), chips)]
        })
    };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot");
    let filter = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("filter-chip-selected"))
        .expect("expected filter chip in semantics snapshot");
    assert_eq!(filter.role, fret_core::SemanticsRole::Checkbox);
    assert_eq!(filter.flags.checked, Some(true));
    assert_eq!(
        filter.flags.checked_state,
        Some(fret_core::SemanticsCheckedState::True)
    );

    let input = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("input-chip-unselected"))
        .expect("expected input chip in semantics snapshot");
    assert_eq!(input.role, fret_core::SemanticsRole::Checkbox);
    assert_eq!(input.flags.checked, Some(false));
    assert_eq!(
        input.flags.checked_state,
        Some(fret_core::SemanticsCheckedState::False)
    );
}

#[test]
fn icon_toggle_button_checked_transition_scene_structure_is_stable() {
    use fret_icons::ids;
    use fret_ui_material3::{
        IconToggleButton, MaterialDesignVariant, with_material_design_variant,
    };

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Dark, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let checked_model = app.models.insert(false);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(240.0)),
    );

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            with_material_design_variant(cx, MaterialDesignVariant::Expressive, |cx| {
                let button = IconToggleButton::new(checked_model.clone(), ids::ui::CHECK)
                    .a11y_label("icon toggle button")
                    .test_id("icon-toggle-button")
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), button)]
            })
        })
    };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let button_bounds = {
        let button_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("icon-toggle-button") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .expect("expected icon-toggle-button in semantics snapshot");

        ui.debug_node_visual_bounds(button_node)
            .expect("expected icon-toggle-button visual bounds")
    };

    let press_at = Point::new(
        Px(button_bounds.origin.x.0 + button_bounds.size.width.0 * 0.5),
        Px(button_bounds.origin.y.0 + button_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), press_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));

    let mut baseline_structure: Option<Vec<SceneSig>> = None;
    let mut baseline_quads: Option<Vec<QuadGeomSig>> = None;
    let mut baseline_clip_corners: Option<(i32, i32, i32, i32)> = None;
    let mut saw_corner_change = false;
    for frame in 0..24 {
        app.advance_frame();
        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        // Ignore the first couple frames: focus + modality may settle after the click.
        if (2..7).contains(&frame) {
            let sig = scene_signature(&scene);
            if let Some(prev) = baseline_structure.as_ref() {
                assert_eq!(
                    sig, *prev,
                    "expected IconToggleButton to keep a stable scene structure while checked corner morph is active"
                );
            } else {
                baseline_structure = Some(sig);
            }

            let corners = scene
                .ops()
                .iter()
                .find_map(|op| match op {
                    SceneOp::PushClipRRect { corner_radii, .. } => Some(*corner_radii),
                    _ => None,
                })
                .expect("expected PushClipRRect while rendering IconToggleButton");

            let sig = (
                ((corners.top_left.0 * 10.0).round()) as i32,
                ((corners.top_right.0 * 10.0).round()) as i32,
                ((corners.bottom_right.0 * 10.0).round()) as i32,
                ((corners.bottom_left.0 * 10.0).round()) as i32,
            );

            match baseline_clip_corners {
                None => baseline_clip_corners = Some(sig),
                Some(prev) if sig != prev => saw_corner_change = true,
                Some(_) => {}
            }
        }

        if frame >= 16 {
            let geom = scene_quad_geometry_signature(&scene);
            if let Some(prev) = baseline_quads.as_ref() {
                assert_eq!(
                    geom, *prev,
                    "expected IconToggleButton to keep stable quad geometry after checked morph settles"
                );
            } else {
                baseline_quads = Some(geom);
            }
        }
    }

    assert!(
        saw_corner_change,
        "expected IconToggleButton quad corner radii to change during checked morph"
    );
}

#[test]
fn switch_pressed_scene_structure_is_stable() {
    use fret_ui_material3::Switch;

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
            Size::new(Px(320.0), Px(240.0)),
        );

        let selected = app.models_mut().insert(false);
        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let switch = Switch::new(selected.clone())
                        .a11y_label("switch")
                        .test_id("switch")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), switch)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let switch_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("switch") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .expect("expected switch in semantics snapshot");
        let switch_bounds = ui
            .debug_node_visual_bounds(switch_node)
            .expect("expected switch visual bounds");
        let press_at = Point::new(
            Px(switch_bounds.origin.x.0 + switch_bounds.size.width.0 * 0.5),
            Px(switch_bounds.origin.y.0 + switch_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut baseline_structure: Option<Vec<SceneSig>> = None;
        let mut baseline_quads: Option<Vec<QuadGeomSig>> = None;
        for frame in 0..24 {
            app.advance_frame();
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

            if (2..7).contains(&frame) {
                let sig = scene_signature(&scene);
                if let Some(prev) = baseline_structure.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Switch to keep a stable scene structure while pressed ({label})"
                    );
                } else {
                    baseline_structure = Some(sig);
                }
            }

            if frame >= 16 {
                let sig = scene_quad_geometry_signature(&scene);
                if let Some(prev) = baseline_quads.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Switch to keep stable quad geometry after animations settle ({label})"
                    );
                } else {
                    baseline_quads = Some(sig);
                }
            }
        }

        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    }
}

#[test]
fn switch_icons_pressed_scene_structure_is_stable() {
    use fret_ui_material3::Switch;

    let schemes = [
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

    let scenarios = [
        ("icons_both.unselected", false, false),
        ("icons_both.selected", true, false),
        ("icons_selected_only.unselected", false, true),
        ("icons_selected_only.selected", true, true),
    ];

    for (mode, variant, label) in schemes {
        for (scenario, initial_selected, selected_only) in scenarios {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices;
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(320.0), Px(240.0)),
            );

            let selected = app.models_mut().insert(initial_selected);
            let test_id = "switch-icons";
            let render = |ui: &mut UiTree<TestHost>,
                          app: &mut TestHost,
                          services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let mut switch = Switch::new(selected.clone())
                        .a11y_label("switch")
                        .test_id(test_id);
                    if selected_only {
                        switch = switch.show_only_selected_icon(true);
                    } else {
                        switch = switch.icons(true);
                    }

                    let el = switch.into_element(cx);
                    vec![with_padding(cx, Px(32.0), el)]
                })
            };

            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let switch_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        if node.test_id.as_deref() == Some(test_id) {
                            Some(node.id)
                        } else {
                            None
                        }
                    })
                })
                .unwrap_or_else(|| panic!("expected switch semantics node ({label}, {scenario})"));
            let switch_bounds = ui
                .debug_node_visual_bounds(switch_node)
                .unwrap_or_else(|| panic!("expected switch visual bounds ({label}, {scenario})"));
            let press_at = Point::new(
                Px(switch_bounds.origin.x.0 + switch_bounds.size.width.0 * 0.5),
                Px(switch_bounds.origin.y.0 + switch_bounds.size.height.0 * 0.5),
            );

            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_down(PointerId(1), press_at),
            );

            let mut baseline_structure: Option<Vec<SceneSig>> = None;
            let mut baseline_quads: Option<Vec<QuadGeomSig>> = None;
            for frame in 0..24 {
                app.advance_frame();
                let root = render(&mut ui, &mut app, &mut services);
                ui.set_root(root);
                ui.layout_all(&mut app, &mut services, bounds, 1.0);

                let mut scene = Scene::default();
                ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

                if (2..7).contains(&frame) {
                    let sig = scene_signature(&scene);
                    if let Some(prev) = baseline_structure.as_ref() {
                        assert_eq!(
                            sig, *prev,
                            "expected Switch icons scene structure to be stable while pressed ({label}, {scenario})"
                        );
                    } else {
                        baseline_structure = Some(sig);
                    }
                }

                if frame >= 16 {
                    let sig = scene_quad_geometry_signature(&scene);
                    if let Some(prev) = baseline_quads.as_ref() {
                        assert_eq!(
                            sig, *prev,
                            "expected Switch icons quad geometry to be stable after animations settle ({label}, {scenario})"
                        );
                    } else {
                        baseline_quads = Some(sig);
                    }
                }
            }

            ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
        }
    }
}

#[test]
fn switch_selected_only_icon_persists_during_toggle_animation() {
    use fret_core::SceneOp;
    use fret_ui_material3::Switch;

    fn svg_icon_op_count(scene: &Scene) -> usize {
        scene
            .ops()
            .iter()
            .filter(|op| matches!(op, SceneOp::SvgMaskIcon { .. } | SceneOp::SvgImage { .. }))
            .count()
    }

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
            Size::new(Px(320.0), Px(240.0)),
        );

        let selected = app.models_mut().insert(true);
        let test_id = "switch-selected-only-icon";
        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let el = Switch::new(selected.clone())
                        .show_only_selected_icon(true)
                        .a11y_label("switch")
                        .test_id(test_id)
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), el)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert!(
            svg_icon_op_count(&scene) >= 1,
            "expected selected-only icon to be painted when selected ({label})"
        );

        let switch_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some(test_id) {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected switch semantics node ({label})"));
        let switch_bounds = ui
            .debug_node_visual_bounds(switch_node)
            .unwrap_or_else(|| panic!("expected switch visual bounds ({label})"));
        let press_at = Point::new(
            Px(switch_bounds.origin.x.0 + switch_bounds.size.width.0 * 0.5),
            Px(switch_bounds.origin.y.0 + switch_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );
        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));

        let mut saw_icon_gone_after_settle = false;
        for frame in 0..120usize {
            app.advance_frame();
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
            let count = svg_icon_op_count(&scene);

            if frame < 2 {
                assert!(
                    count >= 1,
                    "expected selected-only icon to persist while toggle animation starts ({label}, frame={frame}, count={count})"
                );
            }

            if frame >= 60 && count == 0 {
                saw_icon_gone_after_settle = true;
            }
        }

        assert!(
            saw_icon_gone_after_settle,
            "expected selected-only icon to be removed after toggle settles ({label})"
        );
    }
}

#[test]
fn checkbox_pressed_scene_structure_is_stable() {
    use fret_ui_material3::Checkbox;

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
            Size::new(Px(320.0), Px(240.0)),
        );

        let checked = app.models_mut().insert(false);
        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let checkbox = Checkbox::new(checked.clone())
                        .a11y_label("checkbox")
                        .test_id("checkbox")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), checkbox)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let checkbox_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("checkbox") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .expect("expected checkbox in semantics snapshot");
        let checkbox_bounds = ui
            .debug_node_visual_bounds(checkbox_node)
            .expect("expected checkbox visual bounds");
        let press_at = Point::new(
            Px(checkbox_bounds.origin.x.0 + checkbox_bounds.size.width.0 * 0.5),
            Px(checkbox_bounds.origin.y.0 + checkbox_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut baseline_structure: Option<Vec<SceneSig>> = None;
        let mut baseline_quads: Option<Vec<QuadGeomSig>> = None;
        for frame in 0..24 {
            app.advance_frame();
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

            if (2..7).contains(&frame) {
                let sig = scene_signature(&scene);
                if let Some(prev) = baseline_structure.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Checkbox to keep a stable scene structure while pressed ({label})"
                    );
                } else {
                    baseline_structure = Some(sig);
                }
            }

            if frame >= 16 {
                let sig = scene_quad_geometry_signature(&scene);
                if let Some(prev) = baseline_quads.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Checkbox to keep stable quad geometry after animations settle ({label})"
                    );
                } else {
                    baseline_quads = Some(sig);
                }
            }
        }

        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    }
}

#[test]
fn checkbox_tristate_semantics_and_toggle_outcomes() {
    use fret_ui_material3::Checkbox;

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Dark, DynamicVariant::Expressive);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(240.0)),
    );

    let checked = app.models_mut().insert(None::<bool>);
    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let checkbox = Checkbox::new_optional(checked.clone())
                .a11y_label("checkbox")
                .test_id("checkbox-tristate")
                .into_element(cx);
            vec![with_padding(cx, Px(32.0), checkbox)]
        })
    };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("checkbox-tristate"))
        .expect("expected tristate checkbox in semantics snapshot");
    assert_eq!(
        node.flags.checked, None,
        "expected indeterminate checkbox to map to checked: None"
    );

    let checkbox_bounds = ui
        .debug_node_visual_bounds(node.id)
        .expect("expected checkbox visual bounds");
    let press_at = Point::new(
        Px(checkbox_bounds.origin.x.0 + checkbox_bounds.size.width.0 * 0.5),
        Px(checkbox_bounds.origin.y.0 + checkbox_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), press_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));

    assert_eq!(
        app.models().get_cloned(&checked),
        Some(Some(true)),
        "expected tristate checkbox to toggle indeterminate -> checked"
    );

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("checkbox-tristate"))
        .expect("expected tristate checkbox in semantics snapshot");
    assert_eq!(node.flags.checked, Some(true));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), press_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));

    assert_eq!(
        app.models().get_cloned(&checked),
        Some(Some(false)),
        "expected tristate checkbox to toggle checked -> unchecked"
    );
}

#[test]
fn menu_pressed_scene_structure_is_stable() {
    use fret_ui_material3::menu::{Menu, MenuEntry, MenuItem};

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
            Size::new(Px(360.0), Px(260.0)),
        );

        let entries = vec![
            MenuEntry::Item(MenuItem::new("A").test_id("menu-item-a")),
            MenuEntry::Item(MenuItem::new("B").test_id("menu-item-b")),
            MenuEntry::Item(MenuItem::new("C").test_id("menu-item-c")),
        ];

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let menu = Menu::new()
                        .entries(entries.clone())
                        .a11y_label("menu")
                        .test_id("menu")
                        .into_element(cx);
                    vec![with_padding(cx, Px(24.0), menu)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let item_b: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("menu-item-b") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .expect("expected menu-item-b in semantics snapshot");
        let item_bounds = ui
            .debug_node_visual_bounds(item_b)
            .expect("expected menu-item-b visual bounds");
        let press_at = Point::new(
            Px(item_bounds.origin.x.0 + item_bounds.size.width.0 * 0.5),
            Px(item_bounds.origin.y.0 + item_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut baseline_structure: Option<Vec<SceneSig>> = None;
        let mut baseline_quads: Option<Vec<QuadGeomSig>> = None;
        for frame in 0..24 {
            app.advance_frame();
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

            if (2..7).contains(&frame) {
                let sig = scene_signature(&scene);
                if let Some(prev) = baseline_structure.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Menu to keep a stable scene structure while pressed ({label})"
                    );
                } else {
                    baseline_structure = Some(sig);
                }
            }

            if frame >= 16 {
                let sig = scene_quad_geometry_signature(&scene);
                if let Some(prev) = baseline_quads.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Menu to keep stable quad geometry after animations settle ({label})"
                    );
                } else {
                    baseline_quads = Some(sig);
                }
            }
        }

        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    }
}

#[test]
fn menu_style_overrides_apply_to_container_and_label() {
    use fret_core::Color;
    use fret_ui_kit::{ColorRef, WidgetStateProperty};
    use fret_ui_material3::menu::{Menu, MenuEntry, MenuItem, MenuStyle};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(260.0)),
    );

    let override_bg = Color {
        r: 0.9,
        g: 0.1,
        b: 0.2,
        a: 1.0,
    };
    let override_label = Color {
        r: 0.1,
        g: 0.8,
        b: 0.3,
        a: 1.0,
    };

    let style = MenuStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(override_bg))))
        .item_label_color(WidgetStateProperty::new(Some(ColorRef::Color(
            override_label,
        ))));

    let entries = vec![
        MenuEntry::Item(MenuItem::new("A").test_id("menu-item-a")),
        MenuEntry::Item(MenuItem::new("B").test_id("menu-item-b")),
    ];

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "root",
        |cx| {
            let menu = Menu::new()
                .entries(entries.clone())
                .a11y_label("menu")
                .test_id("menu")
                .style(style.clone())
                .into_element(cx);
            vec![with_padding(cx, Px(24.0), menu)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scene.ops().iter().any(|op| {
            matches!(
                op,
                SceneOp::Quad { background, .. } if background.paint == fret_core::Paint::Solid(override_bg)
            )
        }),
        "expected MenuStyle.container_background to affect at least one quad background"
    );
    assert!(
        scene
            .ops()
            .iter()
            .any(|op| {
                matches!(
                    op,
                    SceneOp::Text { paint, .. } if paint.paint == fret_core::Paint::Solid(override_label)
                )
            }),
        "expected MenuStyle.item_label_color to affect at least one text draw op"
    );
}

#[test]
fn dialog_focus_is_contained_and_restored_across_schemes() {
    use fret_ui_material3::{Button, Dialog, DialogAction};

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
            Size::new(Px(420.0), Px(320.0)),
        );

        let open = app.models_mut().insert(false);

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let dialog = Dialog::new(open.clone())
                        .headline("Dialog")
                        .supporting_text("Body")
                        .actions(vec![DialogAction::new("OK").test_id("dialog-ok")])
                        .test_id("dialog")
                        .into_element(
                            cx,
                            |cx| {
                                let trigger = Button::new("Open dialog")
                                    .test_id("dialog-trigger")
                                    .into_element(cx);
                                with_padding(cx, Px(24.0), trigger)
                            },
                            |_cx| Vec::new(),
                        );
                    vec![dialog]
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
                    if node.test_id.as_deref() == Some("dialog-trigger") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected dialog-trigger in semantics snapshot ({label})"));
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
            "expected modal barrier root while dialog is open ({label})"
        );
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|node| node.test_id.as_deref() == Some("dialog.scrim")),
            "expected dialog scrim node while dialog is open ({label})"
        );
        assert_ne!(
            ui.focus(),
            Some(trigger_node),
            "expected focus to move into dialog layer while open ({label})"
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
        for _ in 0..40 {
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
            "expected dialog barrier to unmount after close transition ({label})"
        );
    }
}

#[test]
fn dialog_style_overrides_apply_to_container_and_text() {
    use fret_core::Color;
    use fret_ui_kit::{ColorRef, WidgetStateProperty};
    use fret_ui_material3::{Button, Dialog, DialogAction, DialogStyle};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(320.0)),
    );

    let open = app.models_mut().insert(true);

    let override_bg = Color {
        r: 0.2,
        g: 0.2,
        b: 0.9,
        a: 1.0,
    };
    let override_headline = Color {
        r: 0.9,
        g: 0.9,
        b: 0.2,
        a: 1.0,
    };
    let override_supporting = Color {
        r: 0.8,
        g: 0.2,
        b: 0.8,
        a: 1.0,
    };

    let style = DialogStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(override_bg))))
        .headline_color(WidgetStateProperty::new(Some(ColorRef::Color(
            override_headline,
        ))))
        .supporting_text_color(WidgetStateProperty::new(Some(ColorRef::Color(
            override_supporting,
        ))));

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let dialog = Dialog::new(open.clone())
                .headline("Dialog")
                .supporting_text("Body")
                .actions(vec![DialogAction::new("OK").test_id("dialog-ok")])
                .style(style.clone())
                .test_id("dialog")
                .into_element(
                    cx,
                    |cx| {
                        let trigger = Button::new("Underlay focus probe")
                            .test_id("dialog-trigger")
                            .into_element(cx);
                        with_padding(cx, Px(24.0), trigger)
                    },
                    |_cx| Vec::new(),
                );
            vec![dialog]
        })
    };

    let mut scene = None;
    for _ in 0..3 {
        use fret_ui_kit::OverlayController;

        app.advance_frame();
        OverlayController::begin_frame(&mut app, window);

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut next = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut next, 1.0);
        scene = Some(next);
    }

    let scene = scene.expect("expected rendered scene");

    assert!(
        scene.ops().iter().any(|op| {
            matches!(
                op,
                SceneOp::Quad { background, .. } if background.paint == fret_core::Paint::Solid(override_bg)
            )
        }),
        "expected DialogStyle.container_background to affect at least one quad background"
    );
    assert!(
        scene
            .ops()
            .iter()
            .any(|op| {
                matches!(
                    op,
                    SceneOp::Text { paint, .. } if paint.paint == fret_core::Paint::Solid(override_headline)
                )
            }),
        "expected DialogStyle.headline_color to affect at least one text draw op"
    );
    assert!(
        scene.ops().iter().any(|op| {
            matches!(
                op,
                SceneOp::Text { paint, .. } if paint.paint == fret_core::Paint::Solid(override_supporting)
            )
        }),
        "expected DialogStyle.supporting_text_color to affect at least one text draw op"
    );
}

#[test]
fn dialog_scrim_dismisses_without_activating_underlay() {
    use fret_ui_material3::{Button, Dialog, DialogAction};

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
            Size::new(Px(420.0), Px(320.0)),
        );

        let open = app.models_mut().insert(false);
        let underlay_toggled = app.models_mut().insert(false);

        let open_model = open.clone();
        let underlay_model = underlay_toggled.clone();
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                let open_model = open_model.clone();
                let underlay_model = underlay_model.clone();
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let underlay_model = underlay_model.clone();
                    let underlay = cx.pressable(
                        fret_ui::element::PressableProps {
                            enabled: true,
                            focusable: false,
                            a11y: fret_ui::element::PressableA11y {
                                test_id: Some(std::sync::Arc::<str>::from("underlay-fullscreen")),
                                ..Default::default()
                            },
                            layout: {
                                let mut l = fret_ui::element::LayoutStyle::default();
                                l.position = fret_ui::element::PositionStyle::Absolute;
                                l.size.width = fret_ui::element::Length::Fill;
                                l.size.height = fret_ui::element::Length::Fill;
                                l.inset = fret_ui::element::InsetStyle {
                                    top: Some(Px(0.0)).into(),
                                    right: Some(Px(0.0)).into(),
                                    bottom: Some(Px(0.0)).into(),
                                    left: Some(Px(0.0)).into(),
                                };
                                l
                            },
                            ..Default::default()
                        },
                        move |cx, _st| {
                            cx.pressable_toggle_bool(&underlay_model);
                            Vec::new()
                        },
                    );

                    let dialog = Dialog::new(open_model.clone())
                        .headline("Dialog")
                        .supporting_text("Body")
                        .actions(vec![DialogAction::new("OK").test_id("dialog-ok")])
                        .test_id("dialog")
                        .into_element(
                            cx,
                            move |cx| {
                                let trigger = Button::new("Open dialog")
                                    .test_id("dialog-trigger")
                                    .into_element(cx);
                                with_padding(cx, Px(24.0), trigger)
                            },
                            |_cx| Vec::new(),
                        );
                    vec![underlay, dialog]
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
                snapshot.nodes.iter().find_map(|node| {
                    (node.test_id.as_deref() == Some("dialog-trigger")).then_some(node.id)
                })
            })
            .unwrap_or_else(|| panic!("expected dialog-trigger in semantics snapshot ({label})"));
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
            "expected modal barrier root while dialog is open ({label})"
        );
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|node| node.test_id.as_deref() == Some("dialog.scrim")),
            "expected dialog scrim node while dialog is open ({label})"
        );
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|node| node.test_id.as_deref() == Some("underlay-fullscreen")),
            "expected underlay-fullscreen node while dialog is open ({label})"
        );

        let click_at = Point::new(Px(4.0), Px(4.0));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), click_at),
        );
        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

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
            app.models().get_copied(&open),
            Some(false),
            "expected dialog to dismiss on scrim press ({label})"
        );
        assert_eq!(
            app.models().get_copied(&underlay_toggled),
            Some(false),
            "expected dialog scrim to prevent underlay activation ({label})"
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected dialog to restore focus to trigger after scrim dismissal ({label})"
        );
    }
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

#[test]
fn tooltip_opens_and_closes_on_hover_across_schemes() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Button, PlainTooltip, TooltipProvider};

    fn render_tooltip_root(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
    ) -> NodeId {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            TooltipProvider::new()
                .delay_duration_frames(0)
                .skip_delay_duration_frames(0)
                .with_elements(cx, |cx| {
                    let trigger = Button::new("Trigger")
                        .test_id("tooltip-trigger")
                        .into_element(cx);
                    let tooltip = PlainTooltip::new(trigger, "Tip")
                        .open_delay_frames(Some(0))
                        .close_delay_frames(Some(0))
                        .into_element(cx);
                    vec![with_padding(cx, Px(24.0), tooltip)]
                })
        })
    }

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
            Size::new(Px(420.0), Px(320.0)),
        );

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render_tooltip_root(ui, app, services, window, bounds),
        );

        let trigger_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("tooltip-trigger") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected tooltip-trigger in semantics snapshot ({label})"));
        let trigger_bounds = ui
            .debug_node_visual_bounds(trigger_node)
            .expect("expected tooltip-trigger bounds");
        let hover_at = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_move(PointerId(1), hover_at),
        );

        let mut opened = false;
        for _ in 0..6 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render_tooltip_root(ui, app, services, window, bounds),
            );

            let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
            if stack.stack.iter().any(|entry| {
                entry.kind == OverlayStackEntryKind::Tooltip && entry.open && entry.visible
            }) {
                opened = true;
                break;
            }
        }
        assert!(opened, "expected tooltip to open on hover ({label})");

        let unhover_at = Point::new(Px(1.0), Px(1.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_move(PointerId(1), unhover_at),
        );

        let mut closed = false;
        for _ in 0..6 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render_tooltip_root(ui, app, services, window, bounds),
            );

            let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
            if !stack
                .stack
                .iter()
                .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible)
            {
                closed = true;
                break;
            }
        }
        assert!(closed, "expected tooltip to close after unhover ({label})");
    }
}

#[test]
fn rich_tooltip_opens_and_closes_on_hover_smoke() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Button, RichTooltip, TooltipProvider};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Dark, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(320.0)),
    );

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            TooltipProvider::new()
                .delay_duration_frames(0)
                .skip_delay_duration_frames(0)
                .with_elements(cx, |cx| {
                    let trigger = Button::new("Trigger")
                        .test_id("tooltip-trigger")
                        .into_element(cx);
                    let tooltip = RichTooltip::new(trigger, "Supporting text")
                        .title("Title")
                        .open_delay_frames(Some(0))
                        .close_delay_frames(Some(0))
                        .into_element(cx);
                    vec![with_padding(cx, Px(24.0), tooltip)]
                })
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
            snapshot.nodes.iter().find_map(|node| {
                if node.test_id.as_deref() == Some("tooltip-trigger") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected tooltip-trigger in semantics snapshot");
    let trigger_bounds = ui
        .debug_node_visual_bounds(trigger_node)
        .expect("expected tooltip-trigger bounds");
    let hover_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), hover_at),
    );

    let mut opened = false;
    for _ in 0..6 {
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
        if stack
            .stack
            .iter()
            .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible)
        {
            opened = true;
            break;
        }
    }
    assert!(opened, "expected rich tooltip to open on hover");

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), Point::new(Px(0.0), Px(0.0))),
    );

    let mut closed = false;
    for _ in 0..10 {
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
            .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible)
        {
            closed = true;
            break;
        }
    }
    assert!(closed, "expected rich tooltip to close after unhover");
}

#[test]
fn tooltip_does_not_open_on_touch_move() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Button, PlainTooltip, TooltipProvider};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Dark, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(320.0)),
    );

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            TooltipProvider::new()
                .delay_duration_frames(0)
                .skip_delay_duration_frames(0)
                .with_elements(cx, |cx| {
                    let trigger = Button::new("Trigger")
                        .test_id("tooltip-trigger")
                        .into_element(cx);
                    let tooltip = PlainTooltip::new(trigger, "Tip")
                        .open_delay_frames(Some(0))
                        .close_delay_frames(Some(0))
                        .into_element(cx);
                    vec![with_padding(cx, Px(24.0), tooltip)]
                })
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
            snapshot.nodes.iter().find_map(|node| {
                if node.test_id.as_deref() == Some("tooltip-trigger") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected tooltip-trigger in semantics snapshot");
    let trigger_bounds = ui
        .debug_node_visual_bounds(trigger_node)
        .expect("expected tooltip-trigger bounds");
    let hover_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move_touch(PointerId(1), hover_at),
    );

    for _ in 0..6 {
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
        assert!(
            !stack
                .stack
                .iter()
                .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible),
            "expected tooltip to remain closed when primary pointer is touch"
        );
    }
}

#[test]
fn tooltip_is_click_through_and_does_not_block_underlay_activation_across_schemes() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Button, PlainTooltip, TooltipProvider};

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
            Size::new(Px(560.0), Px(420.0)),
        );

        let underlay_toggled = app.models_mut().insert(false);
        let underlay_toggled_for_render = underlay_toggled.clone();
        let render = move |ui: &mut UiTree<TestHost>,
                           app: &mut TestHost,
                           services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                TooltipProvider::new()
                    .delay_duration_frames(0)
                    .skip_delay_duration_frames(0)
                    .with_elements(cx, |cx| {
                        let trigger = Button::new("Trigger")
                            .test_id("tooltip-trigger")
                            .into_element(cx);
                        let tooltip = PlainTooltip::new(trigger, "Tip")
                            .open_delay_frames(Some(0))
                            .close_delay_frames(Some(0))
                            .into_element(cx);

                        let underlay_toggled = underlay_toggled_for_render.clone();
                        let underlay = cx.pressable(
                            fret_ui::element::PressableProps {
                                layout: {
                                    let mut l = fret_ui::element::LayoutStyle::default();
                                    l.size.width = fret_ui::element::Length::Px(Px(160.0));
                                    l.size.height = fret_ui::element::Length::Px(Px(40.0));
                                    l
                                },
                                a11y: fret_ui::element::PressableA11y {
                                    test_id: Some(std::sync::Arc::<str>::from("tooltip-underlay")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            move |cx, _st| {
                                cx.pressable_toggle_bool(&underlay_toggled);
                                Vec::new()
                            },
                        );

                        let mut props = fret_ui::element::FlexProps::default();
                        props.direction = fret_core::Axis::Vertical;
                        props.gap = fret_ui::element::SpacingLength::Px(Px(24.0));
                        vec![cx.flex(props, move |_cx| vec![tooltip, underlay])]
                    })
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
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("tooltip-trigger") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected tooltip-trigger in semantics snapshot ({label})"));
        let trigger_bounds = ui
            .debug_node_visual_bounds(trigger_node)
            .expect("expected tooltip-trigger bounds");
        let hover_at = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        let underlay_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("tooltip-underlay") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected tooltip-underlay in semantics snapshot ({label})"));
        let underlay_bounds = ui
            .debug_node_visual_bounds(underlay_node)
            .expect("expected tooltip-underlay bounds");
        let click_at = Point::new(
            Px(underlay_bounds.origin.x.0 + underlay_bounds.size.width.0 * 0.5),
            Px(underlay_bounds.origin.y.0 + underlay_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_move(PointerId(1), hover_at),
        );

        let mut opened = false;
        for _ in 0..8 {
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
            if stack
                .stack
                .iter()
                .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible)
            {
                opened = true;
                break;
            }
        }
        assert!(opened, "expected tooltip to open on hover ({label})");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), click_at),
        );
        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

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
            app.models().get_copied(&underlay_toggled),
            Some(true),
            "expected tooltip to be click-through and allow underlay activation ({label})"
        );
        assert_eq!(
            ui.focus(),
            Some(underlay_node),
            "expected underlay to receive focus when clicking through tooltip ({label})"
        );

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
                .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible)
            {
                closed = true;
                break;
            }
        }
        assert!(
            closed,
            "expected tooltip to close after outside press without blocking underlay ({label})"
        );
    }
}

#[test]
fn segmented_button_semantics_roles_match_compose_baseline() {
    use std::collections::BTreeSet;

    use fret_ui::element::FlexProps;
    use fret_ui_material3::{SegmentedButtonItem, SegmentedButtonSet};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Dark, DynamicVariant::Expressive);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(260.0)),
    );

    let single_value: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("alpha"));
    let multi_value: Model<BTreeSet<Arc<str>>> = app
        .models_mut()
        .insert([Arc::<str>::from("alpha")].into_iter().collect());

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let single = SegmentedButtonSet::single(single_value.clone())
                .items(vec![
                    SegmentedButtonItem::new("alpha", "Alpha").test_id("segmented-single-alpha"),
                    SegmentedButtonItem::new("beta", "Beta").test_id("segmented-single-beta"),
                    SegmentedButtonItem::new("gamma", "Gamma")
                        .disabled(true)
                        .test_id("segmented-single-gamma-disabled"),
                ])
                .a11y_label("single segmented")
                .test_id("segmented-single")
                .into_element(cx);

            let multi = SegmentedButtonSet::multi(multi_value.clone())
                .items(vec![
                    SegmentedButtonItem::new("alpha", "Alpha").test_id("segmented-multi-alpha"),
                    SegmentedButtonItem::new("beta", "Beta").test_id("segmented-multi-beta"),
                ])
                .a11y_label("multi segmented")
                .test_id("segmented-multi")
                .into_element(cx);

            let mut props = FlexProps::default();
            props.direction = fret_core::Axis::Vertical;
            props.gap = fret_ui::element::SpacingLength::Px(Px(16.0));
            let content = cx.flex(props, |_cx| vec![single, multi]);
            vec![with_padding(cx, Px(24.0), content)]
        })
    };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");

    let find = |id: &str| -> &fret_core::SemanticsNode {
        snap.nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some(id))
            .unwrap_or_else(|| panic!("expected semantics node {id}"))
    };

    let group_single = find("segmented-single");
    assert_eq!(group_single.role, fret_core::SemanticsRole::RadioGroup);

    let alpha = find("segmented-single-alpha");
    assert_eq!(alpha.role, fret_core::SemanticsRole::RadioButton);
    assert_eq!(alpha.flags.checked, Some(true));
    assert_eq!(
        alpha.flags.checked_state,
        Some(fret_core::SemanticsCheckedState::True)
    );
    assert!(
        !alpha.flags.selected,
        "radio buttons should not set selected"
    );

    let beta = find("segmented-single-beta");
    assert_eq!(beta.role, fret_core::SemanticsRole::RadioButton);
    assert_eq!(beta.flags.checked, Some(false));
    assert_eq!(
        beta.flags.checked_state,
        Some(fret_core::SemanticsCheckedState::False)
    );
    assert!(
        !beta.flags.selected,
        "radio buttons should not set selected"
    );

    let multi_alpha = find("segmented-multi-alpha");
    assert_eq!(multi_alpha.role, fret_core::SemanticsRole::Checkbox);
    assert_eq!(multi_alpha.flags.checked, Some(true));
    assert_eq!(
        multi_alpha.flags.checked_state,
        Some(fret_core::SemanticsCheckedState::True)
    );
    assert!(
        !multi_alpha.flags.selected,
        "checkboxes should not set selected"
    );

    let multi_beta = find("segmented-multi-beta");
    assert_eq!(multi_beta.role, fret_core::SemanticsRole::Checkbox);
    assert_eq!(multi_beta.flags.checked, Some(false));
    assert_eq!(
        multi_beta.flags.checked_state,
        Some(fret_core::SemanticsCheckedState::False)
    );
    assert!(
        !multi_beta.flags.selected,
        "checkboxes should not set selected"
    );
}

#[test]
fn material3_autocomplete_semantics_v1() {
    use fret_core::SemanticsRole;
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Autocomplete, AutocompleteItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let model = app.models_mut().insert(String::new());
    let selected_value = app
        .models_mut()
        .insert(Some(Arc::<str>::from("beta")) as Option<Arc<str>>);
    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let ac = Autocomplete::new(model.clone())
                    .selected_value(selected_value.clone())
                    .items(items.clone())
                    .a11y_label("autocomplete")
                    .test_id("material3-autocomplete")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), ac)]
            })
        };

    // Frame 1: build stable input id + bounds.
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-autocomplete")).then_some(node.id)
            })
        })
        .expect("expected material3-autocomplete input node in semantics snapshot");

    ui.set_focus(Some(input_node));

    // Frame 2: focus visible to the widget.
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    // Open via keyboard.
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    // Frame 3/4: overlay created, then relationships stabilize (controls/active-descendant).
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to be open after ArrowDown"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete"))
        .expect("combobox input node");
    assert_eq!(input.role, SemanticsRole::ComboBox);
    assert!(
        input.flags.expanded,
        "combobox input should report expanded=true while open"
    );

    let list = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete.listbox"))
        .expect("listbox node");
    assert!(
        input.controls.contains(&list.id),
        "combobox input should control the listbox"
    );
    assert!(
        list.labelled_by.contains(&input.id),
        "listbox should be labelled by the combobox input"
    );

    let active = input
        .active_descendant
        .expect("active_descendant should be set");
    let active_node = snap
        .nodes
        .iter()
        .find(|n| n.id == active)
        .expect("active_descendant should reference a node in the snapshot");
    assert_eq!(active_node.role, SemanticsRole::ListBoxOption);

    let beta = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete.option.beta"))
        .expect("expected beta option node");
    assert!(beta.flags.selected, "expected beta to be marked selected");

    // Typing still works while the overlay is open.
    ui.set_focus(Some(input.id));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::TextInput("a".to_string()),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete"))
        .expect("combobox input node after typing");
    assert_eq!(input.value.as_deref(), Some("a"));
}

#[test]
fn material3_autocomplete_filters_items_by_query_v1() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Autocomplete, AutocompleteItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let model = app.models_mut().insert(String::new());
    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let ac = Autocomplete::new(model.clone())
                    .items(items.clone())
                    .a11y_label("autocomplete")
                    .test_id("material3-autocomplete")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), ac)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-autocomplete")).then_some(node.id)
            })
        })
        .expect("expected material3-autocomplete input node in semantics snapshot");
    ui.set_focus(Some(input_node));

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::TextInput("ga".to_string()),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to be open after typing"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .any(|n| { n.test_id.as_deref() == Some("material3-autocomplete.option.gamma") }),
        "expected gamma option after typing 'ga'"
    );
    assert!(
        !snap
            .nodes
            .iter()
            .any(|n| { n.test_id.as_deref() == Some("material3-autocomplete.option.alpha") }),
        "expected alpha option to be filtered out after typing 'ga'"
    );
}

#[test]
fn material3_autocomplete_enter_commits_and_does_not_reopen_v1() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Autocomplete, AutocompleteItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let model = app.models_mut().insert(String::new());
    let selected_value = app.models_mut().insert(None::<Arc<str>>);
    let selected_value_for_render = selected_value.clone();
    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let ac = Autocomplete::new(model.clone())
                    .selected_value(selected_value_for_render.clone())
                    .items(items.clone())
                    .a11y_label("autocomplete")
                    .test_id("material3-autocomplete")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), ac)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let input_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-autocomplete")).then_some(node.id)
            })
        })
        .expect("expected material3-autocomplete input node in semantics snapshot");
    ui.set_focus(Some(input_node));

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to be open after ArrowDown"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Enter));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Enter));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        !stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected autocomplete popover overlay to remain closed after Enter commit"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-autocomplete"))
        .expect("combobox input node after Enter");
    assert_eq!(input.value.as_deref(), Some("Alpha"));

    let selected = app.models_mut().get_cloned(&selected_value).unwrap_or(None);
    assert_eq!(
        selected.as_deref(),
        Some("alpha"),
        "expected selected_value model to be committed on Enter"
    );
}

#[test]
fn material3_exposed_dropdown_reverts_query_to_committed_selection_on_blur_v1() {
    use fret_ui::element::{FlexProps, Length};
    use fret_ui_material3::{AutocompleteItem, ExposedDropdown, TextField, TextFieldVariant};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let selected_value = app
        .models_mut()
        .insert(Some(Arc::<str>::from("beta")) as Option<Arc<str>>);
    let query = app.models_mut().insert(String::new());
    let query_for_render = query.clone();
    let other = app.models_mut().insert(String::new());

    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let exposed = ExposedDropdown::new(selected_value.clone())
                    .query(query_for_render.clone())
                    .items(items.clone())
                    .a11y_label("exposed dropdown")
                    .test_id("material3-exposed-dropdown")
                    .into_element(cx);

                let other = TextField::new(other.clone())
                    .variant(TextFieldVariant::Outlined)
                    .label("Other")
                    .test_id("other-field")
                    .into_element(cx);

                let mut column = FlexProps::default();
                column.direction = fret_core::Axis::Vertical;
                column.gap = fret_ui::element::SpacingLength::Px(Px(24.0));
                column.layout.size.width = Length::Fill;

                let content = cx.flex(column, |_cx| vec![exposed, other]);
                vec![with_padding(cx, Px(24.0), content)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    assert_eq!(
        app.models_mut().get_cloned(&query).unwrap_or_default(),
        "Beta",
        "expected query to synchronize from the committed selection while blurred"
    );

    let (input_node, other_node): (NodeId, NodeId) = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            let input = snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-exposed-dropdown")).then_some(node.id)
            })?;
            let other = snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("other-field")).then_some(node.id)
            })?;
            Some((input, other))
        })
        .expect("expected input and other nodes in semantics snapshot");

    ui.set_focus(Some(input_node));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let _ = app.models_mut().update(&query, |v| *v = "ga".to_string());
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    assert_eq!(
        app.models_mut().get_cloned(&query).unwrap_or_default(),
        "ga",
        "expected query to remain editable while focused"
    );

    ui.set_focus(Some(other_node));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    assert_eq!(
        app.models_mut().get_cloned(&query).unwrap_or_default(),
        "Beta",
        "expected query to revert to the committed selection label on blur"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-exposed-dropdown"))
        .expect("combobox input node after blur");
    assert_eq!(input.value.as_deref(), Some("Beta"));
}

#[test]
fn material3_exposed_dropdown_trailing_icon_toggles_overlay_v1() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{AutocompleteItem, ExposedDropdown};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let selected_value = app.models_mut().insert(None::<Arc<str>>);
    let query = app.models_mut().insert(String::new());
    let query_for_render = query.clone();

    let items: Arc<[AutocompleteItem]> = Arc::from(vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]);

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let exposed = ExposedDropdown::new(selected_value.clone())
                    .query(query_for_render.clone())
                    .items(items.clone())
                    .a11y_label("exposed dropdown")
                    .test_id("material3-exposed-dropdown")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), exposed)]
            })
        };

    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let icon_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("material3-exposed-dropdown.trailing-icon"))
                    .then_some(node.id)
            })
        })
        .expect("expected trailing icon node in semantics snapshot");

    let icon_bounds = ui
        .debug_node_visual_bounds(icon_node)
        .expect("expected trailing icon bounds");
    let click_at = Point::new(
        Px(icon_bounds.origin.x.0 + icon_bounds.size.width.0 * 0.5),
        Px(icon_bounds.origin.y.0 + icon_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected popover overlay to be open after clicking the trailing icon"
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-exposed-dropdown"))
        .expect("expected exposed dropdown input node");
    assert_eq!(input.role, SemanticsRole::ComboBox);
    assert!(
        input.flags.expanded,
        "exposed dropdown input should report expanded=true while open"
    );

    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("material3-exposed-dropdown.listbox"))
        .expect("expected exposed dropdown listbox node");
    assert_eq!(listbox.role, SemanticsRole::ListBox);
    assert!(
        input.controls.contains(&listbox.id),
        "exposed dropdown input should control its listbox"
    );
    assert!(
        listbox.labelled_by.contains(&input.id),
        "exposed dropdown listbox should be labelled by its input"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));
    run_overlay_frame_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        !stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        }),
        "expected popover overlay to be closed after clicking the trailing icon again"
    );
}

#[test]
fn dropdown_menu_dismisses_and_restores_focus_across_schemes() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::DropdownMenu;
    use fret_ui_material3::menu::{MenuEntry, MenuItem};

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
            Size::new(Px(560.0), Px(420.0)),
        );

        let open = app.models_mut().insert(false);
        let underlay_toggled = app.models_mut().insert(false);

        let open_model = open.clone();
        let underlay_model = underlay_toggled.clone();
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let menu = DropdownMenu::new(open_model.clone())
                        .a11y_label("menu")
                        .test_id("dropdown")
                        .into_element(
                            cx,
                            |cx| {
                                cx.pressable_with_id(
                                    fret_ui::element::PressableProps {
                                        layout: {
                                            let mut l = fret_ui::element::LayoutStyle::default();
                                            l.size.width = fret_ui::element::Length::Px(Px(120.0));
                                            l.size.height = fret_ui::element::Length::Px(Px(40.0));
                                            l
                                        },
                                        a11y: fret_ui::element::PressableA11y {
                                            test_id: Some(std::sync::Arc::<str>::from(
                                                "dropdown-trigger",
                                            )),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    |_cx, _st, _id| Vec::new(),
                                )
                            },
                            |_cx| {
                                vec![
                                    MenuEntry::Item(MenuItem::new("A").test_id("dropdown-item-a")),
                                    MenuEntry::Item(MenuItem::new("B").test_id("dropdown-item-b")),
                                    MenuEntry::Item(MenuItem::new("C").test_id("dropdown-item-c")),
                                ]
                            },
                        );

                    let underlay_toggled = underlay_model.clone();
                    let underlay = cx.pressable(
                        fret_ui::element::PressableProps {
                            layout: {
                                let mut l = fret_ui::element::LayoutStyle::default();
                                l.size.width = fret_ui::element::Length::Px(Px(160.0));
                                l.size.height = fret_ui::element::Length::Px(Px(40.0));
                                l
                            },
                            a11y: fret_ui::element::PressableA11y {
                                test_id: Some(std::sync::Arc::<str>::from("underlay-toggle")),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |cx, _st| {
                            cx.pressable_toggle_bool(&underlay_toggled);
                            Vec::new()
                        },
                    );

                    let mut props = fret_ui::element::FlexProps::default();
                    props.direction = fret_core::Axis::Vertical;
                    props.gap = fret_ui::element::SpacingLength::Px(Px(220.0));
                    vec![cx.flex(props, move |_cx| vec![menu, underlay])]
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
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("dropdown-trigger") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected dropdown-trigger in semantics snapshot ({label})"));
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

        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        assert!(
            stack
                .stack
                .iter()
                .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open),
            "expected dropdown menu overlay to be open ({label})"
        );

        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));

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
            app.models().get_copied(&open),
            Some(false),
            "expected dropdown menu to close on Escape ({label})"
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected dropdown menu to restore focus to trigger on Escape ({label})"
        );

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

        let underlay_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("underlay-toggle") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected underlay-toggle in semantics snapshot ({label})"));
        let underlay_bounds = ui
            .debug_node_visual_bounds(underlay_node)
            .expect("expected underlay-toggle bounds");
        let click_at = Point::new(
            Px(underlay_bounds.origin.x.0 + underlay_bounds.size.width.0 * 0.5),
            Px(underlay_bounds.origin.y.0 + underlay_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), click_at),
        );
        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

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
            app.models().get_copied(&open),
            Some(false),
            "expected dropdown menu to close on outside press ({label})"
        );
        assert_eq!(
            app.models().get_copied(&underlay_toggled),
            Some(false),
            "expected dropdown menu to prevent underlay activation on outside press ({label})"
        );

        let mut saw_unmount = false;
        for _ in 0..60 {
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
                .any(|e| e.kind == OverlayStackEntryKind::Popover && e.visible)
            {
                saw_unmount = true;
                break;
            }
        }
        assert!(
            saw_unmount,
            "expected dropdown menu popover layer to unmount after close ({label})"
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected dropdown menu to restore focus to trigger on outside press ({label})"
        );
    }
}

#[test]
fn chip_set_roving_treats_trailing_action_focus_as_active_chip() {
    use fret_ui_material3::{ChipSet, ChipSetItem, InputChip, SuggestionChip};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let chip_a_selected = app.models_mut().insert(false);

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let chip_a = InputChip::new(chip_a_selected.clone(), "Alpha")
                .trailing_icon(fret_icons::ids::ui::CLOSE)
                .on_trailing_icon_activate(Arc::new(|_host, _acx, _reason| {}))
                .test_id("chip-a");

            let chip_b = SuggestionChip::new("Beta").test_id("chip-b");

            let set = ChipSet::new(vec![ChipSetItem::from(chip_a), ChipSetItem::from(chip_b)])
                .a11y_label("chips")
                .test_id("chip-set")
                .into_element(cx);

            vec![with_padding(cx, Px(24.0), set)]
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

    let chip_a_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find_map(|node| (node.test_id.as_deref() == Some("chip-a")).then_some(node.id))
        })
        .expect("expected chip-a in semantics snapshot");

    let chip_a_trailing_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("chip-a.trailing-icon")).then_some(node.id)
            })
        })
        .expect("expected chip-a.trailing-icon in semantics snapshot");

    let chip_b_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find_map(|node| (node.test_id.as_deref() == Some("chip-b")).then_some(node.id))
        })
        .expect("expected chip-b in semantics snapshot");

    ui.set_focus(Some(chip_a_node));
    assert_eq!(ui.focus(), Some(chip_a_node));

    // ArrowRight should move focus to the trailing action inside the chip (handled by the chip),
    // not rove to the next chip.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    assert_eq!(
        ui.focus(),
        Some(chip_a_trailing_node),
        "expected ArrowRight to focus trailing action (chip-internal navigation)",
    );

    // ArrowRight again should bubble to ChipSet roving (chip-internal handler does not consume),
    // and move focus to the next chip.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    assert_eq!(
        ui.focus(),
        Some(chip_b_node),
        "expected ChipSet roving to treat trailing-focus as within the active chip",
    );
}

#[test]
fn radio_pressed_scene_structure_is_stable() {
    use fret_ui_material3::Radio;

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
            Size::new(Px(320.0), Px(240.0)),
        );

        let selected = app.models_mut().insert(false);
        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let radio = Radio::new(selected.clone())
                        .a11y_label("radio")
                        .test_id("radio")
                        .into_element(cx);
                    vec![with_padding(cx, Px(32.0), radio)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let radio_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("radio") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .expect("expected radio in semantics snapshot");
        let radio_bounds = ui
            .debug_node_visual_bounds(radio_node)
            .expect("expected radio visual bounds");
        let press_at = Point::new(
            Px(radio_bounds.origin.x.0 + radio_bounds.size.width.0 * 0.5),
            Px(radio_bounds.origin.y.0 + radio_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut baseline_structure: Option<Vec<SceneSig>> = None;
        let mut baseline_quads: Option<Vec<QuadGeomSig>> = None;
        for frame in 0..24 {
            app.advance_frame();
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

            if (2..7).contains(&frame) {
                let sig = scene_signature(&scene);
                if let Some(prev) = baseline_structure.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Radio to keep a stable scene structure while pressed ({label})"
                    );
                } else {
                    baseline_structure = Some(sig);
                }
            }

            if frame >= 16 {
                let sig = scene_quad_geometry_signature(&scene);
                if let Some(prev) = baseline_quads.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Radio to keep stable quad geometry after animations settle ({label})"
                    );
                } else {
                    baseline_quads = Some(sig);
                }
            }
        }

        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    }
}
