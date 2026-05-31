use std::sync::Arc;

use fret_core::{
    AppWindowId, DrawOrder, Edges, Event, KeyCode, Modifiers, NodeId, Point, PointerId, Px, Rect,
    Scene, SceneOp, Size, Transform2D, UiServices,
};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::{Theme, UiTree};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod interaction_harness;
mod support;

use interaction_harness::{QuadGeomSig, SceneSig, scene_quad_geometry_signature, scene_signature};
use support::events::{key_down, key_up, pointer_down, pointer_up};
use support::goldens::run_overlay_frame;
use support::host::{FakeUiServices, TestHost};
use support::layout::{find_first_bounds_with_size, paint_alpha, with_padding};
use support::theme::apply_material_theme;

// Choice/control and action-surface interaction regressions.

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
