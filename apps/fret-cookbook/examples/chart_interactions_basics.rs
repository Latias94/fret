use std::cell::RefCell;
use std::rc::Rc;

use delinea::data::{Column, DataTable};
use delinea::engine::window::DataWindow;
use delinea::{Action, AxisKind, AxisPointerSpec, AxisPointerTrigger, AxisPointerType, AxisScale};
use fret::{advanced::prelude::*, shadcn};
use fret_app::{CommandMeta, CommandScope};
use fret_chart::ChartCanvas;
use fret_core::{AppWindowId, Px, SemanticsRole};
use fret_runtime::CommandId;
use fret_ui::element::{LayoutStyle, Length, SemanticsDecoration, SemanticsProps};
use fret_ui::retained_bridge::RetainedSubtreeProps;

const ROOT_NAME: &str = "cookbook-chart-interactions-basics";

const TEST_ID_ROOT: &str = "cookbook.chart_interactions_basics.root";
const TEST_ID_CANVAS: &str = "cookbook.chart_interactions_basics.canvas";
const TEST_ID_ZOOM_IN: &str = "cookbook.chart_interactions_basics.zoom_in";
const TEST_ID_ZOOM_OUT: &str = "cookbook.chart_interactions_basics.zoom_out";
const TEST_ID_RESET_VIEW: &str = "cookbook.chart_interactions_basics.reset_view";
const TEST_ID_X_SPAN: &str = "cookbook.chart_interactions_basics.x_span";
const TEST_ID_HOVER_INDEX: &str = "cookbook.chart_interactions_basics.hover_index";
const TEST_ID_SELECTED_INDEX: &str = "cookbook.chart_interactions_basics.selected_index";

const CMD_ZOOM_IN: &str = "cookbook.chart.zoom_in";
const CMD_ZOOM_OUT: &str = "cookbook.chart.zoom_out";
const CMD_RESET_VIEW: &str = "cookbook.chart.reset_view";
const CMD_SELECT_HOVER: &str = "cookbook.chart.select_hover";
const CMD_CLEAR_SELECTION: &str = "cookbook.chart.clear_selection";

type SharedChartEngine = Rc<RefCell<delinea::engine::ChartEngine>>;

#[derive(Debug, Clone, Copy)]
struct ChartIds {
    dataset: delinea::ids::DatasetId,
    grid: delinea::ids::GridId,
    x_axis: delinea::ids::AxisId,
    y_axis: delinea::ids::AxisId,
    series: delinea::ids::SeriesId,
    x_field: delinea::ids::FieldId,
    y_field: delinea::ids::FieldId,
}

fn chart_ids() -> ChartIds {
    ChartIds {
        dataset: delinea::ids::DatasetId::new(1),
        grid: delinea::ids::GridId::new(1),
        x_axis: delinea::AxisId::new(1),
        y_axis: delinea::AxisId::new(2),
        series: delinea::ids::SeriesId::new(1),
        x_field: delinea::FieldId::new(1),
        y_field: delinea::FieldId::new(2),
    }
}

fn base_x_window(n: usize) -> DataWindow {
    let max = (n.saturating_sub(1)) as f64;
    DataWindow { min: 0.0, max }
}

fn base_y_window(values: &[f64]) -> DataWindow {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for v in values {
        if v.is_finite() {
            min = min.min(*v);
            max = max.max(*v);
        }
    }

    if !min.is_finite() || !max.is_finite() || !(max > min) {
        return DataWindow { min: 0.0, max: 1.0 };
    }

    let pad = (max - min) * 0.1;
    DataWindow {
        min: min - pad,
        max: max + pad,
    }
}

fn zoom_window(base: DataWindow, current: DataWindow, factor: f64) -> DataWindow {
    if !(factor.is_finite() && factor > 0.0) {
        return current;
    }

    let base_span = (base.max - base.min).max(1e-9);
    let current_span = (current.max - current.min).max(1e-9);
    let target_span = (current_span * factor).clamp(1e-6, base_span);
    let center = (current.min + current.max) * 0.5;

    let mut out = DataWindow {
        min: center - target_span * 0.5,
        max: center + target_span * 0.5,
    };

    if out.min < base.min {
        let delta = base.min - out.min;
        out.min += delta;
        out.max += delta;
    }
    if out.max > base.max {
        let delta = out.max - base.max;
        out.min -= delta;
        out.max -= delta;
    }

    out.min = out.min.max(base.min);
    out.max = out.max.min(base.max);
    if !(out.max > out.min) {
        out = base;
    }
    out
}

fn install_commands(app: &mut KernelApp) {
    let scope = CommandScope::Widget;

    app.commands_mut().register(
        CommandId::from(CMD_ZOOM_IN),
        CommandMeta::new("Zoom in (X)")
            .with_description("Zoom the X axis window in by 2x (app-driven).")
            .with_category("Chart")
            .with_scope(scope),
    );
    app.commands_mut().register(
        CommandId::from(CMD_ZOOM_OUT),
        CommandMeta::new("Zoom out (X)")
            .with_description("Zoom the X axis window out by 2x (app-driven).")
            .with_category("Chart")
            .with_scope(scope),
    );
    app.commands_mut().register(
        CommandId::from(CMD_RESET_VIEW),
        CommandMeta::new("Reset view")
            .with_description("Reset the axis windows to a known baseline.")
            .with_category("Chart")
            .with_scope(scope),
    );
    app.commands_mut().register(
        CommandId::from(CMD_SELECT_HOVER),
        CommandMeta::new("Select hovered point")
            .with_description("Copy the current axis pointer hit into an app-owned selection.")
            .with_category("Chart")
            .with_scope(scope),
    );
    app.commands_mut().register(
        CommandId::from(CMD_CLEAR_SELECTION),
        CommandMeta::new("Clear selection")
            .with_description("Clear the app-owned selection.")
            .with_category("Chart")
            .with_scope(scope),
    );
}

struct ChartInteractionsWindowState {
    ids: ChartIds,
    engine: SharedChartEngine,
    base_x: DataWindow,
    base_y: DataWindow,
    x_window: DataWindow,
    y_window: DataWindow,
    selected: Option<u32>,
}

fn init_window(_app: &mut KernelApp, _window: AppWindowId) -> ChartInteractionsWindowState {
    let ids = chart_ids();

    let x: Vec<f64> = (0..12).map(|i| i as f64).collect();
    let y: Vec<f64> = vec![
        186.0, 305.0, 237.0, 73.0, 209.0, 214.0, 198.0, 265.0, 172.0, 142.0, 223.0, 190.0,
    ];

    let spec = delinea::ChartSpec {
        id: delinea::ids::ChartId::new(1),
        viewport: None,
        datasets: vec![delinea::DatasetSpec {
            id: ids.dataset,
            fields: vec![
                delinea::FieldSpec {
                    id: ids.x_field,
                    column: 0,
                },
                delinea::FieldSpec {
                    id: ids.y_field,
                    column: 1,
                },
            ],
            ..Default::default()
        }],
        grids: vec![delinea::GridSpec { id: ids.grid }],
        axes: vec![
            delinea::AxisSpec {
                id: ids.x_axis,
                name: Some("Month".to_string()),
                kind: AxisKind::X,
                grid: ids.grid,
                position: None,
                scale: AxisScale::default(),
                range: None,
            },
            delinea::AxisSpec {
                id: ids.y_axis,
                name: Some("Users".to_string()),
                kind: AxisKind::Y,
                grid: ids.grid,
                position: None,
                scale: AxisScale::default(),
                range: None,
            },
        ],
        data_zoom_x: vec![],
        data_zoom_y: vec![],
        tooltip: None,
        axis_pointer: Some(AxisPointerSpec {
            enabled: true,
            trigger: AxisPointerTrigger::Axis,
            pointer_type: AxisPointerType::Line,
            label: Default::default(),
            snap: false,
            trigger_distance_px: 14.0,
            throttle_px: 0.75,
        }),
        visual_maps: vec![],
        series: vec![delinea::SeriesSpec {
            id: ids.series,
            name: Some("Desktop".to_string()),
            kind: delinea::SeriesKind::Line,
            dataset: ids.dataset,
            encode: delinea::SeriesEncode {
                x: ids.x_field,
                y: ids.y_field,
                y2: None,
            },
            x_axis: ids.x_axis,
            y_axis: ids.y_axis,
            stack: None,
            stack_strategy: Default::default(),
            bar_layout: Default::default(),
            area_baseline: None,
            lod: None,
        }],
    };

    let mut engine = delinea::engine::ChartEngine::new(spec).expect("chart spec should be valid");
    let mut table = DataTable::default();
    table.push_column(Column::F64(x));
    table.push_column(Column::F64(y.clone()));
    engine.datasets_mut().insert(ids.dataset, table);

    let base_x = base_x_window(y.len());
    let base_y = base_y_window(&y);

    engine.apply_action(Action::SetViewWindow2D {
        x_axis: ids.x_axis,
        y_axis: ids.y_axis,
        x: Some(base_x),
        y: Some(base_y),
    });

    ChartInteractionsWindowState {
        ids,
        engine: Rc::new(RefCell::new(engine)),
        base_x,
        base_y,
        x_window: base_x,
        y_window: base_y,
        selected: None,
    }
}

fn chart_canvas(
    cx: &mut ElementContext<'_, KernelApp>,
    st: &ChartInteractionsWindowState,
) -> AnyElement {
    let engine = st.engine.clone();

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;

    let props = RetainedSubtreeProps::new::<KernelApp>(move |ui| {
        use fret_ui::retained_bridge::UiTreeRetainedExt as _;

        let mut canvas = ChartCanvas::new_shared(engine.clone());
        canvas.set_input_map(fret_chart::input_map::ChartInputMap::default());
        canvas.set_accessibility_layer(true);

        let node = ui.create_node_retained(canvas.test_id(TEST_ID_CANVAS));
        ui.set_node_view_cache_flags(node, true, true, false);
        node
    })
    .with_layout(layout);

    cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        vec![cx.retained_subtree(props)]
    })
}

fn view(
    cx: &mut ElementContext<'_, KernelApp>,
    st: &mut ChartInteractionsWindowState,
) -> ViewElements {
    let theme = Theme::global(&*cx.app).snapshot();

    let (x_span, hover_index) = {
        let x_span = (st.x_window.max - st.x_window.min).max(0.0);

        let engine = st.engine.borrow();
        let out = engine.output();
        let hover_index = out
            .axis_pointer
            .as_ref()
            .and_then(|o| o.hit.map(|h| h.data_index))
            .map(|v| v as f64)
            .unwrap_or(-1.0);

        (x_span, hover_index)
    };

    let toolbar = ui::h_flex(|cx| {
        let x_span_badge = shadcn::Badge::new(format!("X span: {x_span:.2}"))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(TEST_ID_X_SPAN)
                    .numeric_value(x_span)
                    .numeric_range(0.0, (st.base_x.max - st.base_x.min).max(1.0)),
            );

        let hover_badge = shadcn::Badge::new(format!("Hover index: {hover_index:.0}"))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(TEST_ID_HOVER_INDEX)
                    .numeric_value(hover_index)
                    .numeric_range(-1.0, (st.base_x.max - st.base_x.min).max(1.0)),
            );

        let selected_index = st.selected.map(|v| v as f64).unwrap_or(-1.0);
        let selected_badge = shadcn::Badge::new(format!("Selected index: {selected_index:.0}"))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(TEST_ID_SELECTED_INDEX)
                    .numeric_value(selected_index)
                    .numeric_range(-1.0, (st.base_x.max - st.base_x.min).max(1.0)),
            );

        ui::children![
            cx;
            shadcn::Button::new("Zoom in (X)")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_ZOOM_IN)
                .test_id(TEST_ID_ZOOM_IN),
            shadcn::Button::new("Zoom out (X)")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(CMD_ZOOM_OUT)
                .test_id(TEST_ID_ZOOM_OUT),
            shadcn::Button::new("Reset view")
                .variant(shadcn::ButtonVariant::Outline)
                .on_click(CMD_RESET_VIEW)
                .test_id(TEST_ID_RESET_VIEW),
            shadcn::Button::new("Select hovered")
                .variant(shadcn::ButtonVariant::Outline)
                .on_click(CMD_SELECT_HOVER),
            shadcn::Button::new("Clear selection")
                .variant(shadcn::ButtonVariant::Ghost)
                .on_click(CMD_CLEAR_SELECTION),
            x_span_badge,
            hover_badge,
            selected_badge,
        ]
    })
    .gap(Space::N2)
    .items_center();

    let canvas = chart_canvas(cx, st);

    let canvas_shell = ui::container(|_cx| vec![canvas])
        .bg(ColorRef::Color(theme.color_token("card")))
        .border_1()
        .rounded(Radius::Lg)
        .p(Space::N2)
        .w_full()
        .h_full()
        .min_h(Px(420.0));

    let card = shadcn::Card::build(|cx, out| {
        out.push_ui(
            cx,
            shadcn::CardHeader::build(|cx, out| {
                out.push_ui(cx, shadcn::CardTitle::new("Chart interactions basics"));
                out.push_ui(
                    cx,
                    shadcn::CardDescription::new(
                        "Minimal shared delinea engine + retained ChartCanvas. App-owned zoom + selection; axis pointer hover for exploration.",
                    ),
                );
            }),
        );
        out.push_ui(
            cx,
            shadcn::CardContent::build(|cx, out| {
                out.push_ui(
                    cx,
                    ui::v_flex(|cx| ui::children![cx; toolbar, canvas_shell])
                        .gap(Space::N3)
                        .w_full()
                        .h_full()
                        .min_w_0(),
                );
            }),
        );
    })
    .ui()
    .w_full()
    .h_full()
    .max_w(Px(1100.0));

    let root = fret_cookbook::scaffold::centered_page_muted_ui(cx, TEST_ID_ROOT, card);

    vec![cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Group,
            test_id: None,
            ..Default::default()
        },
        |_cx| vec![root],
    )]
    .into()
}

fn on_command(
    _app: &mut KernelApp,
    _services: &mut dyn fret_core::UiServices,
    _window: AppWindowId,
    _ui: &mut fret_ui::UiTree<KernelApp>,
    st: &mut ChartInteractionsWindowState,
    command: &CommandId,
) {
    let cmd = command.as_str();

    if cmd == CMD_CLEAR_SELECTION {
        st.selected = None;
        return;
    }

    if cmd == CMD_SELECT_HOVER {
        let engine = st.engine.borrow();
        let hit = engine.output().axis_pointer.as_ref().and_then(|o| o.hit);
        if let Some(hit) = hit {
            st.selected = Some(hit.data_index);
        }
        return;
    }

    let mut engine = st.engine.borrow_mut();
    let current_x = st.x_window;

    match cmd {
        CMD_ZOOM_IN => {
            let window = zoom_window(st.base_x, current_x, 0.5);
            st.x_window = window;
            engine.apply_action(Action::SetDataWindowX {
                axis: st.ids.x_axis,
                window: Some(window),
            });
        }
        CMD_ZOOM_OUT => {
            let window = zoom_window(st.base_x, current_x, 2.0);
            st.x_window = window;
            engine.apply_action(Action::SetDataWindowX {
                axis: st.ids.x_axis,
                window: Some(window),
            });
        }
        CMD_RESET_VIEW => {
            st.selected = None;
            st.x_window = st.base_x;
            st.y_window = st.base_y;
            engine.apply_action(Action::SetViewWindow2D {
                x_axis: st.ids.x_axis,
                y_axis: st.ids.y_axis,
                x: Some(st.base_x),
                y: Some(st.base_y),
            });
        }
        _ => {}
    }
}

fn configure_driver(
    driver: UiAppDriver<ChartInteractionsWindowState>,
) -> UiAppDriver<ChartInteractionsWindowState> {
    driver.on_command(on_command)
}

fn main() -> anyhow::Result<()> {
    let builder = ui_app_with_hooks(ROOT_NAME, init_window, view, configure_driver)
        .with_main_window("cookbook-chart-interactions-basics", (1120.0, 820.0))
        .with_command_default_keybindings()
        .install_app(install_commands)
        .install_app(shadcn::install_app)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096)
        .with_lucide_icons();

    #[cfg(feature = "cookbook-diag")]
    let builder = builder.with_default_diagnostics();

    builder.run().map_err(anyhow::Error::from)
}
