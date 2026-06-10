use super::*;
use fret_runtime::Model;
use fret_ui::element::Elements;

const COMBO_TEST_ID: &str = "imui-combo-selectable";
const POPUP_ID: &str = "imui-combo-selectable-popup";
const FIRST_OPTION_TEST_ID: &str = "imui-combo-selectable.option.0";
const ITEMS: [&str; 2] = ["Alpha", "Beta"];

pub(super) struct ComboDirectSelectionScenario {
    window: AppWindowId,
    bounds: Rect,
    ui: UiTree<TestHost>,
    app: TestHost,
    services: FakeTextService,
    selected_model: Model<Option<Arc<str>>>,
    selected: Rc<RefCell<Option<Arc<str>>>>,
}

impl ComboDirectSelectionScenario {
    pub(super) fn new() -> Self {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(220.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let selected_model = app.models_mut().insert(None::<Arc<str>>);

        Self {
            window,
            bounds,
            ui,
            app,
            services: FakeTextService::default(),
            selected_model,
            selected: Rc::new(RefCell::new(None::<Arc<str>>)),
        }
    }

    pub(super) fn render_frame(&mut self) {
        let selected_model = self.selected_model.clone();
        let selected_out = self.selected.clone();
        let _root = run_frame(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            self.window,
            self.bounds,
            COMBO_TEST_ID,
            |cx| render_selectable_combo(cx, selected_model, selected_out),
        );
    }

    pub(super) fn click_trigger(&mut self) {
        let trigger = point_for_test_id(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            self.bounds,
            COMBO_TEST_ID,
        );
        click_at(&mut self.ui, &mut self.app, &mut self.services, trigger);
    }

    pub(super) fn click_first_option(&mut self) {
        let first_option = point_for_test_id(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            self.bounds,
            FIRST_OPTION_TEST_ID,
        );
        click_at(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            first_option,
        );
    }

    pub(super) fn advance_frame(&mut self) {
        self.app.advance_frame();
    }

    pub(super) fn selected(&self) -> Option<Arc<str>> {
        self.selected.borrow().clone()
    }

    pub(super) fn has_first_option(&mut self) -> bool {
        has_test_id(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            self.bounds,
            FIRST_OPTION_TEST_ID,
        )
    }
}

fn render_selectable_combo(
    cx: &mut ElementContext<'_, TestHost>,
    selected_model: Model<Option<Arc<str>>>,
    selected_out: Rc<RefCell<Option<Arc<str>>>>,
) -> Elements {
    crate::imui_raw(cx, |ui| {
        let current = ui
            .cx_mut()
            .app
            .models()
            .get_cloned(&selected_model)
            .unwrap_or(None);
        let preview = current
            .clone()
            .unwrap_or_else(|| Arc::<str>::from("Select..."));
        let current_for_rows = current.clone();
        let model_for_rows = selected_model.clone();
        let _ = ui.combo_with_options(
            POPUP_ID,
            "Mode",
            preview,
            ComboOptions {
                test_id: Some(Arc::from(COMBO_TEST_ID)),
                ..Default::default()
            },
            move |ui| {
                for (index, item) in ITEMS.iter().enumerate() {
                    let is_selected = current_for_rows
                        .as_ref()
                        .is_some_and(|value| value.as_ref() == *item);
                    let row = ui.selectable_with_options(
                        *item,
                        SelectableOptions {
                            selected: is_selected,
                            test_id: Some(Arc::from(format!(
                                "imui-combo-selectable.option.{index}"
                            ))),
                            ..Default::default()
                        },
                    );
                    if row.clicked() {
                        let next = Some(Arc::<str>::from(*item));
                        let _ = ui
                            .cx_mut()
                            .app
                            .models_mut()
                            .update(&model_for_rows, |value| *value = next.clone());
                        ui.close_popup(POPUP_ID);
                    }
                }
            },
        );
        let now = ui
            .cx_mut()
            .app
            .models()
            .get_cloned(&selected_model)
            .unwrap_or(None);
        selected_out.replace(now);
    })
}
