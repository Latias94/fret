use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_core::{AppWindowId, Corners, Edges, Event, Px};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
};
use fret_ui::{Invalidation, Theme, UiTree, VirtualListScrollHandle};
use fret_ui_kit::OverlayController;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::headless::table::{
    ColumnPinningState, GroupedColumnMode, RowKey, TableState, create_column_helper,
};
use fret_ui_shadcn::button::{Button, ButtonSize, ButtonVariant};
use fret_ui_shadcn::context_menu::{ContextMenu, ContextMenuEntry, ContextMenuItem};
use fret_ui_shadcn::dropdown_menu::{
    DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuEntry, DropdownMenuItem, DropdownMenuLabel,
    DropdownMenuRadioGroup, DropdownMenuRadioItemSpec,
};
use std::sync::Arc;
use std::time::Instant;

const CMD_CLOSE: &str = "table_demo.close";
const CMD_GROUP_CLEAR: &str = "table_demo.group.clear";
const CMD_GROUP_SET_ROLE: &str = "table_demo.group.set.role";
const CMD_GROUP_SET_NAME: &str = "table_demo.group.set.name";
const CMD_GROUP_TOGGLE_ID: &str = "table_demo.group.toggle.id";
const CMD_GROUP_TOGGLE_NAME: &str = "table_demo.group.toggle.name";
const CMD_GROUP_TOGGLE_ROLE: &str = "table_demo.group.toggle.role";
const CMD_GROUP_TOGGLE_SCORE: &str = "table_demo.group.toggle.score";

#[derive(Debug, Clone)]
struct DemoRow {
    id: u32,
    name: Arc<str>,
    role: Arc<str>,
    score: i32,
}

struct TableDemoWindowState {
    ui: UiTree<App>,
    table_state: Model<TableState>,
    rows: Arc<[DemoRow]>,
    scroll: VirtualListScrollHandle,
    view_options_open: Model<bool>,
    enable_grouping: Model<bool>,
    grouped_column_mode: Model<Option<Arc<str>>>,
    header_menu_id_open: Model<bool>,
    header_menu_name_open: Model<bool>,
    header_menu_role_open: Model<bool>,
    header_menu_score_open: Model<bool>,
    started_at: Instant,
    frame: u64,
    profile_frames_left: u64,
    exit_after_frames: Option<u64>,
}

#[derive(Default)]
struct TableDemoDriver;

impl TableDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> TableDemoWindowState {
        let started_at = Instant::now();
        let profile_frames_left = std::env::var_os("FRET_TABLE_DEMO_PROFILE_FRAMES")
            .and_then(|v| v.to_string_lossy().parse::<u64>().ok())
            .unwrap_or(0);
        let exit_after_frames = std::env::var_os("FRET_TABLE_DEMO_EXIT_AFTER_FRAMES")
            .and_then(|v| v.to_string_lossy().parse::<u64>().ok());

        let gen_started = Instant::now();
        let rows: Arc<[DemoRow]> = (0..10_000)
            .map(|i| DemoRow {
                id: i as u32,
                name: Arc::from(format!("User {i}")),
                role: Arc::from(if i % 7 == 0 { "Admin" } else { "Member" }),
                score: ((i * 31) % 997) as i32,
            })
            .collect::<Vec<_>>()
            .into();
        let gen_elapsed = gen_started.elapsed();

        if profile_frames_left > 0 {
            tracing::info!(
                "table_demo: generated {} rows in {:.2}ms",
                rows.len(),
                gen_elapsed.as_secs_f64() * 1000.0
            );
        }

        let mut table_state = TableState::default();
        table_state.pagination.page_size = rows.len();
        table_state.column_sizing.insert("id".into(), 72.0);
        table_state.column_sizing.insert("name".into(), 200.0);
        table_state.column_sizing.insert("role".into(), 140.0);
        table_state.column_sizing.insert("score".into(), 100.0);
        table_state.column_pinning = ColumnPinningState {
            left: vec!["id".into()],
            right: vec!["score".into()],
        };
        let table_state = app.models_mut().insert(table_state);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        TableDemoWindowState {
            ui,
            table_state,
            rows,
            scroll: VirtualListScrollHandle::new(),
            view_options_open: app.models_mut().insert(false),
            enable_grouping: app.models_mut().insert(true),
            grouped_column_mode: app.models_mut().insert(Some(Arc::from("reorder"))),
            header_menu_id_open: app.models_mut().insert(false),
            header_menu_name_open: app.models_mut().insert(false),
            header_menu_role_open: app.models_mut().insert(false),
            header_menu_score_open: app.models_mut().insert(false),
            started_at,
            frame: 0,
            profile_frames_left,
            exit_after_frames,
        }
    }
}

impl WinitAppDriver for TableDemoDriver {
    type WindowState = TableDemoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn hot_reload_window(
        &mut self,
        app: &mut App,
        _services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
    ) {
        crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        context
            .state
            .ui
            .propagate_model_changes(context.app, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        context
            .state
            .ui
            .propagate_global_changes(context.app, changed);
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

        match command.as_str() {
            CMD_CLOSE => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
                return;
            }
            CMD_GROUP_CLEAR => {
                let _ = app.models_mut().update(&state.table_state, clear_grouping);
                app.request_redraw(window);
                return;
            }
            CMD_GROUP_SET_ROLE => {
                let _ = app
                    .models_mut()
                    .update(&state.table_state, |st| set_grouping(st, "role"));
                app.request_redraw(window);
                return;
            }
            CMD_GROUP_SET_NAME => {
                let _ = app
                    .models_mut()
                    .update(&state.table_state, |st| set_grouping(st, "name"));
                app.request_redraw(window);
                return;
            }
            CMD_GROUP_TOGGLE_ID => {
                let _ = app
                    .models_mut()
                    .update(&state.table_state, |st| toggle_grouping(st, "id"));
                app.request_redraw(window);
                return;
            }
            CMD_GROUP_TOGGLE_NAME => {
                let _ = app
                    .models_mut()
                    .update(&state.table_state, |st| toggle_grouping(st, "name"));
                app.request_redraw(window);
                return;
            }
            CMD_GROUP_TOGGLE_ROLE => {
                let _ = app
                    .models_mut()
                    .update(&state.table_state, |st| toggle_grouping(st, "role"));
                app.request_redraw(window);
                return;
            }
            CMD_GROUP_TOGGLE_SCORE => {
                let _ = app
                    .models_mut()
                    .update(&state.table_state, |st| toggle_grouping(st, "score"));
                app.request_redraw(window);
                return;
            }
            _ => {}
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

        if let Event::KeyDown { key, modifiers, .. } = event {
            if modifiers.ctrl || modifiers.alt || modifiers.shift || modifiers.meta {
                state.ui.dispatch_event(app, services, event);
                return;
            }

            match *key {
                fret_core::KeyCode::Escape => {
                    app.push_effect(Effect::Window(WindowRequest::Close(window)));
                    return;
                }
                fret_core::KeyCode::Home => {
                    state
                        .scroll
                        .scroll_to_item(0, fret_ui::ScrollStrategy::Start);
                    app.request_redraw(window);
                    return;
                }
                fret_core::KeyCode::End => {
                    state.scroll.scroll_to_bottom();
                    app.request_redraw(window);
                    return;
                }
                _ => {}
            }
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
            ..
        } = context;

        OverlayController::begin_frame(app, window);

        let frame_started = Instant::now();
        let rows = state.rows.clone();
        let scroll = state.scroll.clone();
        let table_state = state.table_state.clone();
        let view_options_open = state.view_options_open.clone();
        let enable_grouping_model = state.enable_grouping.clone();
        let grouped_column_mode_model = state.grouped_column_mode.clone();
        let header_menu_id_open = state.header_menu_id_open.clone();
        let header_menu_name_open = state.header_menu_name_open.clone();
        let header_menu_role_open = state.header_menu_role_open.clone();
        let header_menu_score_open = state.header_menu_score_open.clone();
        let root =
            declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
                .render_root("table-demo", move |cx| {
                    cx.observe_model(&table_state, Invalidation::Layout);
                    cx.observe_model(&view_options_open, Invalidation::Layout);
                    cx.observe_model(&enable_grouping_model, Invalidation::Layout);
                    cx.observe_model(&grouped_column_mode_model, Invalidation::Layout);

                    let (selected, sorting) = cx
                        .app
                        .models()
                        .read(&table_state, |st| {
                            let selected = st.row_selection.len();
                            let sorting = st
                                .sorting
                                .first()
                                .map(|s| {
                                    format!(
                                        "{}:{}",
                                        s.column.as_ref(),
                                        if s.desc { "desc" } else { "asc" }
                                    )
                                })
                                .unwrap_or_else(|| "<none>".to_string());
                            (selected, sorting)
                        })
                        .unwrap_or((0, "<none>".to_string()));

                    let theme = Theme::global(&*cx.app).clone();

                    let mut root_layout = LayoutStyle::default();
                    root_layout.size.width = Length::Fill;
                    root_layout.size.height = Length::Fill;

                    let mut table_slot = LayoutStyle::default();
                    table_slot.size.width = Length::Fill;
                    table_slot.size.height = Length::Fill;
                    table_slot.flex.grow = 1.0;
                    table_slot.flex.basis = Length::Px(Px(0.0));
                    table_slot.overflow = Overflow::Clip;

                    let helper = create_column_helper::<DemoRow>();
                    let columns = vec![
                        helper
                            .clone()
                            .accessor("id", |r| r.id)
                            .facet_key_by(|r| r.id as u64),
                        helper
                            .clone()
                            .accessor("name", |r| r.name.clone())
                            .facet_str_by(|r| r.name.as_ref()),
                        helper
                            .clone()
                            .accessor("role", |r| r.role.clone())
                            .facet_str_by(|r| r.role.as_ref()),
                        helper
                            .accessor("score", |r| r.score)
                            .facet_key_by(|r| r.score as u64),
                    ];

                    let rows = rows.clone();
                    let scroll = scroll.clone();
                    let table_state = table_state.clone();
                    let view_options_open = view_options_open.clone();
                    let enable_grouping_model = enable_grouping_model.clone();
                    let grouped_column_mode_model = grouped_column_mode_model.clone();
                    let header_menu_id_open = header_menu_id_open.clone();
                    let header_menu_name_open = header_menu_name_open.clone();
                    let header_menu_role_open = header_menu_role_open.clone();
                    let header_menu_score_open = header_menu_score_open.clone();

                    let enable_grouping = cx
                        .watch_model(&enable_grouping_model)
                        .copied()
                        .unwrap_or(true);
                    let grouped_column_mode = cx
                        .watch_model(&grouped_column_mode_model)
                        .cloned()
                        .flatten();
                    let grouped_column_mode = match grouped_column_mode.as_deref() {
                        Some("remove") => GroupedColumnMode::Remove,
                        Some("none") => GroupedColumnMode::None,
                        _ => GroupedColumnMode::Reorder,
                    };

                    vec![cx.container(
                        ContainerProps {
                            layout: root_layout,
                            background: Some(theme.color_required("background")),
                            padding: Edges::all(theme.metric_required("metric.padding.md")),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.flex(
                                FlexProps {
                                    layout: root_layout,
                                    direction: fret_core::Axis::Vertical,
                                    gap: Px(8.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Stretch,
                                    wrap: false,
                                },
                                move |cx| {
                                    let header: Arc<str> = Arc::from(format!(
                                        "Table demo (virtualized) | rows={} | selected={} | sorting={} | click header to sort | drag handle to resize | [Home]/[End] scroll | [Esc] close",
                                        rows.len(),
                                        selected,
                                        sorting
                                    ));

                                    vec![
                                        cx.text(header),
                                        cx.flex(
                                            FlexProps {
                                                layout: {
                                                    let mut layout = LayoutStyle::default();
                                                    layout.size.width = Length::Fill;
                                                    layout.size.height = Length::Auto;
                                                    layout
                                                },
                                                direction: fret_core::Axis::Horizontal,
                                                gap: Px(8.0),
                                                padding: Edges::all(Px(0.0)),
                                                justify: MainAlign::Start,
                                                align: CrossAlign::Center,
                                                wrap: false,
                                            },
                                            move |cx| {
                                                let open = view_options_open.clone();
                                                let open_for_trigger = open.clone();
                                                let enable_grouping =
                                                    enable_grouping_model.clone();
                                                let grouped_column_mode =
                                                    grouped_column_mode_model.clone();

                                                vec![DropdownMenu::new(open).into_element(
                                                    cx,
                                                    |cx| {
                                                        Button::new("Table options")
                                                            .variant(ButtonVariant::Outline)
                                                            .size(ButtonSize::Sm)
                                                            .toggle_model(open_for_trigger)
                                                            .into_element(cx)
                                                    },
                                                    |_cx| {
                                                        vec![
                                                            DropdownMenuEntry::Label(
                                                                DropdownMenuLabel::new("Grouping"),
                                                            ),
                                                            DropdownMenuEntry::CheckboxItem(
                                                                DropdownMenuCheckboxItem::new(
                                                                    enable_grouping,
                                                                    "Enable grouping",
                                                                ),
                                                            ),
                                                            DropdownMenuEntry::Item(
                                                                DropdownMenuItem::new(
                                                                    "Clear grouping",
                                                                )
                                                                .on_select(CommandId::from(CMD_GROUP_CLEAR)),
                                                            ),
                                                            DropdownMenuEntry::Separator,
                                                            DropdownMenuEntry::Item(
                                                                DropdownMenuItem::new(
                                                                    "Group by Role",
                                                                )
                                                                .on_select(CommandId::from(CMD_GROUP_SET_ROLE)),
                                                            ),
                                                            DropdownMenuEntry::Item(
                                                                DropdownMenuItem::new(
                                                                    "Group by Name",
                                                                )
                                                                .on_select(CommandId::from(CMD_GROUP_SET_NAME)),
                                                            ),
                                                            DropdownMenuEntry::Separator,
                                                            DropdownMenuEntry::Label(
                                                                DropdownMenuLabel::new(
                                                                    "Grouped column mode",
                                                                ),
                                                            ),
                                                            DropdownMenuEntry::RadioGroup(
                                                                DropdownMenuRadioGroup::new(
                                                                    grouped_column_mode,
                                                                )
                                                                .item(
                                                                    DropdownMenuRadioItemSpec::new(
                                                                        "reorder",
                                                                        "Reorder",
                                                                    ),
                                                                )
                                                                .item(
                                                                    DropdownMenuRadioItemSpec::new(
                                                                        "remove",
                                                                        "Remove",
                                                                    ),
                                                                )
                                                                .item(
                                                                    DropdownMenuRadioItemSpec::new(
                                                                        "none",
                                                                        "None",
                                                                    ),
                                                                ),
                                                            ),
                                                        ]
                                                    },
                                                )]
                                            },
                                        ),
                                        cx.container(
                                            ContainerProps {
                                                layout: table_slot,
                                                background: Some(theme.color_required("card")),
                                                border: Edges::all(Px(1.0)),
                                                border_color: Some(theme.color_required("border")),
                                                corner_radii: Corners::all(
                                                    theme.metric_required("metric.radius.md"),
                                                ),
                                                ..Default::default()
                                            },
                                            move |cx| {
                                                vec![fret_ui_kit::declarative::table::table_virtualized(
                                                    cx,
                                                    &rows,
                                                    &columns,
                                                    table_state.clone(),
                                                    &scroll,
                                                    1,
                                                    &|row: &DemoRow, _i| RowKey(row.id as u64),
                                                    None,
                                                    fret_ui_kit::declarative::table::TableViewProps {
                                                        overscan: 8,
                                                        column_resize_mode:
                                                            fret_ui_kit::headless::table::ColumnResizeMode::OnEnd,
                                                        enable_column_grouping: enable_grouping,
                                                        grouped_column_mode,
                                                        ..Default::default()
                                                    },
                                                    |_row| None,
                                                    |cx, col, sort_state| {
                                                        let label_base: Arc<str> = match col.id.as_ref() {
                                                            "id" => Arc::from("ID"),
                                                            "name" => Arc::from("Name"),
                                                            "role" => Arc::from("Role"),
                                                            "score" => Arc::from("Score"),
                                                            _ => Arc::from(col.id.as_ref()),
                                                        };
                                                        let label: Arc<str> = if let Some(desc) = sort_state {
                                                            let mut s = label_base.to_string();
                                                            s.push_str(if desc { " ▼" } else { " ▲" });
                                                            Arc::from(s)
                                                        } else {
                                                            label_base
                                                        };
                                                        let open = match col.id.as_ref() {
                                                            "id" => header_menu_id_open.clone(),
                                                            "name" => header_menu_name_open.clone(),
                                                            "role" => header_menu_role_open.clone(),
                                                            "score" => header_menu_score_open.clone(),
                                                            _ => header_menu_role_open.clone(),
                                                        };

                                                        let id: Arc<str> = Arc::from(col.id.as_ref());
                                                        let groupable = col.facet_key_fn.is_some()
                                                            || col.facet_str_fn.is_some();
                                                        let is_grouped = cx
                                                            .app
                                                            .models()
                                                            .read(&table_state, |st| {
                                                                st.grouping.iter().any(|c| {
                                                                    c.as_ref() == id.as_ref()
                                                                })
                                                            })
                                                            .unwrap_or(false);
                                                        let has_grouping = cx
                                                            .app
                                                            .models()
                                                            .read(&table_state, |st| {
                                                                !st.grouping.is_empty()
                                                            })
                                                            .unwrap_or(false);

                                                        vec![ContextMenu::new(open).into_element(
                                                            cx,
                                                            |cx| cx.text(label.clone()),
                                                            |_cx| {
                                                                let mut entries = Vec::new();
                                                                if groupable {
                                                                    let action = if is_grouped {
                                                                        "Ungroup"
                                                                    } else {
                                                                        "Group by this column"
                                                                    };
                                                                    if let Some(cmd) =
                                                                        group_toggle_command_for_column(id.as_ref())
                                                                    {
                                                                        entries.push(
                                                                            ContextMenuEntry::Item(
                                                                                ContextMenuItem::new(
                                                                                    action,
                                                                                )
                                                                                .on_select(cmd),
                                                                            ),
                                                                        );
                                                                    } else {
                                                                        entries.push(
                                                                            ContextMenuEntry::Item(
                                                                                ContextMenuItem::new(
                                                                                    "Grouping not available",
                                                                                )
                                                                                .disabled(true),
                                                                            ),
                                                                        );
                                                                    }
                                                                } else {
                                                                    entries.push(
                                                                        ContextMenuEntry::Item(
                                                                            ContextMenuItem::new(
                                                                                "Grouping not available",
                                                                            )
                                                                            .disabled(true),
                                                                        ),
                                                                    );
                                                                }
                                                                if has_grouping {
                                                                    entries.push(
                                                                        ContextMenuEntry::Separator,
                                                                    );
                                                                    entries.push(
                                                                        ContextMenuEntry::Item(
                                                                            ContextMenuItem::new(
                                                                                "Clear grouping",
                                                                            )
                                                                            .on_select(
                                                                                CommandId::from(
                                                                                    CMD_GROUP_CLEAR,
                                                                                ),
                                                                            ),
                                                                        ),
                                                                    );
                                                                }
                                                                entries
                                                            },
                                                        )]
                                                    },
                                                    |cx, row, col| {
                                                        let text = match col.id.as_ref() {
                                                            "id" => row.original.id.to_string(),
                                                            "name" => row.original.name.as_ref().to_string(),
                                                            "role" => row.original.role.as_ref().to_string(),
                                                            "score" => row.original.score.to_string(),
                                                            _ => "".to_string(),
                                                        };
                                                        vec![cx.text(text)]
                                                    },
                                                    None,
                                                )]
                                            },
                                        ),
                                    ]
                                },
                            )]
                        },
                    )]
                });

        state.ui.set_root(root);
        OverlayController::render(&mut state.ui, app, services, window, bounds);
        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);
        scene.clear();

        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        let layout_started = Instant::now();
        frame.layout_all();
        let layout_elapsed = layout_started.elapsed();
        let paint_started = Instant::now();
        frame.paint_all(scene);
        let paint_elapsed = paint_started.elapsed();

        state.frame = state.frame.saturating_add(1);
        if state.profile_frames_left > 0 {
            state.profile_frames_left = state.profile_frames_left.saturating_sub(1);
            let since_start = state.started_at.elapsed();
            let frame_elapsed = frame_started.elapsed();
            tracing::info!(
                "table_demo: frame={} since_start={:.2}ms total={:.2}ms layout={:.2}ms paint={:.2}ms",
                state.frame,
                since_start.as_secs_f64() * 1000.0,
                frame_elapsed.as_secs_f64() * 1000.0,
                layout_elapsed.as_secs_f64() * 1000.0,
                paint_elapsed.as_secs_f64() * 1000.0
            );
        }

        if let Some(limit) = state.exit_after_frames {
            if state.frame >= limit {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
                return;
            }
        }

        if state.profile_frames_left > 0 || state.exit_after_frames.is_some() {
            app.request_redraw(window);
        }
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        None
    }
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter({
            let mut filter = tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap());

            if std::env::var_os("FRET_TABLE_DEMO_PROFILE_FRAMES").is_some()
                || std::env::var_os("FRET_TABLE_DEMO_EXIT_AFTER_FRAMES").is_some()
            {
                filter = filter
                    .add_directive("fret_examples=info".parse().unwrap())
                    .add_directive("fret_ui_kit=info".parse().unwrap());
            }

            filter
        })
        .try_init();

    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());

    let config = WinitRunnerConfig {
        main_window_title: "fret-demo table_demo".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    crate::run_native_demo(config, app, TableDemoDriver::default()).context("run table_demo app")
}

fn clear_grouping(st: &mut TableState) {
    st.grouping.clear();
    st.expanding = Default::default();
    st.pagination.page_index = 0;
}

fn set_grouping(st: &mut TableState, id: &'static str) {
    clear_grouping(st);
    st.grouping.push(Arc::<str>::from(id));
}

fn toggle_grouping(st: &mut TableState, id: &'static str) {
    let id = Arc::<str>::from(id);
    if let Some(i) = st.grouping.iter().position(|c| c.as_ref() == id.as_ref()) {
        st.grouping.remove(i);
    } else {
        st.grouping.push(id);
    }
    st.expanding = Default::default();
    st.pagination.page_index = 0;
}

fn group_toggle_command_for_column(id: &str) -> Option<CommandId> {
    let cmd = match id {
        "id" => CMD_GROUP_TOGGLE_ID,
        "name" => CMD_GROUP_TOGGLE_NAME,
        "role" => CMD_GROUP_TOGGLE_ROLE,
        "score" => CMD_GROUP_TOGGLE_SCORE,
        _ => return None,
    };
    Some(CommandId::from(cmd))
}
