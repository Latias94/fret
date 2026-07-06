use std::sync::Arc;

use fret::app::prelude::*;
use fret::{Defaults, FretApp, shadcn};
use fret_app::{CommandId, Model};
use fret_core::Px;
use fret_runtime::ModelStore;
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle};
use fret_ui_editor::composites::{
    InspectorPanel, InspectorPanelOptions, PropertyGrid, PropertyGroup, PropertyGroupOptions,
    PropertyRow,
};
use fret_ui_editor::controls::{
    EditorTextSelectionBehavior, EditorThemePresetPicker, EditorThemePresetPickerOptions,
    TextField, TextFieldBlurBehavior, TextFieldDraftController, TextFieldOptions, TextFieldOutcome,
};
use fret_ui_editor::theme::EditorThemePresetV1;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::{IntoUiElementInExt as _, Space};
use fret_workspace::WorkspaceFrame;

const ENV_EDITOR_PRESET: &str = "FRET_EDITOR_NOTES_DEMO_PRESET";
const HOST_BASE_COLOR: shadcn::themes::ShadcnBaseColor = shadcn::themes::ShadcnBaseColor::Slate;
const HOST_DEFAULT_SCHEME: shadcn::themes::ShadcnColorScheme =
    shadcn::themes::ShadcnColorScheme::Dark;
const TEST_ID_ROOT: &str = "editor-notes-demo.root";
const TEST_ID_LEFT_RAIL: &str = "editor-notes-demo.left-rail";
const TEST_ID_SELECTION: &str = "editor-notes-demo.selection";
const TEST_ID_COLLECTION: &str = "editor-notes-demo.collection";
const TEST_ID_COLLECTION_SUMMARY: &str = "editor-notes-demo.collection.summary";
const TEST_ID_COLLECTION_LIST: &str = "editor-notes-demo.collection.list";
const TEST_ID_SELECT_MATERIAL: &str = "editor-notes-demo.selection.material";
const TEST_ID_SELECT_LIGHT: &str = "editor-notes-demo.selection.light";
const TEST_ID_SELECT_CAMERA: &str = "editor-notes-demo.selection.camera";
const TEST_ID_CENTER: &str = "editor-notes-demo.center";
const TEST_ID_CENTER_PREVIEW: &str = "editor-notes-demo.center.preview";
const TEST_ID_INSPECTOR: &str = "editor-notes-demo.inspector";
const TEST_ID_INSPECTOR_CONTENT: &str = "editor-notes-demo.inspector.content";
const TEST_ID_RIGHT_RAIL: &str = "editor-notes-demo.right-rail";
const TEST_ID_NAME: &str = "editor-notes-demo.inspector.name";
const TEST_ID_NOTES: &str = "editor-notes-demo.inspector.notes";
const TEST_ID_NOTES_COMMITTED: &str = "editor-notes-demo.inspector.notes.committed";
const TEST_ID_NOTES_OUTCOME: &str = "editor-notes-demo.inspector.notes.outcome";
const TEST_ID_NOTES_DRAFT_STATUS: &str = "editor-notes-demo.inspector.notes.draft-status";
const TEST_ID_DRAFT_COMMIT_COMMAND: &str = "editor-notes-demo.inspector.notes.commit-draft";
const TEST_ID_DRAFT_DISCARD_COMMAND: &str = "editor-notes-demo.inspector.notes.discard-draft";
const TEST_ID_SUMMARY_COMMAND: &str = "editor-notes-demo.inspector.summary-command";
const TEST_ID_SUMMARY_STATUS: &str = "editor-notes-demo.inspector.summary-status";
const TEST_ID_THEME_PRESET_PICKER: &str = "editor-notes-demo.inspector.theme-preset";

pub(crate) mod act {
    fret::actions!([
        SelectMaterial = "editor_notes_demo.select.material.v1",
        SelectLight = "editor_notes_demo.select.light.v1",
        SelectCamera = "editor_notes_demo.select.camera.v1"
    ]);
}

struct EditorNotesModelOwner<'a> {
    models: &'a mut ModelStore,
}

impl<'a> EditorNotesModelOwner<'a> {
    fn new(models: &'a mut ModelStore) -> Self {
        Self { models }
    }

    fn set_text(&mut self, model: &Model<String>, value: impl Into<String>) -> bool {
        let value = value.into();
        self.models
            .update(model, |slot| {
                *slot = value;
                true
            })
            .unwrap_or(false)
    }
}

#[derive(Clone)]
struct EditorAssetModels {
    name: Model<String>,
    notes: Model<String>,
    notes_outcome: Model<String>,
    summary_status: Model<String>,
}

impl EditorAssetModels {
    fn new(models: &mut ModelStore, title: &str, name: &str, notes: &str) -> Self {
        Self {
            name: models.insert(name.to_string()),
            notes: models.insert(notes.to_string()),
            notes_outcome: models.insert("Idle".to_string()),
            summary_status: models.insert(format!("Ready to copy summary for {title}.")),
        }
    }

    fn name_text_model(&self) -> Model<String> {
        self.name.clone()
    }

    fn notes_text_model(&self) -> Model<String> {
        self.notes.clone()
    }

    fn set_notes_outcome(&self, models: &mut ModelStore, value: impl Into<String>) -> bool {
        EditorNotesModelOwner::new(models).set_text(&self.notes_outcome, value)
    }

    fn set_summary_status(&self, models: &mut ModelStore, value: impl Into<String>) -> bool {
        EditorNotesModelOwner::new(models).set_text(&self.summary_status, value)
    }
}

pub(crate) struct EditorAssetSnapshot {
    pub(crate) name_value: String,
    pub(crate) committed_notes: String,
    pub(crate) notes_outcome: String,
    pub(crate) summary_status: String,
}

#[derive(Clone)]
pub(crate) struct EditorThemePresetBinding {
    preset: Model<EditorThemePresetV1>,
}

impl EditorThemePresetBinding {
    pub(crate) fn new(app: &mut App) -> Self {
        let theme_preset = fret_ui_editor::theme::installed_editor_theme_preset_v1(app)
            .unwrap_or(EditorThemePresetV1::Default);
        Self {
            preset: app.models_mut().insert(theme_preset),
        }
    }

    fn picker_model(&self) -> Model<EditorThemePresetV1> {
        self.preset.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum EditorAssetSelection {
    #[default]
    Material,
    Light,
    Camera,
}

#[derive(Clone)]
pub(crate) struct EditorAssetState {
    selection: EditorAssetSelection,
    title: Arc<str>,
    subtitle: Arc<str>,
    name_id_source: Arc<str>,
    notes_id_source: Arc<str>,
    models: EditorAssetModels,
}

pub(crate) struct EditorNotesDemoView {
    assets: Arc<[EditorAssetState]>,
    theme: EditorThemePresetBinding,
}

pub(crate) fn install_editor_notes_demo_theme(app: &mut App) {
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
        Self {
            assets: default_editor_assets(app),
            theme: EditorThemePresetBinding::new(app),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let selected = cx.state().local_init(|| EditorAssetSelection::Material);
        cx.actions()
            .local(&selected)
            .set::<act::SelectMaterial>(EditorAssetSelection::Material);
        cx.actions()
            .local(&selected)
            .set::<act::SelectLight>(EditorAssetSelection::Light);
        cx.actions()
            .local(&selected)
            .set::<act::SelectCamera>(EditorAssetSelection::Camera);

        let theme = cx.theme_snapshot();
        let selected = cx.state().watch(&selected).layout().value_or_default();
        let asset = self.asset(selected).clone();
        let snapshot = editor_asset_paint_snapshot(cx, &asset);

        let selection_panel = render_selection_panel(cx, selected);

        let center = render_center_panel(
            cx,
            asset.clone(),
            snapshot.name_value,
            snapshot.committed_notes.clone(),
            snapshot.notes_outcome.clone(),
            "WorkspaceFrame owns the outer shell slots; fret-ui-editor owns the reusable inspector content.",
            "This center region is app-local content, while both side regions are mounted through the existing workspace shell seam.",
        );
        let inspector = render_inspector_panel(
            cx,
            asset,
            self.theme.clone(),
            committed_line_count_label(&snapshot.committed_notes),
            snapshot.notes_outcome,
            snapshot.summary_status,
        );
        let left_rail = ui::container(|_cx| [selection_panel])
            .w_px(Px(256.0))
            .flex_shrink_0()
            .h_full()
            .into_element_in(cx)
            .test_id(TEST_ID_LEFT_RAIL);
        let right_rail = ui::container(|_cx| [inspector])
            .w_px(Px(360.0))
            .flex_shrink_0()
            .h_full()
            .into_element_in(cx)
            .test_id(TEST_ID_RIGHT_RAIL);
        let frame = WorkspaceFrame::new(center)
            .left(left_rail)
            .right(right_rail)
            .background(Some(theme.color_token("background")))
            .into_element_in(cx);

        ui::container(|_cx| [frame])
            .p(Space::N4)
            .size_full()
            .into_element_in(cx)
            .test_id(TEST_ID_ROOT)
            .into()
    }
}

impl EditorNotesDemoView {
    fn asset(&self, selection: EditorAssetSelection) -> &EditorAssetState {
        editor_asset_for_selection(&self.assets, selection)
    }
}

pub(crate) fn default_editor_assets(app: &mut App) -> Arc<[EditorAssetState]> {
    vec![
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
    .into()
}

pub(crate) fn editor_asset_for_selection<'a>(
    assets: &'a [EditorAssetState],
    selection: EditorAssetSelection,
) -> &'a EditorAssetState {
    assets
        .iter()
        .find(|asset| asset.selection == selection)
        .unwrap_or_else(|| &assets[0])
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
        models: EditorAssetModels::new(app.models_mut(), title, name, notes),
    }
}

pub(crate) fn editor_asset_paint_snapshot(
    cx: &mut AppUi<'_, '_>,
    asset: &EditorAssetState,
) -> EditorAssetSnapshot {
    let (name_value, committed_notes, notes_outcome, summary_status) =
        cx.data().selector_model_paint(
            (
                &asset.models.name,
                &asset.models.notes,
                &asset.models.notes_outcome,
                &asset.models.summary_status,
            ),
            |(name, committed_notes, notes_outcome, summary_status)| {
                (name, committed_notes, notes_outcome, summary_status)
            },
        );

    EditorAssetSnapshot {
        name_value,
        committed_notes,
        notes_outcome,
        summary_status,
    }
}

fn editor_asset_summary_command_status(asset: &EditorAssetState) -> String {
    format!("Copied summary: {} · {}", asset.title, asset.subtitle)
}

fn editor_notes_draft_action_status(asset: &EditorAssetState, action: &str) -> String {
    format!("{action}: {} · TextField draft controller", asset.title)
}

fn editor_notes_readout_text<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    decl_text::text_control_readout(cx, text)
}

fn editor_notes_section_text<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    decl_text::text_section_chrome_label(cx, text)
}

fn editor_notes_paragraph_text<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    decl_text::text_paragraph(cx, text)
}

fn selection_button<'a, Cx>(
    cx: &mut Cx,
    label: impl Into<Arc<str>>,
    selected: bool,
    action: CommandId,
    test_id: &'static str,
) -> AnyElement
where
    Cx: fret::app::ElementContextAccess<'a, App>,
{
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
        .into_element_in(cx)
}

fn editor_selection_label(selection: EditorAssetSelection) -> &'static str {
    match selection {
        EditorAssetSelection::Material => "Material",
        EditorAssetSelection::Light => "Key Light",
        EditorAssetSelection::Camera => "Camera",
    }
}

fn editor_selection_subtitle(selection: EditorAssetSelection) -> &'static str {
    match selection {
        EditorAssetSelection::Material => "Surface authoring metadata",
        EditorAssetSelection::Light => "Shot review notes",
        EditorAssetSelection::Camera => "Sequence continuity notes",
    }
}

fn editor_collection_row_label(selection: EditorAssetSelection, selected: bool) -> Arc<str> {
    let state = if selected { "active" } else { "available" };
    Arc::from(format!(
        "{} · {} · {state}",
        editor_selection_label(selection),
        editor_selection_subtitle(selection)
    ))
}

fn editor_collection_status_label(selected: EditorAssetSelection) -> String {
    format!(
        "3 shell-mounted assets · active: {} · app-owned collection proof",
        editor_selection_label(selected)
    )
}

pub(crate) fn render_selection_panel<'a, Cx>(
    cx: &mut Cx,
    selected: EditorAssetSelection,
) -> AnyElement
where
    Cx: fret::app::ElementContextAccess<'a, App>,
{
    let header = shadcn::CardHeader::new([
        ui::v_flex(|cx| {
            ui::children![
                cx;
                shadcn::CardTitle::new("Scene collection"),
                shadcn::CardDescription::new(
                    "Shell-mounted collection proof: choose an editor-owned surface, then blur Notes to Name to keep a local draft alive.",
                ),
            ]
        })
        .gap(Space::N1)
        .into_element_in(cx),
    ]);

    let material_button = selection_button(
        cx,
        editor_collection_row_label(
            EditorAssetSelection::Material,
            selected == EditorAssetSelection::Material,
        ),
        selected == EditorAssetSelection::Material,
        act::SelectMaterial.into(),
        TEST_ID_SELECT_MATERIAL,
    );
    let light_button = selection_button(
        cx,
        editor_collection_row_label(
            EditorAssetSelection::Light,
            selected == EditorAssetSelection::Light,
        ),
        selected == EditorAssetSelection::Light,
        act::SelectLight.into(),
        TEST_ID_SELECT_LIGHT,
    );
    let camera_button = selection_button(
        cx,
        editor_collection_row_label(
            EditorAssetSelection::Camera,
            selected == EditorAssetSelection::Camera,
        ),
        selected == EditorAssetSelection::Camera,
        act::SelectCamera.into(),
        TEST_ID_SELECT_CAMERA,
    );

    let body = ui::v_flex(move |cx| {
        let collection_summary =
            editor_notes_readout_text(cx, editor_collection_status_label(selected))
                .test_id(TEST_ID_COLLECTION_SUMMARY);
        let collection_list = ui::v_flex(|_cx| [material_button, light_button, camera_button])
            .gap(Space::N2)
            .test_id(TEST_ID_COLLECTION_LIST)
            .into_element(cx);
        ui::children![cx; collection_summary, collection_list]
    })
    .gap(Space::N3)
    .into_element_in(cx)
    .test_id(TEST_ID_COLLECTION);

    shadcn::Card::new(ui::children![
        cx;
        header,
        shadcn::CardContent::new([body]),
    ])
    .ui()
    .w_full()
    .into_element_in(cx)
    .test_id(TEST_ID_SELECTION)
}

pub(crate) fn render_center_panel<'a, Cx>(
    cx: &mut Cx,
    asset: EditorAssetState,
    name_value: String,
    committed_notes: String,
    notes_outcome: String,
    ownership_note: &'static str,
    committed_notes_intro: &'static str,
) -> AnyElement
where
    Cx: fret::app::ElementContextAccess<'a, App>,
{
    let preview_text = if committed_notes.trim().is_empty() {
        "No committed notes yet. Edit Notes in the inspector, then blur back to Name to keep the local draft alive.".to_string()
    } else {
        committed_notes.clone()
    };
    let note_summary = committed_line_count_label(&committed_notes);
    let outcome_label = if notes_outcome.is_empty() {
        "Idle".to_string()
    } else {
        notes_outcome
    };
    let title = asset.title.clone();
    let subtitle = asset.subtitle.clone();
    let header = shadcn::CardHeader::new([ui::v_flex(|cx| {
        ui::children![
            cx;
            shadcn::CardTitle::new(title.clone()),
            shadcn::CardDescription::new(subtitle.clone()),
            editor_notes_paragraph_text(cx, ownership_note),
        ]
    })
    .gap(Space::N1)
    .into_element_in(cx)]);
    let content = shadcn::CardContent::new([ui::v_flex(|cx| {
        ui::children![
            cx;
            ui::h_flex(|cx| {
                ui::children![
                    cx;
                    ui::v_flex(|cx| {
                        ui::children![
                            cx;
                            editor_notes_section_text(cx, "Active asset"),
                            editor_notes_paragraph_text(cx, name_value.clone()),
                        ]
                    })
                    .gap(Space::N1)
                    .into_element(cx),
                    ui::v_flex(|cx| {
                        ui::children![
                            cx;
                            editor_notes_section_text(cx, "Inspector state"),
                            editor_notes_readout_text(cx, note_summary.clone()),
                            editor_notes_readout_text(cx, format!("Last action: {outcome_label}")),
                        ]
                    })
                    .gap(Space::N1)
                    .items_end()
                    .into_element(cx),
                ]
            })
            .items_start()
            .justify_between()
            .w_full()
            .gap(Space::N4)
            .into_element(cx),
            ui::v_flex(|cx| {
                ui::children![
                    cx;
                    editor_notes_section_text(cx, "Committed notes"),
                    editor_notes_paragraph_text(cx, committed_notes_intro),
                    editor_notes_paragraph_text(cx, preview_text),
                ]
            })
            .gap(Space::N2)
            .w_full()
            .p(Space::N3)
            .rounded_md()
            .border_1()
            .test_id(TEST_ID_CENTER_PREVIEW)
            .into_element(cx),
        ]
    })
    .gap(Space::N4)
    .w_full()
    .min_w_0()
    .into_element_in(cx)]);

    shadcn::Card::new(ui::children![cx; header, content])
        .ui()
        .size_full()
        .min_w_0()
        .into_element_in(cx)
        .test_id(TEST_ID_CENTER)
}

pub(crate) fn render_inspector_panel<'a, Cx>(
    cx: &mut Cx,
    asset: EditorAssetState,
    theme: EditorThemePresetBinding,
    committed_label: String,
    outcome_label: String,
    summary_status: String,
) -> AnyElement
where
    Cx: fret::app::ElementContextAccess<'a, App>,
{
    let subtitle = asset.subtitle.clone();
    let title = asset.title.clone();
    let summary_status_next = editor_asset_summary_command_status(&asset);
    let draft_commit_status = editor_notes_draft_action_status(&asset, "Draft committed");
    let draft_discard_status = editor_notes_draft_action_status(&asset, "Draft discarded");
    let draft_status_label = editor_notes_draft_status_label(&outcome_label, &committed_label);
    let draft_controller = cx.elements().keyed_slot_state(
        (
            "editor-notes-demo.notes.draft-controller",
            asset.notes_id_source.clone(),
        ),
        TextFieldDraftController::new,
        |controller| controller.clone(),
    );

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
        .into_element_in(
            cx,
            move |cx, _panel_cx| vec![editor_notes_readout_text(cx, subtitle.clone())],
            move |cx, _panel_cx| {
                vec![
                    PropertyGroup::new("Metadata")
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
                                        PropertyRow::new(),
                                        |cx| row_cx.label_text(cx, "Name"),
                                        |cx| {
                                            TextField::new(asset.models.name_text_model())
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
                                        PropertyRow::new(),
                                        |cx| row_cx.label_text(cx, "Notes"),
                                        |cx| {
                                            TextField::new(asset.models.notes_text_model())
                                            .on_outcome(Some(Arc::new({
                                                let models = asset.models.clone();
                                                move |host, action_cx, outcome: TextFieldOutcome| {
                                                    let next = match outcome {
                                                        TextFieldOutcome::Committed => "Committed",
                                                        TextFieldOutcome::Canceled => "Canceled",
                                                    };
                                                    let _ = models
                                                        .set_notes_outcome(host.models_mut(), next);
                                                    host.request_redraw(action_cx.window);
                                                }
                                            })))
                                            .options(TextFieldOptions {
                                                id_source: Some(asset.notes_id_source.clone()),
                                                multiline: true,
                                                stable_line_boxes: true,
                                                min_height: Some(Px(120.0)),
                                                blur_behavior: TextFieldBlurBehavior::PreserveDraft,
                                                draft_controller: Some(draft_controller.clone()),
                                                test_id: Some(Arc::from(TEST_ID_NOTES)),
                                                ..Default::default()
                                            })
                                            .into_element(cx)
                                        },
                                        |_cx| None,
                                    ));

                                    rows.push(row_cx.row_with(
                                        cx,
                                        PropertyRow::new(),
                                        |cx| row_cx.label_text(cx, "Committed"),
                                        |cx| {
                                            editor_notes_readout_text(cx, committed_label.clone())
                                                .test_id(TEST_ID_NOTES_COMMITTED)
                                        },
                                        |_cx| None,
                                    ));

                                    rows.push(row_cx.row_with(
                                        cx,
                                        PropertyRow::new(),
                                        |cx| row_cx.label_text(cx, "Last action"),
                                        |cx| {
                                            editor_notes_readout_text(cx, outcome_label.clone())
                                                .test_id(TEST_ID_NOTES_OUTCOME)
                                        },
                                        |_cx| None,
                                    ));

                                    rows.push(row_cx.row_with(
                                        cx,
                                        PropertyRow::new(),
                                        |cx| row_cx.label_text(cx, "Draft status"),
                                        |cx| {
                                            editor_notes_readout_text(
                                                cx,
                                                draft_status_label.clone(),
                                            )
                                            .test_id(TEST_ID_NOTES_DRAFT_STATUS)
                                        },
                                        |_cx| None,
                                    ));

                                    rows.push(row_cx.row_with(
                                        cx,
                                        PropertyRow::new(),
                                        |cx| row_cx.label_text(cx, "Draft actions"),
                                        |cx| {
                                            ui::h_flex(|cx| {
                                                ui::children![
                                                    cx;
                                                    shadcn::Button::new("Commit draft")
                                                        .variant(shadcn::ButtonVariant::Secondary)
                                                        .size(shadcn::ButtonSize::Sm)
                                                        .on_activate(fret_ui_kit::on_activate({
                                                            let models = asset.models.clone();
                                                            let draft_commit_status =
                                                                draft_commit_status.clone();
                                                            let draft_controller =
                                                                draft_controller.clone();
                                                            move |host, action_cx, _reason| {
                                                                if draft_controller
                                                                    .commit(host, action_cx)
                                                                {
                                                                    let _ = models
                                                                        .set_notes_outcome(
                                                                            host.models_mut(),
                                                                            "Committed",
                                                                        );
                                                                    let _ = models
                                                                        .set_summary_status(
                                                                            host.models_mut(),
                                                                            draft_commit_status
                                                                                .clone(),
                                                                        );
                                                                    host.request_redraw(
                                                                        action_cx.window,
                                                                    );
                                                                }
                                                            }
                                                        }))
                                                        .test_id(TEST_ID_DRAFT_COMMIT_COMMAND)
                                                        .ui()
                                                        .into_element_in(cx),
                                                    shadcn::Button::new("Discard draft")
                                                        .variant(shadcn::ButtonVariant::Ghost)
                                                        .size(shadcn::ButtonSize::Sm)
                                                        .on_activate(fret_ui_kit::on_activate({
                                                            let models = asset.models.clone();
                                                            let draft_discard_status =
                                                                draft_discard_status.clone();
                                                            let draft_controller =
                                                                draft_controller.clone();
                                                            move |host, action_cx, _reason| {
                                                                if draft_controller
                                                                    .discard(host, action_cx)
                                                                {
                                                                    let _ = models
                                                                        .set_notes_outcome(
                                                                            host.models_mut(),
                                                                            "Canceled",
                                                                        );
                                                                    let _ = models
                                                                        .set_summary_status(
                                                                            host.models_mut(),
                                                                            draft_discard_status
                                                                                .clone(),
                                                                        );
                                                                    host.request_redraw(
                                                                        action_cx.window,
                                                                    );
                                                                }
                                                            }
                                                        }))
                                                        .test_id(TEST_ID_DRAFT_DISCARD_COMMAND)
                                                        .ui()
                                                        .into_element_in(cx),
                                                ]
                                            })
                                            .gap(Space::N2)
                                            .into_element_in(cx)
                                        },
                                        |_cx| None,
                                    ));

                                    rows.push(row_cx.row_with(
                                        cx,
                                        PropertyRow::new(),
                                        |cx| row_cx.label_text(cx, "Theme preset"),
                                        |cx| {
                                            EditorThemePresetPicker::new(theme.picker_model())
                                                .options(EditorThemePresetPickerOptions {
                                                    label: Some(Arc::from("Editor theme preset")),
                                                    test_id: Some(Arc::from(
                                                        TEST_ID_THEME_PRESET_PICKER,
                                                    )),
                                                    ..Default::default()
                                                })
                                                .into_element(cx)
                                        },
                                        |_cx| None,
                                    ));

                                    rows.push(row_cx.row_with(
                                        cx,
                                        PropertyRow::new(),
                                        |cx| row_cx.label_text(cx, "Summary command"),
                                        |cx| {
                                            shadcn::Button::new("Copy asset summary")
                                                .variant(shadcn::ButtonVariant::Secondary)
                                                .size(shadcn::ButtonSize::Sm)
                                                .on_activate(fret_ui_kit::on_activate({
                                                    let models = asset.models.clone();
                                                    let summary_status_next =
                                                        summary_status_next.clone();
                                                    move |host, action_cx, _reason| {
                                                        let _ = models.set_summary_status(
                                                            host.models_mut(),
                                                            summary_status_next.clone(),
                                                        );
                                                        host.request_redraw(action_cx.window);
                                                    }
                                                }))
                                                .test_id(TEST_ID_SUMMARY_COMMAND)
                                                .ui()
                                                .into_element_in(cx)
                                        },
                                        |_cx| None,
                                    ));

                                    rows.push(row_cx.row_with(
                                        cx,
                                        PropertyRow::new(),
                                        |cx| row_cx.label_text(cx, "Summary status"),
                                        |cx| {
                                            editor_notes_readout_text(cx, summary_status.clone())
                                                .test_id(TEST_ID_SUMMARY_STATUS)
                                        },
                                        |_cx| None,
                                    ));

                                    rows
                                })]
                            },
                        ),
                ]
            },
        )
}

pub(crate) fn committed_line_count_label(text: &str) -> String {
    let lines = text.lines().count();
    match lines {
        0 => "No committed notes".to_string(),
        1 => "1 line committed".to_string(),
        n => format!("{n} lines committed"),
    }
}

pub(crate) fn editor_notes_draft_status_label(outcome: &str, committed_label: &str) -> String {
    match outcome {
        "Committed" => format!("Clean draft · {committed_label}"),
        "Canceled" => format!("Draft canceled · preserved editor text · {committed_label}"),
        _ => format!("Draft preserved until commit · {committed_label}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_notes_model_owner_preserves_text_state_updates() {
        let mut models = ModelStore::default();
        let status = models.insert("Idle".to_string());

        assert!(EditorNotesModelOwner::new(&mut models).set_text(&status, "Committed"));
        assert_eq!(
            models.read(&status, Clone::clone).ok().as_deref(),
            Some("Committed")
        );

        assert!(EditorNotesModelOwner::new(&mut models).set_text(&status, "Canceled"));
        assert_eq!(
            models.read(&status, Clone::clone).ok().as_deref(),
            Some("Canceled")
        );
    }
}
