#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, Effect, WindowRequest};
use fret_core::{AppWindowId, Event, MouseButton, PointerEvent, Rect};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
};
use fret_plot::cartesian::DataPoint;
use fret_plot::linking::{LinkedPlotGroup, LinkedPlotMember, PlotLinkPolicy};
use fret_plot::retained::{
    AreaPlotCanvas, AreaPlotModel, AreaSeries, LinePlotCanvas, LinePlotModel, LinePlotStyle,
    LineSeries, PlotOutput, PlotState,
};
use fret_plot::series::Series;
use fret_runtime::PlatformCapabilities;
use fret_ui::UiTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pane {
    Top,
    Bottom,
}

impl Default for Pane {
    fn default() -> Self {
        Self::Top
    }
}

struct LinkedCursorDemoWindowState {
    top_ui: UiTree<App>,
    top_root: Option<fret_core::NodeId>,
    bottom_ui: UiTree<App>,
    bottom_root: Option<fret_core::NodeId>,
    top_plot: fret_runtime::Model<LinePlotModel>,
    bottom_plot: fret_runtime::Model<AreaPlotModel>,
    top_state: fret_runtime::Model<PlotState>,
    top_output: fret_runtime::Model<PlotOutput>,
    bottom_state: fret_runtime::Model<PlotState>,
    bottom_output: fret_runtime::Model<PlotOutput>,
    linked: LinkedPlotGroup,
    focused_pane: Pane,
    active_pointer_pane: Option<Pane>,
    last_bounds: Rect,
}

#[derive(Default)]
struct LinkedCursorDemoDriver;

impl LinkedCursorDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> LinkedCursorDemoWindowState {
        let n = 4096usize;

        let mut series0: Vec<DataPoint> = Vec::with_capacity(n);
        let mut series1: Vec<DataPoint> = Vec::with_capacity(n);
        let mut series2: Vec<DataPoint> = Vec::with_capacity(n);

        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let x = t * 10.0;
            series0.push(DataPoint {
                x,
                y: (x * 1.25).sin() * 0.75 + (x * 0.33).cos() * 0.25,
            });
            series1.push(DataPoint {
                x,
                y: (x * 1.10).sin() * 0.55 + (x * 0.20).cos() * 0.20 + 0.35,
            });
            series2.push(DataPoint {
                x,
                y: (x * 0.75).sin() * 0.35 + (x * 0.15).cos() * 0.10 - 0.35,
            });
        }

        let top_plot = app.models_mut().insert(LinePlotModel::from_series(vec![
            LineSeries::new("signal A", Series::from_points_sorted(series0, true)),
            LineSeries::new(
                "signal B",
                Series::from_points_sorted(series1.clone(), true),
            ),
            LineSeries::new(
                "signal C",
                Series::from_points_sorted(series2.clone(), true),
            ),
        ]));

        let bottom_plot = app.models_mut().insert(AreaPlotModel::from_series(vec![
            AreaSeries::new("area B", Series::from_points_sorted(series1, true)).fill_alpha(0.18),
            AreaSeries::new("area C", Series::from_points_sorted(series2, true)).fill_alpha(0.18),
        ]));

        let top_state = app.models_mut().insert(PlotState::default());
        let top_output = app.models_mut().insert(PlotOutput::default());
        let bottom_state = app.models_mut().insert(PlotState::default());
        let bottom_output = app.models_mut().insert(PlotOutput::default());

        let mut linked = LinkedPlotGroup::new(PlotLinkPolicy::default());
        linked
            .push(LinkedPlotMember {
                state: top_state.clone(),
                output: top_output.clone(),
            })
            .push(LinkedPlotMember {
                state: bottom_state.clone(),
                output: bottom_output.clone(),
            });

        let mut top_ui: UiTree<App> = UiTree::new();
        top_ui.set_window(window);
        let mut bottom_ui: UiTree<App> = UiTree::new();
        bottom_ui.set_window(window);

        LinkedCursorDemoWindowState {
            top_ui,
            top_root: None,
            bottom_ui,
            bottom_root: None,
            top_plot,
            bottom_plot,
            top_state,
            top_output,
            bottom_state,
            bottom_output,
            linked,
            focused_pane: Pane::default(),
            active_pointer_pane: None,
            last_bounds: Rect::new(
                fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
                fret_core::Size::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            ),
        }
    }

    fn split_y(&self, state: &LinkedCursorDemoWindowState) -> fret_core::Px {
        fret_core::Px(state.last_bounds.origin.y.0 + state.last_bounds.size.height.0 * 0.5)
    }

    fn pane_for_position(
        &self,
        state: &LinkedCursorDemoWindowState,
        position: fret_core::Point,
    ) -> Pane {
        let split_y = self.split_y(state);
        if position.y.0 < split_y.0 {
            Pane::Top
        } else {
            Pane::Bottom
        }
    }

    fn dispatch_to_pane(
        &self,
        pane: Pane,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        state: &mut LinkedCursorDemoWindowState,
        event: &Event,
    ) {
        match pane {
            Pane::Top => state.top_ui.dispatch_event(app, services, event),
            Pane::Bottom => state.bottom_ui.dispatch_event(app, services, event),
        }
    }

    fn dispatch_to_both(
        &self,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        state: &mut LinkedCursorDemoWindowState,
        event: &Event,
    ) {
        state.top_ui.dispatch_event(app, services, event);
        state.bottom_ui.dispatch_event(app, services, event);
    }
}

impl WinitAppDriver for LinkedCursorDemoDriver {
    type WindowState = LinkedCursorDemoWindowState;

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
        crate::hotpatch::reset_ui_tree(app, window, &mut state.top_ui);
        crate::hotpatch::reset_ui_tree(app, window, &mut state.bottom_ui);
        state.top_root = None;
        state.bottom_root = None;
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
            ..
        } = context;

        match event {
            Event::WindowCloseRequested
            | Event::KeyDown {
                key: fret_core::KeyCode::Escape,
                ..
            } => {
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
                return;
            }
            Event::KeyDown { .. } => {
                self.dispatch_to_pane(state.focused_pane, app, services, state, event);
            }
            Event::Pointer(PointerEvent::Down {
                position,
                button: MouseButton::Left,
                ..
            }) => {
                let pane = self.pane_for_position(state, *position);
                state.focused_pane = pane;
                state.active_pointer_pane = Some(pane);
                self.dispatch_to_pane(pane, app, services, state, event);
            }
            Event::Pointer(PointerEvent::Up {
                button: MouseButton::Left,
                ..
            }) => {
                if let Some(active) = state.active_pointer_pane.take() {
                    self.dispatch_to_pane(active, app, services, state, event);
                } else {
                    self.dispatch_to_both(app, services, state, event);
                }
            }
            Event::Pointer(PointerEvent::Move { .. }) => {
                if let Some(active) = state.active_pointer_pane {
                    self.dispatch_to_pane(active, app, services, state, event);
                } else {
                    // Send move to both so the previously hovered plot can clear its cursor/output.
                    // The linked-plot coordinator will prefer the plot that currently has a cursor.
                    self.dispatch_to_both(app, services, state, event);
                }
            }
            Event::Pointer(PointerEvent::Wheel { position, .. }) => {
                let pane = self.pane_for_position(state, *position);
                self.dispatch_to_pane(pane, app, services, state, event);
            }
            _ => {
                self.dispatch_to_both(app, services, state, event);
            }
        }

        state.linked.tick(app);
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

        state.last_bounds = bounds;

        let top_h = fret_core::Px((bounds.size.height.0 * 0.5).max(0.0));
        let bottom_h = fret_core::Px((bounds.size.height.0 - top_h.0).max(0.0));

        let top_bounds = Rect::new(
            bounds.origin,
            fret_core::Size::new(bounds.size.width, top_h),
        );
        let bottom_bounds = Rect::new(
            fret_core::Point::new(bounds.origin.x, fret_core::Px(bounds.origin.y.0 + top_h.0)),
            fret_core::Size::new(bounds.size.width, bottom_h),
        );

        let top_root = state.top_root.get_or_insert_with(|| {
            let style = LinePlotStyle::default();
            let canvas = LinePlotCanvas::new(state.top_plot.clone())
                .style(style)
                .state(state.top_state.clone())
                .output(state.top_output.clone());
            let node = LinePlotCanvas::create_node(&mut state.top_ui, canvas);
            state.top_ui.set_root(node);
            node
        });

        let bottom_root = state.bottom_root.get_or_insert_with(|| {
            let style = LinePlotStyle::default();
            let canvas = AreaPlotCanvas::new(state.bottom_plot.clone())
                .style(style)
                .state(state.bottom_state.clone())
                .output(state.bottom_output.clone());
            let node = AreaPlotCanvas::create_node(&mut state.bottom_ui, canvas);
            state.bottom_ui.set_root(node);
            node
        });

        state.top_ui.set_root(*top_root);
        state.bottom_ui.set_root(*bottom_root);

        state.top_ui.request_semantics_snapshot();
        state.top_ui.ingest_paint_cache_source(scene);
        state.bottom_ui.request_semantics_snapshot();
        state.bottom_ui.ingest_paint_cache_source(scene);

        scene.clear();

        let mut top_frame = fret_ui::UiFrameCx::new(
            &mut state.top_ui,
            app,
            services,
            window,
            top_bounds,
            scale_factor,
        );
        top_frame.layout_all();
        top_frame.paint_all(scene);

        let mut bottom_frame = fret_ui::UiFrameCx::new(
            &mut state.bottom_ui,
            app,
            services,
            window,
            bottom_bounds,
            scale_factor,
        );
        bottom_frame.layout_all();
        bottom_frame.paint_all(scene);
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: AppWindowId,
    ) {
    }
}

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo linked_cursor_demo (linked view/query/cursor)".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(960.0, 760.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    LinkedCursorDemoDriver::default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let app = build_app();
    let config = build_runner_config();
    let driver = build_driver();

    crate::run_native_demo(config, app, driver).context("run linked_cursor_demo app")
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
