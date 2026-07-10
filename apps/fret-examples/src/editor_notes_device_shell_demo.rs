use std::sync::Arc;

use fret::adaptive::{DeviceShellSwitchPolicy, device_shell_switch};
use fret::app::LocalState;
use fret::app::editor::{EditorThemePresetV1, InspectorTextFieldSnapshot};
use fret::app::prelude::*;
use fret::app::{AppElement, AppRenderContext, RenderContextAccess as _, text};
use fret::{Defaults, FretApp, shadcn};
use fret_core::Px;
use fret_ui::Invalidation;
use fret_ui_kit::IntoUiElementInExt as _;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_workspace::WorkspaceFrame;

use crate::editor_notes_demo::{self, EditorAssetSelection, EditorAssetState};

const TEST_ID_ROOT: &str = "editor-notes-device-shell-demo.root";
const TEST_ID_LEFT_RAIL: &str = "editor-notes-device-shell-demo.left-rail";
const TEST_ID_RIGHT_RAIL: &str = "editor-notes-device-shell-demo.right-rail";
const TEST_ID_MOBILE_HEADER: &str = "editor-notes-device-shell-demo.mobile-header";
const TEST_ID_DRAWER_TRIGGER: &str = "editor-notes-device-shell-demo.drawer.trigger";
const TEST_ID_DRAWER_CONTENT: &str = "editor-notes-device-shell-demo.drawer.content";
const TEST_ID_DRAWER_VIEWPORT: &str = "editor-notes-device-shell-demo.drawer.viewport";
const TEST_ID_DRAWER_CLOSE: &str = "editor-notes-device-shell-demo.drawer.close";

const DESKTOP_OWNERSHIP_NOTE: &str = "WorkspaceFrame owns the desktop shell rails; fret-ui-editor still owns the shared inspector content.";
const DESKTOP_COMMITTED_NOTES_INTRO: &str = "This center region is app-local content, while both side regions stay mounted through the desktop workspace shell seam.";
const MOBILE_OWNERSHIP_NOTE: &str = "The device shell now swaps the outer owner to a drawer, while the editor-owned inner panels stay unchanged.";
const MOBILE_COMMITTED_NOTES_INTRO: &str = "This center region stays app-local on compact devices while selection and inspector move into a drawer-owned shell.";

struct EditorNotesDeviceShellDemoView {
    assets: Arc<[EditorAssetState]>,
    theme: LocalState<EditorThemePresetV1>,
}

fn device_shell_section_text<'a, Cx, T>(cx: &mut Cx, text: T) -> AppElement
where
    Cx: AppRenderContext<'a>,
    T: Into<Arc<str>>,
{
    text::section_chrome_label(cx, text)
}

fn device_shell_paragraph_text<'a, Cx, T>(cx: &mut Cx, text: T) -> AppElement
where
    Cx: AppRenderContext<'a>,
    T: Into<Arc<str>>,
{
    text::paragraph(cx, text)
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("editor-notes-device-shell-demo")
        .window("editor_notes_device_shell_demo", (1080.0, 720.0))
        .defaults(Defaults {
            shadcn: false,
            ..Defaults::desktop_app()
        })
        .setup((
            editor_notes_demo::install_editor_notes_demo_theme,
            fret_icons_lucide::app::install,
        ))
        .view::<EditorNotesDeviceShellDemoView>()?
        .run()
        .map_err(anyhow::Error::from)
}

impl View for EditorNotesDeviceShellDemoView {
    fn init(app: &mut App, _window: WindowId) -> Self {
        Self {
            assets: editor_notes_demo::default_editor_assets(app),
            theme: editor_notes_demo::editor_theme_preset_state(app),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let selected = cx.state().local_init(|| EditorAssetSelection::Material);
        let drawer_open = cx.state().local_init(|| false);

        cx.actions()
            .local(&selected)
            .set::<editor_notes_demo::act::SelectMaterial>(EditorAssetSelection::Material);
        cx.actions()
            .local(&selected)
            .set::<editor_notes_demo::act::SelectLight>(EditorAssetSelection::Light);
        cx.actions()
            .local(&selected)
            .set::<editor_notes_demo::act::SelectCamera>(EditorAssetSelection::Camera);

        let theme = cx.theme_snapshot();
        let selected = cx.state().watch(&selected).layout().value_or_default();
        let asset = editor_notes_demo::editor_asset_for_selection(&self.assets, selected).clone();
        let desktop_background = theme.color_token("background");

        let desktop_asset = asset.clone();
        let desktop_theme = self.theme.clone();

        let mobile_asset = asset;
        let drawer_theme = self.theme.clone();

        let shell = device_shell_switch(
            cx,
            Invalidation::Layout,
            DeviceShellSwitchPolicy::default(),
            move |cx| {
                let snapshot = editor_notes_demo::editor_asset_paint_snapshot(cx, &desktop_asset);
                let desktop_name_value = snapshot.name_value;
                let desktop_notes_snapshot = snapshot.notes;
                let selection_panel = editor_notes_demo::render_selection_panel(cx, selected);
                let center = editor_notes_demo::render_center_panel(
                    cx,
                    desktop_asset.clone(),
                    desktop_name_value,
                    desktop_notes_snapshot.committed().to_owned(),
                    desktop_notes_snapshot.outcome,
                    DESKTOP_OWNERSHIP_NOTE,
                    DESKTOP_COMMITTED_NOTES_INTRO,
                );
                let inspector = editor_notes_demo::render_inspector_panel(
                    cx,
                    desktop_asset.clone(),
                    desktop_theme.clone(),
                    desktop_notes_snapshot.clone(),
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

                WorkspaceFrame::new(center)
                    .left(left_rail)
                    .right(right_rail)
                    .background(Some(desktop_background))
                    .into_element_in(cx)
            },
            move |cx| {
                let snapshot = editor_notes_demo::editor_asset_paint_snapshot(cx, &mobile_asset);
                let mobile_name_value = snapshot.name_value;
                let mobile_notes_snapshot = snapshot.notes;
                let center = editor_notes_demo::render_center_panel(
                    cx,
                    mobile_asset.clone(),
                    mobile_name_value,
                    mobile_notes_snapshot.committed().to_owned(),
                    mobile_notes_snapshot.outcome,
                    MOBILE_OWNERSHIP_NOTE,
                    MOBILE_COMMITTED_NOTES_INTRO,
                );

                let drawer_asset = mobile_asset.clone();
                let drawer_notes_snapshot: InspectorTextFieldSnapshot = mobile_notes_snapshot;
                let drawer = shadcn::Drawer::new(drawer_open.clone())
                    .children([
                        shadcn::DrawerPart::trigger(shadcn::DrawerTrigger::build(
                            shadcn::Button::new("Panels")
                                .variant(shadcn::ButtonVariant::Outline)
                                .toggle_model(drawer_open.clone())
                                .test_id(TEST_ID_DRAWER_TRIGGER),
                        )),
                        shadcn::DrawerPart::content_with(move |cx| {
                            let selection_panel =
                                editor_notes_demo::render_selection_panel(cx, selected);
                            let inspector = editor_notes_demo::render_inspector_panel(
                                cx,
                                drawer_asset.clone(),
                                drawer_theme.clone(),
                                drawer_notes_snapshot.clone(),
                            );
                            let body = ui::v_flex(|_cx| [selection_panel, inspector])
                                .gap(Space::N4)
                                .w_full()
                                .min_w_0()
                                .into_element(cx);
                            let body = shadcn::ScrollArea::new([body])
                                .refine_layout(
                                    LayoutRefinement::default()
                                        .w_full()
                                        .h_px(Px(320.0))
                                        .min_w_0()
                                        .min_h_0(),
                                )
                                .viewport_test_id(TEST_ID_DRAWER_VIEWPORT)
                                .into_element(cx);
                            let body = ui::container(|_cx| [body])
                                .px(Space::N4)
                                .w_full()
                                .min_w_0()
                                .into_element(cx);

                            shadcn::DrawerContent::new([])
                                .children(|cx| {
                                    ui::children![
                                        cx;
                                        shadcn::DrawerHeader::new([])
                                            .children(|cx| {
                                                ui::children![
                                                    cx;
                                                    shadcn::DrawerTitle::new("Editor panels"),
                                                    shadcn::DrawerDescription::new(
                                                        "Desktop keeps these panels in WorkspaceFrame rails; compact shells mount the same content in a drawer.",
                                                    )
                                                ]
                                            }),
                                        body,
                                        shadcn::DrawerFooter::new([])
                                            .children(|cx| {
                                                ui::children![
                                                    cx;
                                                    shadcn::DrawerClose::from_scope().child(
                                                        shadcn::Button::new("Close")
                                                            .variant(shadcn::ButtonVariant::Outline)
                                                            .test_id(TEST_ID_DRAWER_CLOSE),
                                                    )
                                                ]
                                            })
                                    ]
                                })
                                .test_id(TEST_ID_DRAWER_CONTENT)
                                .into_element(cx)
                        }),
                    ])
                    .into_element_in(cx);

                let mobile_header = ui::h_flex(|cx| {
                    ui::children![
                        cx;
                        ui::v_flex(|cx| {
                                ui::children![
                                cx;
                                device_shell_section_text(cx, "Compact device shell"),
                                device_shell_paragraph_text(
                                    cx,
                                    "Keep the center surface visible and move auxiliary panels behind a drawer trigger.",
                                ),
                            ]
                        })
                        .gap(Space::N1)
                        .min_w_0()
                        .into_element_in(cx),
                        drawer,
                    ]
                })
                .gap(Space::N3)
                .w_full()
                .items_center()
                .justify_between()
                .into_element_in(cx)
                .test_id(TEST_ID_MOBILE_HEADER);

                let center_region = ui::container(|_cx| [center])
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .into_element_in(cx);

                ui::v_flex(|_cx| [mobile_header, center_region])
                    .gap(Space::N4)
                    .items_stretch()
                    .size_full()
                    .into_element_in(cx)
            },
        );

        ui::container(|_cx| [shell])
            .p(Space::N4)
            .size_full()
            .into_element_in(cx)
            .test_id(TEST_ID_ROOT)
            .into()
    }
}
