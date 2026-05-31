//! Fixed-frame motion regression tests for Material 3 SearchBar.

use fret_core::{AppWindowId, DrawOrder, NodeId, Point, PointerId, Px, Rect, Scene, Size};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::SearchBar;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod support;

use support::events::{pointer_down, pointer_move};
use support::host::{FakeUiServices, TestHost};
use support::interaction_harness::{QuadSig, RectSig, scene_quad_signature};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

fn render_search_bar_frame(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeUiServices,
    window: AppWindowId,
    bounds: Rect,
    query: fret_runtime::Model<String>,
) -> Scene {
    app.advance_frame();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let bar = SearchBar::new(query)
            .a11y_label("Material search")
            .placeholder("Search")
            .test_id("m3-search-bar")
            .into_element(cx);
        vec![with_padding(cx, Px(32.0), bar)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, 1.0);
    scene
}

fn search_bar_node(ui: &UiTree<TestHost>) -> NodeId {
    semantics_node_id_by_test_id(ui, "m3-search-bar").expect("expected m3-search-bar")
}

fn node_center(ui: &UiTree<TestHost>, node: NodeId) -> Point {
    let bounds = ui
        .debug_node_visual_bounds(node)
        .expect("expected SearchBar visual bounds");
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

fn chrome_rect(ui: &UiTree<TestHost>) -> RectSig {
    let chrome = semantics_node_id_by_test_id(ui, "m3-search-bar.chrome")
        .expect("expected m3-search-bar.chrome");
    rect_sig(
        ui.debug_node_visual_bounds(chrome)
            .or_else(|| ui.debug_node_bounds(chrome))
            .expect("expected SearchBar chrome bounds"),
    )
}

fn rect_sig(rect: Rect) -> RectSig {
    RectSig {
        x: px_sig(rect.origin.x),
        y: px_sig(rect.origin.y),
        w: px_sig(rect.size.width),
        h: px_sig(rect.size.height),
    }
}

fn px_sig(px: Px) -> i32 {
    ((px.0 * 10.0).round()) as i32
}

fn state_layer_for_chrome(quads: &[QuadSig], chrome: RectSig) -> Option<QuadSig> {
    quads.iter().copied().find(|q| {
        q.order == DrawOrder(0)
            && q.rect == chrome
            && q.background.a > 0
            && q.background.a < 200
            && q.border.top == 0
            && q.border.right == 0
            && q.border.bottom == 0
            && q.border.left == 0
    })
}

fn ripple_quad(quads: &[QuadSig], chrome: RectSig) -> Option<QuadSig> {
    quads.iter().copied().find(|q| {
        q.order == DrawOrder(1)
            && q.background.a > 0
            && q.rect.w > 0
            && q.rect.h > 0
            && q.rect != chrome
    })
}

fn search_bar_harness() -> (
    TestHost,
    AppWindowId,
    FakeUiServices,
    UiTree<TestHost>,
    Rect,
    fret_runtime::Model<String>,
) {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(980.0), Px(180.0)),
    );
    let query = app.models_mut().insert(String::new());

    (app, window, services, ui, bounds, query)
}

#[test]
fn search_bar_hover_state_layer_animates_between_idle_and_hovered() {
    let (mut app, window, mut services, mut ui, bounds, query) = search_bar_harness();

    let idle_scene = render_search_bar_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        query.clone(),
    );
    let node = search_bar_node(&ui);
    let center = node_center(&ui, node);
    let chrome = chrome_rect(&ui);
    assert!(
        state_layer_for_chrome(&scene_quad_signature(&idle_scene), chrome).is_none(),
        "expected no SearchBar state layer before hover"
    );

    ui.dispatch_event(&mut app, &mut services, &pointer_move(PointerId(1), center));

    let mut intermediate_alpha = None;
    let mut settled_alpha = None;
    for frame in 0..12 {
        let scene = render_search_bar_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            query.clone(),
        );
        let overlay = state_layer_for_chrome(&scene_quad_signature(&scene), chrome);
        if let Some(overlay) = overlay {
            if overlay.background.a > 0 && overlay.background.a < 70 {
                intermediate_alpha = Some(overlay.background.a);
            }
            if frame >= 8 {
                settled_alpha = Some(overlay.background.a);
            }
        }
    }

    let intermediate_alpha =
        intermediate_alpha.expect("expected intermediate SearchBar hover state-layer alpha");
    assert!(
        intermediate_alpha > 0 && intermediate_alpha < 70,
        "expected hover state-layer to animate through an intermediate alpha, got {intermediate_alpha}"
    );

    let settled_alpha = settled_alpha.expect("expected settled SearchBar hover state-layer alpha");
    assert!(
        (70..=90).contains(&settled_alpha),
        "expected settled hover state-layer alpha near Material 0.08, got {settled_alpha}"
    );
}

#[test]
fn search_bar_press_ripple_expands_on_fixed_frames() {
    let (mut app, window, mut services, mut ui, bounds, query) = search_bar_harness();

    let idle_scene = render_search_bar_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        query.clone(),
    );
    let node = search_bar_node(&ui);
    let center = node_center(&ui, node);
    let chrome = chrome_rect(&ui);
    assert!(
        ripple_quad(&scene_quad_signature(&idle_scene), chrome).is_none(),
        "expected no SearchBar ripple before press"
    );

    ui.dispatch_event(&mut app, &mut services, &pointer_down(PointerId(1), center));
    let first_press_scene = render_search_bar_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        query.clone(),
    );
    let first_ripple = ripple_quad(&scene_quad_signature(&first_press_scene), chrome)
        .expect("expected SearchBar ripple on first press frame");

    let mut later_ripple = first_ripple;
    for _ in 0..4 {
        let scene = render_search_bar_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            query.clone(),
        );
        later_ripple = ripple_quad(&scene_quad_signature(&scene), chrome)
            .expect("expected SearchBar ripple to remain active while pressed");
    }

    assert!(
        later_ripple.rect.w > first_ripple.rect.w && later_ripple.rect.h > first_ripple.rect.h,
        "expected SearchBar ripple radius to expand, first={:?}, later={:?}",
        first_ripple.rect,
        later_ripple.rect
    );
}
