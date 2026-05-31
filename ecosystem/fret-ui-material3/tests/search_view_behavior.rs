//! Focused interaction regression tests for Material 3 SearchView.

use fret_core::{
    AppWindowId, Color, Corners, Edges, KeyCode, NodeId, Paint, Px, Rect, Scene, SceneOp,
    SemanticsRole, Size, UiServices,
};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_kit::{ColorRef, OverlayController, OverlayStackEntryKind, WidgetStateProperty};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{SearchBarStyle, SearchView, SearchViewPresentation, SearchViewStyle};

mod support;

use support::events::{key_down, key_up};
use support::goldens::{run_overlay_frame, run_overlay_frame_with_scene_scaled};
use support::host::{FakeUiServices, TestHost};
use support::theme::apply_material_theme;

fn semantics_test_id_layout_bounds(ui: &UiTree<TestHost>, id: &str) -> Rect {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find_map(|node| (node.test_id.as_deref() == Some(id)).then_some(node.id))
        })
        .and_then(|node| {
            ui.debug_node_bounds(node)
                .or_else(|| ui.debug_node_visual_bounds(node))
        })
        .unwrap_or_else(|| panic!("expected live layout bounds for test_id {id}"))
}

fn scene_has_intermediate_opacity(scene: &Scene) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushOpacity { opacity } if *opacity > 0.01 && *opacity < 0.99
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

fn scene_has_full_screen_search_expand_transform(scene: &Scene) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushTransform { transform }
                if transform.a > 0.5
                    && transform.a <= 1.01
                    && transform.d > 0.05
                    && transform.d < 0.99
                    && (transform.a - transform.d).abs() > 0.02
        )
    })
}

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

    for _ in 0..64 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
    }

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

#[test]
fn search_view_docked_overlay_fades_and_expands_on_open_close_frames() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let open = app.models_mut().insert(false);
    let query = app.models_mut().insert(String::new());
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
                    .max_height(Px(240.0))
                    .into_element(cx, |cx| {
                        vec![
                            cx.text("Alpha"),
                            cx.text("Bravo"),
                            cx.text("Charlie"),
                            cx.text("Delta"),
                        ]
                    }),
            ]
        })
    };

    for _ in 0..64 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
    }

    let _ = app.models_mut().update(&open, |v| *v = true);
    let first_open_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    let first_open_overlay = semantics_test_id_layout_bounds(&ui, "m3-search-view.overlay");

    assert!(
        scene_has_intermediate_opacity(&first_open_scene),
        "expected docked SearchView overlay to fade on the first open frame"
    );
    assert!(
        first_open_overlay.size.height.0 > 4.0 && first_open_overlay.size.height.0 < 236.0,
        "expected docked SearchView overlay height to expand on the first open frame, got {}",
        first_open_overlay.size.height.0
    );

    for _ in 0..64 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
    }

    let _ = app.models_mut().update(&open, |v| *v = false);
    let first_close_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    let first_close_overlay = semantics_test_id_layout_bounds(&ui, "m3-search-view.overlay");

    assert!(
        scene_has_intermediate_opacity(&first_close_scene),
        "expected docked SearchView overlay to fade on the first close frame"
    );
    assert!(
        first_close_overlay.size.height.0 > 4.0 && first_close_overlay.size.height.0 < 236.0,
        "expected docked SearchView overlay height to shrink on the first close frame, got {}",
        first_close_overlay.size.height.0
    );
}

#[test]
fn search_view_style_overrides_docked_overlay_paint_contract() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let open = app.models_mut().insert(true);
    let query = app.models_mut().insert(String::from("alpha"));
    let open_model = open.clone();
    let query_model = query.clone();
    let container = Color {
        r: 0.12,
        g: 0.15,
        b: 0.20,
        a: 1.0,
    };
    let divider = Color {
        r: 0.94,
        g: 0.74,
        b: 0.36,
        a: 1.0,
    };
    let style = SearchViewStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(container))))
        .divider_color(WidgetStateProperty::new(Some(ColorRef::Color(divider))))
        .docked_container_corner_radii(WidgetStateProperty::new(Some(Corners::all(Px(16.0)))))
        .body_padding(WidgetStateProperty::new(Some(Edges::all(Px(20.0)))));

    let render = move |ui: &mut UiTree<TestHost>,
                       app: &mut TestHost,
                       services: &mut dyn UiServices| {
        let open = open_model.clone();
        let query = query_model.clone();
        let style = style.clone();
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", move |cx| {
            vec![
                SearchView::new(open, query)
                    .style(style)
                    .test_id("m3-search-view")
                    .placeholder("Search")
                    .max_height(Px(240.0))
                    .into_element(cx, |cx| vec![cx.text("Result alpha")]),
            ]
        })
    };

    let mut scene = Scene::default();
    for _ in 0..64 {
        scene = run_overlay_frame_with_scene_scaled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            1.0,
            true,
            |ui, app, services| render(ui, app, services),
        );
    }

    assert!(
        scene_has_solid_quad_color(&scene, container),
        "container_background override should paint the docked SearchView overlay"
    );
    assert!(
        scene_has_solid_quad_color(&scene, divider),
        "divider_color override should paint the docked SearchView divider"
    );
}

#[test]
fn search_view_style_overrides_full_screen_header_layout_contract() {
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
    let style = SearchViewStyle::default()
        .full_screen_header_container_height(WidgetStateProperty::new(Some(Px(92.0))))
        .header_style(
            SearchBarStyle::default().container_height(WidgetStateProperty::new(Some(Px(64.0)))),
        );

    let render = move |ui: &mut UiTree<TestHost>,
                       app: &mut TestHost,
                       services: &mut dyn UiServices| {
        let open = open_model.clone();
        let query = query_model.clone();
        let style = style.clone();
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", move |cx| {
            vec![
                SearchView::new(open, query)
                    .style(style)
                    .test_id("m3-search-view")
                    .placeholder("Search")
                    .presentation(SearchViewPresentation::FullScreen)
                    .into_element(cx, |cx| vec![cx.text("Result alpha")]),
            ]
        })
    };

    for _ in 0..64 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
    }

    let header_slot = semantics_test_id_layout_bounds(&ui, "m3-search-view.overlay.header-slot");
    assert!(
        (header_slot.size.height.0 - 92.0).abs() <= 0.5,
        "full_screen_header_container_height override should affect header slot layout; bounds={header_slot:?}"
    );

    let header_chrome =
        semantics_test_id_layout_bounds(&ui, "m3-search-view.overlay.header.chrome");
    assert!(
        (header_chrome.size.height.0 - 64.0).abs() <= 0.5,
        "header_style container_height override should affect full-screen header SearchBar; bounds={header_chrome:?}"
    );
}

#[test]
fn search_view_full_screen_overlay_expands_from_input_geometry() {
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

    let open = app.models_mut().insert(false);
    let query = app.models_mut().insert(String::new());
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

    let _ = app.models_mut().update(&open, |v| *v = true);
    let first_open_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    assert!(
        scene_has_intermediate_opacity(&first_open_scene),
        "expected full-screen SearchView overlay to fade on the first open frame"
    );
    assert!(
        scene_has_full_screen_search_expand_transform(&first_open_scene),
        "expected full-screen SearchView overlay to expand from the collapsed input geometry"
    );

    for _ in 0..64 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
    }

    let _ = app.models_mut().update(&open, |v| *v = false);
    let first_close_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );

    assert!(
        scene_has_intermediate_opacity(&first_close_scene),
        "expected full-screen SearchView overlay to fade on the first close frame"
    );
    assert!(
        scene_has_full_screen_search_expand_transform(&first_close_scene),
        "expected full-screen SearchView overlay to collapse toward the input geometry"
    );
}

#[test]
fn search_view_inputs_control_overlay_semantics() {
    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(420.0)),
        );

        let open = app.models_mut().insert(true);
        let query = app.models_mut().insert(String::from("alpha"));
        let open_model = open.clone();
        let query_model = query.clone();

        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                let open = open_model.clone();
                let query = query_model.clone();
                fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "root",
                    move |cx| {
                        vec![
                            SearchView::new(open, query)
                                .test_id("m3-search-view")
                                .placeholder("Search")
                                .into_element(cx, |cx| vec![cx.text("Result alpha")]),
                        ]
                    },
                )
            };

        for capture in [false, false, true] {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                capture,
                |ui, app, services| render(ui, app, services),
            );
        }

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let input = snap
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some("m3-search-view"))
            .expect("docked SearchView input node");
        let overlay = snap
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some("m3-search-view.overlay"))
            .expect("docked SearchView overlay node");

        assert_eq!(overlay.role, SemanticsRole::Panel);
        assert!(
            input.flags.expanded,
            "docked SearchView input should report expanded=true while open"
        );
        assert!(
            input.controls.contains(&overlay.id),
            "docked SearchView input should control the overlay panel"
        );
        assert!(
            overlay.labelled_by.contains(&input.id),
            "docked SearchView overlay should be labelled by the input"
        );
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

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

        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                let open = open_model.clone();
                let query = query_model.clone();
                fret_ui::declarative::render_root(
                    ui,
                    app,
                    services,
                    window,
                    bounds,
                    "root",
                    move |cx| {
                        vec![
                            SearchView::new(open, query)
                                .test_id("m3-search-view")
                                .placeholder("Search")
                                .presentation(SearchViewPresentation::FullScreen)
                                .into_element(cx, |cx| vec![cx.text("Result alpha")]),
                        ]
                    },
                )
            };

        for capture in [false, false, true] {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                capture,
                |ui, app, services| render(ui, app, services),
            );
        }

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let header = snap
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some("m3-search-view.overlay.header"))
            .expect("full-screen SearchView header input node");
        let overlay = snap
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some("m3-search-view.overlay"))
            .expect("full-screen SearchView overlay node");

        assert_eq!(overlay.role, SemanticsRole::Dialog);
        assert!(
            header.flags.expanded,
            "full-screen SearchView header should report expanded=true while open"
        );
        assert!(
            header.controls.contains(&overlay.id),
            "full-screen SearchView header should control the dialog"
        );
        assert!(
            overlay.labelled_by.contains(&header.id),
            "full-screen SearchView dialog should be labelled by the header input"
        );
    }
}
