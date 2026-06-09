use super::*;
use fret_ui_kit::imui::{
    BeginMenuOptions, BeginSubmenuOptions, MenuBarOptions, TabBarOptions, TabItemOptions,
};

pub(super) const TOP_LEVEL_TRIGGER_IDS: [&str; 3] = [
    "imui-geometry.menu.file",
    "imui-geometry.tabs.scene",
    "imui-geometry.tabs.inspector",
];

pub(super) struct MenuTabGeometryScenario {
    window: AppWindowId,
    bounds: Rect,
    ui: UiTree<TestHost>,
    app: TestHost,
    services: FakeTextService,
}

impl MenuTabGeometryScenario {
    pub(super) fn new() -> Self {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(520.0), Px(320.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());

        Self {
            window,
            bounds,
            ui,
            app,
            services: FakeTextService::default(),
        }
    }

    pub(super) fn render_frame(&mut self) {
        let _root = run_frame(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            self.window,
            self.bounds,
            "imui-menu-tab-geometry",
            render_menu_tab_geometry_surface,
        );
    }

    pub(super) fn baseline_for(&mut self, test_ids: &[&'static str]) -> Vec<(&'static str, Rect)> {
        test_ids
            .iter()
            .map(|test_id| (*test_id, self.bounds_for(test_id)))
            .collect()
    }

    pub(super) fn assert_top_level_trigger_state_bounds_stable(
        &mut self,
        baseline: &[(&str, Rect)],
        test_id: &str,
    ) {
        let before = baseline_bounds(baseline, test_id);

        self.hover(test_id, before);
        self.focus(test_id, before);
        self.press(test_id, before, false);
    }

    pub(super) fn select_inspector_tab_and_assert_bounds(&mut self, baseline: &[(&str, Rect)]) {
        let inspector_before = baseline_bounds(baseline, "imui-geometry.tabs.inspector");
        self.click_rect_center(inspector_before);
        self.advance_and_render();

        for test_id in ["imui-geometry.tabs.scene", "imui-geometry.tabs.inspector"] {
            let before = baseline_bounds(baseline, test_id);
            self.assert_bounds_for_state(test_id, before, "selected");
        }
    }

    pub(super) fn open_file_menu_and_assert_bounds(&mut self, baseline: &[(&str, Rect)]) {
        let file_before = baseline_bounds(baseline, "imui-geometry.menu.file");
        self.click_rect_center(file_before);
        self.advance_and_render();
        self.assert_bounds_for_state("imui-geometry.menu.file", file_before, "open");
    }

    pub(super) fn bounds_for(&mut self, test_id: &str) -> Rect {
        control_bounds_for_test_id(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            self.bounds,
            test_id,
        )
    }

    pub(super) fn assert_submenu_state_bounds_stable(&mut self, test_id: &str, before: Rect) {
        self.hover(test_id, before);
        self.focus(test_id, before);
        self.press(test_id, before, true);
        self.advance_and_render();
        self.assert_bounds_for_state(test_id, before, "open");
    }

    fn hover(&mut self, test_id: &str, before: Rect) {
        pointer_move_at(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            center_of_rect(before),
            MouseButtons::default(),
        );
        self.advance_and_render();
        self.assert_bounds_for_state(test_id, before, "hover");
    }

    fn focus(&mut self, test_id: &str, before: Rect) {
        let node = node_for_test_id(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            self.bounds,
            test_id,
        );
        self.ui.set_focus(Some(node));
        self.advance_and_render();
        self.assert_bounds_for_state(test_id, before, "focus");
    }

    fn press(&mut self, test_id: &str, before: Rect, is_click: bool) {
        pointer_down_at(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            center_of_rect(before),
        );
        self.advance_and_render();
        self.assert_bounds_for_state(test_id, before, "pressed");
        pointer_up_at_with_is_click(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            center_of_rect(before),
            is_click,
        );
    }

    fn click_rect_center(&mut self, rect: Rect) {
        click_at(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            center_of_rect(rect),
        );
    }

    fn advance_and_render(&mut self) {
        self.app.advance_frame();
        self.render_frame();
    }

    fn assert_bounds_for_state(&mut self, test_id: &str, before: Rect, state: &str) {
        let after = self.bounds_for(test_id);
        assert_same_rect(test_id, before, after, state);
    }
}

fn render_menu_tab_geometry_surface(cx: &mut ElementContext<'_, TestHost>) -> crate::Elements {
    crate::imui_raw(cx, |ui| {
        ui.vertical(|ui| {
            ui.menu_bar_with_options(
                MenuBarOptions {
                    test_id: Some(Arc::from("imui-geometry.menu.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        BeginMenuOptions {
                            test_id: Some(Arc::from("imui-geometry.menu.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                BeginSubmenuOptions {
                                    test_id: Some(Arc::from("imui-geometry.menu.file.recent")),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Project",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-geometry.menu.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            let _ = ui.menu_item_with_options(
                                "Open",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-geometry.menu.file.open")),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                },
            );

            ui.tab_bar_with_options(
                "geometry-tabs",
                TabBarOptions {
                    test_id: Some(Arc::from("imui-geometry.tabs.root")),
                    ..Default::default()
                },
                |tabs| {
                    tabs.begin_tab_item_with_options(
                        "scene",
                        "Scene",
                        TabItemOptions {
                            default_selected: true,
                            test_id: Some(Arc::from("imui-geometry.tabs.scene")),
                            panel_test_id: Some(Arc::from("imui-geometry.tabs.scene.panel")),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Scene Panel");
                        },
                    );
                    tabs.begin_tab_item_with_options(
                        "inspector",
                        "Inspector",
                        TabItemOptions {
                            test_id: Some(Arc::from("imui-geometry.tabs.inspector")),
                            panel_test_id: Some(Arc::from("imui-geometry.tabs.inspector.panel")),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Inspector Panel");
                        },
                    );
                },
            );
        });
    })
}
