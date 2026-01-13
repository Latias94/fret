#![allow(clippy::arc_with_non_send_sync)]

use super::*;

#[test]
fn canvas_hosts_text_and_releases_on_cleanup() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let paint = |p: &mut crate::canvas::CanvasPainter<'_>| {
        p.text(
            1,
            fret_core::DrawOrder(0),
            Point::new(Px(10.0), Px(10.0)),
            "hello",
            TextStyle::default(),
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            crate::canvas::CanvasTextConstraints::default(),
            p.scale_factor(),
        );
    };

    let mut root: Option<NodeId> = None;
    for pass in 0..2 {
        let node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "canvas-hosts-text",
            |cx| vec![cx.canvas(crate::element::CanvasProps::default(), paint)],
        );
        root.get_or_insert(node);
        ui.set_root(node);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert!(
            scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::Text { .. }))
        );

        if pass == 0 {
            assert_eq!(services.prepare_calls, 1);
            assert_eq!(services.release_calls, 0);
        } else {
            assert_eq!(services.prepare_calls, 1, "text blob should be cached");
        }

        app.advance_frame();
    }

    ui.cleanup_subtree(&mut services, root.expect("root"));
    assert_eq!(
        services.release_calls, 1,
        "canvas should release hosted resources on cleanup"
    );
}

#[test]
fn canvas_hosts_path_and_releases_on_cleanup() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let paint = |p: &mut crate::canvas::CanvasPainter<'_>| {
        let commands = [
            fret_core::PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            fret_core::PathCommand::LineTo(Point::new(Px(10.0), Px(10.0))),
            fret_core::PathCommand::Close,
        ];
        p.path(
            2,
            fret_core::DrawOrder(0),
            Point::new(Px(10.0), Px(10.0)),
            &commands,
            fret_core::PathStyle::Fill(fret_core::FillStyle::default()),
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            p.scale_factor(),
        );
    };

    let mut root: Option<NodeId> = None;
    for pass in 0..2 {
        let node = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "canvas-hosts-path",
            |cx| vec![cx.canvas(crate::element::CanvasProps::default(), paint)],
        );
        root.get_or_insert(node);
        ui.set_root(node);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert!(
            scene
                .ops()
                .iter()
                .any(|op| matches!(op, SceneOp::Path { .. }))
        );

        if pass == 0 {
            assert_eq!(services.path_prepare_calls, 1);
            assert_eq!(services.path_release_calls, 0);
        } else {
            assert_eq!(services.path_prepare_calls, 1, "path should be cached");
        }

        app.advance_frame();
    }

    ui.cleanup_subtree(&mut services, root.expect("root"));
    assert_eq!(
        services.path_release_calls, 1,
        "canvas should release hosted path resources on cleanup"
    );
}
