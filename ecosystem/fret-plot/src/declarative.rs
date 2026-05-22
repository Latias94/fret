use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, FontWeight, Paint, PathStyle, Point, Px, Rect, Size,
    StrokeStyle, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};
use fret_ui::element::{AnyElement, CanvasProps, Length, ManagedSurfaceProps};
use fret_ui::{ElementContext, ElementContextAccess, UiHost};

use crate::cartesian::{AxisScale, DataPoint, PlotTransform, polyline_commands};
use crate::models::LinePlotModel;
use crate::plot::axis::{
    AxisLabelFormatter, AxisTicks, axis_ticks_scaled, log10_tick_label_or_empty,
};
use crate::plot::view::sanitize_data_rect_scaled;
use crate::state::{PlotOutput, PlotOutputSnapshot};
use crate::style::LinePlotStyle;

#[derive(Clone)]
pub struct LinePlotPanelProps {
    pub canvas: CanvasProps,
    pub model: Model<LinePlotModel>,
    pub output: Option<Model<PlotOutput>>,
    pub style: LinePlotStyle,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
}

impl LinePlotPanelProps {
    pub fn new(model: Model<LinePlotModel>) -> Self {
        Self {
            canvas: CanvasProps::default(),
            model,
            output: None,
            style: LinePlotStyle::default(),
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
        }
    }

    pub fn output(mut self, output: Model<PlotOutput>) -> Self {
        self.output = Some(output);
        self
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
    let output = props.output.clone();
    let event_model = model.clone();
    let event_output = output.clone();
    let event_style = style;
    let event_x_scale = x_scale;
    let event_y_scale = y_scale;

    let mut surface = ManagedSurfaceProps::default();
    surface.layout = props.canvas.layout;
    let canvas = props.canvas;
    let element = cx.managed_surface(
        surface,
        |cx| {
            cx.layout_unplaced_children(cx.bounds());
            cx.set_hit_test_rects([cx.bounds()]);
        },
        |cx| {
            let bounds = cx.bounds();
            for child in cx.children().to_vec() {
                cx.paint_child(child, bounds);
            }
        },
        move |cx| {
            let model = model.clone();
            vec![cx.canvas(canvas, move |painter| {
                paint_line_plot_panel(painter, &model, style, x_scale, y_scale);
            })]
        },
    );
    let surface_id = element.id;
    cx.managed_surface_on_event_for(surface_id, move |cx, event| {
        let bounds = cx.bounds();
        let changed = handle_line_plot_panel_event(
            cx.app(),
            bounds,
            event,
            &event_model,
            event_output.as_ref(),
            event_style,
            event_x_scale,
            event_y_scale,
        );
        if changed {
            cx.invalidate_self(fret_ui::Invalidation::Paint);
            cx.request_redraw();
            cx.notify();
        }
    });
    element
}

/// Capability-first adapter for [`line_plot_panel`] when the caller only owns
/// `ElementContextAccess`.
#[track_caller]
pub fn line_plot_panel_in<'a, H: UiHost + 'a + 'static, Cx>(
    cx: &mut Cx,
    props: LinePlotPanelProps,
) -> AnyElement
where
    Cx: ElementContextAccess<'a, H>,
{
    line_plot_panel(cx.elements(), props)
}

fn paint_line_plot_panel(
    painter: &mut CanvasPainter<'_>,
    model: &LinePlotModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    let bounds = painter.bounds();
    let plot = line_plot_inner_rect(bounds, style);
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
    paint_line_plot_grid_and_axes(painter, transform, style);

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
            DrawOrder(20),
            Point::new(Px(0.0), Px(0.0)),
            &commands,
            PathStyle::Stroke(StrokeStyle {
                width: stroke_width,
            }),
            color,
            raster_scale_factor,
        );
    }

    paint_line_plot_legend(painter, model, plot, style);
}

fn handle_line_plot_panel_event<H: UiHost>(
    app: &mut H,
    bounds: Rect,
    event: &Event,
    model: &LinePlotModel,
    output: Option<&Model<PlotOutput>>,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let Some(output) = output else {
        return false;
    };
    let Event::Pointer(fret_core::PointerEvent::Move { position, .. }) = event else {
        return false;
    };
    let snapshot =
        line_plot_pointer_output_snapshot(*position, bounds, model, style, x_scale, y_scale);
    output
        .update(app, |state, _cx| {
            if state.snapshot == snapshot {
                return false;
            }
            state.revision = state.revision.wrapping_add(1);
            state.snapshot = snapshot;
            true
        })
        .ok()
        .unwrap_or(false)
}

fn line_plot_pointer_output_snapshot(
    pointer: Point,
    bounds: Rect,
    model: &LinePlotModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> PlotOutputSnapshot {
    let view_bounds = sanitize_data_rect_scaled(model.data_bounds, x_scale, y_scale);
    let plot = line_plot_inner_rect(bounds, style);
    let cursor = cursor_data_for_line_plot_pointer(pointer, plot, view_bounds, x_scale, y_scale);
    PlotOutputSnapshot {
        view_bounds,
        view_bounds_y2: None,
        view_bounds_y3: None,
        view_bounds_y4: None,
        cursor,
        hover: None,
        query: None,
        drag: None,
    }
}

fn cursor_data_for_line_plot_pointer(
    pointer: Point,
    plot: Rect,
    view_bounds: crate::cartesian::DataRect,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<DataPoint> {
    if !plot.contains(pointer) || plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }
    let transform = PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    };
    let data = transform.px_to_data(pointer);
    (data.x.is_finite() && data.y.is_finite()).then_some(data)
}

fn paint_line_plot_grid_and_axes(
    painter: &mut CanvasPainter<'_>,
    transform: PlotTransform,
    style: LinePlotStyle,
) {
    let plot = transform.viewport;
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return;
    }

    let theme = painter.theme().snapshot();
    let mut grid_color = style
        .grid_color
        .unwrap_or_else(|| theme.color_required("border"));
    grid_color.a *= 0.45;
    let axis_color = style
        .axis_color
        .unwrap_or_else(|| theme.color_required("border"));
    let tick_count = style.tick_count.max(2);

    let x_ticks = axis_ticks_scaled(
        transform.data.x_min,
        transform.data.x_max,
        tick_count,
        AxisTicks::Nice,
        transform.x_scale,
    );
    let y_ticks = axis_ticks_scaled(
        transform.data.y_min,
        transform.data.y_max,
        tick_count,
        AxisTicks::Nice,
        transform.y_scale,
    );

    for x in x_ticks.iter().copied() {
        let Some(px) = transform.data_x_to_px(x) else {
            continue;
        };
        push_vertical_line(
            painter,
            px,
            plot.origin.y,
            plot.size.height,
            DrawOrder(2),
            grid_color,
        );
    }

    for y in y_ticks.iter().copied() {
        let Some(py) = transform.data_y_to_px(y) else {
            continue;
        };
        push_horizontal_line(
            painter,
            plot.origin.x,
            py,
            plot.size.width,
            DrawOrder(2),
            grid_color,
        );
    }

    let baseline_y = transform
        .data_y_to_px(0.0)
        .filter(|y| y.0 >= plot.origin.y.0 && y.0 <= plot.origin.y.0 + plot.size.height.0)
        .unwrap_or_else(|| Px(plot.origin.y.0 + plot.size.height.0 - 1.0));
    let baseline_x = transform
        .data_x_to_px(0.0)
        .filter(|x| x.0 >= plot.origin.x.0 && x.0 <= plot.origin.x.0 + plot.size.width.0)
        .unwrap_or(plot.origin.x);

    push_horizontal_line(
        painter,
        plot.origin.x,
        baseline_y,
        plot.size.width,
        DrawOrder(10),
        axis_color,
    );
    push_vertical_line(
        painter,
        baseline_x,
        plot.origin.y,
        plot.size.height,
        DrawOrder(10),
        axis_color,
    );

    paint_line_plot_axis_tick_labels(painter, transform, style, &x_ticks, &y_ticks);
}

fn push_vertical_line(
    painter: &mut CanvasPainter<'_>,
    x: Px,
    y: Px,
    height: Px,
    order: DrawOrder,
    color: Color,
) {
    if !x.0.is_finite() || !y.0.is_finite() || !height.0.is_finite() || height.0 <= 0.0 {
        return;
    }
    painter.scene().push(fret_core::SceneOp::Quad {
        order,
        rect: Rect::new(Point::new(x, y), Size::new(Px(1.0), height)),
        background: Paint::Solid(color).into(),
        border: Edges::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT).into(),
        corner_radii: Corners::default(),
    });
}

fn push_horizontal_line(
    painter: &mut CanvasPainter<'_>,
    x: Px,
    y: Px,
    width: Px,
    order: DrawOrder,
    color: Color,
) {
    if !x.0.is_finite() || !y.0.is_finite() || !width.0.is_finite() || width.0 <= 0.0 {
        return;
    }
    painter.scene().push(fret_core::SceneOp::Quad {
        order,
        rect: Rect::new(Point::new(x, y), Size::new(width, Px(1.0))),
        background: Paint::Solid(color).into(),
        border: Edges::default(),
        border_paint: Paint::Solid(Color::TRANSPARENT).into(),
        corner_radii: Corners::default(),
    });
}

fn paint_line_plot_legend(
    painter: &mut CanvasPainter<'_>,
    model: &LinePlotModel,
    plot: Rect,
    style: LinePlotStyle,
) {
    if model.series.is_empty() || plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return;
    }

    let theme = painter.theme().snapshot();
    let text_color = style
        .label_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let text_constraints = CanvasTextConstraints {
        max_width: Some(Px((plot.size.width.0 - 36.0).max(24.0))),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };

    let series_count = model.series.len();
    let row_height = Px(18.0);
    let swatch = Size::new(Px(12.0), Px(3.0));
    let gap = Px(6.0);
    let inset = Px(8.0);
    let text_baseline_offset = Px(12.0);
    let x = Px(plot.origin.x.0 + inset.0);
    let mut y = Px(plot.origin.y.0 + inset.0);
    let max_y = plot.origin.y.0 + plot.size.height.0 - inset.0;
    let scope = painter.key_scope(&"fret-plot.declarative.legend");
    let raster_scale_factor = painter.scale_factor();

    for (index, series) in model.series.iter().enumerate() {
        if y.0 + row_height.0 > max_y {
            break;
        }

        let color = series
            .stroke_color
            .unwrap_or_else(|| series_color(style, index, series_count));
        let row_mid = y.0 + row_height.0 * 0.5;
        painter.scene().push(fret_core::SceneOp::Quad {
            order: DrawOrder(30),
            rect: Rect::new(Point::new(x, Px(row_mid - swatch.height.0 * 0.5)), swatch),
            background: Paint::Solid(color).into(),
            border: Edges::default(),
            border_paint: Paint::Solid(Color::TRANSPARENT).into(),
            corner_radii: Corners::default(),
        });

        let key: u64 = painter
            .child_key(scope, &("series", series.id.0, series.label.as_ref()))
            .into();
        let _ = painter.text(
            key,
            DrawOrder(31),
            Point::new(
                Px(x.0 + swatch.width.0 + gap.0),
                Px(y.0 + text_baseline_offset.0),
            ),
            series.label.clone(),
            text_style.clone(),
            text_color,
            text_constraints,
            raster_scale_factor,
        );
        y = Px(y.0 + row_height.0);
    }
}

fn paint_line_plot_axis_tick_labels(
    painter: &mut CanvasPainter<'_>,
    transform: PlotTransform,
    style: LinePlotStyle,
    x_ticks: &[f64],
    y_ticks: &[f64],
) {
    if x_ticks.is_empty() && y_ticks.is_empty() {
        return;
    }

    let plot = transform.viewport;
    let theme = painter.theme().snapshot();
    let text_color = style
        .label_color
        .unwrap_or_else(|| theme.color_required("muted-foreground"));
    let text_style = TextStyle {
        size: Px(12.0),
        weight: FontWeight::NORMAL,
        ..TextStyle::default()
    };
    let constraints = CanvasTextConstraints {
        max_width: Some(Px(72.0)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    };
    let formatter = AxisLabelFormatter::default();
    let x_span = (transform.data.x_max - transform.data.x_min).abs();
    let y_span = (transform.data.y_max - transform.data.y_min).abs();
    let scope = painter.key_scope(&"fret-plot.declarative.axis-labels");
    let raster_scale_factor = painter.scale_factor();

    let x_label_y = Px(plot.origin.y.0 + plot.size.height.0 + 2.0);
    for (index, value) in x_ticks.iter().copied().enumerate() {
        let Some(x) = transform.data_x_to_px(value) else {
            continue;
        };
        let text = axis_tick_label_text(transform.x_scale, &formatter, value, x_span);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(scope, &("x", index, value.to_bits()))
            .into();
        let _ = painter.text(
            key,
            DrawOrder(11),
            Point::new(Px(x.0 - 12.0), x_label_y),
            text,
            text_style.clone(),
            text_color,
            constraints,
            raster_scale_factor,
        );
    }

    let y_label_x = Px((plot.origin.x.0 - style.axis_gap.0 + 4.0).max(0.0));
    for (index, value) in y_ticks.iter().copied().enumerate() {
        let Some(y) = transform.data_y_to_px(value) else {
            continue;
        };
        let text = axis_tick_label_text(transform.y_scale, &formatter, value, y_span);
        if text.is_empty() {
            continue;
        }
        let key: u64 = painter
            .child_key(scope, &("y", index, value.to_bits()))
            .into();
        let _ = painter.text(
            key,
            DrawOrder(11),
            Point::new(y_label_x, y),
            text,
            text_style.clone(),
            text_color,
            constraints,
            raster_scale_factor,
        );
    }
}

fn axis_tick_label_text(
    scale: AxisScale,
    formatter: &AxisLabelFormatter,
    value: f64,
    span: f64,
) -> String {
    if scale == AxisScale::Log10 && formatter.is_number_auto() {
        return log10_tick_label_or_empty(value);
    }
    formatter.format(value, span)
}

fn line_plot_inner_rect(bounds: Rect, style: LinePlotStyle) -> Rect {
    let pad = style.padding.0.max(0.0);
    let axis_gap = style.axis_gap.0.max(0.0);
    Rect::new(
        Point::new(
            Px(bounds.origin.x.0 + pad + axis_gap),
            Px(bounds.origin.y.0 + pad),
        ),
        Size::new(
            Px((bounds.size.width.0 - pad * 2.0 - axis_gap).max(0.0)),
            Px((bounds.size.height.0 - pad * 2.0 - axis_gap).max(0.0)),
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
    use crate::state::PlotOutput;
    use fret_core::{
        AppWindowId, Event, FrameId, MaterialDescriptor, MaterialId, MaterialRegistrationError,
        MaterialService, Modifiers, MouseButtons, PathCommand, PathConstraints, PathId,
        PathMetrics, PathService, PointerEvent, PointerId, PointerType, Scene, SvgId, SvgService,
        TextBlobId, TextConstraints, TextInput, TextMetrics, TextService,
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
        let mut services = FakeServices;
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
        let mut services = FakeServices;
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
        let mut services = FakeServices;
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
        let mut services = FakeServices;
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
}
