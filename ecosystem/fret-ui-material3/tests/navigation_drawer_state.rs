#![cfg(feature = "diagnostics")]

//! Material 3 navigation drawer semantics, layout, and motion parity tests.

use std::sync::Arc;

use fret_core::{
    AppWindowId, Paint, Point, PointerId, Px, Rect, Scene, SceneOp, SemanticsNode,
    SemanticsOrientation, SemanticsRole, Size, UiServices,
};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{
    ModalNavigationDrawer, NavigationDrawer, NavigationDrawerItem, NavigationDrawerVariant,
};

mod support;

use support::events::{pointer_down, pointer_move};
use support::goldens::run_overlay_frame_with_scene_scaled;
use support::host::{FakeUiServices, TestHost};
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(640.0), Px(420.0)),
    )
}

fn harness() -> (TestHost, AppWindowId, FakeUiServices, UiTree<TestHost>) {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    (app, window, services, ui)
}

fn drawer_items(prefix: &str) -> Vec<NavigationDrawerItem> {
    vec![
        NavigationDrawerItem::new("search", "Search", fret_icons::ids::ui::SEARCH)
            .test_id(format!("{prefix}-search")),
        NavigationDrawerItem::new("settings", "Settings", fret_icons::ids::ui::SETTINGS)
            .badge_label("4")
            .test_id(format!("{prefix}-settings")),
        NavigationDrawerItem::new("disabled", "Disabled", fret_icons::ids::ui::SLASH)
            .disabled(true)
            .test_id(format!("{prefix}-disabled")),
    ]
}

fn render_navigation_drawer(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<Arc<str>>,
) {
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds(), "root", |cx| {
            let drawer = NavigationDrawer::new(selected)
                .a11y_label("Material navigation drawer")
                .test_id("m3-navigation-drawer")
                .items(drawer_items("m3-drawer"))
                .into_element(cx);
            vec![drawer]
        });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds(), 1.0);
}

fn semantics_node<'a>(ui: &'a UiTree<TestHost>, test_id: &str) -> &'a SemanticsNode {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(test_id))
        })
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"))
}

fn visual_bounds_by_test_id(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    test_id: &str,
) -> Rect {
    fret_ui::declarative::live_test_id_matches_for_window(app, window, test_id)
        .into_iter()
        .find_map(|m| {
            ui.debug_node_visual_bounds(m.node)
                .or_else(|| ui.debug_node_bounds(m.node))
        })
        .unwrap_or_else(|| panic!("expected visual bounds for test_id {test_id}"))
}

fn assert_px_close(actual: f32, expected: f32, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.5,
        "{context}: expected {expected}px, got {actual}px (delta {delta}px)"
    );
}

fn rect_right(rect: Rect) -> f32 {
    rect.origin.x.0 + rect.size.width.0
}

fn paint(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    bounds: Rect,
) -> Scene {
    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, 1.0);
    scene
}

fn state_layer_alphas_for_chrome(scene: &Scene, chrome: Rect) -> Vec<f32> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match *op {
            SceneOp::Quad {
                rect, background, ..
            } if rect.origin.x.0 >= chrome.origin.x.0 - 0.1
                && rect.origin.y.0 >= chrome.origin.y.0 - 0.1
                && rect.origin.x.0 + rect.size.width.0
                    <= chrome.origin.x.0 + chrome.size.width.0 + 0.1
                && rect.origin.y.0 + rect.size.height.0
                    <= chrome.origin.y.0 + chrome.size.height.0 + 0.1
                && (rect.size.width.0 - chrome.size.width.0).abs() <= 2.1
                && (rect.size.height.0 - chrome.size.height.0).abs() <= 2.1 =>
            {
                match background.paint {
                    Paint::Solid(color) if color.a > 0.0 && color.a < 0.2 => Some(color.a),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

fn largest_negative_horizontal_slide_tx(scene: &Scene) -> Option<f32> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            SceneOp::PushTransform { transform }
                if transform.b.abs() < 0.001
                    && transform.c.abs() < 0.001
                    && (transform.a - 1.0).abs() < 0.001
                    && (transform.d - 1.0).abs() < 0.001
                    && transform.ty.abs() < 0.001
                    && transform.tx < -1.0 =>
            {
                Some(transform.tx)
            }
            _ => None,
        })
        .min_by(|a, b| a.total_cmp(b))
}

fn scene_has_intermediate_opacity(scene: &Scene) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushOpacity { opacity } if *opacity > 0.01 && *opacity < 0.99
        )
    })
}

fn full_screen_scrim_alpha(scene: &Scene, width: f32, height: f32) -> Option<f32> {
    scene.ops().iter().find_map(|op| match op {
        SceneOp::Quad {
            rect, background, ..
        } if (rect.origin.x.0).abs() <= 0.5
            && (rect.origin.y.0).abs() <= 0.5
            && (rect.size.width.0 - width).abs() <= 0.5
            && (rect.size.height.0 - height).abs() <= 0.5 =>
        {
            match background.paint {
                Paint::Solid(color) => Some(color.a),
                _ => None,
            }
        }
        _ => None,
    })
}

#[test]
fn navigation_drawer_exports_vertical_tablist_semantics_and_item_geometry() {
    let (mut app, window, mut services, mut ui) = harness();
    let selected = app.models_mut().insert(Arc::<str>::from("search"));

    render_navigation_drawer(&mut ui, &mut app, &mut services, window, selected);

    let list = semantics_node(&ui, "m3-navigation-drawer");
    assert_eq!(list.role, SemanticsRole::TabList);
    assert_eq!(list.extra.orientation, Some(SemanticsOrientation::Vertical));
    assert_eq!(list.label.as_deref(), Some("Material navigation drawer"));

    let search = semantics_node(&ui, "m3-drawer-search");
    assert_eq!(search.role, SemanticsRole::Tab);
    assert!(search.flags.selected);
    assert_eq!(search.pos_in_set, Some(1));
    assert_eq!(search.set_size, Some(3));

    let disabled = semantics_node(&ui, "m3-drawer-disabled");
    assert_eq!(disabled.role, SemanticsRole::Tab);
    assert!(disabled.flags.disabled);
    assert_eq!(disabled.pos_in_set, Some(3));
    assert_eq!(disabled.set_size, Some(3));

    let drawer = visual_bounds_by_test_id(&ui, &app, window, "m3-navigation-drawer.chrome");
    assert_px_close(drawer.origin.x.0, 0.0, "drawer x");
    assert_px_close(drawer.size.width.0, 360.0, "drawer width");
    assert_px_close(drawer.size.height.0, 420.0, "drawer height");

    let item = visual_bounds_by_test_id(&ui, &app, window, "m3-drawer-search.chrome");
    assert_px_close(
        item.origin.x.0 - drawer.origin.x.0,
        12.0,
        "item horizontal inset",
    );
    assert_px_close(item.size.width.0, 336.0, "active indicator width");
    assert_px_close(item.size.height.0, 56.0, "active indicator height");

    let icon = visual_bounds_by_test_id(&ui, &app, window, "m3-drawer-search.icon");
    let label = visual_bounds_by_test_id(&ui, &app, window, "m3-drawer-search.label");
    assert_px_close(
        icon.origin.x.0 - item.origin.x.0,
        16.0,
        "icon start padding",
    );
    assert_px_close(icon.size.width.0, 24.0, "icon width");
    assert_px_close(label.origin.x.0 - rect_right(icon), 12.0, "icon-label gap");
}

#[test]
fn navigation_drawer_pressed_state_layer_animates_over_item_chrome() {
    let (mut app, window, mut services, mut ui) = harness();
    let selected = app.models_mut().insert(Arc::<str>::from("search"));
    let bounds = bounds();

    render_navigation_drawer(&mut ui, &mut app, &mut services, window, selected.clone());

    let chrome = visual_bounds_by_test_id(&ui, &app, window, "m3-drawer-settings.chrome");
    assert!(
        state_layer_alphas_for_chrome(&paint(&mut ui, &mut app, &mut services, bounds), chrome)
            .is_empty(),
        "idle navigation drawer item should not paint a visible state layer"
    );

    let press_at = Point::new(
        Px(chrome.origin.x.0 + chrome.size.width.0 * 0.5),
        Px(chrome.origin.y.0 + chrome.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), press_at),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), press_at),
    );

    let mut animated = Vec::new();
    for _ in 0..4 {
        app.advance_frame();
        render_navigation_drawer(&mut ui, &mut app, &mut services, window, selected.clone());
        animated.extend(state_layer_alphas_for_chrome(
            &paint(&mut ui, &mut app, &mut services, bounds),
            chrome,
        ));
    }

    assert!(
        animated.iter().any(|alpha| *alpha > 0.001 && *alpha < 0.2),
        "expected pressed navigation drawer item state layer to animate through partial alpha, got {animated:?}"
    );
}

#[test]
fn modal_navigation_drawer_exposes_panel_semantics_and_settled_geometry() {
    let (mut app, window, mut services, mut ui) = harness();
    let open = app.models_mut().insert(true);
    let selected = app.models_mut().insert(Arc::<str>::from("search"));

    let open_model = open.clone();
    let selected_model = selected.clone();
    let render = move |ui: &mut UiTree<TestHost>,
                       app: &mut TestHost,
                       services: &mut dyn UiServices| {
        let open = open_model.clone();
        let selected = selected_model.clone();
        fret_ui::declarative::render_root(ui, app, services, window, bounds(), "root", |cx| {
            let drawer_selected = selected.clone();
            let panel = move |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>| {
                NavigationDrawer::new(drawer_selected.clone())
                    .variant(NavigationDrawerVariant::Modal)
                    .a11y_label("Material modal navigation drawer")
                    .test_id("m3-modal-navigation-drawer-content")
                    .items(drawer_items("m3-modal-drawer"))
                    .into_element(cx)
            };

            let underlay =
                move |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>| cx.text("Underlay");

            let drawer = ModalNavigationDrawer::new(open)
                .open_duration_ms(Some(1))
                .close_duration_ms(Some(1))
                .test_id("m3-modal-navigation-drawer")
                .into_element(cx, panel, underlay);
            vec![drawer]
        })
    };

    for _ in 0..8 {
        run_overlay_frame_with_scene_scaled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            1.0,
            true,
            |ui, app, services| render(ui, app, services),
        );
        if ui.semantics_snapshot().is_some_and(|snapshot| {
            snapshot
                .nodes
                .iter()
                .any(|node| node.test_id.as_deref() == Some("m3-modal-navigation-drawer.panel"))
        }) {
            break;
        }
    }

    let panel_semantics = semantics_node(&ui, "m3-modal-navigation-drawer.panel");
    assert_eq!(panel_semantics.role, SemanticsRole::Dialog);
    assert_eq!(panel_semantics.label.as_deref(), Some("Navigation menu"));

    let scrim_semantics = semantics_node(&ui, "m3-modal-navigation-drawer.scrim");
    assert_eq!(scrim_semantics.label.as_deref(), Some("Close drawer"));

    let content = semantics_node(&ui, "m3-modal-navigation-drawer-content");
    assert_eq!(content.role, SemanticsRole::TabList);
    assert_eq!(
        content.extra.orientation,
        Some(SemanticsOrientation::Vertical)
    );

    let scrim = visual_bounds_by_test_id(&ui, &app, window, "m3-modal-navigation-drawer.scrim");
    assert_px_close(scrim.origin.x.0, 0.0, "scrim x");
    assert_px_close(scrim.origin.y.0, 0.0, "scrim y");
    assert_px_close(scrim.size.width.0, 640.0, "scrim width");
    assert_px_close(scrim.size.height.0, 420.0, "scrim height");

    let panel = visual_bounds_by_test_id(&ui, &app, window, "m3-modal-navigation-drawer.panel");
    assert_px_close(panel.origin.x.0, 0.0, "settled panel x");
    assert_px_close(panel.origin.y.0, 0.0, "settled panel y");
    assert_px_close(panel.size.width.0, 360.0, "settled panel width");
    assert_px_close(panel.size.height.0, 420.0, "settled panel height");
}

#[test]
fn modal_navigation_drawer_slides_from_negative_drawer_width_without_panel_fade() {
    let (mut app, window, mut services, mut ui) = harness();
    let open = app.models_mut().insert(false);
    let selected = app.models_mut().insert(Arc::<str>::from("search"));

    let open_model = open.clone();
    let selected_model = selected.clone();
    let render = move |ui: &mut UiTree<TestHost>,
                       app: &mut TestHost,
                       services: &mut dyn UiServices| {
        let open = open_model.clone();
        let selected = selected_model.clone();
        fret_ui::declarative::render_root(ui, app, services, window, bounds(), "root", |cx| {
            let drawer_selected = selected.clone();
            let panel = move |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>| {
                NavigationDrawer::new(drawer_selected.clone())
                    .variant(NavigationDrawerVariant::Modal)
                    .test_id("m3-motion-drawer-content")
                    .items(drawer_items("m3-motion-drawer"))
                    .into_element(cx)
            };

            let underlay =
                move |cx: &mut fret_ui::elements::ElementContext<'_, TestHost>| cx.text("Underlay");

            let drawer = ModalNavigationDrawer::new(open)
                .test_id("m3-motion-modal-navigation-drawer")
                .into_element(cx, panel, underlay);
            vec![drawer]
        })
    };

    run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    let _ = app.models_mut().update(&open, |v| *v = true);
    let first_open_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds(),
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    let tx = largest_negative_horizontal_slide_tx(&first_open_scene)
        .expect("expected modal drawer to slide with a negative horizontal transform");
    assert!(
        tx > -370.0 && tx < -10.0,
        "expected first open frame to translate from the 360px closed anchor, got tx={tx}"
    );
    assert!(
        !scene_has_intermediate_opacity(&first_open_scene),
        "Material3 modal drawer moves the panel by offset; only the scrim alpha should fade"
    );

    let scrim_alpha = full_screen_scrim_alpha(&first_open_scene, 640.0, 420.0)
        .expect("expected modal drawer scrim quad on the first open frame");
    assert!(
        scrim_alpha > 0.0 && scrim_alpha < 0.5,
        "expected scrim to fade in below the 0.5 Material target alpha, got {scrim_alpha}"
    );
}
