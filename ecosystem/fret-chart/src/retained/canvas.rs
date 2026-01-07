use std::collections::BTreeMap;

use delinea::engine::EngineError;
use delinea::engine::model::{ChartPatch, ModelError, PatchMode};
use delinea::engine::window::DataWindow;
use delinea::marks::{MarkKind, MarkPayloadRef};
use delinea::text::{TextMeasurer, TextMetrics};
use delinea::{Action, ChartEngine, WorkBudget};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, MouseButton, PathCommand, PathConstraints, PathStyle,
    Point, PointerEvent, Px, Rect, SceneOp, StrokeStyle,
};
use fret_ui::UiHost;
use fret_ui::retained_bridge::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};

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

pub struct ChartCanvas {
    engine: ChartEngine,
    style: ChartStyle,
    last_bounds: Rect,
    last_marks_rev: delinea::ids::Revision,
    last_scale_factor_bits: u32,
    cached_paths: BTreeMap<delinea::ids::MarkId, CachedPath>,
    pan_drag: Option<PanDrag>,
}

impl ChartCanvas {
    pub fn new(spec: delinea::ChartSpec) -> Result<Self, ModelError> {
        Ok(Self {
            engine: ChartEngine::new(spec)?,
            style: ChartStyle::default(),
            last_bounds: Rect::default(),
            last_marks_rev: delinea::ids::Revision::default(),
            last_scale_factor_bits: 0,
            cached_paths: BTreeMap::default(),
            pan_drag: None,
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

    pub fn create_node<H: UiHost>(ui: &mut fret_ui::UiTree<H>, canvas: Self) -> fret_core::NodeId {
        use fret_ui::retained_bridge::UiTreeRetainedExt as _;
        ui.create_node_retained(canvas)
    }

    fn sync_viewport(&mut self, bounds: Rect) {
        if self.engine.model().viewport == Some(bounds) {
            return;
        }
        let _ = self.engine.apply_patch(
            ChartPatch {
                viewport: Some(Some(bounds)),
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

    fn apply_view_window_2d(
        &mut self,
        x_axis: delinea::AxisId,
        y_axis: delinea::AxisId,
        x: Option<DataWindow>,
        y: Option<DataWindow>,
    ) {
        self.engine.apply_action(Action::SetViewWindow2D {
            x_axis,
            y_axis,
            x,
            y,
        });
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
        let origin = self.last_bounds.origin;

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
            Event::Pointer(PointerEvent::Move {
                position, buttons, ..
            }) => {
                if let Some(drag) = self.pan_drag
                    && buttons.left
                    && cx.captured == Some(cx.node)
                {
                    let bounds = cx.bounds;
                    let width = bounds.size.width.0;
                    let height = bounds.size.height.0;
                    if width <= 0.0 || height <= 0.0 {
                        return;
                    }

                    let dx = position.x.0 - drag.start_pos.x.0;
                    let dy = position.y.0 - drag.start_pos.y.0;

                    let mut next_x = drag.start_x.pan_by_px(dx, width);
                    let mut next_y = drag.start_y.pan_by_px(-dy, height);

                    let (x_locked_min, x_locked_max) = self.axis_constraints(drag.x_axis);
                    let (y_locked_min, y_locked_max) = self.axis_constraints(drag.y_axis);
                    next_x = next_x.apply_constraints(x_locked_min, x_locked_max);
                    next_y = next_y.apply_constraints(y_locked_min, y_locked_max);

                    self.apply_view_window_2d(drag.x_axis, drag.y_axis, Some(next_x), Some(next_y));

                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                } else {
                    self.engine
                        .apply_action(Action::HoverAt { point: *position });
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
            }
            Event::Pointer(PointerEvent::Down {
                position,
                button,
                modifiers,
                ..
            }) => {
                if self.pan_drag.is_some() {
                    return;
                }
                if *button != MouseButton::Left {
                    return;
                }
                if modifiers.shift
                    || modifiers.ctrl
                    || modifiers.alt
                    || modifiers.meta
                    || modifiers.alt_gr
                {
                    return;
                }

                let Some((x_axis, y_axis)) = self.primary_axes() else {
                    return;
                };
                if self.axis_is_fixed(x_axis).is_some() || self.axis_is_fixed(y_axis).is_some() {
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

                cx.capture_pointer(cx.node);
                cx.stop_propagation();
            }
            Event::Pointer(PointerEvent::Up { button, .. }) => {
                if *button != MouseButton::Left {
                    return;
                }
                if self.pan_drag.take().is_some() && cx.captured == Some(cx.node) {
                    cx.release_pointer_capture();
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
                let Some((x_axis, y_axis)) = self.primary_axes() else {
                    return;
                };

                let bounds = cx.bounds;
                let width = bounds.size.width.0;
                let height = bounds.size.height.0;
                if width <= 0.0 || height <= 0.0 {
                    return;
                }

                let delta_y = delta.y.0;
                if !delta_y.is_finite() {
                    return;
                }

                // Match ImPlot's default feel: zoom factor ~= 2^(delta_y * 0.0025)
                let log2_scale = delta_y * 0.0025;

                let local_x = position.x.0 - bounds.origin.x.0;
                let local_y = position.y.0 - bounds.origin.y.0;
                let center_x = local_x;
                let center_y_from_bottom = height - local_y;

                let zoom_x = !modifiers.ctrl;
                let zoom_y = !modifiers.shift;

                let mut next_x = zoom_x.then(|| {
                    let w = self.current_window_x(x_axis);
                    let (locked_min, locked_max) = self.axis_constraints(x_axis);
                    w.zoom_by_px(center_x, log2_scale, width)
                        .apply_constraints(locked_min, locked_max)
                });

                let mut next_y = zoom_y.then(|| {
                    let w = self.current_window_y(y_axis);
                    let (locked_min, locked_max) = self.axis_constraints(y_axis);
                    w.zoom_by_px(center_y_from_bottom, log2_scale, height)
                        .apply_constraints(locked_min, locked_max)
                });

                if self.axis_is_fixed(x_axis).is_some() {
                    next_x = None;
                }
                if self.axis_is_fixed(y_axis).is_some() {
                    next_y = None;
                }

                if next_x.is_none() && next_y.is_none() {
                    return;
                }

                self.apply_view_window_2d(x_axis, y_axis, next_x, next_y);

                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> fret_core::Size {
        self.last_bounds = cx.bounds;
        self.sync_viewport(cx.bounds);
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.last_bounds = cx.bounds;
        self.sync_viewport(cx.bounds);

        // P0: run the engine synchronously for now.
        let mut measurer = NullTextMeasurer::default();
        let _ = self
            .engine
            .step(&mut measurer, WorkBudget::new(u32::MAX, 0, u32::MAX))
            .map_err(|_e: EngineError| ());

        self.rebuild_paths_if_needed(cx);

        if let Some(background) = self.style.background {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(self.style.draw_order.0.saturating_sub(1)),
                rect: self.last_bounds,
                background,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        }

        cx.scene.push(SceneOp::PushClipRect {
            rect: self.last_bounds,
        });

        for cached in self.cached_paths.values() {
            cx.scene.push(SceneOp::Path {
                order: self.style.draw_order,
                origin: self.last_bounds.origin,
                path: cached.path,
                color: self.style.stroke_color,
            });
        }

        cx.scene.push(SceneOp::PopClip);
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for cached in self.cached_paths.values() {
            services.path().release(cached.path);
        }
        self.cached_paths.clear();
    }
}
