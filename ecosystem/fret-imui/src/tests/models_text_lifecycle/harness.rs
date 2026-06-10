use super::*;
use fret_ui_kit::imui::{ButtonOptions, InputTextOptions};

pub(super) const STABLE_BOUNDS_TEST_ID: &str = "imui-input-text-stable-bounds";
pub(super) const STABLE_BOUNDS_SIBLING_TEST_ID: &str = "imui-input-text-stable-bounds.sibling";
pub(super) const LIFECYCLE_TEST_ID: &str = "imui-input-text-lifecycle";
pub(super) const LIFECYCLE_BLUR_TARGET_TEST_ID: &str = "imui-input-text-lifecycle.blur-target";

pub(super) struct InputTextLifecycleScenario {
    window: AppWindowId,
    bounds: Rect,
    pub(super) ui: UiTree<TestHost>,
    pub(super) app: TestHost,
    pub(super) services: FakeTextService,
}

impl InputTextLifecycleScenario {
    pub(super) fn new(width: f32, height: f32) -> Self {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(width), Px(height)),
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

    pub(super) fn insert_text_model(
        &mut self,
        value: impl Into<String>,
    ) -> fret_runtime::Model<String> {
        self.app.models_mut().insert(value.into())
    }

    pub(super) fn render_stable_bounds_frame(
        &mut self,
        root_name: &str,
        model: &fret_runtime::Model<String>,
    ) -> fret_core::NodeId {
        let model = model.clone();
        run_frame(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            self.window,
            self.bounds,
            root_name,
            |cx| {
                crate::imui_raw(cx, |ui| {
                    ui.vertical(|ui| {
                        let _ = ui.input_text_model_with_options(
                            &model,
                            InputTextOptions {
                                test_id: Some(Arc::from(STABLE_BOUNDS_TEST_ID)),
                                ..Default::default()
                            },
                        );
                        let _ = ui.button_with_options(
                            "Sibling",
                            ButtonOptions {
                                test_id: Some(Arc::from(STABLE_BOUNDS_SIBLING_TEST_ID)),
                                ..Default::default()
                            },
                        );
                    });
                })
            },
        )
    }

    pub(super) fn render_lifecycle_frame(
        &mut self,
        root_name: &str,
        model: &fret_runtime::Model<String>,
        activated_out: &Rc<Cell<bool>>,
        deactivated_out: &Rc<Cell<bool>>,
        edited_out: &Rc<Cell<bool>>,
        after_edit_out: &Rc<Cell<bool>>,
        text_out: &Rc<RefCell<String>>,
    ) -> fret_core::NodeId {
        let model = model.clone();
        run_frame(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            self.window,
            self.bounds,
            root_name,
            |cx| {
                crate::imui_raw(cx, |ui| {
                    ui.vertical(|ui| {
                        let resp = ui.input_text_model_with_options(
                            &model,
                            InputTextOptions {
                                test_id: Some(Arc::from(LIFECYCLE_TEST_ID)),
                                ..Default::default()
                            },
                        );
                        activated_out.set(resp.activated());
                        deactivated_out.set(resp.deactivated());
                        edited_out.set(resp.edited());
                        after_edit_out.set(resp.deactivated_after_edit());

                        let _ = ui.button_with_options(
                            "Blur target",
                            ButtonOptions {
                                test_id: Some(Arc::from(LIFECYCLE_BLUR_TARGET_TEST_ID)),
                                ..Default::default()
                            },
                        );
                    });

                    let current = ui
                        .cx_mut()
                        .app
                        .models()
                        .get_cloned(&model)
                        .unwrap_or_default();
                    text_out.replace(current);
                })
            },
        )
    }

    pub(super) fn bounds_for_test_id(&mut self, test_id: &str) -> Rect {
        let node = node_for_test_id(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            self.bounds,
            test_id,
        );
        self.ui.debug_node_bounds(node).expect("input bounds")
    }

    pub(super) fn point_for_test_id(&mut self, test_id: &str) -> Point {
        point_for_test_id(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            self.bounds,
            test_id,
        )
    }

    pub(super) fn click_at(&mut self, at: Point) {
        click_at(&mut self.ui, &mut self.app, &mut self.services, at);
    }

    pub(super) fn text_input(&mut self, text: &str) {
        text_input_event(&mut self.ui, &mut self.app, &mut self.services, text);
    }

    pub(super) fn advance_frame(&mut self) {
        self.app.advance_frame();
    }

    pub(super) fn model_text(&self, model: &fret_runtime::Model<String>) -> Option<String> {
        self.app.models().get_cloned(model)
    }
}
