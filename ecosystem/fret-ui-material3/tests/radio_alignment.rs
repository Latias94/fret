use fret_core::{
    AppWindowId, DrawOrder, Edges, NodeId, Point, PointerId, Px, Rect, Scene, SceneOp, Size,
    UiServices,
};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::{Theme, UiTree};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod support;

use support::events::{pointer_down, pointer_up};
use support::host::{FakeUiServices, TestHost};
use support::interaction_harness::{
    QuadGeomSig, SceneSig, scene_quad_geometry_signature, scene_signature,
};
use support::layout::{find_first_bounds_with_size, paint_alpha, with_padding};
use support::theme::apply_material_theme;
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
