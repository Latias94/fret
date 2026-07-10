use anyhow::Context as _;
use fret::WindowId;
use fret::app::{self, App, AppLocalStateExt as _, LocalState, RenderContextAccess as _, text};
use fret::style::{
    Axis, ContainerProps, Corners, CrossAlign, Edges, FlexProps, LayoutStyle, Length, MainAlign,
    Overflow, Px, Space, SpacingLength,
};
use fret_ui_kit::ui;
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
struct DemoRow {
    id: u64,
    name: Arc<str>,
    role: Arc<str>,
    score: i32,
}

pub struct DemoWindowState {
    table_state: LocalState<shadcn::TableState>,
    table_output: LocalState<shadcn::DataTableViewOutput>,
    table_recipe: shadcn::DataTableRecipe<DemoRow>,
    rows: Arc<[DemoRow]>,
    started_at: Instant,
    frame: u64,
    profile_frames_left: u64,
    exit_after_frames: Option<u64>,
}

fn datatable_rows() -> Arc<[DemoRow]> {
    (0..10_000)
        .map(|i| DemoRow {
            id: i as u64,
            name: Arc::from(format!("User {i}")),
            role: Arc::from(if i % 7 == 0 { "Admin" } else { "Member" }),
            score: ((i * 31) % 997) as i32,
        })
        .collect::<Vec<_>>()
        .into()
}

fn datatable_columns() -> Arc<[shadcn::ColumnDef<DemoRow>]> {
    let helper = shadcn::create_column_helper::<DemoRow>();
    Arc::from(
        vec![
            helper.clone().accessor("id", |r| r.id),
            helper.clone().accessor_str("name", |r| r.name.as_ref()),
            helper.clone().accessor_str("role", |r| r.role.as_ref()),
            helper.accessor("score", |r| r.score),
        ]
        .into_boxed_slice(),
    )
}

fn datatable_column_labels() -> Vec<shadcn::DataTableColumnLabel> {
    vec![
        shadcn::DataTableColumnLabel::new("id", "ID"),
        shadcn::DataTableColumnLabel::new("name", "Name"),
        shadcn::DataTableColumnLabel::new("role", "Role"),
        shadcn::DataTableColumnLabel::new("score", "Score"),
    ]
}

fn datatable_debug_ids() -> shadcn::TableDebugIds {
    shadcn::TableDebugIds {
        header_row_test_id: Some(Arc::<str>::from("datatable-demo-header-row")),
        header_cell_test_id_prefix: Some(Arc::<str>::from("datatable-demo-header-")),
        row_test_id_prefix: Some(Arc::<str>::from("datatable-demo-row-")),
        ..Default::default()
    }
}

fn create_window_state(app: &mut App, _window: WindowId) -> DemoWindowState {
    let profile_frames_left = std::env::var_os("FRET_DATATABLE_DEMO_PROFILE_FRAMES")
        .or_else(|| std::env::var_os("FRET_TANSTACK_DATATABLE_DEMO_PROFILE_FRAMES"))
        .and_then(|v| v.to_string_lossy().parse::<u64>().ok())
        .unwrap_or(0);
    let exit_after_frames = std::env::var_os("FRET_DATATABLE_DEMO_EXIT_AFTER_FRAMES")
        .or_else(|| std::env::var_os("FRET_TANSTACK_DATATABLE_DEMO_EXIT_AFTER_FRAMES"))
        .and_then(|v| v.to_string_lossy().parse::<u64>().ok());

    let mut table_state = shadcn::TableState::default();
    table_state.pagination.page_size = 50;
    let table_state = app.local_state(table_state);
    let table_output = app.local_state(shadcn::DataTableViewOutput::default());
    let columns = datatable_columns();
    let table_recipe =
        shadcn::DataTableRecipe::new(&table_state, &table_output, columns, |row, _i, _parent| {
            shadcn::RowKey(row.id)
        })
        .column_labels(datatable_column_labels())
        .debug_ids(datatable_debug_ids())
        .toolbar_test_id_prefix("datatable-demo-toolbar")
        .page_sizes(Arc::from([25usize, 50, 100, 250]))
        .table(shadcn::DataTable::new().column_actions_menu(true));

    DemoWindowState {
        table_state,
        table_output,
        table_recipe,
        rows: datatable_rows(),
        started_at: Instant::now(),
        frame: 0,
        profile_frames_left,
        exit_after_frames,
    }
}

fn fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout
}

fn render_datatable_demo(cx: &mut fret::AppUi<'_, '_>, state: &mut DemoWindowState) -> fret::Ui {
    let cx = cx.elements();
    let frame_started = Instant::now();
    state.frame = state.frame.saturating_add(1);

    let rows = Arc::clone(&state.rows);
    let table_state = state.table_state.clone();
    let table_output = state.table_output.clone();
    let table_recipe = state.table_recipe.clone();

    // Subscribe to output changes while keeping output on the app-facing LocalState surface.
    let _ = table_output.layout_value(cx);

    let theme = cx.theme_snapshot();
    let padding = theme.metric_token("metric.padding.md");

    let (selected, sorting) = table_state.layout_read_ref(cx, |st| {
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
    });

    let mut root_layout = fill_layout();
    let mut table_slot = fill_layout();
    table_slot.flex.grow = 1.0;
    table_slot.flex.basis = Length::Px(Px(0.0));
    table_slot.overflow = Overflow::Clip;

    let header = ui::h_row(|cx| {
        [
            shadcn::Button::new("Close")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .on_activate(app::close_window_activate())
                .into_element(cx),
            text::control_readout(
                cx,
                Arc::from(format!("DataTable | selected={selected} sort={sorting}")),
            ),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let table_parts =
        table_recipe.into_elements(cx, rows, 1, |cx, col, row| match col.id.as_ref() {
            "id" => text::table_cell(cx, Arc::from(row.id.to_string())),
            "name" => text::table_cell(cx, Arc::clone(&row.name)),
            "role" => text::table_cell(cx, Arc::clone(&row.role)),
            "score" => text::table_cell(cx, Arc::from(row.score.to_string())),
            _ => text::table_cell(cx, Arc::from("")),
        });

    if state.profile_frames_left > 0 {
        state.profile_frames_left = state.profile_frames_left.saturating_sub(1);
        let since_start = state.started_at.elapsed();
        let frame_elapsed = frame_started.elapsed();
        tracing::info!(
            "datatable_demo: frame={} since_start={:.2}ms render_build={:.2}ms",
            state.frame,
            since_start.as_secs_f64() * 1000.0,
            frame_elapsed.as_secs_f64() * 1000.0
        );
    }

    if let Some(limit) = state.exit_after_frames
        && state.frame >= limit
    {
        app::close_window(cx.app, cx.window);
    }

    if state.profile_frames_left > 0 || state.exit_after_frames.is_some() {
        cx.app.request_redraw(cx.window);
    }

    root_layout.size.width = Length::Fill;
    root_layout.size.height = Length::Fill;
    vec![cx.container(
        ContainerProps {
            layout: root_layout,
            background: Some(theme.color_token("background")),
            ..Default::default()
        },
        move |cx| {
            vec![cx.flex(
                FlexProps {
                    layout: root_layout,
                    direction: Axis::Vertical,
                    gap: SpacingLength::Px(Px(8.0)),
                    padding: Edges::all(padding).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    wrap: false,
                },
                move |cx| {
                    vec![
                        header,
                        table_parts.toolbar,
                        cx.container(
                            ContainerProps {
                                layout: table_slot,
                                background: Some(theme.color_token("card")),
                                border: Edges::all(Px(1.0)),
                                border_color: Some(theme.color_token("border")),
                                corner_radii: Corners::all(theme.metric_token("metric.radius.md")),
                                ..Default::default()
                            },
                            move |_cx| vec![table_parts.table],
                        ),
                        table_parts.pagination,
                    ]
                },
            )]
        },
    )]
    .into()
}

impl fret::app::View for DemoWindowState {
    fn init(app: &mut App, window: WindowId) -> Self {
        create_window_state(app, window)
    }

    fn render(&mut self, cx: &mut fret::AppUi<'_, '_>) -> fret::Ui {
        render_datatable_demo(cx, self)
    }
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap()),
        )
        .try_init();

    fret::FretApp::new("datatable-demo")
        .window("fret-demo datatable_demo", (980.0, 720.0))
        .view::<DemoWindowState>()?
        .run()
        .map_err(anyhow::Error::from)
        .context("run datatable_demo app")
}
