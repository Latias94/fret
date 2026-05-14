use super::*;

#[test]
fn label_identity_explicit_id_controls_hide_suffixes_from_visible_labels() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(460.0), Px(260.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.combo_with_options(
                    "identity-combo",
                    "Mode##combo-label",
                    "Alpha",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-label-identity.combo")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Alpha",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-label-identity.combo.alpha")),
                                ..Default::default()
                            },
                        );
                    },
                );

                ui.menu_bar_with_options(
                    MenuBarOptions {
                        test_id: Some(Arc::from("imui-label-identity.explicit-menu.root")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.begin_menu_with_options(
                            "file",
                            "File###file-menu-label",
                            BeginMenuOptions {
                                test_id: Some(Arc::from("imui-label-identity.explicit-menu.file")),
                                ..Default::default()
                            },
                            |_ui| {},
                        );
                    },
                );

                ui.tab_bar_with_options(
                    "identity-tabs",
                    TabBarOptions {
                        test_id: Some(Arc::from("imui-label-identity.tabs.root")),
                        ..Default::default()
                    },
                    |tabs| {
                        tabs.begin_tab_item_with_options(
                            "scene",
                            "Scene##scene-tab-label",
                            TabItemOptions {
                                default_selected: true,
                                test_id: Some(Arc::from("imui-label-identity.tabs.scene")),
                                panel_test_id: Some(Arc::from(
                                    "imui-label-identity.tabs.scene.panel",
                                )),
                                ..Default::default()
                            },
                            |ui| {
                                ui.text("Scene Panel");
                            },
                        );
                    },
                );

                let _ = ui.collapsing_header_with_options(
                    "identity-header",
                    "Header###header-label",
                    CollapsingHeaderOptions {
                        test_id: Some(Arc::from("imui-label-identity.header.root")),
                        header_test_id: Some(Arc::from("imui-label-identity.header")),
                        ..Default::default()
                    },
                    |ui| {
                        ui.text("Header Panel");
                    },
                );
                let _ = ui.tree_node_with_options(
                    "identity-tree",
                    "Tree##tree-label",
                    TreeNodeOptions {
                        test_id: Some(Arc::from("imui-label-identity.tree")),
                        ..Default::default()
                    },
                    |ui| {
                        ui.text("Tree Panel");
                    },
                );
                ui.separator_text_with_options(
                    "Section##section-label",
                    SeparatorTextOptions {
                        test_id: Some(Arc::from("imui-label-identity.separator")),
                    },
                );
            });
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-label-identity-explicit-id-controls",
        |cx| render(cx),
    );

    assert!(services.prepared.iter().any(|text| text == "Mode"));
    assert!(services.prepared.iter().any(|text| text == "File"));
    assert!(services.prepared.iter().any(|text| text == "Scene"));
    assert!(services.prepared.iter().any(|text| text == "Header"));
    assert!(services.prepared.iter().any(|text| text == "Tree"));
    assert!(services.prepared.iter().any(|text| text == "Section"));
    assert!(
        !services.prepared.iter().any(|text| text.contains("##")
            || text.contains("###")
            || text.contains("combo-label")
            || text.contains("file-menu-label")
            || text.contains("scene-tab-label")
            || text.contains("header-label")
            || text.contains("tree-label")
            || text.contains("section-label")),
        "label identity suffixes should not be painted: {:?}",
        services.prepared
    );
}
