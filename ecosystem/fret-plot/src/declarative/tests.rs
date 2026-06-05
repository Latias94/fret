//! Declarative plot panel regression tests.

use super::geometry::line_plot_inner_rect;
use super::*;
use crate::cartesian::DataPoint;
use crate::models::{
    AreaPlotModel, AreaSeries, BarSeries, BarsPlotModel, CandlestickPlotModel, CandlestickSeries,
    ErrorBar, ErrorBarsPlotModel, ErrorBarsSeries, HeatmapPlotModel, Histogram2DPlotModel,
    HistogramPlotModel, HistogramSeries, LinePlotModel, LineSeries, OhlcPoint, ShadedPlotModel,
    StemsPlotModel, StemsSeries, YAxis,
};
use crate::series::Series;
use crate::state::{
    DragLineX, DragLineY, DragPoint, DragRect, InfLineX, InfLineY, PlotDragOutput, PlotDragPhase,
    PlotImage, PlotOutput, PlotState, PlotText, TagX, TagY,
};
use fret_core::{
    AppWindowId, Color, DrawOrder, Event, FrameId, ImageId, MaterialDescriptor, MaterialId,
    MaterialRegistrationError, MaterialService, Modifiers, MouseButton, MouseButtons, Paint,
    PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, Point, PointerEvent,
    PointerId, PointerType, Px, Rect, Scene, Size, SvgId, SvgService, TextBlobId, TextConstraints,
    TextInput, TextMetrics, TextService, UvRect,
};
use fret_runtime::{
    ClipboardToken, CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession,
    DragSessionId, Effect, EffectSink, GlobalsHost, ImageUploadToken, ModelHost, ModelId,
    ModelStore, ModelsHost, ShareSheetToken, TickId, TimeHost, TimerToken,
};
use fret_ui::UiTree;
use fret_ui::declarative::render_root;
use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Default)]
struct TestHost {
    globals: HashMap<TypeId, Box<dyn Any>>,
    models: ModelStore,
    commands: CommandRegistry,
    drags: HashMap<fret_core::PointerId, DragSession>,
    frame_id: FrameId,
    tick_id: TickId,
    next_timer_token: u64,
    next_clipboard_token: u64,
    next_share_sheet_token: u64,
    next_image_upload_token: u64,
    next_drag_session_id: u64,
}

impl TestHost {
    fn set_frame_id(&mut self, frame_id: FrameId) {
        self.frame_id = frame_id;
    }
}

impl GlobalsHost for TestHost {
    fn set_global<T: Any>(&mut self, value: T) {
        self.globals.insert(TypeId::of::<T>(), Box::new(value));
    }

    fn global<T: Any>(&self) -> Option<&T> {
        self.globals
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }

    fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R {
        let type_id = TypeId::of::<T>();
        let mut value = self
            .globals
            .remove(&type_id)
            .map(|value| *value.downcast::<T>().expect("global type id should match"))
            .unwrap_or_else(init);
        let out = f(&mut value, self);
        self.globals.insert(type_id, Box::new(value));
        out
    }
}

impl ModelHost for TestHost {
    fn models(&self) -> &ModelStore {
        &self.models
    }

    fn models_mut(&mut self) -> &mut ModelStore {
        &mut self.models
    }
}

impl ModelsHost for TestHost {
    fn take_changed_models(&mut self) -> Vec<ModelId> {
        self.models.take_changed_models()
    }
}

impl CommandsHost for TestHost {
    fn commands(&self) -> &CommandRegistry {
        &self.commands
    }
}

impl EffectSink for TestHost {
    fn request_redraw(&mut self, _window: AppWindowId) {}

    fn push_effect(&mut self, _effect: Effect) {}
}

impl TimeHost for TestHost {
    fn tick_id(&self) -> TickId {
        self.tick_id
    }

    fn frame_id(&self) -> FrameId {
        self.frame_id
    }

    fn next_timer_token(&mut self) -> TimerToken {
        let token = TimerToken(self.next_timer_token);
        self.next_timer_token = self.next_timer_token.saturating_add(1);
        token
    }

    fn next_clipboard_token(&mut self) -> ClipboardToken {
        let token = ClipboardToken(self.next_clipboard_token);
        self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
        token
    }

    fn next_share_sheet_token(&mut self) -> ShareSheetToken {
        let token = ShareSheetToken(self.next_share_sheet_token);
        self.next_share_sheet_token = self.next_share_sheet_token.saturating_add(1);
        token
    }

    fn next_image_upload_token(&mut self) -> ImageUploadToken {
        let token = ImageUploadToken(self.next_image_upload_token);
        self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
        token
    }
}

impl DragHost for TestHost {
    fn drag(&self, pointer_id: fret_core::PointerId) -> Option<&DragSession> {
        self.drags.get(&pointer_id)
    }

    fn drag_mut(&mut self, pointer_id: fret_core::PointerId) -> Option<&mut DragSession> {
        self.drags.get_mut(&pointer_id)
    }

    fn cancel_drag(&mut self, pointer_id: fret_core::PointerId) {
        self.drags.remove(&pointer_id);
    }

    fn any_drag_session(&self, predicate: impl FnMut(&DragSession) -> bool) -> bool {
        self.drags.values().any(predicate)
    }

    fn find_drag_pointer_id(
        &self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Option<fret_core::PointerId> {
        self.drags
            .values()
            .find(|session| predicate(session))
            .map(|session| session.pointer_id)
    }

    fn cancel_drag_sessions(
        &mut self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Vec<fret_core::PointerId> {
        let pointer_ids: Vec<_> = self
            .drags
            .values()
            .filter(|session| predicate(session))
            .map(|session| session.pointer_id)
            .collect();
        for pointer_id in &pointer_ids {
            self.drags.remove(pointer_id);
        }
        pointer_ids
    }

    fn begin_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: fret_core::PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        let session_id = DragSessionId(self.next_drag_session_id);
        self.next_drag_session_id = self.next_drag_session_id.saturating_add(1);
        self.drags.insert(
            pointer_id,
            DragSession::new(session_id, pointer_id, source_window, kind, start, payload),
        );
    }

    fn begin_cross_window_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: fret_core::PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        let session_id = DragSessionId(self.next_drag_session_id);
        self.next_drag_session_id = self.next_drag_session_id.saturating_add(1);
        self.drags.insert(
            pointer_id,
            DragSession::new_cross_window(
                session_id,
                pointer_id,
                source_window,
                kind,
                start,
                payload,
            ),
        );
    }
}

#[derive(Default)]
struct FakeServices {
    prepared_text: Vec<String>,
    prepared_paths: Vec<Vec<PathCommand>>,
}

impl TextService for FakeServices {
    fn prepare(
        &mut self,
        input: &TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        self.prepared_text.push(input.text().to_string());
        (
            TextBlobId::default(),
            TextMetrics {
                size: Size::default(),
                baseline: Px(0.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl PathService for FakeServices {
    fn prepare(
        &mut self,
        commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        self.prepared_paths.push(commands.to_vec());
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
}

impl SvgService for FakeServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

impl MaterialService for FakeServices {
    fn register_material(
        &mut self,
        _desc: MaterialDescriptor,
    ) -> Result<MaterialId, MaterialRegistrationError> {
        Err(MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: MaterialId) -> bool {
        true
    }
}

fn line_plot_selection_rects(scene: &Scene) -> Vec<Rect> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            fret_core::SceneOp::Quad {
                order: DrawOrder(5),
                rect,
                ..
            } => Some(*rect),
            _ => None,
        })
        .collect()
}

fn line_plot_reference_line_rects(scene: &Scene) -> Vec<Rect> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            fret_core::SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                ..
            } => Some(*rect),
            _ => None,
        })
        .collect()
}

fn line_plot_image_regions(scene: &Scene) -> Vec<(Rect, UvRect, f32)> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            fret_core::SceneOp::ImageRegion {
                rect, uv, opacity, ..
            } => Some((*rect, *uv, *opacity)),
            _ => None,
        })
        .collect()
}

fn assert_line_plot_selection_rect(rect: Rect, x: f32, y: f32, width: f32, height: f32) {
    assert!(
        (rect.origin.x.0 - x).abs() < 0.01,
        "unexpected selection rect x: expected {x}, got {rect:?}"
    );
    assert!(
        (rect.origin.y.0 - y).abs() < 0.01,
        "unexpected selection rect y: expected {y}, got {rect:?}"
    );
    assert!(
        (rect.size.width.0 - width).abs() < 0.01,
        "unexpected selection rect width: expected {width}, got {rect:?}"
    );
    assert!(
        (rect.size.height.0 - height).abs() < 0.01,
        "unexpected selection rect height: expected {height}, got {rect:?}"
    );
}

mod cursor_readout;
mod legend;
mod overlays;
mod query_box_selection;
mod right_axis;
mod series_paint;
mod view_pan;
mod wheel_zoom;

#[test]
fn line_plot_panel_paints_axes_and_grid_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let mut services = FakeServices::default();
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![LineSeries::new(
            "Series",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 0.25 },
                ],
                true,
            ),
        )]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-axes-grid",
        |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let axis_quads = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(10),
                    ..
                }
            )
        })
        .count();
    assert!(
        axis_quads >= 2,
        "declarative line plot should paint x/y axis lines"
    );

    let grid_quads = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(2),
                    ..
                }
            )
        })
        .count();
    assert!(
        grid_quads >= 2,
        "declarative line plot should paint tick-derived grid lines"
    );

    let line_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(20),
                    ..
                }
            )
        })
        .count();
    assert!(
        line_paths > 0,
        "declarative line plot should keep series paths above grid/axes"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn line_plot_panel_paints_axis_tick_labels_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );
    let mut services = FakeServices::default();
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![LineSeries::new(
            "Series",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 0.25 },
                ],
                true,
            ),
        )]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-axis-labels",
        |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let axis_labels = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Text {
                    order: DrawOrder(11),
                    ..
                }
            )
        })
        .count();
    assert!(
        axis_labels >= 4,
        "declarative line plot should paint x/y tick labels"
    );

    let series_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(20),
                    ..
                }
            )
        })
        .count();
    assert!(
        series_paths > 0,
        "axis label painting should not replace seeded series paths"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn line_plot_panel_drags_right_axis_y_line_output_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let mut services = FakeServices::default();
    let left_series = LineSeries::new(
        "left",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
            true,
        ),
    );
    let right_series = LineSeries::new(
        "right",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
            true,
        ),
    )
    .y_axis(YAxis::Right);
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![left_series, right_series]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    plot_state
        .overlays
        .drag_lines_y
        .push(DragLineY::new(50, 100.0, YAxis::Right));
    let state = app.models_mut().insert(plot_state);
    let output = app.models_mut().insert(PlotOutput::default());
    let style = LinePlotStyle {
        clamp_to_data_bounds: false,
        ..LinePlotStyle::default()
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-right-axis-drag-line-y-output",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .output(output.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(169.0), Px(8.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(169.0), Px(81.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("right-axis draggable Y line should publish drag output");
    match drag {
        PlotDragOutput::LineY { id, axis, y, phase } => {
            assert_eq!(id, 50);
            assert_eq!(axis, YAxis::Right);
            assert_eq!(phase, PlotDragPhase::Update);
            assert!(
                (y - 50.0).abs() < 0.2,
                "dragging to the plot middle should map through right-axis bounds, got {y}"
            );
        }
        other => panic!("expected right-axis LineY drag output, got {other:?}"),
    }

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(169.0), Px(81.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("right-axis draggable Y line should publish drag end output");
    match drag {
        PlotDragOutput::LineY { id, axis, y, phase } => {
            assert_eq!(id, 50);
            assert_eq!(axis, YAxis::Right);
            assert_eq!(phase, PlotDragPhase::End);
            assert!(
                (y - 50.0).abs() < 0.2,
                "drag end should preserve the right-axis mapped value, got {y}"
            );
        }
        other => panic!("expected right-axis LineY drag end output, got {other:?}"),
    }
}

#[test]
fn line_plot_panel_drags_x_line_output_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let mut services = FakeServices::default();
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![LineSeries::new(
            "Series",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
                true,
            ),
        )]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    plot_state
        .overlays
        .drag_lines_x
        .push(DragLineX::new(60, 1.0));
    let state = app.models_mut().insert(plot_state);
    let output = app.models_mut().insert(PlotOutput::default());
    let style = LinePlotStyle {
        clamp_to_data_bounds: false,
        ..LinePlotStyle::default()
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-drag-line-x-output",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .output(output.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(98.0), Px(81.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(169.0), Px(81.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("draggable X line should publish drag output");
    match drag {
        PlotDragOutput::LineX { id, x, phase } => {
            assert_eq!(id, 60);
            assert_eq!(phase, PlotDragPhase::Update);
            assert!(
                (x - 2.0).abs() < 0.03,
                "dragging to the plot middle should map through the X view bounds, got {x}"
            );
        }
        other => panic!("expected LineX drag output, got {other:?}"),
    }

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(169.0), Px(81.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("draggable X line should publish drag end output");
    match drag {
        PlotDragOutput::LineX { id, x, phase } => {
            assert_eq!(id, 60);
            assert_eq!(phase, PlotDragPhase::End);
            assert!(
                (x - 2.0).abs() < 0.03,
                "drag end should preserve the X mapped value, got {x}"
            );
        }
        other => panic!("expected LineX drag end output, got {other:?}"),
    }
}

#[test]
fn line_plot_panel_drags_right_axis_point_output_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let mut services = FakeServices::default();
    let left_series = LineSeries::new(
        "left",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
            true,
        ),
    );
    let right_series = LineSeries::new(
        "right",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
            true,
        ),
    )
    .y_axis(YAxis::Right);
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![left_series, right_series]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    plot_state.overlays.drag_points.push(DragPoint::new(
        70,
        DataPoint { x: 2.0, y: 50.0 },
        YAxis::Right,
    ));
    let state = app.models_mut().insert(plot_state);
    let output = app.models_mut().insert(PlotOutput::default());
    let style = LinePlotStyle {
        clamp_to_data_bounds: false,
        ..LinePlotStyle::default()
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-right-axis-drag-point-output",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .output(output.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(169.0), Px(81.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(240.5), Px(117.5)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("right-axis draggable point should publish drag output");
    match drag {
        PlotDragOutput::Point {
            id,
            axis,
            point,
            phase,
        } => {
            assert_eq!(id, 70);
            assert_eq!(axis, YAxis::Right);
            assert_eq!(phase, PlotDragPhase::Update);
            assert!(
                (point.x - 3.0).abs() < 0.03,
                "dragging point right should map through the X view bounds, got {point:?}"
            );
            assert!(
                (point.y - 25.0).abs() < 0.3,
                "dragging point down should map through right-axis bounds, got {point:?}"
            );
        }
        other => panic!("expected right-axis Point drag output, got {other:?}"),
    }

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(240.5), Px(117.5)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("right-axis draggable point should publish drag end output");
    match drag {
        PlotDragOutput::Point {
            id,
            axis,
            point,
            phase,
        } => {
            assert_eq!(id, 70);
            assert_eq!(axis, YAxis::Right);
            assert_eq!(phase, PlotDragPhase::End);
            assert!(
                (point.x - 3.0).abs() < 0.03 && (point.y - 25.0).abs() < 0.3,
                "drag end should preserve the mapped point, got {point:?}"
            );
        }
        other => panic!("expected right-axis Point drag end output, got {other:?}"),
    }
}

#[test]
fn line_plot_panel_drags_right_axis_rect_output_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let mut services = FakeServices::default();
    let left_series = LineSeries::new(
        "left",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
            true,
        ),
    );
    let right_series = LineSeries::new(
        "right",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
            true,
        ),
    )
    .y_axis(YAxis::Right);
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![left_series, right_series]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    plot_state.overlays.drag_rects.push(DragRect::new(
        80,
        DataRect {
            x_min: 1.0,
            x_max: 3.0,
            y_min: 25.0,
            y_max: 75.0,
        },
        YAxis::Right,
    ));
    let state = app.models_mut().insert(plot_state);
    let output = app.models_mut().insert(PlotOutput::default());
    let style = LinePlotStyle {
        clamp_to_data_bounds: false,
        ..LinePlotStyle::default()
    };

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-right-axis-drag-rect-output",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .output(output.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(169.0), Px(81.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(240.5), Px(117.5)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("right-axis draggable rect should publish drag output");
    match drag {
        PlotDragOutput::Rect {
            id,
            axis,
            rect,
            phase,
        } => {
            assert_eq!(id, 80);
            assert_eq!(axis, YAxis::Right);
            assert_eq!(phase, PlotDragPhase::Update);
            assert!(
                (rect.x_min - 2.0).abs() < 0.03
                    && (rect.x_max - 4.0).abs() < 0.03
                    && (rect.y_min - 0.0).abs() < 0.3
                    && (rect.y_max - 50.0).abs() < 0.3,
                "dragging inside the rect should move the whole right-axis rect, got {rect:?}"
            );
        }
        other => panic!("expected right-axis Rect drag output, got {other:?}"),
    }

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(240.5), Px(117.5)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let drag = output_snapshot
        .drag
        .expect("right-axis draggable rect should publish drag end output");
    match drag {
        PlotDragOutput::Rect {
            id,
            axis,
            rect,
            phase,
        } => {
            assert_eq!(id, 80);
            assert_eq!(axis, YAxis::Right);
            assert_eq!(phase, PlotDragPhase::End);
            assert!(
                (rect.x_min - 2.0).abs() < 0.03
                    && (rect.x_max - 4.0).abs() < 0.03
                    && (rect.y_min - 0.0).abs() < 0.3
                    && (rect.y_max - 50.0).abs() < 0.3,
                "drag end should preserve the mapped rect, got {rect:?}"
            );
        }
        other => panic!("expected right-axis Rect drag end output, got {other:?}"),
    }
}
