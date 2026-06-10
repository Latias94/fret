use std::sync::Arc;

use fret::app::prelude::*;
use fret::imui::{
    editor::{
        self,
        composites::{PropertyRow, PropertyRowOptions},
        controls::{
            ColorEdit, ColorEditOptions, DragValue, DragValueOptions, MiniSearchBox,
            MiniSearchBoxOptions, NumericInput, NumericInputOptions, NumericPresentation,
            TextAssistField, TextAssistFieldOptions, TextAssistFieldSurface, TextAssistItem,
        },
        theme::{EditorThemePresetV1, install_editor_theme_preset_v1},
    },
    prelude::*,
};
use fret::style::Space;
use fret_core::Color;
use fret_runtime::Model;

const TEST_ID_ROOT: &str = "cookbook.imui_editor_controls.root";
const TEST_ID_EXPOSURE: &str = "cookbook.imui_editor_controls.exposure";
const TEST_ID_ROUGHNESS: &str = "cookbook.imui_editor_controls.roughness";
const TEST_ID_TINT: &str = "cookbook.imui_editor_controls.tint";
const TEST_ID_SEARCH: &str = "cookbook.imui_editor_controls.search";
const TEST_ID_ASSIST: &str = "cookbook.imui_editor_controls.assist";
const TEST_ID_ASSIST_LIST: &str = "cookbook.imui_editor_controls.assist.list";
const TEST_ID_PROPERTY_ROW: &str = "cookbook.imui_editor_controls.property_row";

struct ImUiEditorControlsBasicsView {
    exposure: Model<f64>,
    roughness: Model<f64>,
    tint: Model<Color>,
    search: Model<String>,
    assist_query: Model<String>,
    assist_dismissed_query: Model<String>,
    assist_active_item_id: Model<Option<Arc<str>>>,
    assist_items: Arc<[TextAssistItem]>,
}

impl View for ImUiEditorControlsBasicsView {
    fn init(app: &mut App, _window: WindowId) -> Self {
        Self {
            exposure: app.models_mut().insert(1.25),
            roughness: app.models_mut().insert(0.42),
            tint: app
                .models_mut()
                .insert(Color::from_srgb_hex_rgb(0x4f_8c_ff)),
            search: app.models_mut().insert(String::from("Transform")),
            assist_query: app.models_mut().insert(String::from("ca")),
            assist_dismissed_query: app.models_mut().insert(String::new()),
            assist_active_item_id: app.models_mut().insert(None::<Arc<str>>),
            assist_items: vec![
                TextAssistItem::new("camera", "Camera"),
                TextAssistItem::new("canvas", "Canvas"),
                TextAssistItem::new("curve", "Curve"),
            ]
            .into(),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        ui::v_flex(|cx| {
            let controls = cx.column(fret_ui::element::ColumnProps::default(), |cx| {
                imui_raw(cx, |ui| {
                    ui.text("Editor controls and composites");

                    editor::property_row(
                        ui,
                        PropertyRow::new().options(PropertyRowOptions {
                            test_id: Some(Arc::from(TEST_ID_PROPERTY_ROW)),
                            ..Default::default()
                        }),
                        |cx| cx.text("Surface"),
                        |cx| cx.text("Property row composite"),
                        |_cx| None,
                    );

                    let exposure_presentation =
                        NumericPresentation::<f64>::fixed_decimals(2).with_chrome_suffix(" EV");
                    editor::numeric_input(
                        ui,
                        NumericInput::from_presentation(
                            self.exposure.clone(),
                            exposure_presentation,
                        )
                        .options(NumericInputOptions {
                            id_source: Some(Arc::from("cookbook.imui_editor_controls.exposure")),
                            test_id: Some(Arc::from(TEST_ID_EXPOSURE)),
                            ..Default::default()
                        }),
                    );

                    let roughness_presentation = NumericPresentation::<f64>::percent_0_1(0);
                    editor::drag_value(
                        ui,
                        DragValue::from_presentation(
                            self.roughness.clone(),
                            roughness_presentation,
                        )
                        .options(DragValueOptions {
                            id_source: Some(Arc::from("cookbook.imui_editor_controls.roughness")),
                            test_id: Some(Arc::from(TEST_ID_ROUGHNESS)),
                            ..Default::default()
                        }),
                    );

                    editor::color_edit(
                        ui,
                        ColorEdit::new(self.tint.clone()).options(ColorEditOptions {
                            id_source: Some(Arc::from("cookbook.imui_editor_controls.tint")),
                            test_id: Some(Arc::from(TEST_ID_TINT)),
                            ..Default::default()
                        }),
                    );

                    editor::mini_search_box(
                        ui,
                        MiniSearchBox::new(self.search.clone()).options(MiniSearchBoxOptions {
                            test_id: Some(Arc::from(TEST_ID_SEARCH)),
                            ..Default::default()
                        }),
                    );

                    editor::text_assist_field(
                        ui,
                        TextAssistField::new(
                            self.assist_query.clone(),
                            self.assist_dismissed_query.clone(),
                            self.assist_active_item_id.clone(),
                            self.assist_items.clone(),
                        )
                        .options(TextAssistFieldOptions {
                            field: editor::controls::TextFieldOptions {
                                id_source: Some(Arc::from("cookbook.imui_editor_controls.assist")),
                                placeholder: Some(Arc::from("Jump to asset")),
                                test_id: Some(Arc::from(TEST_ID_ASSIST)),
                                ..Default::default()
                            },
                            surface: TextAssistFieldSurface::AnchoredOverlay,
                            list_test_id: Some(Arc::from(TEST_ID_ASSIST_LIST)),
                            item_test_id_prefix: Some(Arc::from(
                                "cookbook.imui_editor_controls.assist.item",
                            )),
                            ..Default::default()
                        }),
                    );
                })
            });

            ui::children![
                cx;
                shadcn::Label::new("Immediate-mode editor controls"),
                cx.text("App code imports these controls through fret::imui::editor.")
                    .test_id("cookbook.imui_editor_controls.summary"),
                controls,
            ]
        })
        .size_full()
        .gap(Space::N4)
        .test_id(TEST_ID_ROOT)
        .into_element_in(cx)
        .into()
    }
}

fn install_editor_cookbook_defaults(app: &mut App) {
    fret_cookbook::install_cookbook_defaults(app);
    install_editor_theme_preset_v1(app, EditorThemePresetV1::ImguiLikeDense);
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-imui-editor-controls-basics")
        .window("cookbook-imui-editor-controls-basics", (760.0, 520.0))
        .setup(install_editor_cookbook_defaults)
        .view::<ImUiEditorControlsBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
