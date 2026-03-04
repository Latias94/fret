use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use fret_core::{AppWindowId, Axis, Edges, Event, Px, Rect, SemanticsRole};
use fret_launch::{
    WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    WinitWindowContext,
};
use fret_runtime::{
    CommandDispatchDecisionV1, CommandDispatchSourceV1, CommandScope, PlatformCapabilities,
    WindowCommandDispatchDiagnosticsStore, WindowPendingCommandDispatchSourceService,
};
use fret_ui::declarative;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow, PressableA11y,
    PressableProps, SemanticsProps, ViewCacheProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{Invalidation, UiTree, VirtualListScrollHandle};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::declarative::file_tree::{FileTreeViewProps, file_tree_view_retained_v0};
use fret_ui_kit::{OverlayController, OverlayPresence, OverlayRequest};
use fret_ui_kit::{TreeItem, TreeState};
use fret_workspace::close_policy::{
    WorkspaceDirtyCloseDecision, WorkspaceDirtyClosePolicy, WorkspaceDirtyCloseRequest,
};
use fret_workspace::layout::{WorkspacePaneTree, WorkspaceWindowLayout};
use fret_workspace::{
    WorkspaceCommandScope, WorkspaceFrame, WorkspacePaneContentFocusTarget, WorkspaceTabStrip,
    workspace_pane_tree_element_with_resize,
};
use std::collections::HashSet;
use std::sync::Arc;

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

fn build_file_tree_items() -> (Vec<TreeItem>, TreeState) {
    let root_count = 80u64;
    let folders_per_root = 6u64;
    let leaves_per_folder = 25u64;

    let mut expanded: HashSet<u64> = HashSet::new();
    let mut roots: Vec<TreeItem> = Vec::with_capacity(root_count as usize);

    for r in 0..root_count {
        let root_id = r;
        expanded.insert(root_id);

        let mut folders: Vec<TreeItem> = Vec::with_capacity(folders_per_root as usize);
        for f in 0..folders_per_root {
            let folder_id = 1_000_000 + r * 100 + f;
            expanded.insert(folder_id);

            let mut leaves: Vec<TreeItem> = Vec::with_capacity(leaves_per_folder as usize);
            for l in 0..leaves_per_folder {
                let leaf_id = 2_000_000 + r * 10_000 + f * 100 + l;
                let label: Arc<str> = Arc::from(format!("leaf_{r}_{f}_{l}"));
                leaves.push(TreeItem::new(leaf_id, label).disabled(leaf_id % 97 == 0));
            }

            folders.push(
                TreeItem::new(folder_id, Arc::<str>::from(format!("dir_{r}_{f}"))).children(leaves),
            );
        }

        roots.push(TreeItem::new(root_id, Arc::<str>::from(format!("root_{r}"))).children(folders));
    }

    (
        roots,
        TreeState {
            selected: None,
            expanded,
        },
    )
}

struct WorkspaceShellWindowState {
    ui: UiTree<App>,
    view_cache_shell: bool,
    window_layout: fret_app::Model<WorkspaceWindowLayout>,
    dirty_close_prompt_open: fret_app::Model<bool>,
    dirty_close_prompt: fret_app::Model<Option<WorkspaceShellDirtyClosePrompt>>,
    file_tree_items: fret_app::Model<Vec<TreeItem>>,
    file_tree_state: fret_app::Model<TreeState>,
    file_tree_scroll: VirtualListScrollHandle,
}

#[derive(Default)]
struct WorkspaceShellDemoDriver;

const CMD_WORKSPACE_SHELL_DEMO_SET_ACTIVE_DIRTY: &str = "workspace.shell_demo.set_active_dirty";
const CMD_WORKSPACE_SHELL_DEMO_CLEAR_ACTIVE_DIRTY: &str = "workspace.shell_demo.clear_active_dirty";
const CMD_WORKSPACE_SHELL_DEMO_DEBUG_CLOSE_ACTIVE_PANE_A: &str =
    "workspace.shell_demo.debug_close_active_in_pane_a";
const CMD_WORKSPACE_SHELL_DEMO_DIRTY_CLOSE_CANCEL: &str = "workspace.shell_demo.dirty_close.cancel";
const CMD_WORKSPACE_SHELL_DEMO_DIRTY_CLOSE_DISCARD: &str =
    "workspace.shell_demo.dirty_close.discard";
const CMD_WORKSPACE_SHELL_DEMO_DIRTY_CLOSE_SAVE_AND_CLOSE: &str =
    "workspace.shell_demo.dirty_close.save_and_close";

const DIRTY_CLOSE_PROMPT_OVERLAY_ID: GlobalElementId = GlobalElementId(0x6a4e_5c1f_8f3b_1c20);

#[derive(Debug, Clone)]
struct WorkspaceShellDirtyClosePrompt {
    pane_id: Arc<str>,
    command: CommandId,
    request: WorkspaceDirtyCloseRequest,
}

struct WorkspaceShellDemoDirtyClosePolicy {
    block: bool,
}

impl WorkspaceDirtyClosePolicy for WorkspaceShellDemoDirtyClosePolicy {
    fn decide_dirty_close(
        &mut self,
        _request: &WorkspaceDirtyCloseRequest,
    ) -> WorkspaceDirtyCloseDecision {
        if self.block {
            WorkspaceDirtyCloseDecision::Block
        } else {
            WorkspaceDirtyCloseDecision::Allow
        }
    }
}

impl WorkspaceShellDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> WorkspaceShellWindowState {
        let view_cache_enabled = env_bool("FRET_EXAMPLES_VIEW_CACHE", false);
        let view_cache_shell = env_bool("FRET_EXAMPLES_VIEW_CACHE_SHELL", false);
        // Diagnostics scripts set `FRET_DIAG=1` and should stay lightweight. Opt into expensive
        // UI debug hotspots explicitly.
        let ui_debug_enabled = env_bool("FRET_UI_DEBUG", false);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(view_cache_enabled);
        ui.set_debug_enabled(ui_debug_enabled);

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

        let window_layout = app.models_mut().insert(window_layout);
        let dirty_close_prompt_open = app.models_mut().insert(false);
        let dirty_close_prompt = app.models_mut().insert(None);

        let (items_value, state_value) = build_file_tree_items();
        let file_tree_items = app.models_mut().insert(items_value);
        let file_tree_state = app.models_mut().insert(state_value);

        WorkspaceShellWindowState {
            ui,
            view_cache_shell,
            window_layout,
            dirty_close_prompt_open,
            dirty_close_prompt,
            file_tree_items,
            file_tree_state,
            file_tree_scroll: VirtualListScrollHandle::new(),
        }
    }

    fn render_ui(
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        state: &mut WorkspaceShellWindowState,
        bounds: Rect,
    ) {
        let view_cache_shell = state.view_cache_shell;
        let window_layout = state.window_layout.clone();
        let dirty_close_prompt_open = state.dirty_close_prompt_open.clone();
        let dirty_close_prompt = state.dirty_close_prompt.clone();
        let file_tree_items = state.file_tree_items.clone();
        let file_tree_state = state.file_tree_state.clone();
        let file_tree_scroll = state.file_tree_scroll.clone();

        let _root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("workspace-shell-demo", move |cx| {
                    cx.observe_model(&window_layout, Invalidation::Layout);
                    cx.observe_model(&dirty_close_prompt_open, Invalidation::Layout);
                    cx.observe_model(&dirty_close_prompt, Invalidation::Layout);
                    cx.observe_model(&file_tree_items, Invalidation::Layout);
                    cx.observe_model(&file_tree_state, Invalidation::Layout);

                    let theme = cx.theme_snapshot();
                    let bg = Some(theme.color_token("background"));
                    let prompt_open = cx
                        .get_model_cloned(&dirty_close_prompt_open, Invalidation::Layout)
                        .unwrap_or(false);
                    if prompt_open {
                        let prompt = cx
                            .get_model_cloned(&dirty_close_prompt, Invalidation::Layout)
                            .unwrap_or(None);

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

                        let dim_bg = Some(theme.color_token("muted"));
                        let dialog_bg = Some(theme.color_token("card"));
                        let border = Some(theme.color_token("border"));

                        let open_model = dirty_close_prompt_open.clone();
                        let prompt_model = dirty_close_prompt.clone();

                        let cancel_cmd =
                            CommandId::new(Arc::<str>::from(CMD_WORKSPACE_SHELL_DEMO_DIRTY_CLOSE_CANCEL));
                        let discard_cmd =
                            CommandId::new(Arc::<str>::from(CMD_WORKSPACE_SHELL_DEMO_DIRTY_CLOSE_DISCARD));
                        let save_cmd = CommandId::new(Arc::<str>::from(
                            CMD_WORKSPACE_SHELL_DEMO_DIRTY_CLOSE_SAVE_AND_CLOSE,
                        ));

                        let overlay_root = cx.container(
                            ContainerProps {
                                layout: fill_layout(),
                                ..Default::default()
                            },
                            move |cx| {
                                let button = |cx: &mut fret_ui::ElementContext<'_, App>,
                                              test_id: &str,
                                              label: &str,
                                              cmd: CommandId| {
                                    let test_id: Arc<str> = Arc::from(test_id);
                                    let label: Arc<str> = Arc::from(label);
                                    cx.pressable(
                                        PressableProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Auto;
                                                layout.size.height = Length::Px(Px(28.0));
                                                layout
                                            },
                                            enabled: true,
                                            focusable: false,
                                            a11y: PressableA11y {
                                                role: Some(SemanticsRole::Button),
                                                label: Some(label.clone()),
                                                test_id: Some(test_id),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        move |cx, _state| {
                                            cx.pressable_add_on_activate(Arc::new(
                                                move |host, acx, _reason| {
                                                    host.dispatch_command(Some(acx.window), cmd.clone());
                                                },
                                            ));
                                            vec![cx.container(
                                                ContainerProps {
                                                    layout: fill_layout(),
                                                    padding: Edges::all(Px(8.0)).into(),
                                                    ..Default::default()
                                                },
                                                move |cx| vec![cx.text(label.clone())],
                                            )]
                                        },
                                    )
                                };

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
                                                test_id: Some(Arc::from(
                                                    "workspace-shell-dirty-close-prompt",
                                                )),
                                                ..Default::default()
                                            },
                                            move |cx| {
                                                vec![
                                                    cx.text(Arc::<str>::from(
                                                        "Dirty close confirmation",
                                                    )),
                                                    cx.text(Arc::<str>::from(format!(
                                                        "reason={reason} active={active_tab} close_count={close_count}"
                                                    ))),
                                                    cx.text(Arc::<str>::from(format!(
                                                        "dirty=[{dirty_list}]"
                                                    ))),
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
                                                            vec![
                                                                button(
                                                                    cx,
                                                                    "workspace-shell-dirty-close-prompt.cancel",
                                                                    "Cancel",
                                                                    cancel_cmd.clone(),
                                                                ),
                                                                button(
                                                                    cx,
                                                                    "workspace-shell-dirty-close-prompt.discard",
                                                                    "Discard && Close",
                                                                    discard_cmd.clone(),
                                                                ),
                                                                button(
                                                                    cx,
                                                                    "workspace-shell-dirty-close-prompt.save_and_close",
                                                                    "Save && Close",
                                                                    save_cmd.clone(),
                                                                ),
                                                            ]
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

                        let dismiss_handler: fret_ui::action::OnDismissRequest =
                            Arc::new(move |host, _acx, _req| {
                            let _ = host.models_mut().update(&prompt_model, |p| *p = None);
                            let _ = host.models_mut().update(&open_model, |v| *v = false);
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
                        req.dismissible_on_dismiss_request = Some(dismiss_handler);
                        OverlayController::request(cx, req);
                    }

                    let theme_for_left = theme.clone();
                    let left = cx.keyed("workspace_shell.left", move |cx| {
                        let mut props = FileTreeViewProps::default();
                        props.layout = fill_layout();
                        props.layout.overflow = Overflow::Clip;
                        props.debug_root_test_id =
                            Some(Arc::<str>::from("workspace-shell-file-tree-root"));
                        props.debug_row_test_id_prefix =
                            Some(Arc::<str>::from("workspace-shell-file-tree-node"));
                        props.keep_alive = env_usize("FRET_WORKSPACE_SHELL_FILE_TREE_KEEP_ALIVE")
                            .filter(|v| *v > 0);

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
                                                                let active_label: Arc<str> = pane
                                                                    .tabs
                                                                    .active()
                                                                    .cloned()
                                                                    .unwrap_or_else(|| {
                                                                        Arc::from("<none>")
                                                                    });
                                                                let msg: Arc<str> =
                                                                    Arc::from(format!(
                                                                        "pane={} active={} {}",
                                                                        pane.id.as_ref(),
                                                                        active_label.as_ref(),
                                                                        if is_active {
                                                                            "(active)"
                                                                        } else {
                                                                            ""
                                                                        }
                                                                    ));
                                                                vec![cx.text(msg)]
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
                                                    let button = |cx: &mut fret_ui::ElementContext<'_, App>,
                                                                  test_id: &str,
                                                                  label: &str,
                                                                  cmd: CommandId| {
                                                        let test_id: Arc<str> = Arc::from(test_id);
                                                        let label: Arc<str> = Arc::from(label);
                                                        cx.pressable(
                                                            PressableProps {
                                                                layout: {
                                                                    let mut layout =
                                                                        LayoutStyle::default();
                                                                    layout.size.width = Length::Auto;
                                                                    layout.size.height =
                                                                        Length::Px(Px(22.0));
                                                                    layout
                                                                },
                                                                enabled: true,
                                                                focusable: false,
                                                                a11y: PressableA11y {
                                                                    role: Some(SemanticsRole::Button),
                                                                    label: Some(label.clone()),
                                                                    test_id: Some(test_id),
                                                                    ..Default::default()
                                                                },
                                                                ..Default::default()
                                                            },
                                                            move |cx, _state| {
                                                                cx.pressable_add_on_activate(
                                                                    Arc::new(move |host, acx, _reason| {
                                                                        host.dispatch_command(
                                                                            Some(acx.window),
                                                                            cmd.clone(),
                                                                        );
                                                                    }),
                                                                );
                                                                vec![cx.container(
                                                                    ContainerProps {
                                                                        layout: fill_layout(),
                                                                        padding: Edges::all(Px(6.0))
                                                                            .into(),
                                                                        ..Default::default()
                                                                    },
                                                                    move |cx| vec![cx.text(label.clone())],
                                                                )]
                                                            },
                                                        )
                                                    };

                                                    let open_a = CommandId::new(Arc::<str>::from(
                                                        "workspace.tab.open_preview.doc-a-preview-a",
                                                    ));
                                                    let open_b = CommandId::new(Arc::<str>::from(
                                                        "workspace.tab.open_preview.doc-a-preview-b",
                                                    ));
                                                    let commit = CommandId::new(Arc::<str>::from(
                                                        "workspace.tab.commit_preview",
                                                    ));
                                                    let toggle_pin = CommandId::new(Arc::<str>::from(
                                                        "workspace.tab.toggle_pin",
                                                    ));
                                                    let set_dirty = CommandId::new(Arc::<str>::from(
                                                        CMD_WORKSPACE_SHELL_DEMO_SET_ACTIVE_DIRTY,
                                                    ));
                                                    let clear_dirty = CommandId::new(Arc::<str>::from(
                                                        CMD_WORKSPACE_SHELL_DEMO_CLEAR_ACTIVE_DIRTY,
                                                    ));
	                                                    let close_others = CommandId::new(Arc::<str>::from(
	                                                        "workspace.tab.close.others",
	                                                    ));
	                                                    let close_active = CommandId::new(Arc::<str>::from(
	                                                        CMD_WORKSPACE_SHELL_DEMO_DEBUG_CLOSE_ACTIVE_PANE_A,
	                                                    ));
	                                                    let close_left = CommandId::new(Arc::<str>::from(
	                                                        "workspace.tab.close.left",
	                                                    ));
	                                                    let close_right = CommandId::new(Arc::<str>::from(
	                                                        "workspace.tab.close.right",
                                                    ));

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
	                                                                button(
	                                                                    cx,
	                                                                    "workspace-shell-pane-pane-a-debug-mark-dirty",
	                                                                    "Mark dirty",
                                                                    set_dirty.clone(),
                                                                ),
                                                                button(
                                                                    cx,
                                                                    "workspace-shell-pane-pane-a-debug-clear-dirty",
                                                                    "Clear dirty",
                                                                    clear_dirty.clone(),
                                                                ),
	                                                                button(
	                                                                    cx,
	                                                                    "workspace-shell-pane-pane-a-debug-close-active",
	                                                                    "Close active (pane-a)",
	                                                                    close_active.clone(),
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
	                                                                button(
	                                                                    cx,
	                                                                    "workspace-shell-pane-pane-a-debug-open-preview-a",
	                                                                    "Open preview A",
	                                                                    open_a.clone(),
	                                                                ),
	                                                                button(
	                                                                    cx,
	                                                                    "workspace-shell-pane-pane-a-debug-open-preview-b",
	                                                                    "Open preview B",
	                                                                    open_b.clone(),
	                                                                ),
	                                                                button(
	                                                                    cx,
	                                                                    "workspace-shell-pane-pane-a-debug-commit-preview",
	                                                                    "Commit preview",
	                                                                    commit.clone(),
	                                                                ),
	                                                                button(
	                                                                    cx,
	                                                                    "workspace-shell-pane-pane-a-debug-toggle-pin",
	                                                                    "Toggle pin",
	                                                                    toggle_pin.clone(),
	                                                                ),
	                                                                button(
	                                                                    cx,
	                                                                    "workspace-shell-pane-pane-a-debug-close-others",
	                                                                    "Close others",
	                                                                    close_others.clone(),
	                                                                ),
	                                                                button(
	                                                                    cx,
	                                                                    "workspace-shell-pane-pane-a-debug-close-left",
	                                                                    "Close left",
	                                                                    close_left.clone(),
	                                                                ),
	                                                                button(
	                                                                    cx,
	                                                                    "workspace-shell-pane-pane-a-debug-close-right",
	                                                                    "Close right",
	                                                                    close_right.clone(),
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

                    let frame = WorkspaceFrame::new(center)
                        .left(left)
                        .background(bg)
                        .into_element(cx);

                    let out = if view_cache_shell {
                        let mut props = ViewCacheProps::default();
                        props.layout = fill_layout();
                        props.contained_layout = true;
                        cx.view_cache(props, move |_cx| vec![frame])
                    } else {
                        frame
                    };

                    vec![WorkspaceCommandScope::new(window_layout.clone(), out).into_element(cx)]
                });
    }
}

impl WinitAppDriver for WorkspaceShellDemoDriver {
    type WindowState = WorkspaceShellWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        let WinitWindowContext { app, state, .. } = context;
        state.ui.propagate_model_changes(app, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        let WinitWindowContext { app, state, .. } = context;
        state.ui.propagate_global_changes(app, changed);
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: CommandId,
    ) {
        let WinitCommandContext {
            app,
            services,
            window,
            state,
        } = context;

        if matches!(
            command.as_str(),
            CMD_WORKSPACE_SHELL_DEMO_DIRTY_CLOSE_CANCEL
                | CMD_WORKSPACE_SHELL_DEMO_DIRTY_CLOSE_DISCARD
                | CMD_WORKSPACE_SHELL_DEMO_DIRTY_CLOSE_SAVE_AND_CLOSE
        ) {
            let prompt = app.models().get_cloned(&state.dirty_close_prompt).flatten();
            let do_discard = command.as_str() == CMD_WORKSPACE_SHELL_DEMO_DIRTY_CLOSE_DISCARD;
            let do_save = command.as_str() == CMD_WORKSPACE_SHELL_DEMO_DIRTY_CLOSE_SAVE_AND_CLOSE;

            if (do_discard || do_save) && prompt.is_some() {
                let prompt = prompt.unwrap();
                let _ = app.models_mut().update(
                    &state.window_layout,
                    |layout: &mut WorkspaceWindowLayout| {
                        layout.active_pane = Some(prompt.pane_id.clone());
                        let Some(pane) = layout.pane_tree.find_pane_mut(prompt.pane_id.as_ref())
                        else {
                            return;
                        };
                        if let Some(active) = prompt.request.active_tab_id.clone() {
                            let _ = pane.tabs.activate(active);
                        }
                        if do_save {
                            for id in prompt.request.dirty_tabs_in_order.clone() {
                                pane.tabs.set_dirty(id, false);
                            }
                        }
                        let _ = pane.tabs.apply_command(&prompt.command);
                    },
                );
            }

            let _ = app
                .models_mut()
                .update(&state.dirty_close_prompt, |p| *p = None);
            let _ = app
                .models_mut()
                .update(&state.dirty_close_prompt_open, |v| *v = false);
            app.request_redraw(window);
            return;
        }

        if command.as_str() == "window.close" {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        if matches!(
            command.as_str(),
            CMD_WORKSPACE_SHELL_DEMO_SET_ACTIVE_DIRTY | CMD_WORKSPACE_SHELL_DEMO_CLEAR_ACTIVE_DIRTY
        ) {
            let dirty = command.as_str() == CMD_WORKSPACE_SHELL_DEMO_SET_ACTIVE_DIRTY;
            let did_apply = app
                .models_mut()
                .update(
                    &state.window_layout,
                    |layout: &mut WorkspaceWindowLayout| {
                        let Some(pane) = layout.pane_tree.find_pane_mut("pane-a") else {
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
                        pane.tabs.set_dirty(active, dirty);
                        true
                    },
                )
                .unwrap_or(false);
            if did_apply {
                app.request_redraw(window);
            }
            return;
        }

        if command.as_str() == CMD_WORKSPACE_SHELL_DEMO_DEBUG_CLOSE_ACTIVE_PANE_A {
            let close_cmd = CommandId::new(Arc::<str>::from("workspace.tab.close"));

            let block_dirty_close =
                env_bool("FRET_WORKSPACE_SHELL_DEBUG_DIRTY_CLOSE_POLICY", false);
            let mut dirty_close_policy = WorkspaceShellDemoDirtyClosePolicy {
                block: block_dirty_close,
            };

            let update = app.models_mut().update(
                &state.window_layout,
                |layout: &mut WorkspaceWindowLayout| {
                    layout.active_pane = Some(Arc::from("pane-a"));
                    layout
                        .apply_command_with_close_policy(&close_cmd, Some(&mut dirty_close_policy))
                },
            );
            let outcome = update.unwrap_or(fret_workspace::tabs::WorkspaceApplyCommandOutcome {
                applied: false,
                blocked_dirty_close: None,
            });

            if let Some(req) = outcome.blocked_dirty_close.clone() {
                let _ = app.models_mut().update(&state.dirty_close_prompt, |p| {
                    *p = Some(WorkspaceShellDirtyClosePrompt {
                        pane_id: Arc::from("pane-a"),
                        command: close_cmd.clone(),
                        request: req,
                    });
                });
                let _ = app
                    .models_mut()
                    .update(&state.dirty_close_prompt_open, |v| *v = true);
            }

            if outcome.applied || outcome.blocked_dirty_close.is_some() {
                app.request_redraw(window);
            }
            return;
        }

        // Important: for "app model" commands (e.g. workspace tab operations), we still want to
        // apply the command even if some UI subtree reports it as handled (e.g. a context menu
        // item dispatching the command while focused inside the menu overlay).
        //
        // Diagnostics note: because the model application runs before UI command hooks, some UI
        // hooks become non-idempotent (e.g. close-by-id after the tab is already removed). Capture
        // pending source metadata up front so we can still emit a stable command dispatch trace
        // entry for the driver-applied outcome (ADR 0307).
        let pending_source = app.with_global_mut(
            WindowPendingCommandDispatchSourceService::default,
            |svc, app| {
                svc.consume(window, app.tick_id(), &command)
                    .unwrap_or_else(CommandDispatchSourceV1::programmatic)
            },
        );
        app.with_global_mut(
            WindowPendingCommandDispatchSourceService::default,
            |svc, app| {
                svc.record(window, app.tick_id(), command.clone(), pending_source);
            },
        );

        let block_dirty_close = env_bool("FRET_WORKSPACE_SHELL_DEBUG_DIRTY_CLOSE_POLICY", false);
        let mut dirty_close_policy = WorkspaceShellDemoDirtyClosePolicy {
            block: block_dirty_close,
        };
        let update = app.models_mut().update(
            &state.window_layout,
            |layout: &mut WorkspaceWindowLayout| {
                let active_pane_id = layout.active_pane.clone();
                (
                    layout.apply_command_with_close_policy(&command, Some(&mut dirty_close_policy)),
                    active_pane_id,
                )
            },
        );
        let (outcome, active_pane_id) = update.unwrap_or((
            fret_workspace::tabs::WorkspaceApplyCommandOutcome {
                applied: false,
                blocked_dirty_close: None,
            },
            None,
        ));

        let did_dispatch_ui = state.ui.dispatch_command(app, services, &command);
        if (outcome.applied || outcome.blocked_dirty_close.is_some()) && !did_dispatch_ui {
            let handled_by_scope = app
                .commands()
                .get(command.clone())
                .map(|m| m.scope)
                .or(Some(CommandScope::Window));
            app.with_global_mut(
                WindowCommandDispatchDiagnosticsStore::default,
                |store, app| {
                    store.record(CommandDispatchDecisionV1 {
                        seq: 0,
                        frame_id: app.frame_id(),
                        tick_id: app.tick_id(),
                        window,
                        command: command.clone(),
                        source: pending_source,
                        handled: true,
                        handled_by_element: None,
                        handled_by_scope,
                        handled_by_driver: true,
                        stopped: false,
                        started_from_focus: false,
                        used_default_root_fallback: false,
                    });
                },
            );
        }

        if let Some(req) = outcome.blocked_dirty_close.clone() {
            if let Some(pane_id) = active_pane_id {
                let _ = app.models_mut().update(&state.dirty_close_prompt, |p| {
                    *p = Some(WorkspaceShellDirtyClosePrompt {
                        pane_id,
                        command: command.clone(),
                        request: req,
                    });
                });
                let _ = app
                    .models_mut()
                    .update(&state.dirty_close_prompt_open, |v| *v = true);
            }
        }

        if outcome.applied || did_dispatch_ui {
            app.request_redraw(window);
        }
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
        } = context;
        if matches!(event, Event::WindowCloseRequested) {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }
        state.ui.dispatch_event(app, services, event);
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        } = context;

        OverlayController::begin_frame(app, window);
        Self::render_ui(app, services, window, state, bounds);
        OverlayController::render(&mut state.ui, app, services, window, bounds);

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        let inspection_active = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                svc.wants_inspection_active(window)
            });
        state.ui.set_inspection_active(inspection_active);

        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();

        let semantics_snapshot = state.ui.semantics_snapshot_arc();
        let drive = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            svc.drive_script_for_window(
                app,
                window,
                bounds,
                scale_factor,
                Some(&mut state.ui),
                semantics_snapshot.as_deref(),
            )
        });

        if drive.request_redraw {
            app.request_redraw(window);
            app.push_effect(Effect::RequestAnimationFrame(window));
        }

        let mut injected_any = false;
        for event in drive.events {
            injected_any = true;
            state.ui.dispatch_event(app, services, &event);
        }

        if injected_any {
            let mut deferred_effects: Vec<Effect> = Vec::new();
            loop {
                let effects = app.flush_effects();
                if effects.is_empty() {
                    break;
                }

                let mut applied_any_command = false;
                for effect in effects {
                    match effect {
                        Effect::Command { window: w, command } => {
                            if w.is_none() || w == Some(window) {
                                self.handle_command(
                                    WinitCommandContext {
                                        app,
                                        services,
                                        window,
                                        state,
                                    },
                                    command,
                                );
                                applied_any_command = true;
                            } else {
                                deferred_effects.push(Effect::Command { window: w, command });
                            }
                        }
                        other => deferred_effects.push(other),
                    }
                }

                if !applied_any_command {
                    break;
                }
            }
            for effect in deferred_effects {
                app.push_effect(effect);
            }

            state.ui.request_semantics_snapshot();
            let mut frame =
                fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
            frame.layout_all();
        }

        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.paint_all(scene);

        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
            svc.record_snapshot(
                app,
                window,
                bounds,
                scale_factor,
                &mut state.ui,
                element_runtime,
                scene,
            );
            let _ = svc.maybe_dump_if_triggered();
            if svc.is_enabled() {
                app.push_effect(Effect::RequestAnimationFrame(window));
            }
        });
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

    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    fret_workspace::commands::register_workspace_commands(app.commands_mut());
    fret_app::install_command_default_keybindings_into_keymap(&mut app);

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo workspace_shell_demo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(1280.0, 720.0),
        ..Default::default()
    };

    let driver = WorkspaceShellDemoDriver::default();
    fret::run_native_demo(config, app, driver).context("run workspace_shell_demo app")
}
