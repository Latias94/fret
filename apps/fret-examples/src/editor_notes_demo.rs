use std::sync::Arc;

use fret::app::prelude::*;
use fret::{shadcn, Defaults, FretApp};
use fret_app::{CommandId, Model};
use fret_core::Px;
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle, TextProps};
use fret_ui::Theme;
use fret_ui_editor::composites::{
    InspectorPanel, InspectorPanelOptions, PropertyGrid, PropertyGroup, PropertyGroupOptions,
    PropertyRow,
};
use fret_ui_editor::controls::{
    EditorTextSelectionBehavior, TextField, TextFieldBlurBehavior, TextFieldOptions,
    TextFieldOutcome,
};
use fret_ui_kit::declarative::{ElementContextThemeExt as _, ModelWatchExt as _};
use fret_ui_kit::{ColorRef, Space};

const ENV_EDITOR_PRESET: &str = "FRET_EDITOR_NOTES_DEMO_PRESET";
const HOST_BASE_COLOR: shadcn::themes::ShadcnBaseColor = shadcn::themes::ShadcnBaseColor::Slate;
const HOST_DEFAULT_SCHEME: shadcn::themes::ShadcnColorScheme =
    shadcn::themes::ShadcnColorScheme::Dark;
const TEST_ID_ROOT: &str = "editor-notes-demo.root";
const TEST_ID_SELECTION: &str = "editor-notes-demo.selection";
const TEST_ID_SELECT_MATERIAL: &str = "editor-notes-demo.selection.material";
const TEST_ID_SELECT_LIGHT: &str = "editor-notes-demo.selection.light";
const TEST_ID_SELECT_CAMERA: &str = "editor-notes-demo.selection.camera";
const TEST_ID_INSPECTOR: &str = "editor-notes-demo.inspector";
const TEST_ID_INSPECTOR_CONTENT: &str = "editor-notes-demo.inspector.content";
const TEST_ID_NAME: &str = "editor-notes-demo.inspector.name";
const TEST_ID_NOTES: &str = "editor-notes-demo.inspector.notes";
const TEST_ID_NOTES_COMMITTED: &str = "editor-notes-demo.inspector.notes.committed";
const TEST_ID_NOTES_OUTCOME: &str = "editor-notes-demo.inspector.notes.outcome";

mod act {
    fret::actions!([
        SelectMaterial = "editor_notes_demo.select.material.v1",
        SelectLight = "editor_notes_demo.select.light.v1",
        SelectCamera = "editor_notes_demo.select.camera.v1"
    ]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum EditorAssetSelection {
    #[default]
    Material,
    Light,
    Camera,
}

#[derive(Clone)]
struct EditorAssetState {
    selection: EditorAssetSelection,
    title: Arc<str>,
    subtitle: Arc<str>,
    name_id_source: Arc<str>,
    notes_id_source: Arc<str>,
    name_model: Model<String>,
    notes_model: Model<String>,
    notes_outcome_model: Model<String>,
}

struct EditorNotesDemoView {
    assets: Arc<[EditorAssetState]>,
}

fn install_editor_notes_demo_theme(app: &mut App) {
    shadcn::themes::apply_shadcn_new_york(app, HOST_BASE_COLOR, HOST_DEFAULT_SCHEME);
    fret_ui_editor::theme::install_editor_theme_preset_v1(
        app,
        crate::editor_theme_preset_from_env(ENV_EDITOR_PRESET)
            .unwrap_or(fret_ui_editor::theme::EditorThemePresetV1::Default),
    );
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("editor-notes-demo")
        .window("editor_notes_demo", (1080.0, 720.0))
        .defaults(Defaults {
            shadcn: false,
            ..Defaults::desktop_app()
        })
        .setup((
            install_editor_notes_demo_theme,
            fret_icons_lucide::app::install,
        ))
        .view::<EditorNotesDemoView>()?
        .run()
        .map_err(anyhow::Error::from)
}

impl View for EditorNotesDemoView {
    fn init(app: &mut App, _window: WindowId) -> Self {
        let assets: Arc<[EditorAssetState]> = vec![
            make_asset_state(
                app,
                EditorAssetSelection::Material,
                "Material",
                "Surface authoring metadata",
                "Weathered Steel",
                "Review roughness breakup.\nValidate clear-coat against the hero shot.",
            ),
            make_asset_state(
                app,
                EditorAssetSelection::Light,
                "Key Light",
                "Shot review notes",
                "Key Light A",
                "Keep the rim subtle on close-ups.\nRevisit exposure after fog tuning.",
            ),
            make_asset_state(
                app,
                EditorAssetSelection::Camera,
                "Camera",
                "Sequence continuity notes",
                "ShotCam_Main",
                "Preserve this draft across blur.\nCommit only when the sequence note is ready.",
            ),
        ]
        .into();

        Self { assets }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let selected = cx.state().local_init(|| EditorAssetSelection::Material);
        cx.actions()
            .local_set::<act::SelectMaterial, EditorAssetSelection>(
                &selected,
                EditorAssetSelection::Material,
            );
        cx.actions()
            .local_set::<act::SelectLight, EditorAssetSelection>(
                &selected,
                EditorAssetSelection::Light,
            );
        cx.actions()
            .local_set::<act::SelectCamera, EditorAssetSelection>(
                &selected,
                EditorAssetSelection::Camera,
            );

        let theme = Theme::global(&*cx.app).snapshot();
        let selected = cx.state().watch(&selected).layout().value_or_default();
        let asset = self.asset(selected).clone();
        let committed_notes = cx
            .watch_model(&asset.notes_model)
            .paint()
            .value_or_default();
        let notes_outcome = cx
            .watch_model(&asset.notes_outcome_model)
            .paint()
            .value_or_else(|| "Idle".to_string());

        let selection_panel = {
            let header = shadcn::CardHeader::new([
                ui::v_flex(|cx| {
                    ui::children![
                        cx;
                        shadcn::CardTitle::new("Scene outline"),
                        shadcn::CardDescription::new(
                            "Select an editor-owned surface, then blur Notes to Name to keep a local draft alive.",
                        ),
                    ]
                })
                .gap(Space::N1)
                .into_element(cx),
            ]);

            let material_button = selection_button(
                cx,
                "Material",
                selected == EditorAssetSelection::Material,
                act::SelectMaterial.into(),
                TEST_ID_SELECT_MATERIAL,
            );
            let light_button = selection_button(
                cx,
                "Key Light",
                selected == EditorAssetSelection::Light,
                act::SelectLight.into(),
                TEST_ID_SELECT_LIGHT,
            );
            let camera_button = selection_button(
                cx,
                "Camera",
                selected == EditorAssetSelection::Camera,
                act::SelectCamera.into(),
                TEST_ID_SELECT_CAMERA,
            );

            let body = ui::v_flex(|_cx| [material_button, light_button, camera_button])
                .gap(Space::N2)
                .into_element(cx);

            shadcn::Card::new(ui::children![
                cx;
                header,
                shadcn::CardContent::new([body]),
            ])
            .ui()
            .w_px(Px(240.0))
            .into_element(cx)
            .test_id(TEST_ID_SELECTION)
        };

        let inspector = render_inspector_panel(
            cx,
            asset,
            committed_line_count_label(&committed_notes),
            notes_outcome,
        );
        let content = ui::h_flex(|_cx| [selection_panel, inspector])
            .gap(Space::N4)
            .size_full()
            .into_element(cx);

        ui::container(|_cx| [content])
            .bg(ColorRef::Color(theme.color_token("background")))
            .p(Space::N4)
            .size_full()
            .into_element(cx)
            .test_id(TEST_ID_ROOT)
            .into()
    }
}

impl EditorNotesDemoView {
    fn asset(&self, selection: EditorAssetSelection) -> &EditorAssetState {
        self.assets
            .iter()
            .find(|asset| asset.selection == selection)
            .unwrap_or_else(|| &self.assets[0])
    }
}

fn make_asset_state(
    app: &mut App,
    selection: EditorAssetSelection,
    title: &'static str,
    subtitle: &'static str,
    name: &'static str,
    notes: &'static str,
) -> EditorAssetState {
    let key = match selection {
        EditorAssetSelection::Material => "material",
        EditorAssetSelection::Light => "light",
        EditorAssetSelection::Camera => "camera",
    };

    EditorAssetState {
        selection,
        title: Arc::from(title),
        subtitle: Arc::from(subtitle),
        name_id_source: Arc::from(format!("editor-notes-demo.asset.{key}.name")),
        notes_id_source: Arc::from(format!("editor-notes-demo.asset.{key}.notes")),
        name_model: app.models_mut().insert(name.to_string()),
        notes_model: app.models_mut().insert(notes.to_string()),
        notes_outcome_model: app.models_mut().insert("Idle".to_string()),
    }
}

fn selection_button(
    cx: &mut AppUi<'_, '_>,
    label: &'static str,
    selected: bool,
    action: CommandId,
    test_id: &'static str,
) -> AnyElement {
    let variant = if selected {
        shadcn::ButtonVariant::Default
    } else {
        shadcn::ButtonVariant::Secondary
    };
    shadcn::Button::new(label)
        .variant(variant)
        .on_click(action)
        .test_id(test_id)
        .ui()
        .w_full()
        .into_element(cx)
}

fn render_inspector_panel(
    cx: &mut AppUi<'_, '_>,
    asset: EditorAssetState,
    committed_label: String,
    outcome_label: String,
) -> AnyElement {
    let subtitle = asset.subtitle.clone();
    let title = asset.title.clone();
    let notes_outcome_model = asset.notes_outcome_model.clone();

    InspectorPanel::new(None)
        .options(InspectorPanelOptions {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            title: Some(title.clone()),
            test_id: Some(Arc::from(TEST_ID_INSPECTOR)),
            content_test_id: Some(Arc::from(TEST_ID_INSPECTOR_CONTENT)),
            ..Default::default()
        })
        .into_element(
            cx,
            move |cx, _panel_cx| {
                let muted = cx.theme_snapshot().color_token("muted-foreground");
                let subtitle_text = cx.text_props(TextProps {
                    layout: Default::default(),
                    text: subtitle.clone(),
                    style: None,
                    color: Some(muted),
                    align: fret_core::TextAlign::Start,
                    wrap: fret_core::TextWrap::Word,
                    overflow: fret_core::TextOverflow::Clip,
                    ink_overflow: Default::default(),
                });
                vec![subtitle_text]
            },
            move |cx, _panel_cx| {
                vec![PropertyGroup::new("Metadata")
                    .options(PropertyGroupOptions {
                        test_id: Some(Arc::from("editor-notes-demo.inspector.group.metadata")),
                        ..Default::default()
                    })
                    .into_element(
                        cx,
                        |_cx| None,
                        move |cx| {
                            vec![PropertyGrid::new().into_element(cx, move |cx, row_cx| {
                                let mut rows = Vec::new();

                                rows.push(row_cx.row_with(
                                    cx,
                                    PropertyRow::new().options(row_cx.row_options.clone()),
                                    |cx| cx.text("Name"),
                                    |cx| {
                                        TextField::new(asset.name_model.clone())
                                            .options(TextFieldOptions {
                                                id_source: Some(asset.name_id_source.clone()),
                                                selection_behavior:
                                                    EditorTextSelectionBehavior::SelectAllOnFocus,
                                                clear_button: true,
                                                test_id: Some(Arc::from(TEST_ID_NAME)),
                                                ..Default::default()
                                            })
                                            .into_element(cx)
                                    },
                                    |_cx| None,
                                ));

                                rows.push(row_cx.row_with(
                                    cx,
                                    PropertyRow::new().options(row_cx.row_options.clone()),
                                    |cx| cx.text("Notes"),
                                    |cx| {
                                        TextField::new(asset.notes_model.clone())
                                            .on_outcome(Some(Arc::new({
                                                let notes_outcome_model =
                                                    notes_outcome_model.clone();
                                                move |host, action_cx, outcome: TextFieldOutcome| {
                                                    let next = match outcome {
                                                        TextFieldOutcome::Committed => "Committed",
                                                        TextFieldOutcome::Canceled => "Canceled",
                                                    };
                                                    let _ = host.models_mut().update(
                                                        &notes_outcome_model,
                                                        |text: &mut String| {
                                                            text.clear();
                                                            text.push_str(next);
                                                        },
                                                    );
                                                    host.request_redraw(action_cx.window);
                                                }
                                            })))
                                            .options(TextFieldOptions {
                                                id_source: Some(asset.notes_id_source.clone()),
                                                multiline: true,
                                                stable_line_boxes: true,
                                                min_height: Some(Px(120.0)),
                                                blur_behavior: TextFieldBlurBehavior::PreserveDraft,
                                                test_id: Some(Arc::from(TEST_ID_NOTES)),
                                                ..Default::default()
                                            })
                                            .into_element(cx)
                                    },
                                    |_cx| None,
                                ));

                                rows.push(row_cx.row_with(
                                    cx,
                                    PropertyRow::new().options(row_cx.row_options.clone()),
                                    |cx| cx.text("Committed"),
                                    |cx| {
                                        cx.text(committed_label.clone())
                                            .test_id(TEST_ID_NOTES_COMMITTED)
                                    },
                                    |_cx| None,
                                ));

                                rows.push(row_cx.row_with(
                                    cx,
                                    PropertyRow::new().options(row_cx.row_options.clone()),
                                    |cx| cx.text("Last action"),
                                    |cx| {
                                        cx.text(outcome_label.clone())
                                            .test_id(TEST_ID_NOTES_OUTCOME)
                                    },
                                    |_cx| None,
                                ));

                                rows
                            })]
                        },
                    )]
            },
        )
}

fn committed_line_count_label(text: &str) -> String {
    let lines = text.lines().count();
    match lines {
        0 => "No committed notes".to_string(),
        1 => "1 line committed".to_string(),
        n => format!("{n} lines committed"),
    }
}
