use fret::advanced::view::AppRenderDataExt as _;
use fret::app::{AppRenderContext, text};
use fret::imui::{UiWriterImUiFacadeExt as _, imui_build, kit};
use fret::{shadcn, shadcn::themes::ShadcnColorScheme};
use fret_app::{App, CommandId, Model};
use fret_core::{AppWindowId, Axis, Edges, Px, SemanticsRole};
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow, PressableA11y,
    PressableProps, SemanticsProps, ViewCacheProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{Invalidation, VirtualListScrollHandle};
use fret_ui_editor::composites::{
    InspectorPanel, InspectorPanelOptions, PropertyGrid, PropertyGroup, PropertyGroupOptions,
};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::declarative::file_tree::{FileTreeViewProps, file_tree_view_retained_v0};
use fret_ui_kit::{
    LayoutRefinement, MetricRef, OverlayController, OverlayPresence, OverlayRequest, Space,
    TreeItem, TreeState,
};
use fret_workspace::commands::{act, tab_open_preview_command, tab_pin_command, typed_command_id};
use fret_workspace::layout::{WorkspacePaneTree, WorkspaceWindowLayout};
use fret_workspace::{
    WorkspaceCommandScope, WorkspaceFrame, WorkspacePaneContentFocusTarget, WorkspaceTabStrip,
    WorkspaceWorkbench, WorkspaceWorkbenchCommandOutcome, workspace_pane_tree_element_with_resize,
};
use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

#[path = "state.rs"]
mod state;

use state::{WorkspaceShellWindowState, act as shell_act, build_file_tree_items};

const ENV_WORKSPACE_SHELL_EDITOR_PRESET: &str = "FRET_WORKSPACE_SHELL_EDITOR_PRESET";
const WORKSPACE_SHELL_HOST_BASE_COLOR: shadcn::themes::ShadcnBaseColor =
    shadcn::themes::ShadcnBaseColor::Slate;
const WORKSPACE_SHELL_HOST_DEFAULT_SCHEME: ShadcnColorScheme = ShadcnColorScheme::Dark;
const DIRTY_CLOSE_PROMPT_OVERLAY_ID: GlobalElementId = GlobalElementId(0x6a4e_5c1f_8f3b_1c20);

fn env_bool(name: &str, default: bool) -> bool {
    let Some(v) = std::env::var_os(name).filter(|v| !v.is_empty()) else {
        return default;
    };
    let v = v.to_string_lossy().trim().to_ascii_lowercase();
    !(v == "0" || v == "false" || v == "no" || v == "off")
}

fn env_usize(name: &str) -> Option<usize> {
    let Some(v) = std::env::var_os(name).filter(|v| !v.is_empty()) else {
        return None;
    };
    let v = v.to_string_lossy();
    v.trim().parse::<usize>().ok()
}

fn selected_workspace_shell_editor_theme_preset() -> Option<fret_ui_editor::theme::EditorThemePreset>
{
    crate::editor_theme_preset_from_env(ENV_WORKSPACE_SHELL_EDITOR_PRESET)
}

fn install_workspace_shell_theme(app: &mut App) {
    if let Some(preset) = selected_workspace_shell_editor_theme_preset() {
        shadcn::themes::apply_shadcn_new_york(
            app,
            WORKSPACE_SHELL_HOST_BASE_COLOR,
            WORKSPACE_SHELL_HOST_DEFAULT_SCHEME,
        );
        fret_ui_editor::theme::install_editor_theme_preset(app, preset);
    }
}

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn fixed_width_fill_height(width: Px) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(width);
    layout.size.height = Length::Fill;
    layout.flex.shrink = 0.0;
    layout
}

#[derive(Clone)]
struct WorkspaceShellEditorRailState {
    active_pane_label: Arc<str>,
    active_tab_label: Arc<str>,
    active_tab_count: usize,
    active_dirty_count: usize,
    two_row_pinned: bool,
    prompt_open: bool,
    frame_stage_label: Arc<str>,
}

#[derive(Clone)]
struct WorkspaceShellPaneProofState {
    pane_id: Arc<str>,
    active_tab_label: Arc<str>,
    tab_count: usize,
    dirty_count: usize,
    is_active: bool,
    two_row_pinned: bool,
    prompt_open: bool,
}

struct WorkspaceShellModelBundle {
    window_layout: Model<WorkspaceWindowLayout>,
    workbench: WorkspaceWorkbench,
    tabstrip_two_row_pinned: Model<bool>,
    file_tree_items: Model<Vec<TreeItem>>,
    file_tree_state: Model<TreeState>,
}

impl WorkspaceShellModelBundle {
    fn new(
        app: &mut App,
        window_layout: WorkspaceWindowLayout,
        file_tree_items: Vec<TreeItem>,
        file_tree_state: TreeState,
        block_dirty_close: bool,
    ) -> Self {
        let window_layout = app.models_mut().insert(window_layout);
        let workbench =
            WorkspaceWorkbench::new(app.models_mut(), window_layout.clone(), block_dirty_close);
        Self {
            window_layout,
            workbench,
            tabstrip_two_row_pinned: app.models_mut().insert(false),
            file_tree_items: app.models_mut().insert(file_tree_items),
            file_tree_state: app.models_mut().insert(file_tree_state),
        }
    }
}

fn apply_dirty_close_overlay_focus_policy(
    request: &mut OverlayRequest,
    initial_focus: GlobalElementId,
) {
    request.initial_focus = Some(initial_focus);
    request.on_close_auto_focus = Some(Arc::new(|_host, _cx, request| {
        request.prevent_default();
    }));
}

fn workspace_shell_command_button<'a, Cx>(
    cx: &mut Cx,
    test_id: &str,
    label: &str,
    cmd: CommandId,
    height: Px,
    padding: Px,
) -> fret_ui::element::AnyElement
where
    Cx: fret::app::ElementContextAccess<'a, App>,
{
    workspace_shell_command_button_with_focus_policy(
        cx, test_id, label, cmd, height, padding, false,
    )
    .0
}

fn workspace_shell_command_button_with_focus_policy<'a, Cx>(
    cx: &mut Cx,
    test_id: &str,
    label: &str,
    cmd: CommandId,
    height: Px,
    padding: Px,
    focusable: bool,
) -> (fret_ui::element::AnyElement, GlobalElementId)
where
    Cx: fret::app::ElementContextAccess<'a, App>,
{
    let cx = cx.elements();
    let test_id: Arc<str> = Arc::from(test_id);
    let label: Arc<str> = Arc::from(label);
    let mut button_id = None;
    let element = cx.pressable_with_id(
        PressableProps {
            layout: {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Auto;
                layout.size.height = Length::Px(height);
                layout
            },
            enabled: true,
            focusable,
            a11y: PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(label.clone()),
                test_id: Some(test_id),
                ..Default::default()
            },
            ..Default::default()
        },
        |cx, _state, id| {
            button_id = Some(id);
            let cmd = cmd.clone();
            cx.pressable_add_on_activate(Arc::new(move |host, acx, reason| {
                host.record_pending_command_dispatch_source(acx, &cmd, reason);
                host.dispatch_command(Some(acx.window), cmd.clone());
            }));
            vec![cx.container(
                ContainerProps {
                    layout: fill_layout(),
                    padding: Edges::all(padding).into(),
                    ..Default::default()
                },
                move |cx| vec![text::button_label(cx, label.clone())],
            )]
        },
    );
    (
        element,
        button_id.expect("command button should expose its element id"),
    )
}

fn workspace_shell_readout_text<'a, Cx>(
    cx: &mut Cx,
    text: impl Into<Arc<str>>,
) -> fret_ui::element::AnyElement
where
    Cx: AppRenderContext<'a>,
{
    text::control_readout(cx, text)
}

fn workspace_shell_section_chrome_label<'a, Cx>(
    cx: &mut Cx,
    text: impl Into<Arc<str>>,
) -> fret_ui::element::AnyElement
where
    Cx: AppRenderContext<'a>,
{
    text::section_chrome_label(cx, text)
}

fn workspace_shell_paragraph_text<'a, Cx>(
    cx: &mut Cx,
    text: impl Into<Arc<str>>,
) -> fret_ui::element::AnyElement
where
    Cx: AppRenderContext<'a>,
{
    text::paragraph(cx, text)
}

fn workspace_shell_editor_rail<'a, Cx>(
    cx: &mut Cx,
    state: WorkspaceShellEditorRailState,
) -> fret_ui::element::AnyElement
where
    Cx: fret::app::ElementContextAccess<'a, App>,
{
    let cx = cx.elements();
    let background = cx.theme_snapshot().color_token("background");
    let border = cx.theme_snapshot().color_token("border");
    let shell_group_test_id: Arc<str> = Arc::from("workspace-shell-editor-rail-group-shell");
    let selection_group_test_id: Arc<str> =
        Arc::from("workspace-shell-editor-rail-group-selection");
    let WorkspaceShellEditorRailState {
        active_pane_label,
        active_tab_label,
        active_tab_count,
        active_dirty_count,
        two_row_pinned,
        prompt_open,
        frame_stage_label,
    } = state;
    let inspector = InspectorPanel::new(None)
        .options(InspectorPanelOptions {
            layout: fill_layout(),
            title: Some(Arc::from("Editor Rail")),
            test_id: Some(Arc::from("workspace-shell-editor-rail")),
            header_test_id: Some(Arc::from("workspace-shell-editor-rail-header")),
            content_test_id: Some(Arc::from("workspace-shell-editor-rail-content")),
            ..Default::default()
        })
        .into_element_in(
            cx,
            move |cx, _panel_cx| {
                vec![workspace_shell_paragraph_text(
                    cx,
                    "Workspace shell slot + editor-owned inner panel",
                )]
            },
            move |cx, _panel_cx| {
                vec![
                    PropertyGroup::new("Selection")
                        .options(PropertyGroupOptions {
                            collapsible: false,
                            test_id: Some(selection_group_test_id),
                            ..Default::default()
                        })
                        .into_element_in(
                            cx,
                            |_cx| None,
                            move |cx| {
                                vec![PropertyGrid::new().into_element_in(cx, move |cx, row_cx| {
                                    vec![
                                        row_cx.row(
                                            cx,
                                            |cx| row_cx.label_text(cx, "Active pane"),
                                            |cx| {
                                                workspace_shell_readout_text(
                                                    cx,
                                                    active_pane_label.clone(),
                                                )
                                            },
                                        ),
                                        row_cx.row(
                                            cx,
                                            |cx| row_cx.label_text(cx, "Active tab"),
                                            |cx| {
                                                workspace_shell_readout_text(
                                                    cx,
                                                    active_tab_label.clone(),
                                                )
                                            },
                                        ),
                                        row_cx.row(
                                            cx,
                                            |cx| row_cx.label_text(cx, "Tabs in pane"),
                                            |cx| {
                                                workspace_shell_readout_text(
                                                    cx,
                                                    format!("{active_tab_count}"),
                                                )
                                            },
                                        ),
                                        row_cx.row(
                                            cx,
                                            |cx| row_cx.label_text(cx, "Dirty tabs"),
                                            |cx| {
                                                workspace_shell_readout_text(
                                                    cx,
                                                    format!("{active_dirty_count}"),
                                                )
                                            },
                                        ),
                                    ]
                                })]
                            },
                        ),
                    PropertyGroup::new("Shell")
                        .options(PropertyGroupOptions {
                            collapsible: false,
                            test_id: Some(shell_group_test_id),
                            ..Default::default()
                        })
                        .into_element_in(
                            cx,
                            |_cx| None,
                            move |cx| {
                                vec![PropertyGrid::new().into_element_in(cx, move |cx, row_cx| {
                                    vec![
                                        row_cx.row(
                                            cx,
                                            |cx| row_cx.label_text(cx, "Left slot"),
                                            |cx| workspace_shell_readout_text(cx, "File tree rail"),
                                        ),
                                        row_cx.row(
                                            cx,
                                            |cx| row_cx.label_text(cx, "Right slot"),
                                            |cx| workspace_shell_readout_text(cx, "Editor rail"),
                                        ),
                                        row_cx.row(
                                            cx,
                                            |cx| row_cx.label_text(cx, "Two-row tabs"),
                                            |cx| {
                                                workspace_shell_readout_text(
                                                    cx,
                                                    if two_row_pinned { "Pinned" } else { "Auto" },
                                                )
                                            },
                                        ),
                                        row_cx.row(
                                            cx,
                                            |cx| row_cx.label_text(cx, "Dirty close prompt"),
                                            |cx| {
                                                workspace_shell_readout_text(
                                                    cx,
                                                    if prompt_open { "Open" } else { "Closed" },
                                                )
                                            },
                                        ),
                                        row_cx.row(
                                            cx,
                                            |cx| row_cx.label_text(cx, "Frame pipeline"),
                                            |cx| {
                                                workspace_shell_readout_text(
                                                    cx,
                                                    frame_stage_label.clone(),
                                                )
                                                .test_id("workspace-shell-frame-stage-trace")
                                            },
                                        ),
                                    ]
                                })]
                            },
                        ),
                ]
            },
        );

    cx.container(
        ContainerProps {
            layout: fixed_width_fill_height(Px(320.0)),
            background: Some(background),
            border: Edges {
                left: Px(1.0),
                ..Default::default()
            },
            border_color: Some(border),
            ..Default::default()
        },
        move |_cx| vec![inspector],
    )
}

fn workspace_shell_pane_proof<'a, Cx>(
    cx: &mut Cx,
    state: WorkspaceShellPaneProofState,
) -> fret_ui::element::AnyElement
where
    Cx: fret::app::ElementContextAccess<'a, App>,
{
    let cx = cx.elements();
    fret_ui_kit::ui::v_flex_build(move |cx, out| {
        imui_build(cx, out, move |ui| {
            let WorkspaceShellPaneProofState {
                pane_id,
                active_tab_label,
                tab_count,
                dirty_count,
                is_active,
                two_row_pinned,
                prompt_open,
            } = state;

            let shell_id = format!("workspace-shell-pane-{}-proof.shell", pane_id);
            let shell_viewport_id = format!("{}.viewport", shell_id);
            let shell_content_id = format!("{}.content", shell_id);
            let toolbar_id = format!("workspace-shell-pane-{}-proof.toolbar", pane_id);
            let toolbar_viewport_id = format!("{}.viewport", toolbar_id);
            let tabs_id = format!("workspace-shell-pane-{}-proof.tabs", pane_id);
            let tabs_viewport_id = format!("{}.viewport", tabs_id);
            let inspector_id = format!("workspace-shell-pane-{}-proof.inspector", pane_id);
            let inspector_viewport_id = format!("{}.viewport", inspector_id);
            let status_id = format!("workspace-shell-pane-{}-proof.status", pane_id);
            let status_viewport_id = format!("{}.viewport", status_id);

            ui.child_region_with_options(
                shell_id.as_str(),
                kit::ChildRegionOptions {
                    layout: LayoutRefinement::default().w_full().h_full(),
                    scroll: kit::ScrollOptions {
                        viewport_test_id: Some(Arc::from(shell_viewport_id)),
                        ..Default::default()
                    },
                    test_id: Some(Arc::from(shell_id.clone())),
                    content_test_id: Some(Arc::from(shell_content_id)),
                    ..Default::default()
                },
                |ui| {
                    ui.text(format!("Shell-mounted pane proof for {pane_id}"));
                    ui.text_wrapped(
                        "Nested child regions stay app-composed in the workspace shell; no helper widening is required for this slice.",
                    );

                    ui.child_region_with_options(
                        toolbar_id.as_str(),
                        kit::ChildRegionOptions {
                            layout: LayoutRefinement::default().w_full().h_px(Px(60.0)),
                            scroll: kit::ScrollOptions {
                                viewport_test_id: Some(Arc::from(toolbar_viewport_id)),
                                ..Default::default()
                            },
                            test_id: Some(Arc::from(toolbar_id.clone())),
                            content_test_id: Some(Arc::from(format!("{toolbar_id}.content"))),
                            ..Default::default()
                        },
                        |ui| {
                            ui.separator_text("Toolbar");
                            ui.horizontal_with_options(
                                kit::HorizontalOptions {
                                    gap: MetricRef::space(Space::N2),
                                    ..Default::default()
                                },
                                |ui| {
                                    ui.text("Open Preview");
                                    ui.text("Split Right");
                                    ui.text("Float");
                                    ui.text("Focus Inspector");
                                },
                            );
                        },
                    );

                    ui.horizontal_with_options(
                        kit::HorizontalOptions {
                            gap: MetricRef::space(Space::N2),
                            items: fret_ui_kit::Items::Stretch,
                            ..Default::default()
                        },
                        |ui| {
                            ui.child_region_with_options(
                                tabs_id.as_str(),
                                kit::ChildRegionOptions {
                                    layout: LayoutRefinement::default().w_px(Px(200.0)).h_px(Px(148.0)),
                                    scroll: kit::ScrollOptions {
                                        viewport_test_id: Some(Arc::from(tabs_viewport_id)),
                                        ..Default::default()
                                    },
                                    test_id: Some(Arc::from(tabs_id.clone())),
                                    content_test_id: Some(Arc::from(format!("{tabs_id}.content"))),
                                    ..Default::default()
                                },
                                |ui| {
                                    ui.separator_text("Tabs");
                                    ui.text(format!("Active tab: {active_tab_label}"));
                                    ui.text(format!("Tabs in pane: {tab_count}"));
                                    ui.text(if two_row_pinned {
                                        "Tab strip layout: pinned two-row"
                                    } else {
                                        "Tab strip layout: auto"
                                    });
                                },
                            );

                            ui.child_region_with_options(
                                inspector_id.as_str(),
                                kit::ChildRegionOptions {
                                    layout: LayoutRefinement::default().w_full().h_px(Px(148.0)),
                                    scroll: kit::ScrollOptions {
                                        viewport_test_id: Some(Arc::from(inspector_viewport_id)),
                                        ..Default::default()
                                    },
                                    test_id: Some(Arc::from(inspector_id.clone())),
                                    content_test_id: Some(Arc::from(format!(
                                        "{inspector_id}.content"
                                    ))),
                                    ..Default::default()
                                },
                                |ui| {
                                    ui.separator_text("Inspector");
                                    ui.text(format!("Pane id: {pane_id}"));
                                    ui.text(if is_active {
                                        "Focus state: active pane"
                                    } else {
                                        "Focus state: passive pane"
                                    });
                                    ui.text(format!("Dirty tabs: {dirty_count}"));
                                },
                            );
                        },
                    );

                    ui.child_region_with_options(
                        status_id.as_str(),
                        kit::ChildRegionOptions {
                            layout: LayoutRefinement::default().w_full().h_px(Px(76.0)),
                            scroll: kit::ScrollOptions {
                                viewport_test_id: Some(Arc::from(status_viewport_id)),
                                ..Default::default()
                            },
                            test_id: Some(Arc::from(status_id.clone())),
                            content_test_id: Some(Arc::from(format!("{status_id}.content"))),
                            ..Default::default()
                        },
                        |ui| {
                            ui.separator_text("Status");
                            ui.text(if prompt_open {
                                "Dirty close prompt: open"
                            } else {
                                "Dirty close prompt: closed"
                            });
                            ui.text_wrapped("Decision: keep the current `child_region` seam for M3.");
                        },
                    );
                },
            );
        });
    })
    .w_full()
    .h_full()
    .into_element(cx)
}

fn create_window_state(app: &mut App, _window: AppWindowId) -> WorkspaceShellWindowState {
    let view_cache_shell = env_bool("FRET_EXAMPLES_VIEW_CACHE_SHELL", false);

    let mut window_layout = WorkspaceWindowLayout::new("window-1", "pane-a");
    window_layout.pane_tree = WorkspacePaneTree::split(
        Axis::Horizontal,
        0.62,
        WorkspacePaneTree::leaf("pane-a"),
        WorkspacePaneTree::leaf("pane-b"),
    );
    window_layout.active_pane = Some(Arc::from("pane-a"));

    if let Some(pane) = window_layout.pane_tree.find_pane_mut("pane-a") {
        pane.tabs.open_and_activate(Arc::from("doc-a-0"));
        pane.tabs.open_and_activate(Arc::from("doc-a-1"));
        pane.tabs.open_and_activate(Arc::from("doc-a-2"));
    }
    if let Some(pane) = window_layout.pane_tree.find_pane_mut("pane-b") {
        pane.tabs.open_and_activate(Arc::from("doc-b-0"));
        pane.tabs.open_and_activate(Arc::from("doc-b-1"));
    }

    let (items_value, state_value) = build_file_tree_items();
    let block_dirty_close = env_bool("FRET_WORKSPACE_SHELL_DEBUG_DIRTY_CLOSE_POLICY", false);
    let models = WorkspaceShellModelBundle::new(
        app,
        window_layout,
        items_value,
        state_value,
        block_dirty_close,
    );

    WorkspaceShellWindowState {
        view_cache_shell,
        window_layout: models.window_layout,
        workbench: models.workbench,
        tabstrip_two_row_pinned: models.tabstrip_two_row_pinned,
        file_tree_items: models.file_tree_items,
        file_tree_state: models.file_tree_state,
        file_tree_scroll: VirtualListScrollHandle::new(),
        frame_stage_frame: None,
        frame_stages: Vec::new(),
        completed_frame_stages: Vec::new(),
    }
}

fn render_workspace_shell(
    cx: &mut fret::AppRenderCx<'_>,
    state: &mut WorkspaceShellWindowState,
) -> fret::Ui {
    let frame_stage_label = state.completed_frame_stage_label();
    let view_cache_shell = state.view_cache_shell;
    let window_layout = state.window_layout.clone();
    let dirty_close_prompt_open = state.workbench.dirty_close_prompt_open().clone();
    let dirty_close_prompt = state.workbench.dirty_close_prompt().clone();
    let tabstrip_two_row_pinned = state.tabstrip_two_row_pinned.clone();
    let file_tree_items = state.file_tree_items.clone();
    let file_tree_state = state.file_tree_state.clone();
    let file_tree_scroll = state.file_tree_scroll.clone();

    cx.observe_model(&window_layout, Invalidation::Layout);
    cx.observe_model(&dirty_close_prompt_open, Invalidation::Layout);
    cx.observe_model(&dirty_close_prompt, Invalidation::Layout);
    cx.observe_model(&tabstrip_two_row_pinned, Invalidation::Layout);
    cx.observe_model(&file_tree_items, Invalidation::Layout);
    cx.observe_model(&file_tree_state, Invalidation::Layout);

    let theme = cx.theme_snapshot();
    let bg = Some(theme.color_token("background"));
    let (prompt_open, prompt): (bool, Option<fret_workspace::WorkspaceDirtyClosePrompt>) =
        cx.data().selector_model_layout(
            (&dirty_close_prompt_open, &dirty_close_prompt),
            |(prompt_open, prompt)| (prompt_open, prompt),
        );
    if prompt_open {
        let (reason, dirty_list, active_tab, close_count) = prompt
            .as_ref()
            .map(|p| {
                let reason = Arc::<str>::from(format!("{:?}", p.request.reason));
                let dirty_list = Arc::<str>::from(
                    p.request
                        .dirty_tabs_in_order
                        .iter()
                        .map(|t| t.as_ref())
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                let active_tab = p
                    .request
                    .active_tab_id
                    .as_ref()
                    .map(|t| Arc::<str>::from(t.as_ref()))
                    .unwrap_or_else(|| Arc::from("<none>"));
                let close_count = p.request.target_tabs_in_order.len();
                (reason, dirty_list, active_tab, close_count)
            })
            .unwrap_or_else(|| {
                (
                    Arc::from("<unknown>"),
                    Arc::from("<unknown>"),
                    Arc::from("<none>"),
                    0,
                )
            });
        let prompt_label = Arc::<str>::from(format!(
            "Dirty close confirmation reason={reason} active={active_tab} close_count={close_count} dirty=[{dirty_list}]"
        ));

        let dim_bg = Some(theme.color_token("muted"));
        let dialog_bg = Some(theme.color_token("card"));
        let border = Some(theme.color_token("border"));

        let cancel_cmd = typed_command_id::<act::WorkspaceDirtyCloseCancel>();
        let discard_cmd = typed_command_id::<act::WorkspaceDirtyCloseDiscard>();
        let save_cmd = typed_command_id::<act::WorkspaceDirtyCloseSaveAndClose>();
        let cancel_button_id = Rc::new(Cell::new(None));
        let cancel_button_id_for_overlay = cancel_button_id.clone();

        let overlay_root = cx.container(
                            ContainerProps {
                                layout: fill_layout(),
                                ..Default::default()
                            },
                            move |cx| {
                                let mut center = FlexProps {
                                    layout: fill_layout(),
                                    direction: Axis::Vertical,
                                    gap: fret_ui::element::SpacingLength::Px(Px(12.0)),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                    ..Default::default()
                                };
                                center.layout.size.width = Length::Fill;
                                center.layout.size.height = Length::Fill;

                                let mut dialog_container_layout = LayoutStyle::default();
                                dialog_container_layout.size.width = Length::Px(Px(520.0));
                                dialog_container_layout.size.height = Length::Auto;

                                let dialog = cx.container(
                                    ContainerProps {
                                        layout: dialog_container_layout,
                                        background: dialog_bg,
                                        border: Edges::all(Px(1.0)),
                                        border_color: border,
                                        padding: Edges::all(Px(16.0)).into(),
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        vec![cx.semantics(
                                            SemanticsProps {
                                                layout: fill_layout(),
                                                role: SemanticsRole::Dialog,
                                                label: Some(prompt_label.clone()),
                                                test_id: Some(Arc::from(
                                                    "workspace-shell-dirty-close-prompt",
                                                )),
                                                ..Default::default()
                                            },
                                            move |cx| {
                                                vec![
                                                    workspace_shell_section_chrome_label(
                                                        cx,
                                                        "Dirty close confirmation",
                                                    ),
                                                    workspace_shell_readout_text(
                                                        cx,
                                                        Arc::<str>::from(format!(
                                                            "reason={reason} active={active_tab} close_count={close_count}"
                                                        )),
                                                    ),
                                                    workspace_shell_readout_text(
                                                        cx,
                                                        Arc::<str>::from(format!(
                                                            "dirty=[{dirty_list}]"
                                                        )),
                                                    ),
                                                    cx.flex(
                                                        FlexProps {
                                                            layout: {
                                                                let mut layout =
                                                                    LayoutStyle::default();
                                                                layout.size.width = Length::Fill;
                                                                layout.size.height = Length::Auto;
                                                                layout
                                                            },
                                                            direction: Axis::Horizontal,
                                                            gap:
                                                                fret_ui::element::SpacingLength::Px(
                                                                    Px(12.0),
                                                                ),
                                                            justify: MainAlign::End,
                                                            align: CrossAlign::Center,
                                                            wrap: false,
                                                            ..Default::default()
                                                        },
                                                        move |cx| {
                                                            let (cancel, cancel_id) =
                                                                workspace_shell_command_button_with_focus_policy(
                                                                    cx,
                                                                    "workspace-shell-dirty-close-prompt.cancel",
                                                                    "Cancel",
                                                                    cancel_cmd.clone(),
                                                                    Px(28.0),
                                                                    Px(8.0),
                                                                    true,
                                                                );
                                                            cancel_button_id_for_overlay
                                                                .set(Some(cancel_id));
                                                            let (discard, _) =
                                                                workspace_shell_command_button_with_focus_policy(
                                                                    cx,
                                                                    "workspace-shell-dirty-close-prompt.discard",
                                                                    "Discard && Close",
                                                                    discard_cmd.clone(),
                                                                    Px(28.0),
                                                                    Px(8.0),
                                                                    true,
                                                                );
                                                            let (save, _) =
                                                                workspace_shell_command_button_with_focus_policy(
                                                                    cx,
                                                                    "workspace-shell-dirty-close-prompt.save_and_close",
                                                                    "Save && Close",
                                                                    save_cmd.clone(),
                                                                    Px(28.0),
                                                                    Px(8.0),
                                                                    true,
                                                                );
                                                            vec![cancel, discard, save]
                                                        },
                                                    ),
                                                ]
                                            },
                                        )]
                                    },
                                );

                                vec![cx.container(
                                    ContainerProps {
                                        layout: fill_layout(),
                                        background: dim_bg,
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        vec![cx.flex(center, move |_cx| {
                                            vec![dialog]
                                        })]
                                    },
                                )]
                            },
                        );
        let cancel_button_id = cancel_button_id
            .get()
            .expect("dirty-close Cancel button should expose its element id");

        let dismiss_handler: fret_ui_kit::primitives::dismissable_layer::OnDismissRequest =
            Arc::new(move |host, acx, req| {
                let reason = match req.reason {
                    fret_ui_kit::primitives::dismissable_layer::DismissReason::OutsidePress {
                        ..
                    } => fret_ui::action::ActivateReason::Pointer,
                    _ => fret_ui::action::ActivateReason::Keyboard,
                };
                let command = typed_command_id::<act::WorkspaceDirtyCloseCancel>();
                host.record_pending_command_dispatch_source(acx, &command, reason);
                host.dispatch_command(Some(acx.window), command);
            });

        let mut req = OverlayRequest::modal(
            DIRTY_CLOSE_PROMPT_OVERLAY_ID,
            None,
            dirty_close_prompt_open.clone(),
            OverlayPresence::instant(true),
            vec![overlay_root],
        );
        req.root_name = Some(OverlayController::modal_root_name(
            DIRTY_CLOSE_PROMPT_OVERLAY_ID,
        ));
        apply_dirty_close_overlay_focus_policy(&mut req, cancel_button_id);
        req.dismissible_on_dismiss_request = Some(dismiss_handler);
        OverlayController::request(cx, req);
    }

    let theme_for_left = theme.clone();
    let left = cx.keyed("workspace_shell.left", move |cx| {
        let mut props = FileTreeViewProps::default();
        props.layout = fill_layout();
        props.layout.overflow = Overflow::Clip;
        props.debug_root_test_id = Some(Arc::<str>::from("workspace-shell-file-tree-root"));
        props.debug_row_test_id_prefix = Some(Arc::<str>::from("workspace-shell-file-tree-node"));
        props.keep_alive =
            env_usize("FRET_WORKSPACE_SHELL_FILE_TREE_KEEP_ALIVE").filter(|v| *v > 0);

        cx.container(
            ContainerProps {
                layout: fixed_width_fill_height(Px(280.0)),
                background: Some(theme_for_left.color_token("card")),
                border: Edges::all(Px(1.0)),
                border_color: Some(theme_for_left.color_token("border")),
                ..Default::default()
            },
            move |cx| {
                vec![file_tree_view_retained_v0(
                    cx,
                    file_tree_items.clone(),
                    file_tree_state.clone(),
                    &file_tree_scroll,
                    props.clone(),
                )]
            },
        )
    });

    let two_row_pinned = cx
        .data()
        .selector_model_layout(&tabstrip_two_row_pinned, |two_row_pinned| two_row_pinned);
    let (active_pane_label, active_tab_label, active_tab_count, active_dirty_count): (
        Arc<str>,
        Arc<str>,
        usize,
        usize,
    ) = cx.data().selector_model_layout(&window_layout, |layout| {
        let active_pane_label = layout
            .active_pane
            .clone()
            .unwrap_or_else(|| Arc::from("<none>"));
        let (active_tab_label, active_tab_count, active_dirty_count) = layout
            .active_pane
            .as_ref()
            .and_then(|pane_id| layout.pane_tree.find_pane(pane_id.as_ref()))
            .map(|pane| {
                let active_tab_label = pane
                    .tabs
                    .active()
                    .cloned()
                    .unwrap_or_else(|| Arc::from("<none>"));
                let active_tab_count = pane.tabs.tabs().len();
                let active_dirty_count = pane.tabs.dirty_in_tab_order().len();
                (active_tab_label, active_tab_count, active_dirty_count)
            })
            .unwrap_or_else(|| (Arc::from("<none>"), 0, 0));
        (
            active_pane_label,
            active_tab_label,
            active_tab_count,
            active_dirty_count,
        )
    });
    let theme_for_center = theme.clone();
    let window_layout_for_center = window_layout.clone();
    let center = cx.keyed("workspace_shell.center", move |cx| {
                        let mut render_pane =
                            move |cx: &mut fret_ui::ElementContext<'_, App>,
                                  pane: &fret_workspace::layout::WorkspacePaneLayout,
                                  is_active: bool,
                                  tab_drag| {
                                let title = |id: &str| Arc::<str>::from(id);
                                let strip =
                                    WorkspaceTabStrip::from_workspace_tabs(&pane.tabs, title)
                                        .separate_pinned_row(two_row_pinned)
                                        .pane_id(pane.id.clone())
                                        .tab_drag_model(tab_drag)
                                        .test_id_root(Arc::<str>::from(format!(
                                            "workspace-shell-pane-{}-tab-strip",
                                            pane.id.as_ref()
                                        )))
                                        .tab_test_id_prefix(Arc::<str>::from(format!(
                                            "workspace-shell-pane-{}-tab",
                                            pane.id.as_ref()
                                        )))
                                        .into_element(cx);

                                let mut body_layout = LayoutStyle::default();
                                body_layout.size.width = Length::Fill;
                                body_layout.size.height = Length::Fill;
                                body_layout.flex.grow = 1.0;
                                body_layout.flex.basis = Length::Px(Px(0.0));
                                body_layout.overflow = Overflow::Clip;

                                let pane_content_bg = Some(theme_for_center.color_token("muted"));
                                let pane_active_label: Arc<str> = pane
                                    .tabs
                                    .active()
                                    .cloned()
                                    .unwrap_or_else(|| Arc::from("<none>"));
                                let pane_tab_count = pane.tabs.tabs().len();
                                let pane_dirty_count = pane.tabs.dirty_in_tab_order().len();
                                let pane_proof_state = WorkspaceShellPaneProofState {
                                    pane_id: pane.id.clone(),
                                    active_tab_label: pane_active_label.clone(),
                                    tab_count: pane_tab_count,
                                    dirty_count: pane_dirty_count,
                                    is_active,
                                    two_row_pinned,
                                    prompt_open,
                                };

                                let pane_root_test_id: Arc<str> = Arc::from(format!(
                                    "workspace-shell-pane-{}-root",
                                    pane.id.as_ref()
                                ));
                                cx.semantics(
                                    SemanticsProps {
                                        layout: fill_layout(),
                                        role: SemanticsRole::Panel,
                                        test_id: Some(pane_root_test_id),
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        vec![cx.flex(
                                            FlexProps {
                                                layout: fill_layout(),
                                                direction: Axis::Vertical,
                                                gap: fret_ui::element::SpacingLength::Px(Px(0.0)),
                                                justify: MainAlign::Start,
                                                align: CrossAlign::Stretch,
                                                wrap: false,
                                                ..Default::default()
                                            },
                                            move |cx| {
                                                let content = cx.pressable(
                                                    PressableProps {
                                                        layout: body_layout,
                                                        enabled: true,
                                                        focusable: true,
                                                        a11y: PressableA11y {
                                                            role: Some(SemanticsRole::TextField),
                                                            label: Some(Arc::from("Pane content")),
                                                            test_id: Some(Arc::from(format!(
                                                                "workspace-shell-pane-{}-content",
                                                                pane.id.as_ref()
                                                            ))),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                    move |cx, _state| {
                                                        vec![cx.container(
                                                            ContainerProps {
                                                                layout: fill_layout(),
                                                                background: pane_content_bg,
                                                                ..Default::default()
                                                            },
                                                            move |cx| {
                                                                vec![workspace_shell_pane_proof(
                                                                    cx,
                                                                    pane_proof_state.clone(),
                                                                )]
                                                            },
                                                        )]
                                                    },
                                                );
                                                let content = WorkspacePaneContentFocusTarget::new(
                                                    pane.id.clone(),
                                                    content,
                                                )
                                                .into_element(cx);
                                                let debug_preview = (env_bool(
                                                    "FRET_WORKSPACE_SHELL_DEBUG_PREVIEW",
                                                    false,
                                                ) || env_bool("FRET_DIAG", false)
                                                    || env_bool("FRET_DIAG_DIR", false))
                                                    && pane.id.as_ref() == "pane-a";
                                                let mut children = vec![strip];
                                                if debug_preview {
                                                    let open_a = tab_open_preview_command(
                                                        "doc-a-preview-a",
                                                    )
                                                    .expect("static preview tab id");
                                                    let open_b = tab_open_preview_command(
                                                        "doc-a-preview-b",
                                                    )
                                                    .expect("static preview tab id");
                                                    let commit = typed_command_id::<
                                                        act::WorkspaceTabCommitPreview,
                                                    >();
                                                    let toggle_pin = typed_command_id::<
                                                        act::WorkspaceTabTogglePin,
                                                    >();
                                                    let pin_doc_a_0 = tab_pin_command("doc-a-0")
                                                        .expect("static tab id");
                                                    let pin_doc_a_1 = tab_pin_command("doc-a-1")
                                                        .expect("static tab id");
                                                    let set_dirty = typed_command_id::<
                                                        shell_act::SetActiveDirty,
                                                    >();
                                                    let set_pane_b_dirty = typed_command_id::<
                                                        shell_act::SetPaneBActiveDirty,
                                                    >();
                                                    let clear_dirty = typed_command_id::<
                                                        shell_act::ClearActiveDirty,
                                                    >();
                                                    let toggle_two_row_pinned = typed_command_id::<
                                                        shell_act::ToggleTabstripTwoRowPinned,
                                                    >();
                                                    let close_window = typed_command_id::<
                                                        shell_act::CloseWindow,
                                                    >();
                                                    let close_others = typed_command_id::<
                                                        act::WorkspaceTabCloseOthers,
                                                    >();
                                                    let close_active = typed_command_id::<
                                                        shell_act::DebugCloseActivePaneA,
                                                    >();
                                                    let close_left = typed_command_id::<
                                                        act::WorkspaceTabCloseLeft,
                                                    >();
                                                    let close_right = typed_command_id::<
                                                        act::WorkspaceTabCloseRight,
                                                    >();

                                                    let bar_primary = cx.flex(
                                                        FlexProps {
                                                            layout: {
                                                                let mut layout =
                                                                    LayoutStyle::default();
                                                                layout.size.width = Length::Fill;
                                                                layout.size.height =
                                                                    Length::Px(Px(28.0));
                                                                layout.flex.shrink = 0.0;
                                                                layout
                                                            },
                                                            direction: Axis::Horizontal,
                                                            gap: fret_ui::element::SpacingLength::Px(
                                                                Px(8.0),
                                                            ),
                                                            justify: MainAlign::Start,
                                                            align: CrossAlign::Center,
                                                            wrap: false,
                                                            ..Default::default()
                                                        },
                                                        move |cx| {
                                                            vec![
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-mark-dirty",
                                                                    "Mark dirty",
                                                                    set_dirty.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-clear-dirty",
                                                                    "Clear dirty",
                                                                    clear_dirty.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-close-active",
                                                                    "Close active (pane-a)",
                                                                    close_active.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-mark-pane-b-dirty",
                                                                    "Mark pane-b dirty",
                                                                    set_pane_b_dirty.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                            ]
                                                        },
                                                    );
                                                    let bar_secondary = cx.flex(
                                                        FlexProps {
                                                            layout: {
                                                                let mut layout =
                                                                    LayoutStyle::default();
                                                                layout.size.width = Length::Fill;
                                                                layout.size.height = Length::Auto;
                                                                layout.flex.shrink = 0.0;
                                                                layout
                                                            },
                                                            direction: Axis::Horizontal,
                                                            gap: fret_ui::element::SpacingLength::Px(
                                                                Px(8.0),
                                                            ),
                                                            justify: MainAlign::Start,
                                                            align: CrossAlign::Center,
                                                            wrap: true,
                                                            ..Default::default()
                                                        },
                                                        move |cx| {
                                                            vec![
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-toggle-tabstrip-two-row-pinned",
                                                                    "Toggle pinned row layout",
                                                                    toggle_two_row_pinned.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-pin-doc-a-0",
                                                                    "Pin doc-a-0",
                                                                    pin_doc_a_0.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-pin-doc-a-1",
                                                                    "Pin doc-a-1",
                                                                    pin_doc_a_1.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-open-preview-a",
                                                                    "Open preview A",
                                                                    open_a.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-open-preview-b",
                                                                    "Open preview B",
                                                                    open_b.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-commit-preview",
                                                                    "Commit preview",
                                                                    commit.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-toggle-pin",
                                                                    "Toggle pin",
                                                                    toggle_pin.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-close-others",
                                                                    "Close others",
                                                                    close_others.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-close-left",
                                                                    "Close left",
                                                                    close_left.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-close-right",
                                                                    "Close right",
                                                                    close_right.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                                workspace_shell_command_button(
                                                                    cx,
                                                                    "workspace-shell-debug-close-window",
                                                                    "Close window",
                                                                    close_window.clone(),
                                                                    Px(22.0),
                                                                    Px(6.0),
                                                                ),
                                                            ]
                                                        },
                                                    );
                                                    children.push(bar_primary);
                                                    children.push(bar_secondary);
	                                                }
	                                                children.push(content);
	                                                children
	                                            },
	                                        )]
                                    },
                                )
                            };

                        workspace_pane_tree_element_with_resize(
                            cx,
                            window_layout_for_center.clone(),
                            &mut render_pane,
                        )
                    });
    let right = cx.keyed("workspace_shell.right", move |cx| {
        workspace_shell_editor_rail(
            cx,
            WorkspaceShellEditorRailState {
                active_pane_label,
                active_tab_label,
                active_tab_count,
                active_dirty_count,
                two_row_pinned,
                prompt_open,
                frame_stage_label,
            },
        )
    });

    let frame = WorkspaceFrame::new(center)
        .left(left)
        .right(right)
        .background(bg)
        .into_element(cx);

    let out = if view_cache_shell {
        let mut props = ViewCacheProps::default();
        props.layout = fill_layout();
        props = props.contain_layout_when_bounds_known(true);
        cx.view_cache(props, move |_cx| vec![frame])
    } else {
        frame
    };

    vec![WorkspaceCommandScope::new(window_layout.clone(), out).into_element(cx)].into()
}

impl fret::workspace::WorkspaceWindowState for WorkspaceShellWindowState {
    fn workspace_workbench(&self) -> &WorkspaceWorkbench {
        &self.workbench
    }

    fn save_workspace_dirty_close(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        _request: &fret_workspace::close_policy::WorkspaceDirtyCloseRequest,
    ) -> bool {
        // This diagnostic shell uses synthetic documents with no backing storage. Real apps must
        // persist every requested document here and return false on any save failure.
        true
    }

    fn handle_workspace_command(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        command: &CommandId,
        _source: &fret_runtime::CommandDispatchSourceV1,
    ) -> Option<WorkspaceWorkbenchCommandOutcome> {
        let set_active_dirty = typed_command_id::<shell_act::SetActiveDirty>();
        let set_pane_b_active_dirty = typed_command_id::<shell_act::SetPaneBActiveDirty>();
        let clear_active_dirty = typed_command_id::<shell_act::ClearActiveDirty>();
        if command == &set_active_dirty
            || command == &set_pane_b_active_dirty
            || command == &clear_active_dirty
        {
            let pane_id = if command == &set_pane_b_active_dirty {
                "pane-b"
            } else {
                "pane-a"
            };
            let dirty = command != &clear_active_dirty;
            let mut target = None;
            let applied = app
                .models_mut()
                .update(&self.window_layout, |layout| {
                    let Some(pane) = layout.pane_tree.find_pane_mut(pane_id) else {
                        return false;
                    };
                    let Some(active) = pane
                        .tabs
                        .active()
                        .cloned()
                        .or_else(|| pane.tabs.tabs().first().cloned())
                    else {
                        return false;
                    };
                    let _ = pane.tabs.activate(active.clone());
                    pane.tabs.set_dirty(active.clone(), dirty);
                    target = Some(Arc::from(format!("{pane_id}/{active}")));
                    true
                })
                .unwrap_or(false);
            return Some(WorkspaceWorkbenchCommandOutcome {
                handled: true,
                action_id: None,
                target,
                applied,
                blocked_dirty_close: false,
                close_window: false,
                focus: None,
            });
        }

        if command == &typed_command_id::<shell_act::ToggleTabstripTwoRowPinned>() {
            let applied = app
                .models_mut()
                .update(&self.tabstrip_two_row_pinned, |value| *value = !*value)
                .is_ok();
            return Some(WorkspaceWorkbenchCommandOutcome {
                handled: true,
                action_id: None,
                target: Some(Arc::from("tabstrip")),
                applied,
                blocked_dirty_close: false,
                close_window: false,
                focus: None,
            });
        }

        if command == &typed_command_id::<shell_act::DebugCloseActivePaneA>() {
            let workbench = self.workbench.clone();
            let close = typed_command_id::<act::WorkspaceTabClose>();
            return Some(workbench.apply_command_in_pane(app, window, Arc::from("pane-a"), &close));
        }

        None
    }

    fn handle_workspace_global_changes(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        changed: &[std::any::TypeId],
    ) {
        if selected_workspace_shell_editor_theme_preset().is_some() {
            let _ =
                crate::sync_shadcn_host_theme_then_reapply_editor_preset_on_window_metrics_change(
                    app,
                    window,
                    changed,
                    WORKSPACE_SHELL_HOST_BASE_COLOR,
                    WORKSPACE_SHELL_HOST_DEFAULT_SCHEME,
                );
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    fret::workspace::WorkspaceApp::new("workspace-shell-demo")
        .window("fret-demo workspace_shell_demo", (1280.0, 720.0))
        .setup(install_workspace_shell_theme)
        .ui(create_window_state, render_workspace_shell)?
        .run()
        .map_err(anyhow::Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_ui::element::ElementKind;

    #[test]
    fn workspace_shell_frame_stage_sink_publishes_only_ended_frames() {
        let mut app = App::new();
        let mut state = create_window_state(&mut app, AppWindowId::default());
        let observation = |frame_id, stage| fret::app::UiAppFrameObservation {
            stage,
            window: AppWindowId::default(),
            bounds: fret_core::Rect::default(),
            scale_factor: 1.0,
            tick_id: fret_runtime::TickId(1),
            frame_id: fret_runtime::FrameId(frame_id),
        };

        fret::app::UiAppFrameStageSink::record_frame_stage(
            &mut state,
            observation(1, fret::app::UiAppFrameStage::Begin),
        );
        fret::app::UiAppFrameStageSink::record_frame_stage(
            &mut state,
            observation(1, fret::app::UiAppFrameStage::View),
        );
        fret::app::UiAppFrameStageSink::record_frame_stage(
            &mut state,
            observation(2, fret::app::UiAppFrameStage::Begin),
        );
        assert!(
            state.completed_frame_stages.is_empty(),
            "an interrupted frame must not be published as complete"
        );

        fret::app::UiAppFrameStageSink::record_frame_stage(
            &mut state,
            observation(2, fret::app::UiAppFrameStage::End),
        );

        assert_eq!(
            state.completed_frame_stages,
            vec![
                fret::app::UiAppFrameStage::Begin,
                fret::app::UiAppFrameStage::End,
            ]
        );
        assert_eq!(state.completed_frame_stage_label().as_ref(), "Begin > End");
    }

    #[test]
    fn dirty_close_buttons_opt_into_keyboard_focus() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let (button, button_id) = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            fret_core::Rect::default(),
            "dirty-close-button-test",
            |cx| {
                workspace_shell_command_button_with_focus_policy(
                    cx,
                    "dirty-close.cancel",
                    "Cancel",
                    CommandId::from("test.cancel"),
                    Px(28.0),
                    Px(8.0),
                    true,
                )
            },
        );

        assert_eq!(button.id, button_id);
        let ElementKind::Pressable(props) = button.kind else {
            panic!("dirty-close command should render a pressable");
        };
        assert!(props.focusable);
        assert_eq!(props.a11y.role, Some(SemanticsRole::Button));
    }

    #[test]
    fn dirty_close_overlay_leaves_restore_ownership_to_the_workbench() {
        let mut app = App::new();
        let open = app.models_mut().insert(true);
        let cancel = GlobalElementId(0xcafe);
        let mut request = OverlayRequest::modal(
            DIRTY_CLOSE_PROMPT_OVERLAY_ID,
            None,
            open,
            OverlayPresence::instant(true),
            Vec::new(),
        );
        apply_dirty_close_overlay_focus_policy(&mut request, cancel);
        assert_eq!(request.trigger, None);
        assert_eq!(request.initial_focus, Some(cancel));
        assert!(request.on_close_auto_focus.is_some());
    }
}
