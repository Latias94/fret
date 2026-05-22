use fret_core::{
    Color, Corners, DrawOrder, Edges, Paint, PathStyle, Point, Px, Rect, Size, StrokeStyle,
};
use fret_runtime::Model;
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{AnyElement, CanvasProps, Length};
use fret_ui::{ElementContext, UiHost};

use crate::cartesian::{AxisScale, PlotTransform, polyline_commands};
use crate::models::LinePlotModel;
use crate::plot::view::sanitize_data_rect_scaled;
use crate::style::LinePlotStyle;

#[derive(Clone)]
pub struct LinePlotPanelProps {
    pub canvas: CanvasProps,
    pub model: Model<LinePlotModel>,
    pub style: LinePlotStyle,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
}

impl LinePlotPanelProps {
    pub fn new(model: Model<LinePlotModel>) -> Self {
        Self {
            canvas: CanvasProps::default(),
            model,
            style: LinePlotStyle::default(),
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
        }
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    pub fn x_scale(mut self, scale: AxisScale) -> Self {
        self.x_scale = scale;
        self
    }

    pub fn y_scale(mut self, scale: AxisScale) -> Self {
        self.y_scale = scale;
        self
    }
}

#[track_caller]
pub fn line_plot_panel<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    mut props: LinePlotPanelProps,
) -> AnyElement {
    props.canvas.layout.size.width = Length::Fill;
    props.canvas.layout.size.height = Length::Fill;
    cx.observe_model(&props.model, fret_ui::Invalidation::Paint);

    let model = cx
        .read_model_ref(&props.model, fret_ui::Invalidation::Paint, Clone::clone)
        .expect("line plot model should exist");
    let style = props.style;
    let x_scale = props.x_scale;
    let y_scale = props.y_scale;

    cx.canvas(props.canvas, move |painter| {
        paint_line_plot_panel(painter, &model, style, x_scale, y_scale);
    })
}

fn paint_line_plot_panel(
    painter: &mut CanvasPainter<'_>,
    model: &LinePlotModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    let bounds = painter.bounds();
    let plot = line_plot_inner_rect(bounds, style.padding);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return;
    }

    let background = style
        .background
        .unwrap_or_else(|| painter.theme().snapshot().color_required("surface"));
    painter.scene().push(fret_core::SceneOp::Quad {
        order: DrawOrder(0),
        rect: bounds,
        background: Paint::Solid(background).into(),
        border: if style.border.is_some() {
            Edges::all(style.border_width)
        } else {
            Edges::default()
        },
        border_paint: Paint::Solid(style.border.unwrap_or(Color::TRANSPARENT)).into(),
        corner_radii: Corners::default(),
    });

    let transform = PlotTransform {
        viewport: plot,
        data: sanitize_data_rect_scaled(model.data_bounds, x_scale, y_scale),
        x_scale,
        y_scale,
    };

    let series_count = model.series.len();
    let raster_scale_factor = painter.scale_factor();
    for (index, series) in model.series.iter().enumerate() {
        let Some(points) = series.data.as_slice() else {
            continue;
        };
        let commands = polyline_commands(transform, points);
        if commands.len() < 2 {
            continue;
        }

        let color = series
            .stroke_color
            .unwrap_or_else(|| series_color(style, index, series_count));
        let stroke_width = series.stroke_width.unwrap_or(style.stroke_width);
        painter.path(
            line_plot_series_path_key(series.id.0),
            DrawOrder(1),
            Point::new(Px(0.0), Px(0.0)),
            &commands,
            PathStyle::Stroke(StrokeStyle {
                width: stroke_width,
            }),
            color,
            raster_scale_factor,
        );
    }
}

fn line_plot_inner_rect(bounds: Rect, padding: Px) -> Rect {
    let pad = padding.0.max(0.0);
    Rect::new(
        Point::new(Px(bounds.origin.x.0 + pad), Px(bounds.origin.y.0 + pad)),
        Size::new(
            Px((bounds.size.width.0 - pad * 2.0).max(0.0)),
            Px((bounds.size.height.0 - pad * 2.0).max(0.0)),
        ),
    )
}

fn series_color(style: LinePlotStyle, series_index: usize, series_count: usize) -> Color {
    if series_count <= 1 {
        return style.stroke_color;
    }
    style.series_palette[series_index % style.series_palette.len()]
}

fn line_plot_series_path_key(series_id: u64) -> u64 {
    0x706c_6f74_6c69_6e65_u64 ^ series_id
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartesian::DataPoint;
    use crate::models::{LinePlotModel, LineSeries};
    use crate::series::Series;
    use fret_core::{
        AppWindowId, FrameId, MaterialDescriptor, MaterialId, MaterialRegistrationError,
        MaterialService, PathCommand, PathConstraints, PathId, PathMetrics, PathService, Scene,
        SvgId, SvgService, TextBlobId, TextConstraints, TextInput, TextMetrics, TextService,
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
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
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
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
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
        let mut services = FakeServices;
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
}
