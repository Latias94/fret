use fret_core::{
    AppWindowId, Color, Edges, NodeId, Paint, Point, PointerId, Px, Rect, Scene, SceneOp,
    SemanticsLive, Size,
};
use fret_runtime::{CommandId, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui::action::UiActionHostAdapter;
use fret_ui_kit::{ColorRef, ToastStore, WidgetStateProperty};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{
    Snackbar, SnackbarController, SnackbarDuration, SnackbarHost, SnackbarStyle,
};

mod support;

use support::events::{pointer_down, pointer_up};
use support::goldens::run_overlay_frame_with_scene_scaled;
use support::host::{FakeUiServices, TestHost};
use support::layout::semantics_node_id_by_test_id;
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    )
}

fn layout_bounds_by_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Rect {
    let node = semantics_node_id_by_test_id(ui, test_id)
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"));
    ui.debug_node_bounds(node)
        .or_else(|| ui.debug_node_visual_bounds(node))
        .unwrap_or_else(|| panic!("expected layout bounds for test_id {test_id}"))
}

fn visual_bounds_by_test_id(ui: &UiTree<TestHost>, test_id: &str) -> Rect {
    let node = semantics_node_id_by_test_id(ui, test_id)
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"));
    ui.debug_node_visual_bounds(node)
        .or_else(|| ui.debug_node_bounds(node))
        .unwrap_or_else(|| panic!("expected visual bounds for test_id {test_id}"))
}

fn scene_has_intermediate_opacity(scene: &Scene) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushOpacity { opacity } if *opacity > 0.01 && *opacity < 0.99
        )
    })
}

fn scene_has_snackbar_scale(scene: &Scene) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushTransform { transform }
                if transform.b.abs() < 0.001
                    && transform.c.abs() < 0.001
                    && transform.a > 0.79
                    && transform.a < 1.0
                    && transform.d > 0.79
                    && transform.d < 1.0
        )
    })
}

fn scene_has_y_slide_without_scale(scene: &Scene) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushTransform { transform }
                if (transform.a - 1.0).abs() < 0.001
                    && (transform.d - 1.0).abs() < 0.001
                    && transform.ty.abs() > 0.5
        )
    })
}

fn color_close(actual: Color, expected: Color) -> bool {
    (actual.r - expected.r).abs() <= 0.001
        && (actual.g - expected.g).abs() <= 0.001
        && (actual.b - expected.b).abs() <= 0.001
        && (actual.a - expected.a).abs() <= 0.001
}

fn scene_has_solid_quad_color(scene: &Scene, expected: Color) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::Quad { background, .. }
                if matches!(background.paint, Paint::Solid(color) if color_close(color, expected))
        )
    })
}

fn scene_has_solid_text_color(scene: &Scene, expected: Color) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::Text { paint, .. }
                if matches!(paint.paint, Paint::Solid(color) if color_close(color, expected))
        )
    })
}

fn assert_px_close(actual: f32, expected: f32, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.5,
        "{context}: expected {expected}px, got {actual}px (delta {delta}px)"
    );
}

fn seed_snackbar(app: &mut TestHost, window: AppWindowId) -> fret_runtime::Model<ToastStore> {
    let store = app.models_mut().insert(ToastStore::default());
    let controller = SnackbarController::new(store.clone());
    {
        let mut action_host = UiActionHostAdapter { app };
        let _ = controller.show(
            &mut action_host,
            window,
            Snackbar::new("Update available")
                .supporting_text("Restart the app to apply the latest changes.")
                .action_id("Restart", CommandId::new("m3.snackbar.restart"))
                .duration(SnackbarDuration::Long)
                .test_id("m3-snackbar"),
        );
    }
    store
}

fn render_snackbar_frame(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    store: fret_runtime::Model<ToastStore>,
    capture_semantics: bool,
) -> Scene {
    run_overlay_frame_with_scene_scaled(
        ui,
        app,
        services,
        window,
        bounds(),
        1.0,
        capture_semantics,
        move |ui, app, services| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds(), "root", |cx| {
                vec![SnackbarHost::new(store).max_snackbars(1).into_element(cx)]
            })
        },
    )
}

fn render_snackbar_frame_with_style(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    store: fret_runtime::Model<ToastStore>,
    style: SnackbarStyle,
    capture_semantics: bool,
) -> Scene {
    run_overlay_frame_with_scene_scaled(
        ui,
        app,
        services,
        window,
        bounds(),
        1.0,
        capture_semantics,
        move |ui, app, services| {
            let style = style.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds(), "root", |cx| {
                vec![
                    SnackbarHost::new(store)
                        .max_snackbars(1)
                        .style(style)
                        .into_element(cx),
                ]
            })
        },
    )
}

#[test]
fn snackbar_uses_material_width_offset_live_region_and_dismiss_label() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    let store = seed_snackbar(&mut app, window);

    for _ in 0..64 {
        render_snackbar_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            store.clone(),
            true,
        );
    }

    let snackbar = layout_bounds_by_test_id(&ui, "m3-snackbar");
    assert_px_close(snackbar.size.width.0, 600.0, "settled snackbar width");
    assert_px_close(snackbar.origin.x.0, 130.0, "settled snackbar x");
    assert!(
        snackbar.size.height.0 >= 68.0,
        "two-line snackbar should keep Material 68dp minimum height; bounds={snackbar:?}"
    );

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected snackbar semantics snapshot");
    let close = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("m3-snackbar.close"))
        .expect("expected snackbar close button");
    assert_eq!(close.label.as_deref(), Some("Dismiss"));

    let viewport = snapshot
        .nodes
        .iter()
        .find(|node| node.flags.live == Some(SemanticsLive::Polite))
        .expect("expected polite snackbar live region");
    assert_eq!(viewport.label.as_deref(), Some("Alert"));
}

#[test]
fn snackbar_style_overrides_paint_and_layout_contract() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    let store = seed_snackbar(&mut app, window);

    let background = Color {
        r: 0.12,
        g: 0.18,
        b: 0.26,
        a: 1.0,
    };
    let text = Color {
        r: 0.94,
        g: 0.82,
        b: 0.42,
        a: 1.0,
    };
    let style = SnackbarStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(background))))
        .supporting_text_color(WidgetStateProperty::new(Some(ColorRef::Color(text))))
        .container_padding(WidgetStateProperty::new(Some(Edges {
            left: Px(20.0),
            right: Px(20.0),
            top: Px(12.0),
            bottom: Px(12.0),
        })))
        .container_corner_radius(WidgetStateProperty::new(Some(Px(10.0))))
        .two_line_min_height(WidgetStateProperty::new(Some(Px(96.0))));

    let mut scene = Scene::default();
    for _ in 0..64 {
        scene = render_snackbar_frame_with_style(
            &mut ui,
            &mut app,
            &mut services,
            window,
            store.clone(),
            style.clone(),
            true,
        );
    }

    assert!(
        scene_has_solid_quad_color(&scene, background),
        "expected SnackbarStyle container_background to paint a snackbar quad"
    );
    assert!(
        scene_has_solid_text_color(&scene, text),
        "expected SnackbarStyle supporting_text_color to paint snackbar text"
    );

    let snackbar = layout_bounds_by_test_id(&ui, "m3-snackbar");
    assert!(
        snackbar.size.height.0 >= 95.0,
        "two_line_min_height override should affect snackbar layout; bounds={snackbar:?}"
    );
}

#[test]
fn snackbar_enters_with_material_fade_scale_without_y_slide() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    let store = seed_snackbar(&mut app, window);

    let first_scene = render_snackbar_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        store.clone(),
        true,
    );

    assert!(
        scene_has_intermediate_opacity(&first_scene),
        "expected snackbar to fade on the first open frame"
    );
    assert!(
        scene_has_snackbar_scale(&first_scene),
        "expected snackbar to scale from the Material 0.8 closed size on the first open frame"
    );
    assert!(
        !scene_has_y_slide_without_scale(&first_scene),
        "Material Snackbar host motion should not use the generic Sonner Y-slide enter transform"
    );

    let root: NodeId = semantics_node_id_by_test_id(&ui, "m3-snackbar")
        .expect("expected snackbar semantics node after first open frame");
    assert!(
        ui.debug_node_visual_bounds(root).is_some(),
        "expected first-frame snackbar to stay measurable for hit testing"
    );
}

#[test]
fn snackbar_exits_with_material_fade_scale_without_y_slide() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    let store = seed_snackbar(&mut app, window);

    for _ in 0..64 {
        render_snackbar_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            store.clone(),
            true,
        );
    }

    let close_bounds = visual_bounds_by_test_id(&ui, "m3-snackbar.close");
    let click_at = Point::new(
        Px(close_bounds.origin.x.0 + close_bounds.size.width.0 * 0.5),
        Px(close_bounds.origin.y.0 + close_bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    app.advance_frame();
    let closing_scene = render_snackbar_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        store.clone(),
        true,
    );

    assert!(
        scene_has_intermediate_opacity(&closing_scene),
        "expected snackbar to fade on the first close frame"
    );
    assert!(
        scene_has_snackbar_scale(&closing_scene),
        "expected snackbar to scale toward the Material 0.8 closed size on close"
    );
    assert!(
        !scene_has_y_slide_without_scale(&closing_scene),
        "Material Snackbar close motion should not use the generic Sonner Y-slide transform"
    );
}
