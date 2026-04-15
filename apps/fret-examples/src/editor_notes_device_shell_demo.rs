use std::sync::Arc;

use fret::adaptive::{DeviceShellSwitchPolicy, device_shell_switch};
use fret::app::prelude::*;
use fret::{Defaults, FretApp, shadcn};
use fret_core::Px;
use fret_ui::Invalidation;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::{ColorRef, LayoutRefinement, Space};
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
        let name_value = cx
            .watch_model(&asset.name_model)
            .paint()
            .cloned_or_default();
        let committed_notes = cx
            .watch_model(&asset.notes_model)
            .paint()
            .cloned_or_default();
        let notes_outcome = cx
            .watch_model(&asset.notes_outcome_model)
            .paint()
            .cloned_or_default();
        let committed_label = editor_notes_demo::committed_line_count_label(&committed_notes);
        let desktop_background = theme.color_token("background");

        let desktop_asset = asset.clone();
        let desktop_name_value = name_value.clone();
        let desktop_committed_notes = committed_notes.clone();
        let desktop_notes_outcome = notes_outcome.clone();
        let desktop_committed_label = committed_label.clone();

        let mobile_asset = asset;
        let mobile_name_value = name_value;
        let mobile_committed_notes = committed_notes;
        let mobile_notes_outcome = notes_outcome;
        let mobile_committed_label = committed_label;

        let shell = device_shell_switch(
            cx,
            Invalidation::Layout,
            DeviceShellSwitchPolicy::default(),
            move |cx| {
                let selection_panel = editor_notes_demo::render_selection_panel(cx, selected);
                let center = editor_notes_demo::render_center_panel(
                    cx,
                    desktop_asset.clone(),
                    desktop_name_value.clone(),
                    desktop_committed_notes.clone(),
                    desktop_notes_outcome.clone(),
                    DESKTOP_OWNERSHIP_NOTE,
                    DESKTOP_COMMITTED_NOTES_INTRO,
                );
                let inspector = editor_notes_demo::render_inspector_panel(
                    cx,
                    desktop_asset.clone(),
                    desktop_committed_label.clone(),
                    desktop_notes_outcome.clone(),
                );
                let left_rail = ui::container(|_cx| [selection_panel])
                    .w_px(Px(256.0))
                    .flex_shrink_0()
                    .h_full()
                    .into_element(cx)
                    .test_id(TEST_ID_LEFT_RAIL);
                let right_rail = ui::container(|_cx| [inspector])
                    .w_px(Px(360.0))
                    .flex_shrink_0()
                    .h_full()
                    .into_element(cx)
                    .test_id(TEST_ID_RIGHT_RAIL);

                WorkspaceFrame::new(center)
                    .left(left_rail)
                    .right(right_rail)
                    .background(Some(desktop_background))
                    .into_element(cx)
            },
            move |cx| {
                let center = editor_notes_demo::render_center_panel(
                    cx,
                    mobile_asset.clone(),
                    mobile_name_value,
                    mobile_committed_notes,
                    mobile_notes_outcome.clone(),
                    MOBILE_OWNERSHIP_NOTE,
                    MOBILE_COMMITTED_NOTES_INTRO,
                );

                let drawer_asset = mobile_asset.clone();
                let drawer_committed_label = mobile_committed_label.clone();
                let drawer_notes_outcome = mobile_notes_outcome.clone();
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
                                drawer_committed_label.clone(),
                                drawer_notes_outcome.clone(),
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
                    .into_element(cx);

                let mobile_header = ui::h_flex(|cx| {
                    let muted = cx.theme_snapshot().color_token("muted-foreground");
                    ui::children![
                        cx;
                        ui::v_flex(|cx| {
                            ui::children![
                                cx;
                                ui::text("Compact device shell")
                                    .text_base()
                                    .font_semibold()
                                    .into_element(cx),
                                ui::text("Keep the center surface visible and move auxiliary panels behind a drawer trigger.")
                                    .text_sm()
                                    .text_color(ColorRef::Color(muted))
                                    .wrap(fret_core::TextWrap::Word)
                                    .into_element(cx),
                            ]
                        })
                        .gap(Space::N1)
                        .min_w_0()
                        .into_element(cx),
                        drawer,
                    ]
                })
                .gap(Space::N3)
                .w_full()
                .items_center()
                .justify_between()
                .into_element(cx)
                .test_id(TEST_ID_MOBILE_HEADER);

                let center_region = ui::container(|_cx| [center])
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .into_element(cx);

                ui::v_flex(|_cx| [mobile_header, center_region])
                    .gap(Space::N4)
                    .items_stretch()
                    .size_full()
                    .into_element(cx)
            },
        );

        ui::container(|_cx| [shell])
            .p(Space::N4)
            .size_full()
            .into_element(cx)
            .test_id(TEST_ID_ROOT)
            .into()
    }
}
