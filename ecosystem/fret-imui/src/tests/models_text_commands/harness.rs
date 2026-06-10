use super::*;

pub(super) struct InputTextCommandScenario {
    window: AppWindowId,
    bounds: Rect,
    pub(super) ui: UiTree<TestHost>,
    pub(super) app: TestHost,
    pub(super) services: FakeTextService,
}

impl InputTextCommandScenario {
    pub(super) fn new() -> Self {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(140.0)),
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
                    let _ = ui.input_text_model_with_options(&model, options);
                })
            },
        )
    }

    pub(super) fn click_input(&mut self, root: fret_core::NodeId) {
        let at = first_child_point(&self.ui, root);
        click_at(&mut self.ui, &mut self.app, &mut self.services, at);
    }

    pub(super) fn clear_effects(&mut self) {
        self.app.effects.clear();
    }

    pub(super) fn commands_for_window(&self) -> Vec<fret_runtime::CommandId> {
        self.app
            .effects
            .iter()
            .filter_map(|effect| match effect {
                Effect::Command {
                    window: Some(target_window),
                    command,
                } if *target_window == self.window => Some(command.clone()),
                _ => None,
            })
            .collect()
    }

    pub(super) fn model_text(&self, model: &fret_runtime::Model<String>) -> Option<String> {
        self.app.models().get_cloned(model)
    }
}
