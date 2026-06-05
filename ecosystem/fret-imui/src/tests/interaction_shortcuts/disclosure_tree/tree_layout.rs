use super::*;

#[test]
fn tree_node_children_stack_vertically_inside_open_parents() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tree-node-vertical-stacking",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.tree_node_with_options(
                    "scene",
                    "Scene",
                    fret_ui_kit::imui::TreeNodeOptions {
                        default_open: true,
                        test_id: Some(Arc::from("imui-tree-node-stack.scene")),
                        content_test_id: Some(Arc::from("imui-tree-node-stack.scene.content")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.tree_node_with_options(
                            "geometry",
                            "Geometry",
                            fret_ui_kit::imui::TreeNodeOptions {
                                default_open: true,
                                level: 2,
                                test_id: Some(Arc::from("imui-tree-node-stack.geometry")),
                                content_test_id: Some(Arc::from(
                                    "imui-tree-node-stack.geometry.content",
                                )),
                                ..Default::default()
                            },
                            |ui| {
                                let _ = ui.tree_node_with_options(
                                    "cube",
                                    "Cube",
                                    fret_ui_kit::imui::TreeNodeOptions {
                                        leaf: true,
                                        level: 3,
                                        test_id: Some(Arc::from("imui-tree-node-stack.cube")),
                                        ..Default::default()
                                    },
                                    |_ui| {},
                                );
                                let _ = ui.tree_node_with_options(
                                    "key-light",
                                    "Key light",
                                    fret_ui_kit::imui::TreeNodeOptions {
                                        leaf: true,
                                        level: 3,
                                        test_id: Some(Arc::from("imui-tree-node-stack.key-light")),
                                        ..Default::default()
                                    },
                                    |_ui| {},
                                );
                            },
                        );
                        let _ = ui.tree_node_with_options(
                            "postfx",
                            "Post FX",
                            fret_ui_kit::imui::TreeNodeOptions {
                                leaf: true,
                                level: 2,
                                test_id: Some(Arc::from("imui-tree-node-stack.postfx")),
                                ..Default::default()
                            },
                            |_ui| {},
                        );
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let geometry_bounds = bounds_for_test_id(&ui, "imui-tree-node-stack.geometry");
    let postfx_bounds = bounds_for_test_id(&ui, "imui-tree-node-stack.postfx");
    let cube_bounds = bounds_for_test_id(&ui, "imui-tree-node-stack.cube");
    let key_light_bounds = bounds_for_test_id(&ui, "imui-tree-node-stack.key-light");

    assert!(
        postfx_bounds.origin.y.0 >= geometry_bounds.origin.y.0 + geometry_bounds.size.height.0,
        "expected Post FX to land below Geometry, got geometry={geometry_bounds:?} postfx={postfx_bounds:?}"
    );
    assert!(
        key_light_bounds.origin.y.0 >= cube_bounds.origin.y.0 + cube_bounds.size.height.0,
        "expected Key light to land below Cube, got cube={cube_bounds:?} key_light={key_light_bounds:?}"
    );
}
