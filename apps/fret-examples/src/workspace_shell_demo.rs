use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, WindowRequest};
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use fret_core::{AppWindowId, Axis, Edges, Event, Px, Rect, SemanticsRole};
use fret_launch::{
    WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    SemanticsProps, ViewCacheProps,
};
use fret_ui::{Invalidation, UiTree, VirtualListScrollHandle};
use fret_ui_kit::declarative::file_tree::{FileTreeViewProps, file_tree_view_retained_v0};
use fret_ui_kit::{TreeItem, TreeState};
use fret_workspace::layout::{WorkspacePaneTree, WorkspaceWindowLayout};
use fret_workspace::{WorkspaceFrame, WorkspaceTabStrip, workspace_pane_tree_element_with_resize};
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
    file_tree_items: fret_app::Model<Vec<TreeItem>>,
    file_tree_state: fret_app::Model<TreeState>,
    file_tree_scroll: VirtualListScrollHandle,
}

#[derive(Default)]
struct WorkspaceShellDemoDriver;

impl WorkspaceShellDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> WorkspaceShellWindowState {
        let view_cache_enabled = env_bool("FRET_EXAMPLES_VIEW_CACHE", false);
        let view_cache_shell = env_bool("FRET_EXAMPLES_VIEW_CACHE_SHELL", false);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(view_cache_enabled);
        ui.set_debug_enabled(std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty()));

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

        let (items_value, state_value) = build_file_tree_items();
        let file_tree_items = app.models_mut().insert(items_value);
        let file_tree_state = app.models_mut().insert(state_value);

        WorkspaceShellWindowState {
            ui,
            view_cache_shell,
            window_layout,
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
        let file_tree_items = state.file_tree_items.clone();
        let file_tree_state = state.file_tree_state.clone();
        let file_tree_scroll = state.file_tree_scroll.clone();

        let _root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("workspace-shell-demo", move |cx| {
                    cx.observe_model(&window_layout, Invalidation::Layout);
                    cx.observe_model(&file_tree_items, Invalidation::Layout);
                    cx.observe_model(&file_tree_state, Invalidation::Layout);

                    let theme = cx.theme_snapshot();
                    let bg = Some(theme.color_required("background"));

                    let theme_for_left = theme;
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
                                background: Some(theme_for_left.color_required("card")),
                                border: Edges::all(Px(1.0)),
                                border_color: Some(theme_for_left.color_required("border")),
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

                    let theme_for_center = theme;
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

                                let pane_content_bg =
                                    Some(theme_for_center.color_required("muted"));

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
                                                gap: Px(0.0),
                                                justify: MainAlign::Start,
                                                align: CrossAlign::Stretch,
                                                wrap: false,
                                                ..Default::default()
                                            },
                                            move |cx| {
                                                let header = strip.clone();
                                                let content = cx.container(
                                                    ContainerProps {
                                                        layout: body_layout,
                                                        background: pane_content_bg,
                                                        ..Default::default()
                                                    },
                                                    move |cx| {
                                                        let active_label: Arc<str> = pane
                                                            .tabs
                                                            .active()
                                                            .cloned()
                                                            .unwrap_or_else(|| Arc::from("<none>"));
                                                        let msg: Arc<str> = Arc::from(format!(
                                                            "pane={} active={} {}",
                                                            pane.id.as_ref(),
                                                            active_label.as_ref(),
                                                            if is_active { "(active)" } else { "" }
                                                        ));
                                                        vec![cx.text(msg)]
                                                    },
                                                );
                                                vec![header, content]
                                            },
                                        )]
                                    },
                                )
                            };

                        workspace_pane_tree_element_with_resize(
                            cx,
                            window_layout.clone(),
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

                    vec![out]
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

        if state.ui.dispatch_command(app, services, &command) {
            return;
        }

        if command.as_str() == "window.close" {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        let did_apply = app
            .models_mut()
            .update(
                &state.window_layout,
                |layout: &mut WorkspaceWindowLayout| layout.apply_command(&command),
            )
            .unwrap_or(false);

        if did_apply {
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

        Self::render_ui(app, services, window, state, bounds);

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

        let semantics_snapshot = state.ui.semantics_snapshot();
        let drive = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
            svc.drive_script_for_window(
                app,
                window,
                bounds,
                scale_factor,
                Some(&state.ui),
                semantics_snapshot,
                element_runtime,
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
                &state.ui,
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
        main_window_size: winit::dpi::LogicalSize::new(1080.0, 720.0),
        ..Default::default()
    };

    let driver = WorkspaceShellDemoDriver::default();
    fret_kit::run_native_demo(config, app, driver).context("run workspace_shell_demo app")
}
