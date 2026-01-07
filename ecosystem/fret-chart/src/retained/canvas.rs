use std::collections::BTreeMap;

use delinea::engine::EngineError;
use delinea::engine::model::{ChartPatch, ModelError, PatchMode};
use delinea::engine::window::DataWindow;
use delinea::marks::{MarkKind, MarkPayloadRef};
use delinea::text::{TextMeasurer, TextMetrics};
use delinea::{Action, ChartEngine, WorkBudget};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, FontWeight, KeyCode, Modifiers, MouseButton,
    PathCommand, PathConstraints, PathStyle, Point, PointerEvent, PointerType, Px, Rect, SceneOp,
    Size, StrokeStyle, TextBlobId, TextConstraints, TextOverflow, TextStyle, TextWrap,
};
use fret_ui::UiHost;
use fret_ui::retained_bridge::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};

use crate::input_map::{ChartInputMap, ModifierKey, ModifiersMask};
use crate::retained::style::ChartStyle;

#[derive(Debug, Default)]
struct NullTextMeasurer;

impl TextMeasurer for NullTextMeasurer {
    fn measure(
        &mut self,
        _text: delinea::ids::StringId,
        _style: delinea::text::TextStyleId,
    ) -> TextMetrics {
        TextMetrics::default()
    }
}

#[derive(Debug, Default)]
struct CachedPath {
    path: fret_core::PathId,
}

#[derive(Debug, Clone, Copy)]
struct PanDrag {
    x_axis: delinea::AxisId,
    y_axis: delinea::AxisId,
    start_pos: Point,
    start_x: DataWindow,
    start_y: DataWindow,
}

#[derive(Debug, Clone, Copy)]
struct BoxZoomDrag {
    x_axis: delinea::AxisId,
    y_axis: delinea::AxisId,
    button: MouseButton,
    required_mods: ModifiersMask,
    start_pos: Point,
    current_pos: Point,
    start_x: DataWindow,
    start_y: DataWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AxisRegion {
    Plot,
    XAxis,
    YAxis,
}

#[derive(Debug, Default, Clone, Copy)]
struct ChartLayout {
    bounds: Rect,
    plot: Rect,
    x_axis: Rect,
    y_axis: Rect,
}

pub struct ChartCanvas {
    engine: ChartEngine,
    style: ChartStyle,
    input_map: ChartInputMap,
    last_bounds: Rect,
    last_layout: ChartLayout,
    last_pointer_pos: Option<Point>,
    last_marks_rev: delinea::ids::Revision,
    last_scale_factor_bits: u32,
    cached_paths: BTreeMap<delinea::ids::MarkId, CachedPath>,
    axis_text: Vec<TextBlobId>,
    tooltip_text: Vec<TextBlobId>,
    pan_drag: Option<PanDrag>,
    box_zoom_drag: Option<BoxZoomDrag>,
    lock_x_pan: bool,
    lock_y_pan: bool,
    lock_x_zoom: bool,
    lock_y_zoom: bool,
}

impl ChartCanvas {
    pub fn new(spec: delinea::ChartSpec) -> Result<Self, ModelError> {
        Ok(Self {
            engine: ChartEngine::new(spec)?,
            style: ChartStyle::default(),
            input_map: ChartInputMap::default(),
            last_bounds: Rect::default(),
            last_layout: ChartLayout::default(),
            last_pointer_pos: None,
            last_marks_rev: delinea::ids::Revision::default(),
            last_scale_factor_bits: 0,
            cached_paths: BTreeMap::default(),
            axis_text: Vec::default(),
            tooltip_text: Vec::default(),
            pan_drag: None,
            box_zoom_drag: None,
            lock_x_pan: false,
            lock_y_pan: false,
            lock_x_zoom: false,
            lock_y_zoom: false,
        })
    }

    pub fn engine(&self) -> &ChartEngine {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut ChartEngine {
        &mut self.engine
    }

    pub fn set_style(&mut self, style: ChartStyle) {
        self.style = style;
    }

    pub fn set_input_map(&mut self, map: ChartInputMap) {
        self.input_map = map;
    }

    fn compute_layout(&self, bounds: Rect) -> ChartLayout {
        let mut inner = bounds;
        inner.origin.x.0 += self.style.padding.left.0;
        inner.origin.y.0 += self.style.padding.top.0;
        inner.size.width.0 =
            (inner.size.width.0 - self.style.padding.left.0 - self.style.padding.right.0).max(0.0);
        inner.size.height.0 =
            (inner.size.height.0 - self.style.padding.top.0 - self.style.padding.bottom.0).max(0.0);

        let axis_band_x = self.style.axis_band_x.0.max(0.0);
        let axis_band_y = self.style.axis_band_y.0.max(0.0);

        let plot_w = (inner.size.width.0 - axis_band_x).max(0.0);
        let plot_h = (inner.size.height.0 - axis_band_y).max(0.0);

        let plot = Rect::new(
            Point::new(Px(inner.origin.x.0 + axis_band_x), inner.origin.y),
            Size::new(Px(plot_w), Px(plot_h)),
        );

        let y_axis = Rect::new(inner.origin, Size::new(Px(axis_band_x), Px(plot_h)));
        let x_axis = Rect::new(
            Point::new(plot.origin.x, Px(plot.origin.y.0 + plot.size.height.0)),
            Size::new(Px(plot_w), Px(axis_band_y)),
        );

        ChartLayout {
            bounds,
            plot,
            x_axis,
            y_axis,
        }
    }

    pub fn create_node<H: UiHost>(ui: &mut fret_ui::UiTree<H>, canvas: Self) -> fret_core::NodeId {
        use fret_ui::retained_bridge::UiTreeRetainedExt as _;
        ui.create_node_retained(canvas)
    }

    fn sync_viewport(&mut self, viewport: Rect) {
        if self.engine.model().viewport == Some(viewport) {
            return;
        }
        let _ = self.engine.apply_patch(
            ChartPatch {
                viewport: Some(Some(viewport)),
                ..ChartPatch::default()
            },
            PatchMode::Merge,
        );
    }

    fn primary_axes(&self) -> Option<(delinea::AxisId, delinea::AxisId)> {
        let series_id = *self.engine.model().series_order.first()?;
        let series = self.engine.model().series.get(&series_id)?;
        Some((series.x_axis, series.y_axis))
    }

    fn axis_range(&self, axis: delinea::AxisId) -> delinea::AxisRange {
        self.engine
            .model()
            .axes
            .get(&axis)
            .map(|a| a.range)
            .unwrap_or_default()
    }

    fn axis_is_fixed(&self, axis: delinea::AxisId) -> Option<DataWindow> {
        match self.axis_range(axis) {
            delinea::AxisRange::Fixed { min, max } => {
                let mut w = DataWindow { min, max };
                w.clamp_non_degenerate();
                Some(w)
            }
            _ => None,
        }
    }

    fn axis_constraints(&self, axis: delinea::AxisId) -> (Option<f64>, Option<f64>) {
        match self.axis_range(axis) {
            delinea::AxisRange::Auto => (None, None),
            delinea::AxisRange::LockMin { min } => (Some(min), None),
            delinea::AxisRange::LockMax { max } => (None, Some(max)),
            delinea::AxisRange::Fixed { min, max } => (Some(min), Some(max)),
        }
    }

    fn current_window_x(&mut self, axis: delinea::AxisId) -> DataWindow {
        if let Some(fixed) = self.axis_is_fixed(axis) {
            return fixed;
        }

        if let Some(window) = self.engine.state().data_window_x.get(&axis).copied() {
            return window;
        }

        let mut window = self.compute_axis_extent(axis, true);
        let (locked_min, locked_max) = self.axis_constraints(axis);
        window = window.apply_constraints(locked_min, locked_max);
        window
    }

    fn current_window_y(&mut self, axis: delinea::AxisId) -> DataWindow {
        if let Some(fixed) = self.axis_is_fixed(axis) {
            return fixed;
        }

        if let Some(window) = self.engine.state().data_window_y.get(&axis).copied() {
            return window;
        }

        let mut window = self.compute_axis_extent(axis, false);
        let (locked_min, locked_max) = self.axis_constraints(axis);
        window = window.apply_constraints(locked_min, locked_max);
        window
    }

    fn compute_axis_extent(&mut self, axis: delinea::AxisId, is_x: bool) -> DataWindow {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        let series_cols: Vec<(delinea::DatasetId, usize)> = self
            .engine
            .model()
            .series
            .values()
            .filter_map(|s| {
                let axis_id = if is_x { s.x_axis } else { s.y_axis };
                if axis_id != axis {
                    return None;
                }
                let col = if is_x { s.x_col } else { s.y_col };
                Some((s.dataset, col))
            })
            .collect();

        let store = self.engine.datasets_mut();
        for (dataset_id, col) in series_cols {
            let Some(table) = store.dataset_mut(dataset_id) else {
                continue;
            };
            let Some(values) = table.column_f64(col) else {
                continue;
            };

            for &v in values {
                if !v.is_finite() {
                    continue;
                }
                min = min.min(v);
                max = max.max(v);
            }
        }

        let mut out = if min.is_finite() && max.is_finite() && max > min {
            DataWindow { min, max }
        } else {
            DataWindow { min: 0.0, max: 1.0 }
        };
        out.clamp_non_degenerate();
        out
    }

    fn set_data_window_x(&mut self, axis: delinea::AxisId, window: Option<DataWindow>) {
        self.engine
            .apply_action(Action::SetDataWindowX { axis, window });
    }

    fn set_data_window_y(&mut self, axis: delinea::AxisId, window: Option<DataWindow>) {
        self.engine
            .apply_action(Action::SetDataWindowY { axis, window });
    }

    fn reset_view(&mut self) {
        let Some((x_axis, y_axis)) = self.primary_axes() else {
            return;
        };
        if self.axis_is_fixed(x_axis).is_none() {
            self.set_data_window_x(x_axis, None);
        }
        if self.axis_is_fixed(y_axis).is_none() {
            self.set_data_window_y(y_axis, None);
        }
    }

    fn fit_view_to_data(&mut self) {
        let Some((x_axis, y_axis)) = self.primary_axes() else {
            return;
        };

        if self.axis_is_fixed(x_axis).is_none() {
            let mut w = self.compute_axis_extent(x_axis, true);
            let (locked_min, locked_max) = self.axis_constraints(x_axis);
            w = w.apply_constraints(locked_min, locked_max);
            self.set_data_window_x(x_axis, Some(w));
        }

        if self.axis_is_fixed(y_axis).is_none() {
            let mut w = self.compute_axis_extent(y_axis, false);
            let (locked_min, locked_max) = self.axis_constraints(y_axis);
            w = w.apply_constraints(locked_min, locked_max);
            self.set_data_window_y(y_axis, Some(w));
        }
    }

    fn axis_region(layout: ChartLayout, position: Point) -> AxisRegion {
        if layout.x_axis.contains(position) {
            return AxisRegion::XAxis;
        }
        if layout.y_axis.contains(position) {
            return AxisRegion::YAxis;
        }
        AxisRegion::Plot
    }

    fn is_button_held(button: MouseButton, buttons: fret_core::MouseButtons) -> bool {
        match button {
            MouseButton::Left => buttons.left,
            MouseButton::Right => buttons.right,
            MouseButton::Middle => buttons.middle,
            _ => false,
        }
    }

    fn apply_box_select_modifiers(
        plot_size: Size,
        start: Point,
        end: Point,
        modifiers: Modifiers,
        expand_x: Option<ModifierKey>,
        expand_y: Option<ModifierKey>,
        required: ModifiersMask,
    ) -> (Point, Point) {
        let mut start = start;
        let mut end = end;

        // Matches ImPlot's default selection modifiers:
        // - Alt: expand selection horizontally to plot edge.
        // - Shift: expand selection vertically to plot edge.
        //
        // Note: when a modifier is required to start the drag gesture (e.g. Shift+LMB alternative),
        // treat it as part of the gesture chord and do not implicitly apply edge expansion.
        if expand_x.is_some_and(|k| k.is_pressed(modifiers) && !k.is_required_by(required)) {
            start.x = Px(0.0);
            end.x = plot_size.width;
        }
        if expand_y.is_some_and(|k| k.is_pressed(modifiers) && !k.is_required_by(required)) {
            start.y = Px(0.0);
            end.y = plot_size.height;
        }

        (start, end)
    }

    fn data_at_x_px(window: DataWindow, x_px: f32, viewport_width_px: f32) -> f64 {
        let viewport_width_px = viewport_width_px as f64;
        if !viewport_width_px.is_finite() || viewport_width_px <= 0.0 {
            return window.min;
        }
        let t = ((x_px as f64) / viewport_width_px).clamp(0.0, 1.0);
        window.min + t * window.span()
    }

    fn data_at_y_px_from_bottom(
        window: DataWindow,
        y_px_from_bottom: f32,
        viewport_height_px: f32,
    ) -> f64 {
        let viewport_height_px = viewport_height_px as f64;
        if !viewport_height_px.is_finite() || viewport_height_px <= 0.0 {
            return window.min;
        }
        let t = ((y_px_from_bottom as f64) / viewport_height_px).clamp(0.0, 1.0);
        window.min + t * window.span()
    }

    fn axis_ticks(window: DataWindow, count: usize) -> Vec<f64> {
        nice_ticks(window, count)
    }

    fn format_tick(window: DataWindow, value: f64) -> String {
        format_tick_value(window, value)
    }

    fn clear_axis_text_cache(&mut self, services: &mut dyn fret_core::UiServices) {
        for blob in self.axis_text.drain(..) {
            services.text().release(blob);
        }
    }

    fn clear_tooltip_text_cache(&mut self, services: &mut dyn fret_core::UiServices) {
        for blob in self.tooltip_text.drain(..) {
            services.text().release(blob);
        }
    }

    fn draw_axes<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        self.clear_axis_text_cache(cx.services);

        let layout = self.last_layout;
        if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
            return;
        }

        let Some((x_axis, y_axis)) = self.primary_axes() else {
            return;
        };

        let x_window = self.current_window_x(x_axis);
        let y_window = self.current_window_y(y_axis);

        let axis_order = DrawOrder(self.style.draw_order.0.saturating_add(2));
        let label_order = DrawOrder(self.style.draw_order.0.saturating_add(3));

        let line_w = self.style.axis_line_width.0.max(1.0);
        let tick_len = self.style.axis_tick_length.0.max(0.0);

        // Axis baselines (as thin quads).
        cx.scene.push(SceneOp::Quad {
            order: axis_order,
            rect: Rect::new(
                Point::new(
                    layout.plot.origin.x,
                    Px(layout.plot.origin.y.0 + layout.plot.size.height.0 - line_w * 0.5),
                ),
                Size::new(layout.plot.size.width, Px(line_w)),
            ),
            background: self.style.axis_line_color,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });
        cx.scene.push(SceneOp::Quad {
            order: axis_order,
            rect: Rect::new(
                Point::new(
                    Px(layout.plot.origin.x.0 - line_w * 0.5),
                    layout.plot.origin.y,
                ),
                Size::new(Px(line_w), layout.plot.size.height),
            ),
            background: self.style.axis_line_color,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });

        let text_style = TextStyle {
            size: Px(12.0),
            weight: FontWeight::NORMAL,
            ..TextStyle::default()
        };
        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        let x_tick_count = (layout.plot.size.width.0 / 80.0).round().clamp(2.0, 12.0) as usize;
        let y_tick_count = (layout.plot.size.height.0 / 56.0).round().clamp(2.0, 12.0) as usize;

        // X ticks + labels.
        let mut last_right = f32::NEG_INFINITY;
        for value in Self::axis_ticks(x_window, x_tick_count) {
            let t = ((value - x_window.min) / x_window.span()).clamp(0.0, 1.0) as f32;
            let x_px = layout.plot.origin.x.0 + t * layout.plot.size.width.0;
            let y0 = layout.plot.origin.y.0 + layout.plot.size.height.0;

            cx.scene.push(SceneOp::Quad {
                order: axis_order,
                rect: Rect::new(
                    Point::new(Px(x_px - 0.5 * line_w), Px(y0)),
                    Size::new(Px(line_w), Px(tick_len)),
                ),
                background: self.style.axis_tick_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });

            let label = Self::format_tick(x_window, value);
            let (blob, metrics) = cx.services.text().prepare(&label, &text_style, constraints);

            let label_x = x_px - metrics.size.width.0 * 0.5;
            let label_y = layout.x_axis.origin.y.0
                + (layout.x_axis.size.height.0 - metrics.size.height.0) * 0.5;

            let gap = 4.0;
            let right = label_x + metrics.size.width.0;
            if label_x >= last_right + gap {
                cx.scene.push(SceneOp::Text {
                    order: label_order,
                    origin: Point::new(Px(label_x), Px(label_y)),
                    text: blob,
                    color: self.style.axis_label_color,
                });
                self.axis_text.push(blob);
                last_right = right;
            } else {
                cx.services.text().release(blob);
            }
        }

        // Y ticks + labels.
        let mut last_bottom = f32::NEG_INFINITY;
        for value in Self::axis_ticks(y_window, y_tick_count) {
            let t = ((value - y_window.min) / y_window.span()).clamp(0.0, 1.0) as f32;
            let y_px = layout.plot.origin.y.0 + (1.0 - t) * layout.plot.size.height.0;
            let x0 = layout.plot.origin.x.0;

            cx.scene.push(SceneOp::Quad {
                order: axis_order,
                rect: Rect::new(
                    Point::new(Px(x0 - tick_len), Px(y_px - 0.5 * line_w)),
                    Size::new(Px(tick_len), Px(line_w)),
                ),
                background: self.style.axis_tick_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });

            let label = Self::format_tick(y_window, value);
            let (blob, metrics) = cx.services.text().prepare(&label, &text_style, constraints);

            let label_x = layout.y_axis.origin.x.0
                + (layout.y_axis.size.width.0 - metrics.size.width.0 - 4.0).max(0.0);
            let label_y = y_px - metrics.size.height.0 * 0.5;

            let gap = 2.0;
            let bottom = label_y + metrics.size.height.0;
            if label_y >= last_bottom + gap {
                cx.scene.push(SceneOp::Text {
                    order: label_order,
                    origin: Point::new(Px(label_x), Px(label_y)),
                    text: blob,
                    color: self.style.axis_label_color,
                });
                self.axis_text.push(blob);
                last_bottom = bottom;
            } else {
                cx.services.text().release(blob);
            }
        }
    }

    fn rebuild_paths_if_needed<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        let marks_rev = self.engine.output().marks.revision;
        let scale_factor_bits = cx.scale_factor.to_bits();

        if marks_rev == self.last_marks_rev && scale_factor_bits == self.last_scale_factor_bits {
            return;
        }
        self.last_marks_rev = marks_rev;
        self.last_scale_factor_bits = scale_factor_bits;

        for cached in self.cached_paths.values() {
            cx.services.path().release(cached.path);
        }
        self.cached_paths.clear();

        let marks = &self.engine.output().marks;
        let origin = self.last_layout.plot.origin;

        for node in &marks.nodes {
            if node.kind != MarkKind::Polyline {
                continue;
            }

            let MarkPayloadRef::Polyline(poly) = &node.payload else {
                continue;
            };

            let start = poly.points.start;
            let end = poly.points.end;
            if end <= start || end > marks.arena.points.len() {
                continue;
            }

            let mut commands: Vec<PathCommand> =
                Vec::with_capacity((end - start).saturating_add(1));
            for (i, p) in marks.arena.points[start..end].iter().enumerate() {
                let local = fret_core::Point::new(Px(p.x.0 - origin.x.0), Px(p.y.0 - origin.y.0));
                if i == 0 {
                    commands.push(PathCommand::MoveTo(local));
                } else {
                    commands.push(PathCommand::LineTo(local));
                }
            }

            if commands.len() < 2 {
                continue;
            }

            let stroke_width = poly
                .stroke
                .as_ref()
                .map(|(_, s)| s.width)
                .unwrap_or(self.style.stroke_width);

            let (path, _metrics) = cx.services.path().prepare(
                &commands,
                PathStyle::Stroke(StrokeStyle {
                    width: stroke_width,
                }),
                PathConstraints {
                    scale_factor: cx.scale_factor,
                },
            );

            let mark_id = node.id;
            self.cached_paths.insert(mark_id, CachedPath { path });
        }
    }
}

impl<H: UiHost> Widget<H> for ChartCanvas {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        match event {
            Event::KeyDown { key, modifiers, .. } => {
                let plain = !modifiers.shift
                    && !modifiers.ctrl
                    && !modifiers.alt
                    && !modifiers.alt_gr
                    && !modifiers.meta;
                let lock_mods_ok = !modifiers.alt && !modifiers.alt_gr && !modifiers.meta;

                if lock_mods_ok && *key == KeyCode::KeyL {
                    let Some(pos) = self.last_pointer_pos else {
                        return;
                    };

                    let toggle_pan = modifiers.shift && !modifiers.ctrl;
                    let toggle_zoom = modifiers.ctrl && !modifiers.shift;
                    let toggle_both = !toggle_pan && !toggle_zoom;

                    let layout = self.compute_layout(cx.bounds);
                    match Self::axis_region(layout, pos) {
                        AxisRegion::XAxis => {
                            if toggle_both || toggle_pan {
                                self.lock_x_pan = !self.lock_x_pan;
                            }
                            if toggle_both || toggle_zoom {
                                self.lock_x_zoom = !self.lock_x_zoom;
                            }
                        }
                        AxisRegion::YAxis => {
                            if toggle_both || toggle_pan {
                                self.lock_y_pan = !self.lock_y_pan;
                            }
                            if toggle_both || toggle_zoom {
                                self.lock_y_zoom = !self.lock_y_zoom;
                            }
                        }
                        AxisRegion::Plot => {
                            if toggle_both || toggle_pan {
                                self.lock_x_pan = !self.lock_x_pan;
                                self.lock_y_pan = !self.lock_y_pan;
                            }
                            if toggle_both || toggle_zoom {
                                self.lock_x_zoom = !self.lock_x_zoom;
                                self.lock_y_zoom = !self.lock_y_zoom;
                            }
                        }
                    }

                    self.pan_drag = None;
                    self.box_zoom_drag = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if plain && *key == KeyCode::KeyR {
                    self.reset_view();
                    self.pan_drag = None;
                    self.box_zoom_drag = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if plain && *key == KeyCode::KeyF {
                    self.fit_view_to_data();
                    self.pan_drag = None;
                    self.box_zoom_drag = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }
            Event::Pointer(PointerEvent::Move {
                position, buttons, ..
            }) => {
                self.last_pointer_pos = Some(*position);
                if cx.captured == Some(cx.node) {
                    if let Some(mut drag) = self.box_zoom_drag
                        && Self::is_button_held(drag.button, *buttons)
                    {
                        drag.current_pos = *position;
                        self.box_zoom_drag = Some(drag);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }

                    if let Some(drag) = self.pan_drag
                        && buttons.left
                    {
                        let layout = self.compute_layout(cx.bounds);
                        let width = layout.plot.size.width.0;
                        let height = layout.plot.size.height.0;
                        if width <= 0.0 || height <= 0.0 {
                            return;
                        }

                        let dx = position.x.0 - drag.start_pos.x.0;
                        let dy = position.y.0 - drag.start_pos.y.0;

                        let mut next_x = if self.lock_x_pan {
                            drag.start_x
                        } else {
                            drag.start_x.pan_by_px(dx, width)
                        };
                        let mut next_y = if self.lock_y_pan {
                            drag.start_y
                        } else {
                            drag.start_y.pan_by_px(-dy, height)
                        };

                        let (x_locked_min, x_locked_max) = self.axis_constraints(drag.x_axis);
                        let (y_locked_min, y_locked_max) = self.axis_constraints(drag.y_axis);
                        next_x = next_x.apply_constraints(x_locked_min, x_locked_max);
                        next_y = next_y.apply_constraints(y_locked_min, y_locked_max);

                        if !self.lock_x_pan {
                            self.set_data_window_x(drag.x_axis, Some(next_x));
                        }
                        if !self.lock_y_pan {
                            self.set_data_window_y(drag.y_axis, Some(next_y));
                        }

                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }
                }

                self.engine
                    .apply_action(Action::HoverAt { point: *position });
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            Event::Pointer(PointerEvent::Down {
                position,
                button,
                modifiers,
                click_count,
                pointer_type,
                ..
            }) => {
                self.last_pointer_pos = Some(*position);

                if *pointer_type == PointerType::Mouse
                    && *button == MouseButton::Left
                    && *click_count == 2
                    && !modifiers.shift
                    && !modifiers.ctrl
                    && !modifiers.alt
                    && !modifiers.alt_gr
                    && !modifiers.meta
                {
                    let Some((x_axis, y_axis)) = self.primary_axes() else {
                        return;
                    };
                    let layout = self.compute_layout(cx.bounds);
                    match Self::axis_region(layout, *position) {
                        AxisRegion::XAxis => {
                            if self.axis_is_fixed(x_axis).is_none() {
                                self.set_data_window_x(x_axis, None);
                            }
                        }
                        AxisRegion::YAxis => {
                            if self.axis_is_fixed(y_axis).is_none() {
                                self.set_data_window_y(y_axis, None);
                            }
                        }
                        AxisRegion::Plot => self.reset_view(),
                    }

                    self.pan_drag = None;
                    self.box_zoom_drag = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if let Some(cancel) = self.input_map.box_zoom_cancel
                    && self.box_zoom_drag.is_some()
                    && cancel.matches(*button, *modifiers)
                {
                    self.box_zoom_drag = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if self.input_map.axis_lock_toggle.matches(*button, *modifiers) {
                    let layout = self.compute_layout(cx.bounds);
                    match Self::axis_region(layout, *position) {
                        AxisRegion::XAxis => {
                            self.lock_x_pan = !self.lock_x_pan;
                            self.lock_x_zoom = !self.lock_x_zoom;
                        }
                        AxisRegion::YAxis => {
                            self.lock_y_pan = !self.lock_y_pan;
                            self.lock_y_zoom = !self.lock_y_zoom;
                        }
                        AxisRegion::Plot => {
                            self.lock_x_pan = !self.lock_x_pan;
                            self.lock_x_zoom = !self.lock_x_zoom;
                            self.lock_y_pan = !self.lock_y_pan;
                            self.lock_y_zoom = !self.lock_y_zoom;
                        }
                    }

                    cx.request_focus(cx.node);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if self.pan_drag.is_some() || self.box_zoom_drag.is_some() {
                    return;
                }

                let start_box_primary = self.input_map.box_zoom.matches(*button, *modifiers);
                let start_box_alt = self
                    .input_map
                    .box_zoom_alt
                    .is_some_and(|chord| chord.matches(*button, *modifiers));
                if start_box_primary || start_box_alt {
                    let layout = self.compute_layout(cx.bounds);
                    if !layout.plot.contains(*position) {
                        return;
                    }

                    let Some((x_axis, y_axis)) = self.primary_axes() else {
                        return;
                    };

                    if self.axis_is_fixed(x_axis).is_some() || self.axis_is_fixed(y_axis).is_some()
                    {
                        return;
                    }

                    let required_mods = if start_box_primary {
                        self.input_map.box_zoom.modifiers
                    } else {
                        self.input_map
                            .box_zoom_alt
                            .unwrap_or(self.input_map.box_zoom)
                            .modifiers
                    };

                    let start_x = self.current_window_x(x_axis);
                    let start_y = self.current_window_y(y_axis);

                    self.box_zoom_drag = Some(BoxZoomDrag {
                        x_axis,
                        y_axis,
                        button: *button,
                        required_mods,
                        start_pos: *position,
                        current_pos: *position,
                        start_x,
                        start_y,
                    });

                    cx.request_focus(cx.node);
                    cx.capture_pointer(cx.node);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if !self.input_map.pan.matches(*button, *modifiers) {
                    return;
                }

                let layout = self.compute_layout(cx.bounds);
                if !layout.plot.contains(*position) {
                    return;
                }

                let Some((x_axis, y_axis)) = self.primary_axes() else {
                    return;
                };
                if self.axis_is_fixed(x_axis).is_some() || self.axis_is_fixed(y_axis).is_some() {
                    return;
                }
                if self.lock_x_pan && self.lock_y_pan {
                    return;
                }

                let start_x = self.current_window_x(x_axis);
                let start_y = self.current_window_y(y_axis);

                self.pan_drag = Some(PanDrag {
                    x_axis,
                    y_axis,
                    start_pos: *position,
                    start_x,
                    start_y,
                });

                cx.request_focus(cx.node);
                cx.capture_pointer(cx.node);
                cx.stop_propagation();
            }
            Event::Pointer(PointerEvent::Up {
                position,
                button,
                modifiers,
                ..
            }) => {
                self.last_pointer_pos = Some(*position);
                if let Some(drag) = self.box_zoom_drag
                    && drag.button == *button
                {
                    self.box_zoom_drag = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }

                    let layout = self.compute_layout(cx.bounds);
                    let plot = layout.plot;
                    let width = plot.size.width.0;
                    let height = plot.size.height.0;
                    if width > 0.0 && height > 0.0 {
                        let start_local = Point::new(
                            Px(drag.start_pos.x.0 - plot.origin.x.0),
                            Px(drag.start_pos.y.0 - plot.origin.y.0),
                        );
                        let end_local = Point::new(
                            Px(drag.current_pos.x.0 - plot.origin.x.0),
                            Px(drag.current_pos.y.0 - plot.origin.y.0),
                        );

                        let (start_local, end_local) = Self::apply_box_select_modifiers(
                            plot.size,
                            start_local,
                            end_local,
                            *modifiers,
                            self.input_map.box_zoom_expand_x,
                            self.input_map.box_zoom_expand_y,
                            drag.required_mods,
                        );

                        let w = (start_local.x.0 - end_local.x.0).abs();
                        let h = (start_local.y.0 - end_local.y.0).abs();
                        if w >= 4.0 && h >= 4.0 {
                            let mut x = None;
                            let mut y = None;

                            if self.axis_is_fixed(drag.x_axis).is_none() {
                                if self.lock_x_zoom {
                                    // no-op: keep existing X window
                                } else {
                                    let x0 = start_local.x.0.min(end_local.x.0).clamp(0.0, width);
                                    let x1 = start_local.x.0.max(end_local.x.0).clamp(0.0, width);
                                    let min = Self::data_at_x_px(drag.start_x, x0, width);
                                    let max = Self::data_at_x_px(drag.start_x, x1, width);
                                    let mut window = DataWindow { min, max };
                                    window.clamp_non_degenerate();
                                    let (locked_min, locked_max) =
                                        self.axis_constraints(drag.x_axis);
                                    x = Some(window.apply_constraints(locked_min, locked_max));
                                }
                            }

                            if self.axis_is_fixed(drag.y_axis).is_none() {
                                if self.lock_y_zoom {
                                    // no-op: keep existing Y window
                                } else {
                                    let y0 = start_local.y.0.min(end_local.y.0).clamp(0.0, height);
                                    let y1 = start_local.y.0.max(end_local.y.0).clamp(0.0, height);
                                    let y0_from_bottom = height - y1;
                                    let y1_from_bottom = height - y0;
                                    let min = Self::data_at_y_px_from_bottom(
                                        drag.start_y,
                                        y0_from_bottom,
                                        height,
                                    );
                                    let max = Self::data_at_y_px_from_bottom(
                                        drag.start_y,
                                        y1_from_bottom,
                                        height,
                                    );
                                    let mut window = DataWindow { min, max };
                                    window.clamp_non_degenerate();
                                    let (locked_min, locked_max) =
                                        self.axis_constraints(drag.y_axis);
                                    y = Some(window.apply_constraints(locked_min, locked_max));
                                }
                            }

                            if let Some(x) = x {
                                self.set_data_window_x(drag.x_axis, Some(x));
                            }
                            if let Some(y) = y {
                                self.set_data_window_y(drag.y_axis, Some(y));
                            }
                        }
                    }

                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if self.pan_drag.is_some() && *button == MouseButton::Left {
                    self.pan_drag = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
            }
            Event::Pointer(PointerEvent::Wheel {
                position,
                delta,
                modifiers,
                ..
            }) => {
                self.last_pointer_pos = Some(*position);
                let Some((x_axis, y_axis)) = self.primary_axes() else {
                    return;
                };

                let layout = self.compute_layout(cx.bounds);
                let plot = layout.plot;
                let width = plot.size.width.0;
                let height = plot.size.height.0;
                if width <= 0.0 || height <= 0.0 {
                    return;
                }

                let delta_y = delta.y.0;
                if !delta_y.is_finite() {
                    return;
                }

                if let Some(required) = self.input_map.wheel_zoom_mod
                    && !required.is_pressed(*modifiers)
                {
                    return;
                }

                // Match ImPlot's default feel: zoom factor ~= 2^(delta_y * 0.0025)
                let log2_scale = delta_y * 0.0025;

                let in_plot = plot.contains(*position);
                let in_x_axis = layout.x_axis.contains(*position);
                let in_y_axis = layout.y_axis.contains(*position);
                if !in_plot && !in_x_axis && !in_y_axis {
                    return;
                }

                let local_x = (position.x.0 - plot.origin.x.0).clamp(0.0, width);
                let local_y = (position.y.0 - plot.origin.y.0).clamp(0.0, height);
                let center_x = local_x;
                let center_y_from_bottom = height - local_y;

                let zoom_x = if in_x_axis {
                    true
                } else if in_y_axis {
                    false
                } else {
                    !modifiers.ctrl
                };
                let zoom_y = if in_x_axis {
                    false
                } else if in_y_axis {
                    true
                } else {
                    !modifiers.shift
                };

                let next_x = zoom_x.then(|| {
                    if self.lock_x_zoom {
                        return None;
                    }
                    let w = self.current_window_x(x_axis);
                    let (locked_min, locked_max) = self.axis_constraints(x_axis);
                    Some(
                        w.zoom_by_px(center_x, log2_scale, width)
                            .apply_constraints(locked_min, locked_max),
                    )
                });
                let mut next_x = next_x.flatten();

                let next_y = zoom_y.then(|| {
                    if self.lock_y_zoom {
                        return None;
                    }
                    let w = self.current_window_y(y_axis);
                    let (locked_min, locked_max) = self.axis_constraints(y_axis);
                    Some(
                        w.zoom_by_px(center_y_from_bottom, log2_scale, height)
                            .apply_constraints(locked_min, locked_max),
                    )
                });
                let mut next_y = next_y.flatten();

                if self.axis_is_fixed(x_axis).is_some() {
                    next_x = None;
                }
                if self.axis_is_fixed(y_axis).is_some() {
                    next_y = None;
                }

                if next_x.is_none() && next_y.is_none() {
                    return;
                }

                if let Some(x) = next_x {
                    self.set_data_window_x(x_axis, Some(x));
                }
                if let Some(y) = next_y {
                    self.set_data_window_y(y_axis, Some(y));
                }

                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> fret_core::Size {
        self.last_bounds = cx.bounds;
        self.last_layout = self.compute_layout(cx.bounds);
        self.sync_viewport(self.last_layout.plot);
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.last_bounds = cx.bounds;
        self.last_layout = self.compute_layout(cx.bounds);
        self.sync_viewport(self.last_layout.plot);

        // P0: run the engine synchronously for now.
        let mut measurer = NullTextMeasurer::default();
        let _ = self
            .engine
            .step(&mut measurer, WorkBudget::new(u32::MAX, 0, u32::MAX))
            .map_err(|_e: EngineError| ());

        self.rebuild_paths_if_needed(cx);
        self.clear_tooltip_text_cache(cx.services);

        if let Some(background) = self.style.background {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(self.style.draw_order.0.saturating_sub(1)),
                rect: self.last_layout.bounds,
                background,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        }

        cx.scene.push(SceneOp::PushClipRect {
            rect: self.last_layout.plot,
        });

        for cached in self.cached_paths.values() {
            cx.scene.push(SceneOp::Path {
                order: self.style.draw_order,
                origin: self.last_layout.plot.origin,
                path: cached.path,
                color: self.style.stroke_color,
            });
        }

        let pointer_pos = self.last_pointer_pos;
        let hover_hit = self.engine.output().hover;

        let interaction_idle = self.pan_drag.is_none() && self.box_zoom_drag.is_none();
        let hover_radius_px = self.style.hover_point_size.0.max(4.0) * 3.0;
        let hover_active = interaction_idle
            && pointer_pos.is_some_and(|pos| self.last_layout.plot.contains(pos))
            && hover_hit.is_some_and(|hit| hit.dist2_px <= hover_radius_px * hover_radius_px);

        if hover_active && let (Some(pos), Some(hit)) = (pointer_pos, hover_hit) {
            let overlay_order = DrawOrder(self.style.draw_order.0.saturating_add(2));
            let point_order = DrawOrder(self.style.draw_order.0.saturating_add(3));

            let plot = self.last_layout.plot;
            let crosshair_w = self.style.crosshair_width.0.max(1.0);

            let x = pos
                .x
                .0
                .clamp(plot.origin.x.0, plot.origin.x.0 + plot.size.width.0);
            let y = pos
                .y
                .0
                .clamp(plot.origin.y.0, plot.origin.y.0 + plot.size.height.0);

            cx.scene.push(SceneOp::Quad {
                order: overlay_order,
                rect: Rect::new(
                    Point::new(Px(x - 0.5 * crosshair_w), plot.origin.y),
                    Size::new(Px(crosshair_w), plot.size.height),
                ),
                background: self.style.crosshair_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
            cx.scene.push(SceneOp::Quad {
                order: overlay_order,
                rect: Rect::new(
                    Point::new(plot.origin.x, Px(y - 0.5 * crosshair_w)),
                    Size::new(plot.size.width, Px(crosshair_w)),
                ),
                background: self.style.crosshair_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });

            let r = self.style.hover_point_size.0.max(1.0);
            cx.scene.push(SceneOp::Quad {
                order: point_order,
                rect: Rect::new(
                    Point::new(Px(hit.point_px.x.0 - r), Px(hit.point_px.y.0 - r)),
                    Size::new(Px(2.0 * r), Px(2.0 * r)),
                ),
                background: self.style.hover_point_color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        }

        if let Some(drag) = self.box_zoom_drag {
            let rect =
                rect_from_points_clamped(self.last_layout.plot, drag.start_pos, drag.current_pos);
            if rect.size.width.0 >= 1.0 && rect.size.height.0 >= 1.0 {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(self.style.draw_order.0.saturating_add(1)),
                    rect,
                    background: self.style.selection_fill,
                    border: Edges::all(self.style.selection_stroke_width),
                    border_color: self.style.selection_stroke,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }
        }

        cx.scene.push(SceneOp::PopClip);

        if hover_active
            && let Some(hit) = hover_hit
            && let Some((x_axis, y_axis)) = self.primary_axes()
        {
            let x_window = self.current_window_x(x_axis);
            let y_window = self.current_window_y(y_axis);

            let tooltip_values = (|| {
                let series = self.engine.model().series.get(&hit.series)?;
                let dataset_id = series.dataset;
                let x_col = series.x_col;
                let y_col = series.y_col;

                let table = self
                    .engine
                    .datasets_mut()
                    .datasets
                    .iter()
                    .find_map(|(id, t)| (*id == dataset_id).then_some(t))?;

                let idx = hit.data_index as usize;
                let x = table.column_f64(x_col)?;
                let y = table.column_f64(y_col)?;
                if idx >= x.len() || idx >= y.len() {
                    return None;
                }

                Some((x[idx], y[idx]))
            })();

            if let Some((x_value, y_value)) = tooltip_values {
                let x_label = format_tick_value(x_window, x_value);
                let y_label = format_tick_value(y_window, y_value);
                let label = format!("series: {}  x: {}  y: {}", hit.series.0, x_label, y_label);

                let text_style = TextStyle {
                    size: Px(12.0),
                    weight: FontWeight::NORMAL,
                    ..TextStyle::default()
                };
                let constraints = TextConstraints {
                    max_width: None,
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor: cx.scale_factor,
                };
                let (blob, metrics) = cx.services.text().prepare(&label, &text_style, constraints);

                let pad = self.style.tooltip_padding;
                let w = (metrics.size.width.0 + pad.left.0 + pad.right.0).max(1.0);
                let h = (metrics.size.height.0 + pad.top.0 + pad.bottom.0).max(1.0);

                let bounds = self.last_layout.bounds;
                let x0 = bounds.origin.x.0;
                let y0 = bounds.origin.y.0;
                let x1 = x0 + bounds.size.width.0;
                let y1 = y0 + bounds.size.height.0;

                let offset = 10.0f32;
                let mut tip_x = hit.point_px.x.0 + offset;
                let mut tip_y = hit.point_px.y.0 - h - offset;

                if tip_x + w > x1 {
                    tip_x = hit.point_px.x.0 - w - offset;
                }
                if tip_y < y0 {
                    tip_y = hit.point_px.y.0 + offset;
                }

                if w < bounds.size.width.0 {
                    tip_x = tip_x.clamp(x0, x1 - w);
                } else {
                    tip_x = x0;
                }
                if h < bounds.size.height.0 {
                    tip_y = tip_y.clamp(y0, y1 - h);
                } else {
                    tip_y = y0;
                }

                let tooltip_order = DrawOrder(self.style.draw_order.0.saturating_add(20));
                cx.scene.push(SceneOp::Quad {
                    order: tooltip_order,
                    rect: Rect::new(Point::new(Px(tip_x), Px(tip_y)), Size::new(Px(w), Px(h))),
                    background: self.style.tooltip_background,
                    border: Edges::all(self.style.tooltip_border_width),
                    border_color: self.style.tooltip_border_color,
                    corner_radii: Corners::all(self.style.tooltip_corner_radius),
                });

                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(tooltip_order.0.saturating_add(1)),
                    origin: Point::new(Px(tip_x + pad.left.0), Px(tip_y + pad.top.0)),
                    text: blob,
                    color: self.style.tooltip_text_color,
                });

                self.tooltip_text.push(blob);
            }
        }

        self.draw_axes(cx);
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for cached in self.cached_paths.values() {
            services.path().release(cached.path);
        }
        self.cached_paths.clear();

        for blob in self.axis_text.drain(..) {
            services.text().release(blob);
        }
        for blob in self.tooltip_text.drain(..) {
            services.text().release(blob);
        }
    }
}

fn rect_from_points_clamped(bounds: Rect, a: Point, b: Point) -> Rect {
    let x0 =
        a.x.0
            .min(b.x.0)
            .clamp(bounds.origin.x.0, bounds.origin.x.0 + bounds.size.width.0);
    let x1 =
        a.x.0
            .max(b.x.0)
            .clamp(bounds.origin.x.0, bounds.origin.x.0 + bounds.size.width.0);
    let y0 =
        a.y.0
            .min(b.y.0)
            .clamp(bounds.origin.y.0, bounds.origin.y.0 + bounds.size.height.0);
    let y1 =
        a.y.0
            .max(b.y.0)
            .clamp(bounds.origin.y.0, bounds.origin.y.0 + bounds.size.height.0);

    Rect::new(
        Point::new(Px(x0), Px(y0)),
        Size::new(Px((x1 - x0).max(0.0)), Px((y1 - y0).max(0.0))),
    )
}

fn nice_ticks(window: DataWindow, target_count: usize) -> Vec<f64> {
    let mut out = Vec::new();

    let mut window = window;
    window.clamp_non_degenerate();
    let span = window.span();

    if !span.is_finite() || span <= 0.0 || target_count == 0 {
        out.push(window.min);
        out.push(window.max);
        return out;
    }

    if target_count == 1 {
        out.push(0.5 * (window.min + window.max));
        return out;
    }

    let step = nice_step(span / (target_count as f64 - 1.0));
    if !step.is_finite() || step <= 0.0 {
        out.push(window.min);
        out.push(window.max);
        return out;
    }

    let start = (window.min / step).floor() * step;
    let end = (window.max / step).ceil() * step;
    if !start.is_finite() || !end.is_finite() || end < start {
        out.push(window.min);
        out.push(window.max);
        return out;
    }

    let mut v = start;
    // Prevent pathological loops due to floating rounding.
    let max_steps = 10_000usize;
    for _ in 0..max_steps {
        if v > end + step * 0.5 {
            break;
        }
        let clamped = v.clamp(window.min, window.max);
        if out
            .last()
            .is_none_or(|prev| (clamped - *prev).abs() > step * 0.1)
        {
            out.push(clamped);
        }
        v += step;
    }

    if out.is_empty() {
        out.push(window.min);
        out.push(window.max);
    } else {
        let first = *out.first().unwrap_or(&window.min);
        let last = *out.last().unwrap_or(&window.max);
        if (first - window.min).abs() > step * 0.1 {
            out.insert(0, window.min);
        }
        if (last - window.max).abs() > step * 0.1 {
            out.push(window.max);
        }
    }

    out
}

fn nice_step(raw_step: f64) -> f64 {
    if !raw_step.is_finite() || raw_step <= 0.0 {
        return 0.0;
    }

    let exponent = raw_step.abs().log10().floor();
    let base = 10f64.powf(exponent);
    if !base.is_finite() || base <= 0.0 {
        return raw_step;
    }

    let fraction = raw_step / base;
    let nice_fraction = if fraction <= 1.0 {
        1.0
    } else if fraction <= 2.0 {
        2.0
    } else if fraction <= 5.0 {
        5.0
    } else {
        10.0
    };

    nice_fraction * base
}

fn format_tick_value(window: DataWindow, value: f64) -> String {
    let mut window = window;
    window.clamp_non_degenerate();

    let span = window.span().abs();
    if !span.is_finite() || span <= 0.0 {
        return format!("{value}");
    }

    let step = nice_step(span / 4.0);
    let digits = if step.is_finite() && step > 0.0 {
        let log10 = step.abs().log10();
        (-log10).ceil().clamp(0.0, 8.0) as usize
    } else {
        3
    };

    let mut s = format!("{value:.digits$}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_mapping_is_monotonic() {
        let window = DataWindow {
            min: 10.0,
            max: 20.0,
        };
        let a = ChartCanvas::data_at_x_px(window, 0.0, 100.0);
        let b = ChartCanvas::data_at_x_px(window, 50.0, 100.0);
        let c = ChartCanvas::data_at_x_px(window, 100.0, 100.0);
        assert!(a < b && b < c);
        assert_eq!(a, 10.0);
        assert_eq!(c, 20.0);

        let d = ChartCanvas::data_at_y_px_from_bottom(window, 0.0, 100.0);
        let e = ChartCanvas::data_at_y_px_from_bottom(window, 100.0, 100.0);
        assert_eq!(d, 10.0);
        assert_eq!(e, 20.0);
    }

    #[test]
    fn rect_from_points_is_clamped_to_bounds() {
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(100.0), Px(200.0)),
        );
        let a = Point::new(Px(0.0), Px(0.0));
        let b = Point::new(Px(999.0), Px(999.0));
        let rect = rect_from_points_clamped(bounds, a, b);
        assert_eq!(rect.origin, bounds.origin);
        assert_eq!(rect.size, bounds.size);
    }

    #[test]
    fn nice_ticks_include_endpoints() {
        let window = DataWindow { min: 0.2, max: 9.7 };
        let ticks = nice_ticks(window, 5);
        assert!(!ticks.is_empty());
        assert_eq!(*ticks.first().unwrap(), window.min);
        assert_eq!(*ticks.last().unwrap(), window.max);
    }
}
