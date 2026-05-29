use fret_core::{
    AppWindowId, Color, Paint, Point, Px, Rect, Scene, SceneOp, SemanticsRole, Size, UiServices,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;
use fret_ui::scroll::ScrollHandle;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{TopAppBar, TopAppBarAction, TopAppBarScrollBehavior, TopAppBarVariant};

mod interaction_harness;
mod support;

use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(220.0)),
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

fn render_large_top_app_bar(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    scroll: &ScrollHandle,
    offset_y: f32,
) -> Scene {
    scroll.set_viewport_size(Size::new(Px(520.0), Px(220.0)));
    scroll.set_content_size(Size::new(Px(520.0), Px(1000.0)));
    scroll.set_offset(Point::new(Px(0.0), Px(offset_y)));

    let scroll = scroll.clone();
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds(), "root", |cx| {
            let bar = TopAppBar::new("Library")
                .variant(TopAppBarVariant::Large)
                .scroll_behavior(TopAppBarScrollBehavior::exit_until_collapsed(scroll))
                .navigation_icon(
                    TopAppBarAction::new(fret_icons::ids::ui::CHEVRON_RIGHT)
                        .a11y_label("Navigate")
                        .test_id("m3-top-app-bar-nav"),
                )
                .actions(vec![
                    TopAppBarAction::new(fret_icons::ids::ui::SEARCH)
                        .a11y_label("Search")
                        .test_id("m3-top-app-bar-search"),
                ])
                .a11y_label("Material top app bar")
                .test_id("m3-top-app-bar")
                .into_element(cx);
            vec![bar]
        });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds(), 1.0);

    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds(), &mut scene, 1.0);
    scene
}

fn visual_bounds_by_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Rect {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find_map(|node| (node.test_id.as_deref() == Some(test_id)).then_some(node.id))
        })
        .and_then(|node| {
            ui.debug_node_visual_bounds(node)
                .or_else(|| ui.debug_node_bounds(node))
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

fn intermediate_opacities(scene: &Scene) -> Vec<f32> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            SceneOp::PushOpacity { opacity } if *opacity > 0.01 && *opacity < 0.99 => {
                Some(*opacity)
            }
            _ => None,
        })
        .collect()
}

fn app_bar_background(scene: &Scene, expected_height: f32) -> Color {
    scene
        .ops()
        .iter()
        .find_map(|op| match op {
            SceneOp::Quad {
                rect, background, ..
            } if rect.origin.x.0.abs() <= 0.5
                && rect.origin.y.0.abs() <= 0.5
                && (rect.size.width.0 - 520.0).abs() <= 0.5
                && (rect.size.height.0 - expected_height).abs() <= 0.5 =>
            {
                match background.paint {
                    Paint::Solid(color) => Some(color),
                    _ => None,
                }
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("expected app bar background quad with height {expected_height}"))
}

fn color_distance(a: Color, b: Color) -> f32 {
    (a.r - b.r).abs() + (a.g - b.g).abs() + (a.b - b.b).abs() + (a.a - b.a).abs()
}

#[test]
fn top_app_bar_exposes_toolbar_semantics_role() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(220.0)),
    );

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let bar = TopAppBar::new("TopAppBar")
                    .variant(TopAppBarVariant::Small)
                    .a11y_label("Material 3 Top App Bar")
                    .test_id("top-app-bar")
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), bar)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let node = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some("top-app-bar"))
        })
        .expect("expected top-app-bar in semantics snapshot");

    assert_eq!(
        node.role,
        SemanticsRole::Toolbar,
        "expected top app bar semantics role to be Toolbar",
    );
}

#[test]
fn top_app_bar_large_exposes_material_parts_and_collapse_geometry() {
    let (mut app, window, mut services, mut ui) = harness();
    let scroll = ScrollHandle::default();

    let _scene = render_large_top_app_bar(&mut ui, &mut app, &mut services, window, &scroll, 0.0);

    let chrome = visual_bounds_by_test_id(&ui, "m3-top-app-bar.chrome");
    assert_px_close(chrome.origin.x.0, 0.0, "expanded chrome x");
    assert_px_close(chrome.size.width.0, 520.0, "expanded chrome width");
    assert_px_close(chrome.size.height.0, 152.0, "expanded large height");

    let expanded_title = visual_bounds_by_test_id(&ui, "m3-top-app-bar.expanded-title");
    assert_px_close(expanded_title.origin.x.0, 16.0, "expanded title x");

    render_large_top_app_bar(&mut ui, &mut app, &mut services, window, &scroll, 44.0);
    let half = visual_bounds_by_test_id(&ui, "m3-top-app-bar.chrome");
    assert_px_close(half.size.height.0, 108.0, "half-collapsed large height");

    render_large_top_app_bar(&mut ui, &mut app, &mut services, window, &scroll, 88.0);
    let collapsed = visual_bounds_by_test_id(&ui, "m3-top-app-bar.chrome");
    assert_px_close(collapsed.size.height.0, 64.0, "collapsed large height");

    let _collapsed_title = visual_bounds_by_test_id(&ui, "m3-top-app-bar.collapsed-title");

    let node = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some("m3-top-app-bar"))
        })
        .expect("expected top app bar semantics after collapse");
    assert_eq!(node.role, SemanticsRole::Toolbar);
    assert_eq!(node.label.as_deref(), Some("Material top app bar"));
}

#[test]
fn top_app_bar_large_scroll_fraction_drives_title_alpha_and_container_color() {
    let (mut app, window, mut services, mut ui) = harness();
    let scroll = ScrollHandle::default();

    let expanded_scene =
        render_large_top_app_bar(&mut ui, &mut app, &mut services, window, &scroll, 0.0);
    let half_scene =
        render_large_top_app_bar(&mut ui, &mut app, &mut services, window, &scroll, 44.0);
    let collapsed_scene =
        render_large_top_app_bar(&mut ui, &mut app, &mut services, window, &scroll, 88.0);

    let opacities = intermediate_opacities(&half_scene);
    assert!(
        opacities
            .iter()
            .any(|opacity| *opacity > 0.01 && *opacity < 0.25),
        "expected Compose TopTitleAlphaEasing to keep top title alpha below 0.25 at half collapse; opacities={opacities:?}"
    );
    assert!(
        opacities
            .iter()
            .any(|opacity| (*opacity - 0.5).abs() <= 0.02),
        "expected expanded title alpha to remain linear at half collapse; opacities={opacities:?}"
    );

    let expanded_color = app_bar_background(&expanded_scene, 152.0);
    let half_color = app_bar_background(&half_scene, 108.0);
    let collapsed_color = app_bar_background(&collapsed_scene, 64.0);

    assert!(
        color_distance(expanded_color, half_color) > 0.001,
        "expected half-collapsed color to move away from expanded color"
    );
    assert!(
        color_distance(half_color, collapsed_color) > 0.001,
        "expected half-collapsed color to interpolate instead of snapping to scrolled color"
    );
}
