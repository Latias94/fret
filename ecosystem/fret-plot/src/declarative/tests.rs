//! Declarative plot panel regression tests.

use super::geometry::line_plot_inner_rect;
use super::*;
use crate::cartesian::{DataPoint, DataRect};
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
use fret_ui::element::{CanvasProps, LayoutStyle, Length};
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
mod drag_output;
mod legend;
mod overlays;
mod query_box_selection;
mod right_axis;
mod series_paint;
mod view_pan;
mod wheel_zoom;

#[test]
fn line_plot_panel_props_builder_projects_canvas_layout_and_size_fields() {
    let mut app = TestHost::default();
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(Vec::new()));

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fraction(0.5);
    layout.size.height = Length::Px(Px(160.0));
    layout.size.min_height = Some(Length::Px(Px(80.0)));

    let props = LinePlotPanelProps::new(model.clone()).layout(layout);
    assert_eq!(props.canvas.layout, layout);

    let props = LinePlotPanelProps::new(model.clone())
        .width(Length::Fraction(0.75))
        .height_px(Px(240.0));
    assert_eq!(props.canvas.layout.size.width, Length::Fraction(0.75));
    assert_eq!(props.canvas.layout.size.height, Length::Px(Px(240.0)));

    let props = LinePlotPanelProps::new(model.clone())
        .size(Length::Fill, Length::Px(Px(180.0)))
        .size_px(Px(320.0), Px(120.0));
    assert_eq!(props.canvas.layout.size.width, Length::Px(Px(320.0)));
    assert_eq!(props.canvas.layout.size.height, Length::Px(Px(120.0)));

    let mut canvas = CanvasProps::default();
    canvas.layout.size.width = Length::Px(Px(220.0));
    let props = LinePlotPanelProps::new(model).canvas(canvas);
    assert_eq!(props.canvas.layout.size.width, Length::Px(Px(220.0)));
}

#[test]
fn all_plot_panel_props_builder_project_fixed_height_fields() {
    let mut app = TestHost::default();
    let expected = Length::Px(Px(144.0));

    let area = app
        .models_mut()
        .insert(AreaPlotModel::from_series(Vec::new()));
    assert_eq!(
        AreaPlotPanelProps::new(area)
            .height_px(Px(144.0))
            .canvas
            .layout
            .size
            .height,
        expected
    );

    let bars = app
        .models_mut()
        .insert(BarsPlotModel::from_series(Vec::new()));
    assert_eq!(
        BarsPlotPanelProps::new(bars)
            .height_px(Px(144.0))
            .canvas
            .layout
            .size
            .height,
        expected
    );

    let candlestick = app
        .models_mut()
        .insert(CandlestickPlotModel::from_series(Vec::new()));
    assert_eq!(
        CandlestickPlotPanelProps::new(candlestick)
            .height_px(Px(144.0))
            .canvas
            .layout
            .size
            .height,
        expected
    );

    let error_bars = app
        .models_mut()
        .insert(ErrorBarsPlotModel::from_series(Vec::new()));
    assert_eq!(
        ErrorBarsPlotPanelProps::new(error_bars)
            .height_px(Px(144.0))
            .canvas
            .layout
            .size
            .height,
        expected
    );

    let heatmap_bounds = DataRect {
        x_min: 0.0,
        x_max: 1.0,
        y_min: 0.0,
        y_max: 1.0,
    };
    let heatmap = app
        .models_mut()
        .insert(HeatmapPlotModel::new(heatmap_bounds, 1, 1, vec![0.0]));
    assert_eq!(
        HeatmapPlotPanelProps::new(heatmap)
            .height_px(Px(144.0))
            .canvas
            .layout
            .size
            .height,
        expected
    );

    let histogram = app
        .models_mut()
        .insert(HistogramPlotModel::from_series(Vec::new()));
    assert_eq!(
        HistogramPlotPanelProps::new(histogram)
            .height_px(Px(144.0))
            .canvas
            .layout
            .size
            .height,
        expected
    );

    let histogram2d =
        app.models_mut()
            .insert(Histogram2DPlotModel::new(heatmap_bounds, 1, 1, vec![0.0]));
    assert_eq!(
        Histogram2DPlotPanelProps::new(histogram2d)
            .height_px(Px(144.0))
            .canvas
            .layout
            .size
            .height,
        expected
    );

    let shaded = app
        .models_mut()
        .insert(ShadedPlotModel::from_series(Vec::new()));
    assert_eq!(
        ShadedPlotPanelProps::new(shaded)
            .height_px(Px(144.0))
            .canvas
            .layout
            .size
            .height,
        expected
    );

    let stems = app
        .models_mut()
        .insert(StemsPlotModel::from_series(Vec::new()));
    assert_eq!(
        StemsPlotPanelProps::new(stems)
            .height_px(Px(144.0))
            .canvas
            .layout
            .size
            .height,
        expected
    );
}

#[test]
fn line_plot_panel_respects_explicit_canvas_height_on_declarative_path() {
    let mut app = TestHost::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(280.0)),
    );
    let mut services = FakeServices::default();
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![LineSeries::new(
            "Series",
            Series::from_points_sorted(
                vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 1.0 }],
                true,
            ),
        )]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-fixed-height",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).height_px(Px(160.0)),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let root_children = ui.debug_node_children(root);
    assert_eq!(root_children.len(), 1);
    let panel_bounds = ui
        .debug_node_bounds(root_children[0])
        .expect("line plot panel should have layout bounds");
    assert_eq!(panel_bounds.size.width, Px(320.0));
    assert_eq!(panel_bounds.size.height, Px(160.0));
}

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
