use fret::app::prelude::*;
use fret::app::{LocalState, RenderContextAccess as _};
use fret::chart::delinea::data::{Column, DataTable};
use fret::chart::{
    self, ChartCanvasOutput, ChartEngine, ChartInputMap, DataWindow,
    delinea::{
        self, Action, AxisKind, AxisPointerSpec, AxisPointerTrigger, AxisPointerType, AxisScale,
    },
};
use fret::children::UiElementSinkExt as _;
use fret::commands::{CommandId, CommandMeta, CommandScope};
use fret::semantics::{SemanticsDecoration, SemanticsRole};
use fret::style::{ColorRef, Radius, Space};

const ROOT_NAME: &str = "cookbook-chart-interactions-basics";

mod act {
    fret::actions!([
        ZoomIn = "cookbook.chart.zoom_in",
        ZoomOut = "cookbook.chart.zoom_out",
        ResetView = "cookbook.chart.reset_view",
        SelectHover = "cookbook.chart.select_hover",
        ClearSelection = "cookbook.chart.clear_selection",
    ]);
}

const TEST_ID_ROOT: &str = "cookbook.chart_interactions_basics.root";
const TEST_ID_CANVAS: &str = "cookbook.chart_interactions_basics.canvas";
const TEST_ID_ZOOM_IN: &str = "cookbook.chart_interactions_basics.zoom_in";
const TEST_ID_ZOOM_OUT: &str = "cookbook.chart_interactions_basics.zoom_out";
const TEST_ID_RESET_VIEW: &str = "cookbook.chart_interactions_basics.reset_view";
const TEST_ID_X_SPAN: &str = "cookbook.chart_interactions_basics.x_span";
const TEST_ID_HOVER_INDEX: &str = "cookbook.chart_interactions_basics.hover_index";
const TEST_ID_SELECTED_INDEX: &str = "cookbook.chart_interactions_basics.selected_index";

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

fn install_commands(app: &mut App) {
    let scope = CommandScope::Widget;

    let zoom_in: CommandId = act::ZoomIn.into();
    app.commands_mut().register(
        zoom_in,
        CommandMeta::new("Zoom in (X)")
            .with_description("Zoom the X axis window in by 2x (app-driven).")
            .with_category("Chart")
            .with_scope(scope),
    );

    let zoom_out: CommandId = act::ZoomOut.into();
    app.commands_mut().register(
        zoom_out,
        CommandMeta::new("Zoom out (X)")
            .with_description("Zoom the X axis window out by 2x (app-driven).")
            .with_category("Chart")
            .with_scope(scope),
    );

    let reset_view: CommandId = act::ResetView.into();
    app.commands_mut().register(
        reset_view,
        CommandMeta::new("Reset view")
            .with_description("Reset the axis windows to a known baseline.")
            .with_category("Chart")
            .with_scope(scope),
    );

    let select_hover: CommandId = act::SelectHover.into();
    app.commands_mut().register(
        select_hover,
        CommandMeta::new("Select hovered point")
            .with_description("Copy the current axis pointer hit into an app-owned selection.")
            .with_category("Chart")
            .with_scope(scope),
    );

    let clear_selection: CommandId = act::ClearSelection.into();
    app.commands_mut().register(
        clear_selection,
        CommandMeta::new("Clear selection")
            .with_description("Clear the app-owned selection.")
            .with_category("Chart")
            .with_scope(scope),
    );
}

struct ChartInteractionsView {
    ids: ChartIds,
    spec: delinea::ChartSpec,
    engine: LocalState<ChartEngine>,
    output: LocalState<ChartCanvasOutput>,
    base_x: DataWindow,
    base_y: DataWindow,
    x_window: LocalState<DataWindow>,
    y_window: LocalState<DataWindow>,
    selected: LocalState<Option<u32>>,
}

fn build_chart(app: &mut App) -> ChartInteractionsView {
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

    let mut engine = ChartEngine::new(spec.clone()).expect("chart spec should be valid");
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

    ChartInteractionsView {
        ids,
        spec,
        engine: app.local_state(engine),
        output: app.local_state(ChartCanvasOutput::default()),
        base_x,
        base_y,
        x_window: app.local_state(base_x),
        y_window: app.local_state(base_y),
        selected: app.local_state(None),
    }
}

impl ChartInteractionsView {
    fn bind_actions(&self, cx: &mut AppUi<'_, '_>) {
        let ids = self.ids;
        let base_x = self.base_x;
        cx.actions()
            .locals_with((&self.x_window, &self.engine))
            .on::<act::ZoomIn>(move |tx, (x_window, engine)| {
                let current = tx.value(&x_window);
                let next = zoom_window(base_x, current, 0.5);
                let ok = tx.set(&x_window, next);
                tx.update(&engine, move |engine| {
                    engine.apply_action(Action::SetDataWindowX {
                        axis: ids.x_axis,
                        window: Some(next),
                    });
                }) && ok
            });

        let ids = self.ids;
        let base_x = self.base_x;
        cx.actions()
            .locals_with((&self.x_window, &self.engine))
            .on::<act::ZoomOut>(move |tx, (x_window, engine)| {
                let current = tx.value(&x_window);
                let next = zoom_window(base_x, current, 2.0);
                let ok = tx.set(&x_window, next);
                tx.update(&engine, move |engine| {
                    engine.apply_action(Action::SetDataWindowX {
                        axis: ids.x_axis,
                        window: Some(next),
                    });
                }) && ok
            });

        let ids = self.ids;
        let base_x = self.base_x;
        let base_y = self.base_y;
        cx.actions()
            .locals_with((&self.x_window, &self.y_window, &self.selected, &self.engine))
            .on::<act::ResetView>(move |tx, (x_window, y_window, selected, engine)| {
                let ok = tx.set(&x_window, base_x);
                let ok = tx.set(&y_window, base_y) && ok;
                let ok = tx.set(&selected, None) && ok;
                tx.update(&engine, move |engine| {
                    engine.apply_action(Action::SetViewWindow2D {
                        x_axis: ids.x_axis,
                        y_axis: ids.y_axis,
                        x: Some(base_x),
                        y: Some(base_y),
                    });
                }) && ok
            });

        cx.actions()
            .locals_with((&self.engine, &self.selected))
            .on::<act::SelectHover>(|tx, (engine, selected)| {
                let hit = tx
                    .read_ref(&engine, |engine| {
                        engine.output().axis_pointer.as_ref().and_then(|o| o.hit)
                    })
                    .ok()
                    .flatten();
                let Some(hit) = hit else {
                    return false;
                };
                tx.set(&selected, Some(hit.data_index))
            });

        cx.actions()
            .local(&self.selected)
            .set::<act::ClearSelection>(None);
    }
}

impl View for ChartInteractionsView {
    fn init(app: &mut App, _window: WindowId) -> Self {
        build_chart(app)
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = cx.theme_snapshot();
        self.bind_actions(cx);

        let x_window = self.x_window.layout_value(cx);
        let x_span = (x_window.max - x_window.min).max(0.0);
        let hover_index = self.engine.paint_read_ref(cx, |engine| {
            engine
                .output()
                .axis_pointer
                .as_ref()
                .and_then(|o| o.hit.map(|h| h.data_index))
                .map(|v| v as f64)
                .unwrap_or(-1.0)
        });
        let selected_index = self
            .selected
            .layout_value(cx)
            .map(|v| v as f64)
            .unwrap_or(-1.0);

        let toolbar = ui::h_flex(|cx| {
            let x_span_badge = shadcn::Badge::new(format!("X span: {x_span:.2}"))
                .variant(shadcn::BadgeVariant::Secondary)
                .a11y(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Generic)
                        .test_id(TEST_ID_X_SPAN)
                        .numeric_value(x_span)
                        .numeric_range(0.0, (self.base_x.max - self.base_x.min).max(1.0)),
                );

            let hover_badge = shadcn::Badge::new(format!("Hover index: {hover_index:.0}"))
                .variant(shadcn::BadgeVariant::Secondary)
                .a11y(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Generic)
                        .test_id(TEST_ID_HOVER_INDEX)
                        .numeric_value(hover_index)
                        .numeric_range(-1.0, (self.base_x.max - self.base_x.min).max(1.0)),
                );

            let selected_badge = shadcn::Badge::new(format!("Selected index: {selected_index:.0}"))
                .variant(shadcn::BadgeVariant::Secondary)
                .a11y(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Generic)
                        .test_id(TEST_ID_SELECTED_INDEX)
                        .numeric_value(selected_index)
                        .numeric_range(-1.0, (self.base_x.max - self.base_x.min).max(1.0)),
                );

            ui::children![
                cx;
                shadcn::Button::new("Zoom in (X)")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .action(act::ZoomIn)
                    .test_id(TEST_ID_ZOOM_IN),
                shadcn::Button::new("Zoom out (X)")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .action(act::ZoomOut)
                    .test_id(TEST_ID_ZOOM_OUT),
                shadcn::Button::new("Reset view")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::ResetView)
                    .test_id(TEST_ID_RESET_VIEW),
                shadcn::Button::new("Select hovered")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::SelectHover),
                shadcn::Button::new("Clear selection")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .action(act::ClearSelection),
                x_span_badge,
                hover_badge,
                selected_badge,
            ]
        })
        .gap(Space::N2)
        .items_center();

        let canvas = chart::ChartCanvas::new(self.spec.clone())
            .engine(&self.engine)
            .output(&self.output)
            .input_map(ChartInputMap::default())
            .accessibility_layer(true)
            .test_id(TEST_ID_CANVAS)
            .contain_layout_when_bounds_known(true)
            .into_element(cx);

        let canvas_shell = ui::container(|_cx| vec![canvas])
            .bg(ColorRef::Color(theme.color_token("card")))
            .border_1()
            .rounded(Radius::Lg)
            .p(Space::N2)
            .w_full()
            .h_full()
            .min_h(Px(420.0));

        let card = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("Chart interactions basics"),
                        shadcn::card_description(
                            "Minimal shared delinea engine + declarative chart canvas panel. App-owned zoom + selection; axis pointer hover for exploration.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::children![
                        cx;
                        ui::v_flex(|cx| ui::children![cx; toolbar, canvas_shell])
                            .gap(Space::N3)
                            .w_full()
                            .h_full()
                            .min_w_0(),
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .h_full()
        .max_w(Px(1100.0))
        .a11y(SemanticsDecoration::default().role(SemanticsRole::Group));

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    let builder = FretApp::new(ROOT_NAME)
        .window("cookbook-chart-interactions-basics", (1120.0, 820.0))
        .setup(install_commands)
        .setup((shadcn::app::install, fret_icons_lucide::app::install))
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<ChartInteractionsView>()?
        .with_command_default_keybindings()
        .with_ui_assets_budgets(64 * 1024 * 1024, 4096, 16 * 1024 * 1024, 4096);

    #[cfg(feature = "cookbook-diag")]
    let builder = builder.with_default_diagnostics();

    builder.run().map_err(anyhow::Error::from)
}
