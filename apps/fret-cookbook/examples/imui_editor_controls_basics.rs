use std::sync::Arc;

use fret::app::LocalState;
use fret::app::RenderContextAccess as _;
use fret::app::prelude::*;
use fret::imui::{
    editor::{
        self,
        composites::{
            InspectorPanel, InspectorPanelOptions, PropertyGrid, PropertyGridOptions,
            PropertyGroup, PropertyGroupOptions,
        },
        controls::{
            ColorEdit, ColorEditLocalStateExt, ColorEditOptions, DragValue, DragValueLocalStateExt,
            DragValueOptions, MiniSearchBox, MiniSearchBoxLocalStateExt, MiniSearchBoxOptions,
            NumericInput, NumericInputLocalStateExt, NumericInputOptions, NumericPresentation,
            TextAssistField, TextAssistFieldLocalStateExt, TextAssistFieldOptions,
            TextAssistFieldSurface, TextAssistItem, TextFieldOptions,
        },
        theme::{EditorThemePreset, install_editor_theme_preset},
    },
    prelude::*,
};
use fret::style::{Color, ColorRef, Space};

const TEST_ID_ROOT: &str = "cookbook.imui_editor_controls.root";
const TEST_ID_EXPOSURE: &str = "cookbook.imui_editor_controls.exposure";
const TEST_ID_ROUGHNESS: &str = "cookbook.imui_editor_controls.roughness";
const TEST_ID_TINT: &str = "cookbook.imui_editor_controls.tint";
const TEST_ID_SEARCH: &str = "cookbook.imui_editor_controls.search";
const TEST_ID_ASSIST: &str = "cookbook.imui_editor_controls.assist";
const TEST_ID_ASSIST_LIST: &str = "cookbook.imui_editor_controls.assist.list";
const TEST_ID_SUMMARY: &str = "cookbook.imui_editor_controls.summary";
const TEST_ID_INSPECTOR: &str = "cookbook.imui_editor_controls.inspector";
const TEST_ID_GRID: &str = "cookbook.imui_editor_controls.grid";

struct ImUiEditorControlsBasicsView {
    exposure: LocalState<f64>,
    roughness: LocalState<f64>,
    tint: LocalState<Color>,
    search: LocalState<String>,
    assist_query: LocalState<String>,
    assist_dismissed_query: LocalState<String>,
    assist_active_item_id: LocalState<Option<Arc<str>>>,
    assist_items: Arc<[TextAssistItem]>,
}

impl View for ImUiEditorControlsBasicsView {
    fn init(app: &mut App, _window: WindowId) -> Self {
        Self {
            exposure: app.local_state(1.25),
            roughness: app.local_state(0.42),
            tint: app.local_state(Color::from_srgb_hex_rgb(0x4f_8c_ff)),
            search: app.local_state(String::from("Transform")),
            assist_query: app.local_state(String::from("ca")),
            assist_dismissed_query: app.local_state(String::new()),
            assist_active_item_id: app.local_state(None::<Arc<str>>),
            assist_items: vec![
                TextAssistItem::new("camera", "Camera"),
                TextAssistItem::new("canvas", "Canvas"),
                TextAssistItem::new("curve", "Curve"),
            ]
            .into(),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = cx.theme_snapshot();
        let foreground = theme.color_token("foreground");
        let muted_foreground = theme.color_token("muted-foreground");

        let exposure = self.exposure.clone();
        let roughness = self.roughness.clone();
        let tint = self.tint.clone();
        let search = self.search.clone();
        let assist_query = self.assist_query.clone();
        let assist_dismissed_query = self.assist_dismissed_query.clone();
        let assist_active_item_id = self.assist_active_item_id.clone();
        let assist_items = self.assist_items.clone();

        let surface = ui::v_flex(move |cx| {
            let header = ui::v_flex(move |cx| {
                ui::children![
                    cx;
                    ui::text("Immediate-mode editor controls")
                        .font_semibold()
                        .text_base()
                        .text_color(ColorRef::Color(foreground)),
                    ui::text("App code imports these controls through fret::imui::editor.")
                        .text_sm()
                        .text_color(ColorRef::Color(muted_foreground))
                        .test_id(TEST_ID_SUMMARY),
                ]
            })
            .gap(Space::N1);

            let editor_panel = ui::v_flex(move |cx| {
                imui_in(cx, move |ui| {
                let exposure = exposure.clone();
                let roughness = roughness.clone();
                let tint = tint.clone();
                let search = search.clone();
                let assist_query = assist_query.clone();
                let assist_dismissed_query = assist_dismissed_query.clone();
                let assist_active_item_id = assist_active_item_id.clone();
                let assist_items = assist_items.clone();

                editor::inspector_panel(
                    ui,
                    InspectorPanel::new(None).options(InspectorPanelOptions {
                        title: Some(Arc::from("Object inspector")),
                        test_id: Some(Arc::from(TEST_ID_INSPECTOR)),
                        content_test_id: Some(Arc::from(
                            "cookbook.imui_editor_controls.inspector.content",
                        )),
                        ..Default::default()
                    }),
                    |_cx, _panel| Vec::new(),
                    move |cx, _panel| {
                        let exposure = exposure.clone();
                        let tint = tint.clone();
                        let search = search.clone();
                        let assist_query = assist_query.clone();
                        let assist_dismissed_query = assist_dismissed_query.clone();
                        let assist_active_item_id = assist_active_item_id.clone();
                        let assist_items = assist_items.clone();

                        vec![
                            PropertyGroup::new("Editor controls")
                                .options(PropertyGroupOptions {
                                    collapsible: false,
                                    ..Default::default()
                                })
                                .into_element(
                                    cx,
                                    |_cx| None,
                                    move |cx| {
                                        vec![
                                            PropertyGrid::new()
                                                .options(PropertyGridOptions {
                                                    test_id: Some(Arc::from(TEST_ID_GRID)),
                                                    ..Default::default()
                                                })
                                                .into_element(cx, move |cx, row_cx| {
                                                    let exposure_presentation =
                                                        NumericPresentation::<f64>::fixed_decimals(2)
                                                            .with_chrome_suffix(" EV");
                                                    let roughness_presentation =
                                                        NumericPresentation::<f64>::percent_0_1(0);

                                                    vec![
                                                        row_cx.row(
                                                            cx,
                                                            |cx| row_cx.label_text(cx, "Exposure"),
                                                            |cx| {
                                                                NumericInput::from_local_presentation(
                                                                    &exposure,
                                                                    exposure_presentation,
                                                                )
                                                                .options(NumericInputOptions {
                                                                    id_source: Some(Arc::from(
                                                                        "cookbook.imui_editor_controls.exposure",
                                                                    )),
                                                                    test_id: Some(Arc::from(
                                                                        TEST_ID_EXPOSURE,
                                                                    )),
                                                                    ..Default::default()
                                                                })
                                                                .into_element(cx)
                                                            },
                                                        ),
                                                        row_cx.row(
                                                            cx,
                                                            |cx| row_cx.label_text(cx, "Roughness"),
                                                            |cx| {
                                                                DragValue::from_local_presentation(
                                                                    &roughness,
                                                                    roughness_presentation,
                                                                )
                                                                .options(DragValueOptions {
                                                                    id_source: Some(Arc::from(
                                                                        "cookbook.imui_editor_controls.roughness",
                                                                    )),
                                                                    test_id: Some(Arc::from(
                                                                        TEST_ID_ROUGHNESS,
                                                                    )),
                                                                    ..Default::default()
                                                                })
                                                                .into_element(cx)
                                                            },
                                                        ),
                                                        row_cx.row(
                                                            cx,
                                                            |cx| row_cx.label_text(cx, "Tint"),
                                                            |cx| {
                                                                ColorEdit::new_local(&tint)
                                                                    .options(ColorEditOptions {
                                                                        id_source: Some(Arc::from(
                                                                            "cookbook.imui_editor_controls.tint",
                                                                        )),
                                                                        test_id: Some(Arc::from(
                                                                            TEST_ID_TINT,
                                                                        )),
                                                                        ..Default::default()
                                                                    })
                                                                    .into_element(cx)
                                                            },
                                                        ),
                                                        row_cx.row(
                                                            cx,
                                                            |cx| row_cx.label_text(cx, "Filter"),
                                                            |cx| {
                                                                MiniSearchBox::new_local(&search)
                                                                    .options(MiniSearchBoxOptions {
                                                                        test_id: Some(Arc::from(
                                                                            TEST_ID_SEARCH,
                                                                        )),
                                                                        ..Default::default()
                                                                    })
                                                                    .into_element(cx)
                                                            },
                                                        ),
                                                        row_cx.row(
                                                            cx,
                                                            |cx| row_cx.label_text(cx, "Asset"),
                                                            |cx| {
                                                                TextAssistField::new_local(
                                                                    &assist_query,
                                                                    &assist_dismissed_query,
                                                                    &assist_active_item_id,
                                                                    assist_items.clone(),
                                                                )
                                                                .options(TextAssistFieldOptions {
                                                                    field: TextFieldOptions {
                                                                        id_source: Some(Arc::from(
                                                                            "cookbook.imui_editor_controls.assist",
                                                                        )),
                                                                        placeholder: Some(Arc::from(
                                                                            "Jump to asset",
                                                                        )),
                                                                        test_id: Some(Arc::from(
                                                                            TEST_ID_ASSIST,
                                                                        )),
                                                                        ..Default::default()
                                                                    },
                                                                    surface:
                                                                        TextAssistFieldSurface::AnchoredOverlay,
                                                                    list_test_id: Some(Arc::from(
                                                                        TEST_ID_ASSIST_LIST,
                                                                    )),
                                                                    item_test_id_prefix: Some(
                                                                        Arc::from(
                                                                            "cookbook.imui_editor_controls.assist.item",
                                                                        ),
                                                                    ),
                                                                    ..Default::default()
                                                                })
                                                                .into_element(cx)
                                                            },
                                                        ),
                                                    ]
                                                }),
                                        ]
                                    },
                                ),
                        ]
                    },
                );
                })
            });

            ui::children![cx; header, editor_panel]
        })
        .w_full()
        .max_w(Px(560.0))
        .gap(Space::N3);

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, surface).into()
    }
}

fn install_editor_cookbook_defaults(app: &mut App) {
    fret_cookbook::install_cookbook_defaults(app);
    install_editor_theme_preset(app, EditorThemePreset::ImguiLikeDense);
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-imui-editor-controls-basics")
        .window("cookbook-imui-editor-controls-basics", (760.0, 520.0))
        .setup(install_editor_cookbook_defaults)
        .view::<ImUiEditorControlsBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
