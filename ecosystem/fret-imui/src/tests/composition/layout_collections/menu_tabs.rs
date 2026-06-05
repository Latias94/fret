use super::*;

#[test]
fn menu_bar_helper_arranges_triggers_horizontally_and_stamps_menubar_semantics() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(140.0)),
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
        "imui-menu-bar",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.menu_bar_with_options(
                    fret_ui_kit::imui::MenuBarOptions {
                        test_id: Some(Arc::from("imui-menu-bar.root")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.begin_menu_with_options(
                            "file",
                            "File",
                            fret_ui_kit::imui::BeginMenuOptions {
                                test_id: Some(Arc::from("imui-menu-bar.file")),
                                ..Default::default()
                            },
                            |_ui| {},
                        );
                        let _ = ui.begin_menu_with_options(
                            "edit",
                            "Edit",
                            fret_ui_kit::imui::BeginMenuOptions {
                                test_id: Some(Arc::from("imui-menu-bar.edit")),
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

    let file = bounds_for_test_id(&ui, "imui-menu-bar.file");
    let edit = bounds_for_test_id(&ui, "imui-menu-bar.edit");
    assert!(edit.origin.x.0 > file.origin.x.0 + file.size.width.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let menubar = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-menu-bar.root"))
        .expect("menubar semantics node");
    assert_eq!(menubar.role, SemanticsRole::MenuBar);
}

#[test]
fn tab_bar_helper_arranges_tabs_horizontally_and_stamps_tab_semantics() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(180.0)),
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
        "imui-tab-bar",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.tab_bar_with_options(
                    "workspace",
                    fret_ui_kit::imui::TabBarOptions {
                        test_id: Some(Arc::from("imui-tab-bar.root")),
                        ..Default::default()
                    },
                    |tabs| {
                        tabs.begin_tab_item_with_options(
                            "scene",
                            "Scene",
                            fret_ui_kit::imui::TabItemOptions {
                                default_selected: true,
                                test_id: Some(Arc::from("imui-tab-bar.scene")),
                                panel_test_id: Some(Arc::from("imui-tab-bar.scene.panel")),
                                ..Default::default()
                            },
                            |ui| {
                                ui.text("Scene Panel");
                            },
                        );
                        tabs.begin_tab_item_with_options(
                            "inspector",
                            "Inspector",
                            fret_ui_kit::imui::TabItemOptions {
                                test_id: Some(Arc::from("imui-tab-bar.inspector")),
                                panel_test_id: Some(Arc::from("imui-tab-bar.inspector.panel")),
                                ..Default::default()
                            },
                            |ui| {
                                ui.text("Inspector Panel");
                            },
                        );
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scene = bounds_for_test_id(&ui, "imui-tab-bar.scene");
    let inspector = bounds_for_test_id(&ui, "imui-tab-bar.inspector");
    assert!(inspector.origin.x.0 > scene.origin.x.0 + scene.size.width.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let tab_list = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-tab-bar.root"))
        .expect("tab list semantics node");
    assert_eq!(tab_list.role, SemanticsRole::TabList);

    let scene_tab = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-tab-bar.scene"))
        .expect("scene tab semantics node");
    assert_eq!(scene_tab.role, SemanticsRole::Tab);
    assert!(scene_tab.flags.selected);

    let scene_panel = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-tab-bar.scene.panel"))
        .expect("scene panel semantics node");
    assert_eq!(scene_panel.role, SemanticsRole::TabPanel);
    assert_eq!(scene_panel.label.as_deref(), Some("Scene"));

    assert!(
        snap.nodes
            .iter()
            .all(|node| node.test_id.as_deref() != Some("imui-tab-bar.inspector.panel")),
        "expected inactive tab panel to stay out of the semantics tree"
    );
}
