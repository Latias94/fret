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

#[test]
fn line_plot_panel_paints_seeded_line_on_declarative_path() {
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
                    DataPoint { x: 0.0, y: 1.0 },
                    DataPoint { x: 1.0, y: 4.0 },
                    DataPoint { x: 2.0, y: 2.0 },
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
        "plot-declarative-line-panel",
        |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let line_paths = scene
        .ops()
        .iter()
        .filter(|op| matches!(op, fret_core::SceneOp::Path { order, .. } if order.0 >= 1))
        .count();
    assert!(
        line_paths > 0,
        "declarative line plot panel should emit at least one path"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn area_plot_panel_paints_area_fill_and_stroke_on_declarative_path() {
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
    let model = app.models_mut().insert(AreaPlotModel::from_series(vec![
        AreaSeries::new(
            "Area",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.2 },
                    DataPoint { x: 1.0, y: 0.8 },
                    DataPoint { x: 2.0, y: 0.4 },
                ],
                true,
            ),
        )
        .fill_alpha(0.25),
    ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-area-panel",
        |cx| vec![area_plot_panel(cx, AreaPlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let fill_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(19),
                    ..
                }
            )
        })
        .count();
    let stroke_paths = scene
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
    assert_eq!(
        fill_paths, 1,
        "declarative area plot should emit one filled area path"
    );
    assert_eq!(
        stroke_paths, 1,
        "declarative area plot should keep the area stroke path"
    );
    assert!(
        services
            .prepared_paths
            .iter()
            .any(|path| path.iter().any(|cmd| matches!(cmd, PathCommand::Close))),
        "area fill path should close back to the baseline"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn error_bars_plot_panel_paints_x_y_errors_caps_and_markers_on_declarative_path() {
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
        .insert(ErrorBarsPlotModel::from_series(vec![
            ErrorBarsSeries::new(
                "measurement",
                Series::from_points_sorted(vec![DataPoint { x: 1.0, y: 1.0 }], true),
            )
            .x_errors(std::sync::Arc::from([ErrorBar::symmetric(0.25)]))
            .y_errors(std::sync::Arc::from([ErrorBar::symmetric(0.5)]))
            .cap_size(Px(5.0))
            .marker_radius(Px(3.0)),
        ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-error-bars-panel",
        |cx| {
            vec![error_bars_plot_panel(
                cx,
                ErrorBarsPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let error_paths = scene
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
    assert_eq!(
        error_paths, 1,
        "declarative error-bars plot should emit one path for the series error bars"
    );

    let error_path = services
        .prepared_paths
        .iter()
        .find(|path| path.len() >= 16)
        .expect("error-bars path should include y-error, x-error, caps, and plus marker");
    assert!(
        !error_path
            .iter()
            .any(|command| matches!(command, PathCommand::Close)),
        "default error-bars markers and caps should be open stroke commands"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn histogram_plot_panel_paints_closed_bin_fill_paths_on_declarative_path() {
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
        .insert(HistogramPlotModel::from_series(vec![
            HistogramSeries::new("histogram", std::sync::Arc::from([0.1, 0.2, 0.8, 1.2, 1.8]))
                .bins(2)
                .range(0.0, 2.0)
                .bar_gap_fraction(0.0),
        ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-histogram-panel",
        |cx| {
            vec![histogram_plot_panel(
                cx,
                HistogramPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let fill_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(19),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        fill_paths, 1,
        "declarative histogram should emit one fill path for the series bins"
    );

    let histogram_path = services
        .prepared_paths
        .iter()
        .find(|path| {
            path.iter()
                .filter(|cmd| matches!(cmd, PathCommand::Close))
                .count()
                >= 2
        })
        .expect("histogram fill path should close each non-empty bin");
    assert_eq!(
        histogram_path
            .iter()
            .filter(|cmd| matches!(cmd, PathCommand::Close))
            .count(),
        2,
        "the fixture should produce two closed histogram bin rectangles"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn heatmap_plot_panel_paints_grid_cells_as_declarative_quads() {
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
    let model = app.models_mut().insert(HeatmapPlotModel::new(
        DataRect {
            x_min: 0.0,
            x_max: 2.0,
            y_min: 0.0,
            y_max: 2.0,
        },
        2,
        2,
        [0.0_f32, 0.5, 0.75, 1.0],
    ));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-heatmap-panel",
        |cx| {
            vec![heatmap_plot_panel(
                cx,
                HeatmapPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let heatmap_quads: Vec<_> = scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            fret_core::SceneOp::Quad {
                order: DrawOrder(2),
                rect,
                ..
            } => Some(*rect),
            _ => None,
        })
        .filter(|rect| rect.size.width.0 > 20.0 && rect.size.height.0 > 20.0)
        .collect();
    assert_eq!(
        heatmap_quads.len(),
        4,
        "declarative heatmap should emit one visible quad per finite grid cell"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn histogram2d_plot_panel_paints_grid_cells_and_default_colorbar_on_declarative_path() {
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
    let model = app.models_mut().insert(Histogram2DPlotModel::new(
        DataRect {
            x_min: 0.0,
            x_max: 2.0,
            y_min: 0.0,
            y_max: 2.0,
        },
        2,
        2,
        [0.0_f32, 2.0, 3.0, 4.0],
    ));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-histogram2d-panel",
        |cx| {
            vec![histogram2d_plot_panel(
                cx,
                Histogram2DPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let histogram2d_quads: Vec<_> = scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            fret_core::SceneOp::Quad {
                order: DrawOrder(2),
                rect,
                ..
            } => Some(*rect),
            _ => None,
        })
        .filter(|rect| rect.size.width.0 > 20.0 && rect.size.height.0 > 20.0)
        .collect();
    assert_eq!(
        histogram2d_quads.len(),
        4,
        "declarative histogram2d should emit one visible quad per finite grid cell"
    );

    let gradient_steps = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(4),
                    ..
                }
            )
        })
        .count();
    assert!(
        gradient_steps >= 8,
        "declarative histogram2d should paint a default colorbar gradient"
    );

    assert!(
        services.prepared_text.iter().any(|text| text == "4.000"),
        "declarative histogram2d colorbar should label the finite maximum value"
    );
    assert!(
        services.prepared_text.iter().any(|text| text == "0.000"),
        "declarative histogram2d colorbar should label the finite minimum value"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn heatmap_plot_panel_paints_default_colorbar_on_declarative_path() {
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
    let model = app.models_mut().insert(HeatmapPlotModel::new(
        DataRect {
            x_min: 0.0,
            x_max: 2.0,
            y_min: 0.0,
            y_max: 2.0,
        },
        2,
        2,
        [0.0_f32, 0.5, 0.75, 1.0],
    ));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-heatmap-colorbar-panel",
        |cx| {
            vec![heatmap_plot_panel(
                cx,
                HeatmapPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let gradient_steps = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(4),
                    ..
                }
            )
        })
        .count();
    assert!(
        gradient_steps >= 8,
        "declarative heatmap should paint a default colorbar gradient"
    );

    assert!(
        services.prepared_text.iter().any(|text| text == "1.000"),
        "declarative heatmap colorbar should label the finite maximum value"
    );
    assert!(
        services.prepared_text.iter().any(|text| text == "0.000"),
        "declarative heatmap colorbar should label the finite minimum value"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn candlestick_plot_panel_paints_wicks_and_up_down_bodies_on_declarative_path() {
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
        .insert(CandlestickPlotModel::from_series(vec![
            CandlestickSeries::new_sorted(
                "ohlc",
                std::sync::Arc::from([
                    OhlcPoint {
                        x: 0.0,
                        open: 1.0,
                        high: 2.0,
                        low: 0.5,
                        close: 1.5,
                    },
                    OhlcPoint {
                        x: 1.0,
                        open: 2.0,
                        high: 2.5,
                        low: 1.0,
                        close: 1.25,
                    },
                ]),
                true,
            )
            .width(0.8),
        ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-candlestick-panel",
        |cx| {
            vec![candlestick_plot_panel(
                cx,
                CandlestickPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let wick_paths = scene
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
    assert_eq!(
        wick_paths, 1,
        "declarative candlestick should emit one wick stroke path"
    );

    let body_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(19),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        body_paths, 2,
        "declarative candlestick should emit separate up and down body fill paths"
    );

    let closed_body_paths = services
        .prepared_paths
        .iter()
        .filter(|path| path.iter().any(|cmd| matches!(cmd, PathCommand::Close)))
        .count();
    assert_eq!(
        closed_body_paths, 2,
        "up and down candle bodies should be closed fill rectangles"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn bars_plot_panel_paints_grouped_and_stacked_closed_fill_paths_on_declarative_path() {
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
    let grouped = BarSeries::new(
        "grouped",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 1.0 }, DataPoint { x: 1.0, y: 2.0 }],
            true,
        ),
    )
    .width(0.8);
    let stacked = BarSeries::new(
        "stacked",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 2.5 }, DataPoint { x: 1.0, y: -1.5 }],
            true,
        ),
    )
    .width(0.8)
    .baseline_by_index(std::sync::Arc::from([1.0, -0.5]));
    let model = app
        .models_mut()
        .insert(BarsPlotModel::from_series(vec![grouped, stacked]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-bars-panel",
        |cx| vec![bars_plot_panel(cx, BarsPlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let fill_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(19),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        fill_paths, 2,
        "declarative bars should emit one fill path per visible series"
    );

    let closed_bar_rects = services
        .prepared_paths
        .iter()
        .filter(|path| {
            path.iter()
                .filter(|cmd| matches!(cmd, PathCommand::Close))
                .count()
                >= 2
        })
        .count();
    assert_eq!(
        closed_bar_rects, 2,
        "grouped and stacked series should each close both bar rectangles"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn shaded_plot_panel_paints_band_fill_and_two_strokes_on_declarative_path() {
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
    let model = app.models_mut().insert(ShadedPlotModel::from_series(vec![
        crate::models::ShadedSeries::new(
            "Band",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.8 },
                    DataPoint { x: 1.0, y: 1.2 },
                    DataPoint { x: 2.0, y: 0.9 },
                ],
                true,
            ),
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.2 },
                    DataPoint { x: 1.0, y: 0.4 },
                    DataPoint { x: 2.0, y: 0.3 },
                ],
                true,
            ),
        )
        .fill_alpha(0.25),
    ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-shaded-panel",
        |cx| {
            vec![shaded_plot_panel(
                cx,
                ShadedPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let fill_paths = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Path {
                    order: DrawOrder(19),
                    ..
                }
            )
        })
        .count();
    let stroke_paths = scene
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
    assert_eq!(
        fill_paths, 1,
        "declarative shaded plot should emit one filled band path"
    );
    assert_eq!(
        stroke_paths, 2,
        "declarative shaded plot should emit upper and lower stroke paths"
    );
    assert!(
        services
            .prepared_paths
            .iter()
            .any(|path| path.iter().any(|cmd| matches!(cmd, PathCommand::Close))),
        "shaded fill path should close the upper/lower band"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn stems_plot_panel_paints_stems_from_baseline_on_declarative_path() {
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
    let model = app.models_mut().insert(StemsPlotModel::from_series(vec![
        StemsSeries::new(
            "Stems",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.2 },
                    DataPoint { x: 1.0, y: 0.8 },
                    DataPoint { x: 2.0, y: 0.4 },
                ],
                true,
            ),
        )
        .baseline(0.0),
    ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-stems-panel",
        |cx| {
            vec![stems_plot_panel(
                cx,
                StemsPlotPanelProps::new(model.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let stem_paths = scene
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
    assert_eq!(
        stem_paths, 1,
        "declarative stems plot should emit one stem path"
    );

    let stem_path = services
        .prepared_paths
        .iter()
        .find(|path| {
            path.windows(2).any(|commands| {
                matches!(
                    (&commands[0], &commands[1]),
                    (PathCommand::MoveTo(_), PathCommand::LineTo(_))
                )
            })
        })
        .expect("stems panel should prepare move/line stem commands");
    assert!(
        stem_path.len() >= 6,
        "three sampled stems should produce at least six path commands; got {stem_path:?}"
    );
    assert!(
        !stem_path
            .iter()
            .any(|cmd| matches!(cmd, PathCommand::Close)),
        "stems should be strokes from the baseline, not closed fills"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
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

#[test]
fn line_plot_panel_paints_right_axis_tick_labels_with_custom_formatters_on_declarative_path() {
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
    let left = LineSeries::new(
        "left",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 4.0 }],
            true,
        ),
    );
    let right = LineSeries::new(
        "right",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 100.0 }],
            true,
        ),
    )
    .y_axis(YAxis::Right);
    let right2 = LineSeries::new(
        "right2",
        Series::from_points_sorted(
            vec![
                DataPoint { x: 0.0, y: 0.0 },
                DataPoint { x: 4.0, y: 1_000.0 },
            ],
            true,
        ),
    )
    .y_axis(YAxis::Right2);
    let right3 = LineSeries::new(
        "right3",
        Series::from_points_sorted(
            vec![
                DataPoint { x: 0.0, y: 0.0 },
                DataPoint { x: 4.0, y: 2_000.0 },
            ],
            true,
        ),
    )
    .y_axis(YAxis::Right3);
    let model = app.models_mut().insert(LinePlotModel::from_series(vec![
        left, right, right2, right3,
    ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-right-axis-labels",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .y2_axis_labels(AxisLabelFormatter::custom(0x5231u64, |v, _span| {
                        format!("R1:{v:.0}")
                    }))
                    .y3_axis_labels(AxisLabelFormatter::custom(0x5232u64, |v, _span| {
                        format!("R2:{v:.0}")
                    }))
                    .y4_axis_labels(AxisLabelFormatter::custom(0x5233u64, |v, _span| {
                        format!("R3:{v:.0}")
                    })),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        services
            .prepared_text
            .iter()
            .any(|text| text.starts_with("R1:")),
        "declarative line plot should use the y2 formatter for right-axis tick labels, got {:?}",
        services.prepared_text
    );
    assert!(
        services
            .prepared_text
            .iter()
            .any(|text| text.starts_with("R2:")),
        "declarative line plot should use the y3 formatter for right2-axis tick labels, got {:?}",
        services.prepared_text
    );
    assert!(
        services
            .prepared_text
            .iter()
            .any(|text| text.starts_with("R3:")),
        "declarative line plot should use the y4 formatter for right3-axis tick labels, got {:?}",
        services.prepared_text
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn line_plot_panel_paints_series_legend_on_declarative_path() {
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
    let series = vec![
        LineSeries::new(
            "Alpha",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 1.0 },
                    DataPoint { x: 1.0, y: 2.0 },
                    DataPoint { x: 2.0, y: 1.5 },
                ],
                true,
            ),
        ),
        LineSeries::new(
            "Beta",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.5 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 2.5 },
                ],
                true,
            ),
        ),
    ];
    let model = app.models_mut().insert(LinePlotModel::from_series(series));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-legend",
        |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let legend_swatches = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(30),
                    ..
                }
            )
        })
        .count();
    assert!(
        legend_swatches >= 2,
        "declarative line plot should paint one legend swatch per series"
    );

    let legend_labels = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Text {
                    order: DrawOrder(31),
                    ..
                }
            )
        })
        .count();
    assert!(
        legend_labels >= 2,
        "declarative line plot should paint one legend label per series"
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
    assert_eq!(
        series_paths, 2,
        "legend painting should not replace seeded series paths"
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn line_plot_panel_paints_right_axis_series_with_right_axis_bounds_on_declarative_path() {
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
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 1.0 }],
            true,
        ),
    );
    let right_series = LineSeries::new(
        "right",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 100.0 }],
            true,
        ),
    )
    .y_axis(YAxis::Right);
    let model = app
        .models_mut()
        .insert(LinePlotModel::from_series(vec![left_series, right_series]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-right-axis-line-panel",
        |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    services.prepared_paths.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let plot = line_plot_inner_rect(bounds, LinePlotStyle::default());
    let right_path = services
        .prepared_paths
        .iter()
        .find(|commands| {
            commands.iter().any(|command| match command {
                PathCommand::LineTo(point) => (point.y.0 - plot.origin.y.0).abs() < 0.5,
                _ => false,
            })
        })
        .cloned();
    assert!(
        right_path.is_some(),
        "declarative right-axis series should use right-axis y bounds and reach the plot top; paths={:?}",
        services.prepared_paths
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn line_plot_panel_paints_right2_and_right3_axis_series_with_axis_bounds_on_declarative_path() {
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
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 1.0 }],
            true,
        ),
    );
    let right2_series = LineSeries::new(
        "right2",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 200.0 }],
            true,
        ),
    )
    .y_axis(YAxis::Right2);
    let right3_series = LineSeries::new(
        "right3",
        Series::from_points_sorted(
            vec![
                DataPoint { x: 0.0, y: 0.0 },
                DataPoint { x: 1.0, y: 3000.0 },
            ],
            true,
        ),
    )
    .y_axis(YAxis::Right3);
    let model = app.models_mut().insert(LinePlotModel::from_series(vec![
        left_series,
        right2_series,
        right3_series,
    ]));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-right23-axis-line-panel",
        |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    services.prepared_paths.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let endpoint_y: Vec<f32> = services
        .prepared_paths
        .iter()
        .filter_map(|commands| {
            commands.iter().find_map(|command| match command {
                PathCommand::LineTo(point) => Some(point.y.0),
                _ => None,
            })
        })
        .collect();
    assert_eq!(
        endpoint_y.len(),
        3,
        "left, right2, and right3 series should each emit a line endpoint; paths={:?}",
        services.prepared_paths
    );
    let right2_endpoint_y = endpoint_y[1];
    assert_eq!(
        endpoint_y
            .iter()
            .skip(1)
            .filter(|y| (**y - right2_endpoint_y).abs() < 0.5)
            .count(),
        2,
        "right2 and right3 series should project their max y values to the same plot-space endpoint through their own y bounds; endpoint_y={endpoint_y:?}, paths={:?}",
        services.prepared_paths
    );

    app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
}

#[test]
fn line_plot_panel_legend_swatch_click_toggles_series_visibility_on_declarative_path() {
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
    let series = vec![
        LineSeries::new(
            "Alpha",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 1.0 },
                    DataPoint { x: 1.0, y: 2.0 },
                    DataPoint { x: 2.0, y: 1.5 },
                ],
                true,
            ),
        ),
        LineSeries::new(
            "Beta",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.5 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 2.5 },
                ],
                true,
            ),
        ),
    ];
    let alpha_id = series[0].id;
    let model = app.models_mut().insert(LinePlotModel::from_series(series));
    let state = app.models_mut().insert(PlotState::default());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-legend-toggle",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).state(state.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
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
    assert_eq!(series_paths, 2);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(42.0), Px(32.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let hidden = state
        .read_ref(&app, |state| state.hidden_series.clone())
        .expect("plot state should be readable");
    assert!(
        hidden.contains(&alpha_id),
        "clicking a declarative legend swatch should hide that series"
    );

    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
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
    assert_eq!(
        series_paths, 1,
        "hidden declarative legend series should be omitted from line painting"
    );
}

#[test]
fn line_plot_panel_legend_label_click_pins_and_unpins_series_on_declarative_path() {
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
    let series = vec![
        LineSeries::new(
            "Alpha",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 1.0 },
                    DataPoint { x: 1.0, y: 2.0 },
                    DataPoint { x: 2.0, y: 1.5 },
                ],
                true,
            ),
        ),
        LineSeries::new(
            "Beta",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.5 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 2.5 },
                ],
                true,
            ),
        ),
    ];
    let beta_id = series[1].id;
    let model = app.models_mut().insert(LinePlotModel::from_series(series));
    let state = app.models_mut().insert(PlotState::default());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-legend-pin",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).state(state.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(64.0), Px(48.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let pinned = state
        .read_ref(&app, |state| state.pinned_series)
        .expect("plot state should be readable");
    assert_eq!(
        pinned,
        Some(beta_id),
        "clicking a declarative legend label should pin that series"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(169.0), Px(81.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    services.prepared_text.clear();
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("Beta: y="),
        "pinned declarative legend series should be kept in cursor readout rows: {prepared_text:?}"
    );
    assert!(
        !prepared_text.contains("Alpha: y="),
        "pinning Beta should filter other declarative cursor readout rows: {prepared_text:?}"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(64.0), Px(48.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let pinned = state
        .read_ref(&app, |state| state.pinned_series)
        .expect("plot state should be readable");
    assert_eq!(
        pinned, None,
        "clicking a pinned declarative legend label should unpin it"
    );

    services.prepared_text.clear();
    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("Alpha: y=") && prepared_text.contains("Beta: y="),
        "unpinning should restore all visible declarative cursor readout rows: {prepared_text:?}"
    );
}

#[test]
fn line_plot_panel_legend_shift_click_solos_and_restores_series_on_declarative_path() {
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
    let series = vec![
        LineSeries::new(
            "Alpha",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 1.0 },
                    DataPoint { x: 1.0, y: 2.0 },
                    DataPoint { x: 2.0, y: 1.5 },
                ],
                true,
            ),
        ),
        LineSeries::new(
            "Beta",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.5 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 2.5 },
                ],
                true,
            ),
        ),
        LineSeries::new(
            "Gamma",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 1.5 },
                    DataPoint { x: 1.0, y: 1.25 },
                    DataPoint { x: 2.0, y: 0.75 },
                ],
                true,
            ),
        ),
    ];
    let alpha_id = series[0].id;
    let beta_id = series[1].id;
    let gamma_id = series[2].id;
    let model = app.models_mut().insert(LinePlotModel::from_series(series));
    let state = app.models_mut().insert(PlotState::default());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-legend-solo",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).state(state.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(42.0), Px(48.0)),
            button: MouseButton::Left,
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let hidden = state
        .read_ref(&app, |state| state.hidden_series.clone())
        .expect("plot state should be readable");
    assert!(
        hidden.contains(&alpha_id) && hidden.contains(&gamma_id) && !hidden.contains(&beta_id),
        "shift-clicking a declarative legend row should solo that series"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
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
    assert_eq!(
        series_paths, 1,
        "soloed declarative legend series should be the only painted line"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(42.0), Px(48.0)),
            button: MouseButton::Left,
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let hidden = state
        .read_ref(&app, |state| state.hidden_series.clone())
        .expect("plot state should be readable");
    assert!(
        hidden.is_empty(),
        "shift-clicking an already-solo declarative legend row should restore all series"
    );

    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
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
    assert_eq!(
        series_paths, 3,
        "restoring declarative legend solo mode should paint every line series again"
    );
}

#[test]
fn line_plot_panel_legend_hover_emphasizes_series_on_declarative_path() {
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
    let alpha_color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let beta_color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    let model = app.models_mut().insert(LinePlotModel::from_series(vec![
        LineSeries::new(
            "Alpha",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 1.0 },
                    DataPoint { x: 1.0, y: 2.0 },
                    DataPoint { x: 2.0, y: 1.5 },
                ],
                true,
            ),
        )
        .color(alpha_color),
        LineSeries::new(
            "Beta",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.5 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 2.5 },
                ],
                true,
            ),
        )
        .color(beta_color),
    ]));
    let state = app.models_mut().insert(PlotState::default());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-legend-hover",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).state(state.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(64.0), Px(32.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let legend_highlights = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(29),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        legend_highlights, 1,
        "hovering a declarative legend row should paint a legend highlight"
    );

    let mut alpha_path_alpha = None;
    let mut beta_path_alpha = None;
    for op in scene.ops() {
        let fret_core::SceneOp::Path {
            order: DrawOrder(20),
            paint,
            ..
        } = op
        else {
            continue;
        };
        if let Paint::Solid(color) = paint.paint {
            if (color.r - alpha_color.r).abs() < 0.001
                && (color.g - alpha_color.g).abs() < 0.001
                && (color.b - alpha_color.b).abs() < 0.001
            {
                alpha_path_alpha = Some(color.a);
            } else if (color.g - beta_color.g).abs() < 0.001 {
                beta_path_alpha = Some(color.a);
            }
        }
    }

    assert_eq!(
        alpha_path_alpha,
        Some(1.0),
        "hovered declarative legend series should keep full opacity"
    );
    assert!(
        beta_path_alpha.is_some_and(|alpha| alpha < 0.5),
        "non-hovered declarative line series should be dimmed while a legend row is hovered"
    );
}

#[test]
fn line_plot_panel_uses_controlled_view_bounds_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    let state = app.models_mut().insert(plot_state);
    let output = app.models_mut().insert(PlotOutput::default());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-controlled-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .output(output.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(169.0), Px(81.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let published = output
        .read_ref(&app, |output| *output)
        .expect("plot output model should be readable");
    assert_eq!(
        published.snapshot.view_bounds,
        DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        },
        "declarative line plot output should publish caller-controlled view bounds"
    );
    let cursor = published
        .snapshot
        .cursor
        .expect("pointer inside the controlled plot region should publish cursor data");
    assert!(
        (cursor.x - 2.0).abs() < 0.04,
        "expected pointer x to map through controlled view bounds, got {:?}",
        cursor
    );
    assert!(
        (cursor.y - 2.0).abs() < 0.08,
        "expected pointer y to map through controlled view bounds, got {:?}",
        cursor
    );
}

#[test]
fn line_plot_panel_pans_controlled_view_bounds_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    let state = app.models_mut().insert(plot_state);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-pan-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).state(state.clone()),
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
            position: Point::new(Px(189.0), Px(81.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(189.0), Px(81.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let updated = state
        .read_ref(&app, |state| (state.view_is_auto, state.view_bounds))
        .expect("plot state should be readable");
    let view = updated
        .1
        .expect("declarative panning should leave an explicit view bounds");
    assert!(
        !updated.0,
        "declarative panning should switch/keep plot view in controlled mode"
    );
    assert!(
        view.x_min < -0.20 && view.x_max < 3.80,
        "dragging right should pan the declarative view left in data space, got {view:?}"
    );
    assert!(
        (view.y_min - 0.0).abs() < 0.001 && (view.y_max - 4.0).abs() < 0.001,
        "horizontal pan should preserve y range, got {view:?}"
    );
}

#[test]
fn line_plot_panel_pan_respects_x_pan_lock_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    plot_state.axis_locks.x.pan = true;
    let state = app.models_mut().insert(plot_state);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-pan-x-lock-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).state(state.clone()),
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
            position: Point::new(Px(189.0), Px(101.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative panning should leave an explicit view bounds");
    assert!(
        (view.x_min - 0.0).abs() < 0.001 && (view.x_max - 4.0).abs() < 0.001,
        "X pan lock should preserve the declarative X range, got {view:?}"
    );
    assert!(
        view.y_min > 0.2 && view.y_max > 4.2,
        "X pan lock should still allow declarative Y panning, got {view:?}"
    );
}

#[test]
fn line_plot_panel_pan_respects_y_pan_lock_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    plot_state.axis_locks.y.pan = true;
    let state = app.models_mut().insert(plot_state);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-pan-y-lock-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).state(state.clone()),
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
            position: Point::new(Px(189.0), Px(101.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative panning should leave an explicit view bounds");
    assert!(
        view.x_min < -0.20 && view.x_max < 3.80,
        "Y pan lock should still allow declarative X panning, got {view:?}"
    );
    assert!(
        (view.y_min - 0.0).abs() < 0.001 && (view.y_max - 4.0).abs() < 0.001,
        "Y pan lock should preserve the declarative Y range, got {view:?}"
    );
}

#[test]
fn line_plot_panel_pan_noops_when_both_axes_locked_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    plot_state.axis_locks.x.pan = true;
    plot_state.axis_locks.y.pan = true;
    let state = app.models_mut().insert(plot_state);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-pan-both-lock-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).state(state.clone()),
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
            position: Point::new(Px(189.0), Px(101.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative panning should preserve explicit view bounds");
    assert_eq!(
        view,
        DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        },
        "panning should not change declarative view bounds when both axes are pan-locked"
    );
}

#[test]
fn line_plot_panel_box_zoom_updates_controlled_view_bounds_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-box-zoom-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
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
            position: Point::new(Px(100.0), Px(50.0)),
            button: MouseButton::Right,
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
            position: Point::new(Px(200.0), Px(120.0)),
            buttons: MouseButtons {
                right: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(200.0), Px(120.0)),
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let updated = state
        .read_ref(&app, |state| (state.view_is_auto, state.view_bounds))
        .expect("plot state should be readable");
    let view = updated
        .1
        .expect("declarative box zoom should leave an explicit view bounds");
    assert!(
        !updated.0,
        "declarative box zoom should switch/keep plot view in controlled mode"
    );
    assert!(
        view.x_min > 0.9 && view.x_max < 2.6,
        "right-button box zoom should narrow the declarative X range, got {view:?}"
    );
    assert!(
        view.y_min > 0.8 && view.y_max < 3.0,
        "right-button box zoom should narrow the declarative Y range, got {view:?}"
    );
}

#[test]
fn line_plot_panel_query_drag_updates_query_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-query-drag",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let alt = Modifiers {
        alt: true,
        ..Modifiers::default()
    };
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(100.0), Px(50.0)),
            button: MouseButton::Left,
            modifiers: alt,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(200.0), Px(120.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: alt,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(200.0), Px(120.0)),
            button: MouseButton::Left,
            modifiers: alt,
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let query = state
        .read_ref(&app, |state| state.query)
        .expect("plot state should be readable")
        .expect("declarative query drag should write a query rect");
    assert!(
        query.x_min > 0.9 && query.x_max < 2.6,
        "Alt+left query drag should map the selected X range into data space, got {query:?}"
    );
    assert!(
        query.y_min > 0.8 && query.y_max < 3.1,
        "Alt+left query drag should map the selected Y range into data space, got {query:?}"
    );
}

#[test]
fn line_plot_panel_query_drag_updates_output_query_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
        "plot-declarative-query-output",
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

    let alt = Modifiers {
        alt: true,
        ..Modifiers::default()
    };
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(100.0), Px(50.0)),
            button: MouseButton::Left,
            modifiers: alt,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(200.0), Px(120.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: alt,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(200.0), Px(120.0)),
            button: MouseButton::Left,
            modifiers: alt,
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let output_snapshot = output
        .read_ref(&app, |output| output.snapshot)
        .expect("plot output should be readable");
    let query = output_snapshot
        .query
        .expect("declarative query drag should publish query output");
    assert!(
        query.x_min > 0.9 && query.x_max < 2.6,
        "query output should include the selected X data range, got {query:?}"
    );
    assert!(
        query.y_min > 0.8 && query.y_max < 3.1,
        "query output should include the selected Y data range, got {query:?}"
    );
    assert_eq!(
        output_snapshot.view_bounds,
        DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        },
        "query output should keep reporting the current declarative view bounds"
    );
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

#[test]
fn line_plot_panel_paints_query_selection_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-query-selection",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let alt = Modifiers {
        alt: true,
        ..Modifiers::default()
    };
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(100.0), Px(50.0)),
            button: MouseButton::Left,
            modifiers: alt,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(200.0), Px(120.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: alt,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut active_scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut active_scene, 1.0);
    let active_rects = line_plot_selection_rects(&active_scene);
    assert_eq!(
        active_rects.len(),
        1,
        "active declarative query drag should paint one selection rectangle"
    );
    assert_line_plot_selection_rect(active_rects[0], 100.0, 50.0, 100.0, 70.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(200.0), Px(120.0)),
            button: MouseButton::Left,
            modifiers: alt,
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut persisted_scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut persisted_scene, 1.0);
    let persisted_rects = line_plot_selection_rects(&persisted_scene);
    assert_eq!(
        persisted_rects.len(),
        1,
        "persisted declarative query state should paint one selection rectangle"
    );
    assert_line_plot_selection_rect(persisted_rects[0], 100.0, 50.0, 100.0, 70.0);
}

#[test]
fn line_plot_panel_paints_box_zoom_selection_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-box-selection",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
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
            position: Point::new(Px(100.0), Px(50.0)),
            button: MouseButton::Right,
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
            position: Point::new(Px(200.0), Px(120.0)),
            buttons: MouseButtons {
                right: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut active_scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut active_scene, 1.0);
    let active_rects = line_plot_selection_rects(&active_scene);
    assert_eq!(
        active_rects.len(),
        1,
        "active declarative box zoom should paint one selection rectangle"
    );
    assert_line_plot_selection_rect(active_rects[0], 100.0, 50.0, 100.0, 70.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(200.0), Px(120.0)),
            button: MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut released_scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut released_scene, 1.0);
    assert!(
        line_plot_selection_rects(&released_scene).is_empty(),
        "box zoom selection rectangle should clear after applying the view change"
    );
}

#[test]
fn line_plot_panel_paints_query_selection_tooltip_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-query-tooltip",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    services.prepared_text.clear();

    let alt = Modifiers {
        alt: true,
        ..Modifiers::default()
    };
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(100.0), Px(50.0)),
            button: MouseButton::Left,
            modifiers: alt,
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(200.0), Px(120.0)),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: alt,
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("query\nx=["),
        "declarative query drag should paint a query selection tooltip, got {prepared_text:?}"
    );
    assert!(
        prepared_text.contains("y=["),
        "declarative query selection tooltip should include y-range text, got {prepared_text:?}"
    );
}

#[test]
fn line_plot_panel_paints_box_zoom_selection_tooltip_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-box-tooltip",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    services.prepared_text.clear();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(100.0), Px(50.0)),
            button: MouseButton::Right,
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
            position: Point::new(Px(200.0), Px(120.0)),
            buttons: MouseButtons {
                right: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("zoom\nx=["),
        "declarative box zoom should paint a zoom selection tooltip, got {prepared_text:?}"
    );
    assert!(
        prepared_text.contains("y=["),
        "declarative box zoom tooltip should include y-range text, got {prepared_text:?}"
    );
}

#[test]
fn line_plot_panel_paints_reference_lines_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    plot_state.overlays.inf_lines_x.push(InfLineX::new(2.0));
    plot_state
        .overlays
        .inf_lines_y
        .push(InfLineY::new(1.0, YAxis::Left));
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-reference-lines",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let reference_lines = line_plot_reference_line_rects(&scene);
    assert!(
        reference_lines.iter().any(|rect| {
            (rect.origin.x.0 - 169.0).abs() < 0.01
                && (rect.origin.y.0 - 8.0).abs() < 0.01
                && (rect.size.width.0 - 1.0).abs() < 0.01
                && (rect.size.height.0 - 146.0).abs() < 0.01
        }),
        "declarative line plot should paint caller-owned X reference line, got {reference_lines:?}"
    );
    assert!(
        reference_lines.iter().any(|rect| {
            (rect.origin.x.0 - 26.0).abs() < 0.01
                && (rect.origin.y.0 - 117.0).abs() < 0.01
                && (rect.size.width.0 - 286.0).abs() < 0.01
                && (rect.size.height.0 - 1.0).abs() < 0.01
        }),
        "declarative line plot should paint caller-owned Y reference line, got {reference_lines:?}"
    );
}

#[test]
fn line_plot_panel_paints_draggable_lines_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
        .push(DragLineX::new(10, 2.0));
    plot_state
        .overlays
        .drag_lines_y
        .push(DragLineY::new(11, 1.0, YAxis::Left));
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-draggable-lines",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let draggable_lines = line_plot_reference_line_rects(&scene);
    assert!(
        draggable_lines.iter().any(|rect| {
            (rect.origin.x.0 - 169.0).abs() < 0.01
                && (rect.origin.y.0 - 8.0).abs() < 0.01
                && (rect.size.width.0 - 1.0).abs() < 0.01
                && (rect.size.height.0 - 146.0).abs() < 0.01
        }),
        "declarative line plot should paint caller-owned draggable X line, got {draggable_lines:?}"
    );
    assert!(
        draggable_lines.iter().any(|rect| {
            (rect.origin.x.0 - 26.0).abs() < 0.01
                && (rect.origin.y.0 - 117.0).abs() < 0.01
                && (rect.size.width.0 - 286.0).abs() < 0.01
                && (rect.size.height.0 - 1.0).abs() < 0.01
        }),
        "declarative line plot should paint caller-owned draggable Y line, got {draggable_lines:?}"
    );
}

#[test]
fn line_plot_panel_paints_draggable_point_and_rect_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    plot_state.overlays.drag_points.push(DragPoint::new(
        20,
        DataPoint { x: 2.0, y: 1.0 },
        YAxis::Left,
    ));
    plot_state.overlays.drag_rects.push(DragRect::new(
        21,
        DataRect {
            x_min: 1.0,
            x_max: 3.0,
            y_min: 1.0,
            y_max: 3.0,
        },
        YAxis::Left,
    ));
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-draggable-point-rect",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let draggable_shapes = line_plot_reference_line_rects(&scene);
    assert!(
        draggable_shapes.iter().any(|rect| {
            (rect.origin.x.0 - 165.0).abs() < 0.01
                && (rect.origin.y.0 - 114.0).abs() < 0.01
                && (rect.size.width.0 - 8.0).abs() < 0.01
                && (rect.size.height.0 - 8.0).abs() < 0.01
        }),
        "declarative line plot should paint caller-owned draggable point, got {draggable_shapes:?}"
    );
    assert!(
        draggable_shapes.iter().any(|rect| {
            (rect.origin.x.0 - 98.0).abs() < 0.01
                && (rect.origin.y.0 - 45.0).abs() < 0.01
                && (rect.size.width.0 - 143.0).abs() < 0.01
                && (rect.size.height.0 - 73.0).abs() < 0.01
        }),
        "declarative line plot should paint caller-owned draggable rect, got {draggable_shapes:?}"
    );
}

#[test]
fn line_plot_panel_paints_right_axis_draggable_shapes_on_declarative_path() {
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
    plot_state.overlays.drag_points.push(DragPoint::new(
        51,
        DataPoint { x: 2.0, y: 50.0 },
        YAxis::Right,
    ));
    plot_state.overlays.drag_rects.push(DragRect::new(
        52,
        DataRect {
            x_min: 1.0,
            x_max: 3.0,
            y_min: 25.0,
            y_max: 75.0,
        },
        YAxis::Right,
    ));
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-right-axis-draggable-shapes",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let draggable_shapes = line_plot_reference_line_rects(&scene);
    assert!(
        draggable_shapes.iter().any(|rect| {
            (rect.origin.x.0 - 26.0).abs() < 0.01
                && (rect.origin.y.0 - 8.0).abs() < 0.01
                && (rect.size.width.0 - 286.0).abs() < 0.01
                && (rect.size.height.0 - 1.0).abs() < 0.01
        }),
        "declarative line plot should paint right-axis draggable Y line, got {draggable_shapes:?}"
    );
    assert!(
        draggable_shapes.iter().any(|rect| {
            (rect.origin.x.0 - 165.0).abs() < 0.01
                && (rect.origin.y.0 - 77.0).abs() < 0.01
                && (rect.size.width.0 - 8.0).abs() < 0.01
                && (rect.size.height.0 - 8.0).abs() < 0.01
        }),
        "declarative line plot should paint right-axis draggable point, got {draggable_shapes:?}"
    );
    assert!(
        draggable_shapes.iter().any(|rect| {
            (rect.origin.x.0 - 98.0).abs() < 0.01
                && (rect.origin.y.0 - 45.0).abs() < 0.01
                && (rect.size.width.0 - 143.0).abs() < 0.01
                && (rect.size.height.0 - 73.0).abs() < 0.01
        }),
        "declarative line plot should paint right-axis draggable rect, got {draggable_shapes:?}"
    );
}

#[test]
fn line_plot_panel_paints_plot_text_overlay_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    plot_state.overlays.text.push(
        PlotText::new(2.0, 1.0, YAxis::Left, "threshold note")
            .background(Color::from_srgb_hex_rgb(0x19_33_4c))
            .padding(Px(4.0))
            .offset(Point::new(Px(4.0), Px(-6.0))),
    );
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-plot-text-overlay",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    services.prepared_text.clear();
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("threshold note"),
        "declarative line plot should prepare caller-owned PlotText overlay text, got {prepared_text:?}"
    );

    let text_backgrounds = line_plot_reference_line_rects(&scene);
    assert!(
        text_backgrounds.iter().any(|rect| {
            (rect.origin.x.0 - 173.0).abs() < 0.01
                && (rect.origin.y.0 - 112.0).abs() < 0.01
                && (rect.size.width.0 - 8.0).abs() < 0.01
                && (rect.size.height.0 - 8.0).abs() < 0.01
        }),
        "declarative line plot should paint retained-compatible PlotText background, got {text_backgrounds:?}"
    );
}

#[test]
fn line_plot_panel_paints_tag_x_and_y_overlays_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
        .tags_x
        .push(TagX::new(2.0).label("X Gate").show_value(false));
    plot_state.overlays.tags_y.push(
        TagY::new(1.0, YAxis::Left)
            .label("Y Gate")
            .show_value(false),
    );
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-tags",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    services.prepared_text.clear();
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("X Gate"),
        "declarative line plot should prepare caller-owned TagX text, got {prepared_text:?}"
    );
    assert!(
        prepared_text.contains("Y Gate"),
        "declarative line plot should prepare caller-owned TagY text, got {prepared_text:?}"
    );

    let tag_rects = line_plot_reference_line_rects(&scene);
    assert!(
        tag_rects.iter().any(|rect| {
            (rect.origin.x.0 - 168.0).abs() < 0.01
                && (rect.origin.y.0 - 146.0).abs() < 0.01
                && (rect.size.width.0 - 2.0).abs() < 0.01
                && (rect.size.height.0 - 8.0).abs() < 0.01
        }),
        "declarative line plot should paint retained-compatible TagX marker, got {tag_rects:?}"
    );
    assert!(
        tag_rects.iter().any(|rect| {
            (rect.origin.x.0 - 26.0).abs() < 0.01
                && (rect.origin.y.0 - 117.0).abs() < 0.01
                && (rect.size.width.0 - 8.0).abs() < 0.01
                && (rect.size.height.0 - 2.0).abs() < 0.01
        }),
        "declarative line plot should paint retained-compatible left-axis TagY marker, got {tag_rects:?}"
    );
}

#[test]
fn line_plot_panel_paints_draggable_overlay_labels_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
        .push(DragLineX::new(30, 2.0).label("X Drag").show_value(false));
    plot_state.overlays.drag_lines_y.push(
        DragLineY::new(31, 1.0, YAxis::Left)
            .label("Y Drag")
            .show_value(false),
    );
    plot_state
        .overlays
        .drag_points
        .push(DragPoint::new(32, DataPoint { x: 2.0, y: 1.0 }, YAxis::Left).label("Point Drag"));
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-drag-labels",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    services.prepared_text.clear();
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("X Drag"),
        "declarative line plot should prepare draggable X-line label text, got {prepared_text:?}"
    );
    assert!(
        prepared_text.contains("Y Drag"),
        "declarative line plot should prepare draggable Y-line label text, got {prepared_text:?}"
    );
    assert!(
        prepared_text.contains("Point Drag"),
        "declarative line plot should prepare draggable point label text, got {prepared_text:?}"
    );

    let label_rects = line_plot_reference_line_rects(&scene);
    assert!(
        label_rects.iter().any(|rect| {
            (rect.origin.x.0 - 168.0).abs() < 0.01
                && (rect.origin.y.0 - 146.0).abs() < 0.01
                && (rect.size.width.0 - 2.0).abs() < 0.01
                && (rect.size.height.0 - 8.0).abs() < 0.01
        }),
        "declarative line plot should paint retained-compatible draggable X-line label marker, got {label_rects:?}"
    );
    assert!(
        label_rects.iter().any(|rect| {
            (rect.origin.x.0 - 26.0).abs() < 0.01
                && (rect.origin.y.0 - 117.0).abs() < 0.01
                && (rect.size.width.0 - 8.0).abs() < 0.01
                && (rect.size.height.0 - 2.0).abs() < 0.01
        }),
        "declarative line plot should paint retained-compatible draggable Y-line label marker, got {label_rects:?}"
    );
}

#[test]
fn line_plot_panel_paints_right_axis_draggable_overlay_labels_on_declarative_path() {
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
    plot_state.overlays.drag_lines_y.push(
        DragLineY::new(40, 100.0, YAxis::Right)
            .label("Right Y Drag")
            .show_value(false),
    );
    plot_state.overlays.drag_points.push(
        DragPoint::new(41, DataPoint { x: 2.0, y: 50.0 }, YAxis::Right).label("Right Point Drag"),
    );
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-right-axis-drag-labels",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    services.prepared_text.clear();
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let prepared_text = services.prepared_text.join("\n");
    assert!(
        prepared_text.contains("Right Y Drag"),
        "declarative line plot should prepare right-axis draggable Y-line label text, got {prepared_text:?}"
    );
    assert!(
        prepared_text.contains("Right Point Drag"),
        "declarative line plot should prepare right-axis draggable point label text, got {prepared_text:?}"
    );

    let label_rects = line_plot_reference_line_rects(&scene);
    assert!(
        label_rects.iter().any(|rect| {
            (rect.origin.x.0 - 304.0).abs() < 0.01
                && (rect.origin.y.0 - 8.0).abs() < 0.01
                && (rect.size.width.0 - 8.0).abs() < 0.01
                && (rect.size.height.0 - 2.0).abs() < 0.01
        }),
        "declarative line plot should paint retained-compatible right-axis draggable Y-line label marker, got {label_rects:?}"
    );
}

#[test]
fn line_plot_panel_paints_plot_image_overlay_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    let uv = UvRect {
        u0: 0.25,
        v0: 0.10,
        u1: 0.75,
        v1: 0.90,
    };
    plot_state.overlays.images.push(
        PlotImage::new(
            ImageId::default(),
            DataRect {
                x_min: 1.0,
                x_max: 3.0,
                y_min: 1.0,
                y_max: 3.0,
            },
            YAxis::Left,
        )
        .uv(uv)
        .opacity(0.5),
    );
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-image-overlay",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let image_regions = line_plot_image_regions(&scene);
    assert!(
        image_regions.iter().any(|(rect, found_uv, opacity)| {
            (rect.origin.x.0 - 97.5).abs() < 0.01
                && (rect.origin.y.0 - 44.5).abs() < 0.01
                && (rect.size.width.0 - 143.0).abs() < 0.01
                && (rect.size.height.0 - 73.0).abs() < 0.01
                && *found_uv == uv
                && (*opacity - 0.5).abs() < 0.01
        }),
        "declarative line plot should paint caller-owned PlotImage overlay, got {image_regions:?}"
    );
}

#[test]
fn line_plot_panel_paints_right_axis_plot_image_overlays_on_declarative_path() {
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
    let right2_series = LineSeries::new(
        "right2",
        Series::from_points_sorted(
            vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 4.0, y: 200.0 }],
            true,
        ),
    )
    .y_axis(YAxis::Right2);
    let right3_series = LineSeries::new(
        "right3",
        Series::from_points_sorted(
            vec![
                DataPoint { x: 0.0, y: 0.0 },
                DataPoint { x: 4.0, y: 3000.0 },
            ],
            true,
        ),
    )
    .y_axis(YAxis::Right3);
    let model = app.models_mut().insert(LinePlotModel::from_series(vec![
        left_series,
        right2_series,
        right3_series,
    ]));
    let mut plot_state = PlotState::default();
    plot_state.view_is_auto = false;
    plot_state.view_bounds = Some(DataRect {
        x_min: 0.0,
        x_max: 4.0,
        y_min: 0.0,
        y_max: 4.0,
    });
    plot_state.overlays.images.push(
        PlotImage::new(
            ImageId::default(),
            DataRect {
                x_min: 1.0,
                x_max: 3.0,
                y_min: 0.0,
                y_max: 200.0,
            },
            YAxis::Right2,
        )
        .opacity(0.42),
    );
    plot_state.overlays.images.push(
        PlotImage::new(
            ImageId::default(),
            DataRect {
                x_min: 1.0,
                x_max: 3.0,
                y_min: 0.0,
                y_max: 3000.0,
            },
            YAxis::Right3,
        )
        .opacity(0.43),
    );
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-right-axis-image-overlays",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let image_regions = line_plot_image_regions(&scene);
    for expected_opacity in [0.42, 0.43] {
        assert!(
            image_regions.iter().any(|(rect, _uv, opacity)| {
                (rect.origin.x.0 - 97.5).abs() < 0.01
                    && (rect.origin.y.0 - 8.0).abs() < 0.01
                    && (rect.size.width.0 - 143.0).abs() < 0.01
                    && (rect.size.height.0 - 146.0).abs() < 0.01
                    && (*opacity - expected_opacity).abs() < 0.01
            }),
            "declarative line plot should paint right-axis PlotImage overlay with opacity {expected_opacity}, got {image_regions:?}"
        );
    }
}

#[test]
fn line_plot_panel_paints_right_axis_tag_y_and_plot_text_overlays_on_declarative_path() {
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
    plot_state.overlays.tags_y.push(
        TagY::new(100.0, YAxis::Right)
            .label("threshold")
            .show_value(true),
    );
    plot_state.overlays.text.push(
        PlotText::new(2.0, 50.0, YAxis::Right, "right-axis note")
            .background(Color::from_srgb_hex_rgb(0x0A141E)),
    );
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-right-axis-tagy-text",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let tag_y_quads = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(3),
                    ..
                }
            )
        })
        .count();
    let tag_y_texts = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Text {
                    order: DrawOrder(3),
                    ..
                }
            )
        })
        .count();
    assert!(
        tag_y_quads >= 2 && tag_y_texts >= 2,
        "declarative line plot should paint right-axis TagY and PlotText overlays"
    );
}
#[test]
fn line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-wheel-zoom-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(169.0), Px(81.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let updated = state
        .read_ref(&app, |state| (state.view_is_auto, state.view_bounds))
        .expect("plot state should be readable");
    let view = updated
        .1
        .expect("declarative wheel zoom should leave an explicit view bounds");
    assert!(
        !updated.0,
        "declarative wheel zoom should switch/keep plot view in controlled mode"
    );
    assert!(
        view.x_max - view.x_min < 4.0 && view.y_max - view.y_min < 4.0,
        "positive wheel delta should zoom the declarative view in around the pointer, got {view:?}"
    );
    assert!(
        view.x_min > 0.0 && view.x_max < 4.0 && view.y_min > 0.0 && view.y_max < 4.0,
        "center wheel zoom should keep the next view inside the previous bounds, got {view:?}"
    );
}

#[test]
fn line_plot_panel_wheel_zoom_shift_modifier_zooms_x_only_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-wheel-x-only-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(169.0), Px(81.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative wheel zoom should leave an explicit view bounds");
    assert!(
        view.x_max - view.x_min < 4.0,
        "Shift+wheel should zoom the declarative X range, got {view:?}"
    );
    assert!(
        (view.y_min - 0.0).abs() < 0.001 && (view.y_max - 4.0).abs() < 0.001,
        "Shift+wheel should preserve the declarative Y range, got {view:?}"
    );
}

#[test]
fn line_plot_panel_wheel_zoom_ctrl_modifier_zooms_y_only_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-wheel-y-only-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(169.0), Px(81.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative wheel zoom should leave an explicit view bounds");
    assert!(
        (view.x_min - 0.0).abs() < 0.001 && (view.x_max - 4.0).abs() < 0.001,
        "Ctrl+wheel should preserve the declarative X range, got {view:?}"
    );
    assert!(
        view.y_max - view.y_min < 4.0,
        "Ctrl+wheel should zoom the declarative Y range, got {view:?}"
    );
}

#[test]
fn line_plot_panel_wheel_zoom_on_x_axis_zooms_x_only_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-wheel-x-axis-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(169.0), Px(163.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative x-axis wheel zoom should leave an explicit view bounds");
    assert!(
        view.x_max - view.x_min < 4.0,
        "wheel over the declarative X axis should zoom the X range, got {view:?}"
    );
    assert!(
        (view.y_min - 0.0).abs() < 0.001 && (view.y_max - 4.0).abs() < 0.001,
        "wheel over the declarative X axis should preserve the Y range, got {view:?}"
    );
}

#[test]
fn line_plot_panel_wheel_zoom_on_y_axis_zooms_y_only_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-wheel-y-axis-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(17.0), Px(81.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative y-axis wheel zoom should leave an explicit view bounds");
    assert!(
        (view.x_min - 0.0).abs() < 0.001 && (view.x_max - 4.0).abs() < 0.001,
        "wheel over the declarative Y axis should preserve the X range, got {view:?}"
    );
    assert!(
        view.y_max - view.y_min < 4.0,
        "wheel over the declarative Y axis should zoom the Y range, got {view:?}"
    );
}

#[test]
fn line_plot_panel_wheel_zoom_respects_x_zoom_lock_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    plot_state.axis_locks.x.zoom = true;
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-wheel-x-lock-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(169.0), Px(81.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative locked wheel zoom should leave an explicit view bounds");
    assert!(
        (view.x_min - 0.0).abs() < 0.001 && (view.x_max - 4.0).abs() < 0.001,
        "X zoom lock should preserve the declarative X range, got {view:?}"
    );
    assert!(
        view.y_max - view.y_min < 4.0,
        "X zoom lock should still allow declarative Y zoom, got {view:?}"
    );
}

#[test]
fn line_plot_panel_wheel_zoom_respects_y_zoom_lock_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    plot_state.axis_locks.y.zoom = true;
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-wheel-y-lock-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(169.0), Px(81.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative locked wheel zoom should leave an explicit view bounds");
    assert!(
        view.x_max - view.x_min < 4.0,
        "Y zoom lock should still allow declarative X zoom, got {view:?}"
    );
    assert!(
        (view.y_min - 0.0).abs() < 0.001 && (view.y_max - 4.0).abs() < 0.001,
        "Y zoom lock should preserve the declarative Y range, got {view:?}"
    );
}

#[test]
fn line_plot_panel_wheel_zoom_noops_when_both_axes_locked_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
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
    plot_state.axis_locks.x.zoom = true;
    plot_state.axis_locks.y.zoom = true;
    let state = app.models_mut().insert(plot_state);
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
        "plot-declarative-wheel-both-lock-view",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone())
                    .state(state.clone())
                    .style(style),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(169.0), Px(81.0)),
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let view = state
        .read_ref(&app, |state| state.view_bounds)
        .expect("plot state should be readable")
        .expect("declarative locked wheel zoom should preserve explicit view bounds");
    assert_eq!(
        view,
        DataRect {
            x_min: 0.0,
            x_max: 4.0,
            y_min: 0.0,
            y_max: 4.0,
        },
        "wheel zoom should not change declarative view bounds when both axes are zoom-locked"
    );
}

#[test]
fn line_plot_panel_updates_output_cursor_on_pointer_move() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
                true,
            ),
        )]));
    let output = app.models_mut().insert(PlotOutput::default());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-pointer-output",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).output(output.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(169.0), Px(81.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let published = output
        .read_ref(&app, |output| *output)
        .expect("plot output model should be readable");
    assert_eq!(published.revision, 1);
    let cursor = published
        .snapshot
        .cursor
        .expect("pointer inside the plot region should publish cursor data");
    assert!(
        (cursor.x - 1.0).abs() < 0.02,
        "expected pointer x to map to the middle of the data domain, got {:?}",
        cursor
    );
    assert!(
        (cursor.y - 0.5).abs() < 0.04,
        "expected pointer y to map to the middle of the data domain, got {:?}",
        cursor
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(4.0), Px(4.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );
    let published = output
        .read_ref(&app, |output| *output)
        .expect("plot output model should be readable");
    assert_eq!(published.revision, 2);
    assert_eq!(published.snapshot.cursor, None);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Path {
                order: DrawOrder(20),
                ..
            }
        )),
        "managed-surface pointer handling must preserve declarative line painting"
    );
}

#[test]
fn line_plot_panel_paints_cursor_readout_without_output_model_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
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
        "plot-declarative-cursor-readout",
        |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(169.0), Px(81.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let cursor_guides = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(3),
                    ..
                }
            )
        })
        .count();
    assert!(
        cursor_guides >= 2,
        "declarative line plot should paint cursor crosshair guides"
    );

    let readout_backgrounds = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(12),
                    ..
                }
            )
        })
        .count();
    assert!(
        readout_backgrounds >= 1,
        "declarative line plot should paint mouse readout overlay chrome"
    );

    let readout_text = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Text {
                    order: DrawOrder(13),
                    ..
                }
            )
        })
        .count();
    assert!(
        readout_text >= 1,
        "declarative line plot should paint mouse readout text"
    );

    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Path {
                order: DrawOrder(20),
                ..
            }
        )),
        "cursor readout painting must preserve declarative line painting"
    );
}

#[test]
fn line_plot_panel_paints_series_readout_rows_on_declarative_cursor_overlay() {
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
            "Alpha",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 0.0 },
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
        "plot-declarative-series-readout",
        |cx| vec![line_plot_panel(cx, LinePlotPanelProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(169.0), Px(81.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let mut prepared_text = services.prepared_text.join("\n");
    prepared_text.make_ascii_lowercase();
    assert!(
        prepared_text.contains("alpha: y="),
        "declarative cursor readout should include per-series readout rows, got {prepared_text:?}"
    );
}

#[test]
fn line_plot_panel_paints_right_axis_series_readout_with_right_axis_formatter_on_declarative_path()
{
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
    let model = app.models_mut().insert(LinePlotModel::from_series(vec![
        LineSeries::new(
            "RightAxis",
            Series::from_points_sorted(
                vec![
                    DataPoint { x: 0.0, y: 0.0 },
                    DataPoint { x: 1.0, y: 1.0 },
                    DataPoint { x: 2.0, y: 0.0 },
                ],
                true,
            ),
        )
        .y_axis(YAxis::Right),
    ]));

    let right_axis_labels =
        AxisLabelFormatter::custom(0x5279_6768_7441, |v, _span| format!("R{v:.1}"));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-right-axis-series-readout",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).y2_axis_labels(right_axis_labels),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(169.0), Px(81.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let mut prepared_text = services.prepared_text.join("\n");
    prepared_text.make_ascii_lowercase();
    assert!(
        prepared_text.contains("rightaxis: y2=r1.0"),
        "right-axis cursor readout should use the right-axis formatter, got {prepared_text:?}"
    );
}

#[test]
fn line_plot_panel_paints_linked_cursor_readout_from_state_on_declarative_path() {
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
                    DataPoint { x: 2.0, y: 0.0 },
                ],
                true,
            ),
        )]));
    let mut plot_state = PlotState::default();
    plot_state.linked_cursor_x = Some(1.0);
    let state = app.models_mut().insert(plot_state);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "plot-declarative-linked-cursor-readout",
        |cx| {
            vec![line_plot_panel(
                cx,
                LinePlotPanelProps::new(model.clone()).state(state.clone()),
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let linked_cursor_guides = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(3),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        linked_cursor_guides, 1,
        "linked cursor should paint one vertical guide when no local cursor is active"
    );

    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Quad {
                order: DrawOrder(12),
                ..
            }
        )),
        "linked cursor should paint readout overlay chrome"
    );
    assert!(
        scene.ops().iter().any(|op| matches!(
            op,
            fret_core::SceneOp::Text {
                order: DrawOrder(13),
                ..
            }
        )),
        "linked cursor should paint readout text"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(169.0), Px(81.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: PointerId(0),
            pointer_type: PointerType::Mouse,
        }),
    );

    scene.clear();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let local_cursor_guides = scene
        .ops()
        .iter()
        .filter(|op| {
            matches!(
                op,
                fret_core::SceneOp::Quad {
                    order: DrawOrder(3),
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        local_cursor_guides, 2,
        "local cursor crosshair should take precedence over linked cursor"
    );
}
