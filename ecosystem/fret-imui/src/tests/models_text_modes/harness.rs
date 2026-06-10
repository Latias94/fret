use super::*;

pub(super) const READ_ONLY_TEST_ID: &str = "imui-input-text-read-only";
pub(super) const SELECT_ALL_TEST_ID: &str = "imui-input-text-select-all-on-focus";
pub(super) const SELECT_ALL_FIRST_TEST_ID: &str = "imui-select-all-first";
pub(super) const SELECT_ALL_SECOND_TEST_ID: &str = "imui-select-all-second";
pub(super) const PASSWORD_TEST_ID: &str = "imui-input-text-password";

pub(super) struct InputTextModeScenario {
    window: AppWindowId,
    bounds: Rect,
    ui: UiTree<TestHost>,
    app: TestHost,
    services: FakeTextService,
}

impl InputTextModeScenario {
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

    pub(super) fn render_input(
        &mut self,
        root_name: &str,
        model: &fret_runtime::Model<String>,
        options: InputTextOptions,
        changed: Option<Rc<Cell<bool>>>,
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
                    let response = ui.input_text_model_with_options(&model, options);
                    if let Some(changed) = changed {
                        changed.set(response.changed());
                    }
                })
            },
        )
    }

    pub(super) fn render_select_all_pair(
        &mut self,
        root_name: &str,
        first: &fret_runtime::Model<String>,
        second: &fret_runtime::Model<String>,
    ) -> fret_core::NodeId {
        let first = first.clone();
        let second = second.clone();
        run_frame(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            self.window,
            self.bounds,
            root_name,
            |cx| {
                crate::imui_raw(cx, |ui| {
                    ui.column(|ui| {
                        let _ = ui.input_text_model_with_options(
                            &first,
                            InputTextOptions {
                                select_all_on_focus: true,
                                test_id: Some(Arc::from(SELECT_ALL_FIRST_TEST_ID)),
                                ..Default::default()
                            },
                        );
                        let _ = ui.input_text_model_with_options(
                            &second,
                            InputTextOptions {
                                test_id: Some(Arc::from(SELECT_ALL_SECOND_TEST_ID)),
                                ..Default::default()
                            },
                        );
                    });
                })
            },
        )
    }

    pub(super) fn first_child_point(&self, root: fret_core::NodeId) -> Point {
        first_child_point(&self.ui, root)
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

    pub(super) fn dispatch_all_timers(&mut self) -> usize {
        dispatch_all_timers(&mut self.ui, &mut self.app, &mut self.services)
    }

    pub(super) fn drain_effects(&mut self) -> Vec<Effect> {
        self.app.effects.drain(..).collect()
    }

    pub(super) fn effects(&self) -> &[Effect] {
        &self.app.effects
    }

    pub(super) fn window(&self) -> AppWindowId {
        self.window
    }

    pub(super) fn dispatch_command(&mut self, command: &fret_runtime::CommandId) -> bool {
        self.ui
            .dispatch_command(&mut self.app, &mut self.services, command)
    }

    pub(super) fn is_command_available(&mut self, command: &fret_runtime::CommandId) -> bool {
        self.ui.is_command_available(&mut self.app, command)
    }

    pub(super) fn clear_prepared_text(&mut self) {
        self.services.prepared.clear();
    }

    pub(super) fn paint_all(&mut self) {
        let mut scene = fret_core::Scene::default();
        self.ui.paint_all(
            &mut self.app,
            &mut self.services,
            self.bounds,
            &mut scene,
            1.0,
        );
    }

    pub(super) fn prepared_texts(&self) -> &[String] {
        &self.services.prepared
    }

    pub(super) fn model_text(&self, model: &fret_runtime::Model<String>) -> Option<String> {
        self.app.models().get_cloned(model)
    }
}
