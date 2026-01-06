//! Retained plot canvas implementation.

mod cache;
mod constraints;
mod readout;
mod util;

use self::cache::{LegendEntry, PreparedText};
pub use self::constraints::AxisConstraints;
use self::constraints::constrain_view_bounds_scaled;
use self::readout::apply_readout_policy;
pub(super) use self::util::contains_point;
use self::util::{dim_color, offset_rect, overlay_rect_in_plot};

use fret_core::geometry::{Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, SceneOp};
use fret_core::{
    Event, FontId, FontWeight, KeyCode, MouseButton, PathId, PointerEvent, SemanticsRole,
    TextBlobId, TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap, UiServices,
};
use fret_runtime::{Model, TextFontStackKey};
use fret_ui::UiHost;
use fret_ui::retained_bridge::{
    Invalidation, LayoutCx, PaintCx, SemanticsCx, UiTreeRetainedExt, Widget,
};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use super::YAxis;
use super::layers::resolve_series_color;
use super::layers::{
    PlotCursorReadoutArgs, PlotHitTestArgs, PlotHover, PlotLayer, PlotPaintArgs, PlotQuad,
    SeriesMeta,
};
use super::layout::{PlotLayout, PlotRegion};
use super::state::{PlotHoverOutput, PlotOutput, PlotOutputSnapshot, PlotState};
use super::style::{LinePlotStyle, MouseReadoutMode};

use crate::cartesian::{AxisScale, DataPoint, DataRect, PlotTransform};
use crate::input_map::{ModifierKey, ModifiersMask, PlotInputMap};
use crate::plot::axis::{AxisLabelFormat, AxisLabelFormatter, AxisTicks, axis_ticks_scaled};
use crate::plot::view::{
    clamp_view_to_data_scaled, clamp_zoom_factors, data_rect_from_plot_points_scaled,
    expand_data_bounds_scaled, local_from_absolute, pan_view_by_px_scaled, sanitize_data_rect,
    sanitize_data_rect_scaled, zoom_view_at_px_scaled,
};
use crate::series::SeriesId;

fn query_rect_from_plot_points_raw(
    view_bounds: DataRect,
    viewport: Size,
    a: Point,
    b: Point,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<DataRect> {
    let viewport_w = viewport.width.0;
    let viewport_h = viewport.height.0;
    if !viewport_w.is_finite() || !viewport_h.is_finite() || viewport_w <= 0.0 || viewport_h <= 0.0
    {
        return None;
    }

    let x0 = a.x.0.min(b.x.0).clamp(0.0, viewport_w);
    let x1 = a.x.0.max(b.x.0).clamp(0.0, viewport_w);
    let y0 = a.y.0.min(b.y.0).clamp(0.0, viewport_h);
    let y1 = a.y.0.max(b.y.0).clamp(0.0, viewport_h);

    let transform = PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), viewport),
        data: view_bounds,
        x_scale,
        y_scale,
    };

    let da = transform.px_to_data(Point::new(Px(x0), Px(y0)));
    let db = transform.px_to_data(Point::new(Px(x1), Px(y1)));

    if !da.x.is_finite() || !da.y.is_finite() || !db.x.is_finite() || !db.y.is_finite() {
        return None;
    }

    Some(DataRect {
        x_min: da.x.min(db.x),
        x_max: da.x.max(db.x),
        y_min: da.y.min(db.y),
        y_max: da.y.max(db.y),
    })
}

fn apply_axis_locks(
    view_before: DataRect,
    mut next: DataRect,
    lock_x: bool,
    lock_y: bool,
) -> DataRect {
    if lock_x {
        next.x_min = view_before.x_min;
        next.x_max = view_before.x_max;
    }
    if lock_y {
        next.y_min = view_before.y_min;
        next.y_max = view_before.y_max;
    }
    next
}

fn log10_decade_exponent(v: f64) -> Option<i32> {
    if !v.is_finite() || v <= 0.0 {
        return None;
    }
    let e = v.log10();
    if !e.is_finite() {
        return None;
    }

    let rounded = e.round();
    let eps = 1.0e-10_f64;
    ((e - rounded).abs() <= eps).then_some(rounded as i32)
}

fn log10_tick_label_or_empty(v: f64) -> String {
    let Some(exp) = log10_decade_exponent(v) else {
        return String::new();
    };
    format!("10^{exp}")
}

#[derive(Debug, Clone, Copy, Default)]
struct AxisLock {
    pan: bool,
    zoom: bool,
}

#[derive(Debug)]
pub struct PlotCanvas<L: PlotLayer + 'static> {
    model: Model<L::Model>,
    style: LinePlotStyle,
    input_map: PlotInputMap,
    x_axis_ticks: AxisTicks,
    y_axis_ticks: AxisTicks,
    y2_axis_ticks: AxisTicks,
    y3_axis_ticks: AxisTicks,
    y4_axis_ticks: AxisTicks,
    x_scale: AxisScale,
    y_scale: AxisScale,
    y2_scale: AxisScale,
    y3_scale: AxisScale,
    y4_scale: AxisScale,
    x_axis_labels: AxisLabelFormatter,
    y_axis_labels: AxisLabelFormatter,
    y2_axis_labels: AxisLabelFormatter,
    y3_axis_labels: AxisLabelFormatter,
    y4_axis_labels: AxisLabelFormatter,
    tooltip_x_labels: AxisLabelFormatter,
    tooltip_y_labels: AxisLabelFormatter,
    layer: L,
    hover: Option<PlotHover>,
    plot_state: PlotState,
    plot_state_model: Option<Model<PlotState>>,
    plot_output: PlotOutput,
    plot_output_model: Option<Model<PlotOutput>>,
    legend_hover: Option<SeriesId>,
    cursor_px: Option<Point>,
    last_pointer_pos: Option<Point>,
    last_scale_factor: f32,
    x_axis_thickness: Px,
    y_axis_thickness: Px,
    y_axis_right_thickness: Px,
    y_axis_right2_thickness: Px,
    y_axis_right3_thickness: Px,
    show_y2_axis: bool,
    show_y3_axis: bool,
    show_y4_axis: bool,
    lock_x: AxisLock,
    lock_y: AxisLock,
    lock_y2: AxisLock,
    lock_y3: AxisLock,
    lock_y4: AxisLock,
    x_constraints: AxisConstraints,
    y_constraints: AxisConstraints,
    y2_constraints: AxisConstraints,
    y3_constraints: AxisConstraints,
    y4_constraints: AxisConstraints,
    pan_button: Option<MouseButton>,
    pan_target: Option<PlotRegion>,
    pan_start_pos: Option<Point>,
    pan_last_pos: Option<Point>,
    box_zoom_start: Option<Point>,
    box_zoom_current: Option<Point>,
    box_zoom_button: Option<MouseButton>,
    box_zoom_required_mods: Option<ModifiersMask>,
    query_drag_button: Option<MouseButton>,
    query_drag_start: Option<Point>,
    query_drag_current: Option<Point>,
    axis_label_key: Option<u64>,
    axis_ticks_x: Vec<f64>,
    axis_ticks_y: Vec<f64>,
    axis_ticks_y2: Vec<f64>,
    axis_ticks_y3: Vec<f64>,
    axis_ticks_y4: Vec<f64>,
    axis_labels_x: Vec<PreparedText>,
    axis_labels_y: Vec<PreparedText>,
    axis_labels_y2: Vec<PreparedText>,
    axis_labels_y3: Vec<PreparedText>,
    axis_labels_y4: Vec<PreparedText>,
    legend_key: Option<u64>,
    legend_entries: Vec<LegendEntry>,
    tooltip_text: Option<PreparedText>,
    mouse_readout_text: Option<PreparedText>,
    linked_cursor_readout_text: Option<PreparedText>,
}

#[cfg(test)]
mod box_select_modifier_tests {
    use super::*;

    #[test]
    fn box_select_modifiers_expand_to_edges() {
        let plot_size = Size::new(Px(100.0), Px(50.0));
        let start = Point::new(Px(10.0), Px(10.0));
        let end = Point::new(Px(20.0), Px(20.0));
        let expand_x = Some(ModifierKey::Alt);
        let expand_y = Some(ModifierKey::Shift);

        let mods_x = fret_core::Modifiers {
            alt: true,
            ..fret_core::Modifiers::default()
        };
        let (sx, ex) = PlotCanvas::<crate::retained::LinePlotLayer>::apply_box_select_modifiers(
            plot_size,
            start,
            end,
            mods_x,
            expand_x,
            expand_y,
            ModifiersMask::NONE,
        );
        assert_eq!(sx.x.0, 0.0);
        assert_eq!(ex.x.0, 100.0);
        assert_eq!(sx.y.0, 10.0);
        assert_eq!(ex.y.0, 20.0);

        let mods_y = fret_core::Modifiers {
            shift: true,
            ..fret_core::Modifiers::default()
        };
        let (sy, ey) = PlotCanvas::<crate::retained::LinePlotLayer>::apply_box_select_modifiers(
            plot_size,
            start,
            end,
            mods_y,
            expand_x,
            expand_y,
            ModifiersMask::NONE,
        );
        assert_eq!(sy.y.0, 0.0);
        assert_eq!(ey.y.0, 50.0);
        assert_eq!(sy.x.0, 10.0);
        assert_eq!(ey.x.0, 20.0);

        let mods_xy = fret_core::Modifiers {
            alt: true,
            shift: true,
            ..fret_core::Modifiers::default()
        };
        let (sxy, exy) = PlotCanvas::<crate::retained::LinePlotLayer>::apply_box_select_modifiers(
            plot_size,
            start,
            end,
            mods_xy,
            expand_x,
            expand_y,
            ModifiersMask::NONE,
        );
        assert_eq!((sxy.x.0, sxy.y.0), (0.0, 0.0));
        assert_eq!((exy.x.0, exy.y.0), (100.0, 50.0));

        let required_shift = ModifiersMask {
            shift: true,
            ..ModifiersMask::NONE
        };
        let (s_req, e_req) =
            PlotCanvas::<crate::retained::LinePlotLayer>::apply_box_select_modifiers(
                plot_size,
                start,
                end,
                mods_y,
                expand_x,
                expand_y,
                required_shift,
            );
        assert_eq!((s_req.x.0, s_req.y.0), (10.0, 10.0));
        assert_eq!((e_req.x.0, e_req.y.0), (20.0, 20.0));
    }
}

impl<L: PlotLayer + 'static> PlotCanvas<L> {
    pub(super) fn with_layer_mut(mut self, f: impl FnOnce(&mut L)) -> Self {
        f(&mut self.layer);
        self
    }

    fn apply_box_select_modifiers(
        plot_size: Size,
        start: Point,
        end: Point,
        modifiers: fret_core::Modifiers,
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

    fn ensure_required_axes_enabled<H: UiHost>(&mut self, app: &mut H) {
        if !self.show_y2_axis {
            let has_y2 = self
                .model
                .read(app, |_app, m| L::data_bounds_y2(m).is_some())
                .unwrap_or(false);
            if has_y2 {
                self.show_y2_axis = true;
            }
        }
        if !self.show_y3_axis {
            let has_y3 = self
                .model
                .read(app, |_app, m| L::data_bounds_y3(m).is_some())
                .unwrap_or(false);
            if has_y3 {
                self.show_y3_axis = true;
            }
        }
        if !self.show_y4_axis {
            let has_y4 = self
                .model
                .read(app, |_app, m| L::data_bounds_y4(m).is_some())
                .unwrap_or(false);
            if has_y4 {
                self.show_y4_axis = true;
            }
        }
    }

    fn fit_view_to_data_now<H: UiHost>(
        &self,
        app: &mut H,
    ) -> (
        DataRect,
        Option<DataRect>,
        Option<DataRect>,
        Option<DataRect>,
    ) {
        let data_bounds = self.read_data_bounds(app);
        let view = if self.style.clamp_to_data_bounds {
            expand_data_bounds_scaled(
                data_bounds,
                self.style.overscroll_fraction,
                self.x_scale,
                self.y_scale,
            )
        } else {
            sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y_scale)
        };
        let view = constrain_view_bounds_scaled(
            view,
            self.x_scale,
            self.y_scale,
            self.x_constraints,
            self.y_constraints,
        );

        let view_y2 = if self.show_y2_axis {
            self.read_data_bounds_y2(app).map(|data_bounds| {
                let y2_bounds = if self.style.clamp_to_data_bounds {
                    expand_data_bounds_scaled(
                        data_bounds,
                        self.style.overscroll_fraction,
                        self.x_scale,
                        self.y2_scale,
                    )
                } else {
                    sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y2_scale)
                };

                let view_y2 = sanitize_data_rect_scaled(
                    DataRect {
                        x_min: view.x_min,
                        x_max: view.x_max,
                        y_min: y2_bounds.y_min,
                        y_max: y2_bounds.y_max,
                    },
                    self.x_scale,
                    self.y2_scale,
                );

                constrain_view_bounds_scaled(
                    view_y2,
                    self.x_scale,
                    self.y2_scale,
                    self.x_constraints,
                    self.y2_constraints,
                )
            })
        } else {
            None
        };

        let view_y3 = if self.show_y3_axis {
            self.read_data_bounds_y3(app).map(|data_bounds| {
                let y3_bounds = if self.style.clamp_to_data_bounds {
                    expand_data_bounds_scaled(
                        data_bounds,
                        self.style.overscroll_fraction,
                        self.x_scale,
                        self.y3_scale,
                    )
                } else {
                    sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y3_scale)
                };

                let view_y3 = sanitize_data_rect_scaled(
                    DataRect {
                        x_min: view.x_min,
                        x_max: view.x_max,
                        y_min: y3_bounds.y_min,
                        y_max: y3_bounds.y_max,
                    },
                    self.x_scale,
                    self.y3_scale,
                );

                constrain_view_bounds_scaled(
                    view_y3,
                    self.x_scale,
                    self.y3_scale,
                    self.x_constraints,
                    self.y3_constraints,
                )
            })
        } else {
            None
        };

        let view_y4 = if self.show_y4_axis {
            self.read_data_bounds_y4(app).map(|data_bounds| {
                let y4_bounds = if self.style.clamp_to_data_bounds {
                    expand_data_bounds_scaled(
                        data_bounds,
                        self.style.overscroll_fraction,
                        self.x_scale,
                        self.y4_scale,
                    )
                } else {
                    sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y4_scale)
                };

                let view_y4 = sanitize_data_rect_scaled(
                    DataRect {
                        x_min: view.x_min,
                        x_max: view.x_max,
                        y_min: y4_bounds.y_min,
                        y_max: y4_bounds.y_max,
                    },
                    self.x_scale,
                    self.y4_scale,
                );

                constrain_view_bounds_scaled(
                    view_y4,
                    self.x_scale,
                    self.y4_scale,
                    self.x_constraints,
                    self.y4_constraints,
                )
            })
        } else {
            None
        };

        (view, view_y2, view_y3, view_y4)
    }

    pub fn with_layer(model: Model<L::Model>, layer: L) -> Self {
        let axis_gap = LinePlotStyle::default().axis_gap;
        Self {
            model,
            style: LinePlotStyle::default(),
            input_map: PlotInputMap::default(),
            x_axis_ticks: AxisTicks::default(),
            y_axis_ticks: AxisTicks::default(),
            y2_axis_ticks: AxisTicks::default(),
            y3_axis_ticks: AxisTicks::default(),
            y4_axis_ticks: AxisTicks::default(),
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
            y2_scale: AxisScale::Linear,
            y3_scale: AxisScale::Linear,
            y4_scale: AxisScale::Linear,
            x_axis_labels: AxisLabelFormatter::default(),
            y_axis_labels: AxisLabelFormatter::default(),
            y2_axis_labels: AxisLabelFormatter::default(),
            y3_axis_labels: AxisLabelFormatter::default(),
            y4_axis_labels: AxisLabelFormatter::default(),
            tooltip_x_labels: AxisLabelFormatter::default(),
            tooltip_y_labels: AxisLabelFormatter::default(),
            layer,
            hover: None,
            plot_state: PlotState::default(),
            plot_state_model: None,
            plot_output: PlotOutput::default(),
            plot_output_model: None,
            legend_hover: None,
            cursor_px: None,
            last_pointer_pos: None,
            last_scale_factor: 1.0,
            x_axis_thickness: axis_gap,
            y_axis_thickness: axis_gap,
            y_axis_right_thickness: Px(0.0),
            y_axis_right2_thickness: Px(0.0),
            y_axis_right3_thickness: Px(0.0),
            show_y2_axis: false,
            show_y3_axis: false,
            show_y4_axis: false,
            lock_x: AxisLock::default(),
            lock_y: AxisLock::default(),
            lock_y2: AxisLock::default(),
            lock_y3: AxisLock::default(),
            lock_y4: AxisLock::default(),
            x_constraints: AxisConstraints::default(),
            y_constraints: AxisConstraints::default(),
            y2_constraints: AxisConstraints::default(),
            y3_constraints: AxisConstraints::default(),
            y4_constraints: AxisConstraints::default(),
            pan_button: None,
            pan_target: None,
            pan_start_pos: None,
            pan_last_pos: None,
            box_zoom_start: None,
            box_zoom_current: None,
            box_zoom_button: None,
            box_zoom_required_mods: None,
            query_drag_button: None,
            query_drag_start: None,
            query_drag_current: None,
            axis_label_key: None,
            axis_ticks_x: Vec::new(),
            axis_ticks_y: Vec::new(),
            axis_ticks_y2: Vec::new(),
            axis_ticks_y3: Vec::new(),
            axis_ticks_y4: Vec::new(),
            axis_labels_x: Vec::new(),
            axis_labels_y: Vec::new(),
            axis_labels_y2: Vec::new(),
            axis_labels_y3: Vec::new(),
            axis_labels_y4: Vec::new(),
            legend_key: None,
            legend_entries: Vec::new(),
            tooltip_text: None,
            mouse_readout_text: None,
            linked_cursor_readout_text: None,
        }
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    pub fn input_map(mut self, input_map: PlotInputMap) -> Self {
        self.input_map = input_map;
        self
    }

    pub fn x_axis_format(mut self, format: AxisLabelFormat) -> Self {
        self.x_axis_ticks = format.ticks();
        let labels = format.labels();
        self.x_axis_labels = labels.clone();
        self.tooltip_x_labels = labels;
        self
    }

    pub fn y_axis_format(mut self, format: AxisLabelFormat) -> Self {
        self.y_axis_ticks = format.ticks();
        let labels = format.labels();
        self.y_axis_labels = labels.clone();
        self.tooltip_y_labels = labels;
        self
    }

    pub fn y2_axis_format(mut self, format: AxisLabelFormat) -> Self {
        self.show_y2_axis = true;
        self.y2_axis_ticks = format.ticks();
        self.y2_axis_labels = format.labels();
        self
    }

    pub fn y3_axis_format(mut self, format: AxisLabelFormat) -> Self {
        self.show_y3_axis = true;
        self.y3_axis_ticks = format.ticks();
        self.y3_axis_labels = format.labels();
        self
    }

    pub fn y4_axis_format(mut self, format: AxisLabelFormat) -> Self {
        self.show_y4_axis = true;
        self.y4_axis_ticks = format.ticks();
        self.y4_axis_labels = format.labels();
        self
    }

    pub fn x_axis_locked(mut self, locked: bool) -> Self {
        self.lock_x = AxisLock {
            pan: locked,
            zoom: locked,
        };
        self
    }

    pub fn y_axis_locked(mut self, locked: bool) -> Self {
        self.lock_y = AxisLock {
            pan: locked,
            zoom: locked,
        };
        self
    }

    pub fn y2_axis_locked(mut self, locked: bool) -> Self {
        self.lock_y2 = AxisLock {
            pan: locked,
            zoom: locked,
        };
        self
    }

    pub fn y3_axis_locked(mut self, locked: bool) -> Self {
        self.lock_y3 = AxisLock {
            pan: locked,
            zoom: locked,
        };
        self
    }

    pub fn y4_axis_locked(mut self, locked: bool) -> Self {
        self.lock_y4 = AxisLock {
            pan: locked,
            zoom: locked,
        };
        self
    }

    pub fn x_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_x.pan = locked;
        self
    }

    pub fn x_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_x.zoom = locked;
        self
    }

    pub fn y_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_y.pan = locked;
        self
    }

    pub fn y_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_y.zoom = locked;
        self
    }

    pub fn y2_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_y2.pan = locked;
        self
    }

    pub fn y2_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_y2.zoom = locked;
        self
    }

    pub fn y3_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_y3.pan = locked;
        self
    }

    pub fn y3_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_y3.zoom = locked;
        self
    }

    pub fn y4_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_y4.pan = locked;
        self
    }

    pub fn y4_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_y4.zoom = locked;
        self
    }

    pub fn x_axis_constraints(mut self, constraints: AxisConstraints) -> Self {
        self.x_constraints = constraints;
        self
    }

    pub fn y_axis_constraints(mut self, constraints: AxisConstraints) -> Self {
        self.y_constraints = constraints;
        self
    }

    pub fn y2_axis_constraints(mut self, constraints: AxisConstraints) -> Self {
        self.y2_constraints = constraints;
        self
    }

    pub fn y3_axis_constraints(mut self, constraints: AxisConstraints) -> Self {
        self.y3_constraints = constraints;
        self
    }

    pub fn y4_axis_constraints(mut self, constraints: AxisConstraints) -> Self {
        self.y4_constraints = constraints;
        self
    }

    pub fn x_axis_scale(mut self, scale: AxisScale) -> Self {
        self.x_scale = scale;
        self
    }

    pub fn y_axis_scale(mut self, scale: AxisScale) -> Self {
        self.y_scale = scale;
        self
    }

    pub fn y2_axis_scale(mut self, scale: AxisScale) -> Self {
        self.y2_scale = scale;
        self
    }

    pub fn y3_axis_scale(mut self, scale: AxisScale) -> Self {
        self.y3_scale = scale;
        self
    }

    pub fn y4_axis_scale(mut self, scale: AxisScale) -> Self {
        self.y4_scale = scale;
        self
    }

    pub fn x_axis_ticks(mut self, ticks: AxisTicks) -> Self {
        self.x_axis_ticks = ticks;
        self
    }

    pub fn y_axis_ticks(mut self, ticks: AxisTicks) -> Self {
        self.y_axis_ticks = ticks;
        self
    }

    pub fn y2_axis_ticks(mut self, ticks: AxisTicks) -> Self {
        self.show_y2_axis = true;
        self.y2_axis_ticks = ticks;
        self
    }

    pub fn y3_axis_ticks(mut self, ticks: AxisTicks) -> Self {
        self.show_y3_axis = true;
        self.y3_axis_ticks = ticks;
        self
    }

    pub fn y4_axis_ticks(mut self, ticks: AxisTicks) -> Self {
        self.show_y4_axis = true;
        self.y4_axis_ticks = ticks;
        self
    }

    pub fn x_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.x_axis_labels = labels.clone();
        self.tooltip_x_labels = labels;
        self
    }

    pub fn y_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.y_axis_labels = labels.clone();
        self.tooltip_y_labels = labels;
        self
    }

    pub fn y2_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.show_y2_axis = true;
        self.y2_axis_labels = labels;
        self
    }

    pub fn y3_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.show_y3_axis = true;
        self.y3_axis_labels = labels;
        self
    }

    pub fn y4_axis_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.show_y4_axis = true;
        self.y4_axis_labels = labels;
        self
    }

    pub fn tooltip_x_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.tooltip_x_labels = labels;
        self
    }

    pub fn tooltip_y_labels(mut self, labels: AxisLabelFormatter) -> Self {
        self.tooltip_y_labels = labels;
        self
    }

    pub fn state(mut self, state: Model<PlotState>) -> Self {
        self.plot_state_model = Some(state);
        self
    }

    pub fn output(mut self, output: Model<PlotOutput>) -> Self {
        self.plot_output_model = Some(output);
        self
    }

    pub fn create_node<H: UiHost>(ui: &mut fret_ui::UiTree<H>, canvas: Self) -> fret_core::NodeId {
        ui.create_node_retained(canvas)
    }

    fn axis_gaps(&self) -> (Px, Px, Px, Px, Px) {
        let min = self.style.axis_gap.0.max(0.0);
        let y = self.y_axis_thickness.0.max(min);
        let y_right = if self.show_y2_axis {
            self.y_axis_right_thickness.0.max(min)
        } else {
            0.0
        };
        let y_right2 = if self.show_y3_axis {
            self.y_axis_right2_thickness.0.max(min)
        } else {
            0.0
        };
        let y_right3 = if self.show_y4_axis {
            self.y_axis_right3_thickness.0.max(min)
        } else {
            0.0
        };
        let x = self.x_axis_thickness.0.max(min);
        (Px(y), Px(y_right), Px(y_right2), Px(y_right3), Px(x))
    }

    fn read_plot_state<H: UiHost>(&self, app: &mut H) -> PlotState {
        if let Some(state) = &self.plot_state_model {
            state
                .read(app, |_app, s| s.clone())
                .unwrap_or_else(|_| self.plot_state.clone())
        } else {
            self.plot_state.clone()
        }
    }

    fn update_plot_state<H: UiHost>(
        &mut self,
        app: &mut H,
        f: impl FnOnce(&mut PlotState),
    ) -> bool {
        if let Some(state) = &self.plot_state_model {
            state.update(app, |s, _cx| f(s)).is_ok()
        } else {
            f(&mut self.plot_state);
            true
        }
    }

    fn publish_plot_output<H: UiHost>(&mut self, app: &mut H, snapshot: PlotOutputSnapshot) {
        if self.plot_output.snapshot == snapshot {
            return;
        }

        self.plot_output.revision = self.plot_output.revision.wrapping_add(1);
        self.plot_output.snapshot = snapshot;

        if let Some(model) = &self.plot_output_model {
            let next = self.plot_output;
            let _ = model.update(app, |s, _cx| {
                *s = next;
            });
        }
    }

    fn publish_current_output_snapshot<H: UiHost>(
        &mut self,
        app: &mut H,
        layout: PlotLayout,
        state: &PlotState,
        view_bounds: DataRect,
        view_bounds_y2: Option<DataRect>,
        view_bounds_y3: Option<DataRect>,
        view_bounds_y4: Option<DataRect>,
    ) {
        let cursor_data = self.cursor_px.and_then(|cursor_px| {
            if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                return None;
            }
            let transform = PlotTransform {
                viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size),
                data: view_bounds,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
            };
            let data = transform.px_to_data(cursor_px);
            (data.x.is_finite() && data.y.is_finite()).then_some(data)
        });

        self.publish_plot_output(
            app,
            PlotOutputSnapshot {
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                cursor: cursor_data,
                hover: self.hover.map(|h| PlotHoverOutput {
                    series_id: h.series_id,
                    data: h.data,
                    value: h.value,
                }),
                query: state.query,
            },
        );
    }

    fn current_view_bounds<H: UiHost>(&self, app: &mut H, state: &PlotState) -> DataRect {
        let view = if state.view_is_auto {
            let data_bounds = self.read_data_bounds(app);
            if self.style.clamp_to_data_bounds {
                expand_data_bounds_scaled(
                    data_bounds,
                    self.style.overscroll_fraction,
                    self.x_scale,
                    self.y_scale,
                )
            } else {
                sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y_scale)
            }
        } else if let Some(view) = state.view_bounds {
            sanitize_data_rect_scaled(view, self.x_scale, self.y_scale)
        } else {
            sanitize_data_rect_scaled(self.read_data_bounds(app), self.x_scale, self.y_scale)
        };

        constrain_view_bounds_scaled(
            view,
            self.x_scale,
            self.y_scale,
            self.x_constraints,
            self.y_constraints,
        )
    }

    fn current_view_bounds_y2<H: UiHost>(
        &self,
        app: &mut H,
        state: &PlotState,
        view_bounds: DataRect,
    ) -> Option<DataRect> {
        if !self.show_y2_axis {
            return None;
        }

        let data_bounds = self.read_data_bounds_y2(app)?;

        let y2_bounds = if state.view_y2_is_auto {
            if self.style.clamp_to_data_bounds {
                expand_data_bounds_scaled(
                    data_bounds,
                    self.style.overscroll_fraction,
                    self.x_scale,
                    self.y2_scale,
                )
            } else {
                sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y2_scale)
            }
        } else if let Some(view) = state.view_bounds_y2 {
            sanitize_data_rect_scaled(view, self.x_scale, self.y2_scale)
        } else {
            sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y2_scale)
        };

        let view = sanitize_data_rect_scaled(
            DataRect {
                x_min: view_bounds.x_min,
                x_max: view_bounds.x_max,
                y_min: y2_bounds.y_min,
                y_max: y2_bounds.y_max,
            },
            self.x_scale,
            self.y2_scale,
        );

        Some(constrain_view_bounds_scaled(
            view,
            self.x_scale,
            self.y2_scale,
            self.x_constraints,
            self.y2_constraints,
        ))
    }

    fn current_view_bounds_y3<H: UiHost>(
        &self,
        app: &mut H,
        state: &PlotState,
        view_bounds: DataRect,
    ) -> Option<DataRect> {
        if !self.show_y3_axis {
            return None;
        }

        let data_bounds = self.read_data_bounds_y3(app)?;

        let y3_bounds = if state.view_y3_is_auto {
            if self.style.clamp_to_data_bounds {
                expand_data_bounds_scaled(
                    data_bounds,
                    self.style.overscroll_fraction,
                    self.x_scale,
                    self.y3_scale,
                )
            } else {
                sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y3_scale)
            }
        } else if let Some(view) = state.view_bounds_y3 {
            sanitize_data_rect_scaled(view, self.x_scale, self.y3_scale)
        } else {
            sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y3_scale)
        };

        let view = sanitize_data_rect_scaled(
            DataRect {
                x_min: view_bounds.x_min,
                x_max: view_bounds.x_max,
                y_min: y3_bounds.y_min,
                y_max: y3_bounds.y_max,
            },
            self.x_scale,
            self.y3_scale,
        );

        Some(constrain_view_bounds_scaled(
            view,
            self.x_scale,
            self.y3_scale,
            self.x_constraints,
            self.y3_constraints,
        ))
    }

    fn current_view_bounds_y4<H: UiHost>(
        &self,
        app: &mut H,
        state: &PlotState,
        view_bounds: DataRect,
    ) -> Option<DataRect> {
        if !self.show_y4_axis {
            return None;
        }

        let data_bounds = self.read_data_bounds_y4(app)?;

        let y4_bounds = if state.view_y4_is_auto {
            if self.style.clamp_to_data_bounds {
                expand_data_bounds_scaled(
                    data_bounds,
                    self.style.overscroll_fraction,
                    self.x_scale,
                    self.y4_scale,
                )
            } else {
                sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y4_scale)
            }
        } else if let Some(view) = state.view_bounds_y4 {
            sanitize_data_rect_scaled(view, self.x_scale, self.y4_scale)
        } else {
            sanitize_data_rect_scaled(data_bounds, self.x_scale, self.y4_scale)
        };

        let view = sanitize_data_rect_scaled(
            DataRect {
                x_min: view_bounds.x_min,
                x_max: view_bounds.x_max,
                y_min: y4_bounds.y_min,
                y_max: y4_bounds.y_max,
            },
            self.x_scale,
            self.y4_scale,
        );

        Some(constrain_view_bounds_scaled(
            view,
            self.x_scale,
            self.y4_scale,
            self.x_constraints,
            self.y4_constraints,
        ))
    }

    fn rebuild_paths_if_needed<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        plot: Rect,
        view_bounds: DataRect,
        view_bounds_y2: Option<DataRect>,
        view_bounds_y3: Option<DataRect>,
        view_bounds_y4: Option<DataRect>,
        hidden: &HashSet<SeriesId>,
    ) -> Vec<(SeriesId, PathId, Color)> {
        let model_revision = self.model.revision(cx.app).unwrap_or(0);
        let Ok(model) = self.model.read(cx.app, |_app, m| m.clone()) else {
            return Vec::new();
        };

        self.layer.paint_paths(
            cx,
            &model,
            PlotPaintArgs {
                model_revision,
                plot,
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
                y2_scale: self.y2_scale,
                y3_scale: self.y3_scale,
                y4_scale: self.y4_scale,
                style: self.style,
                hidden,
            },
        )
    }

    fn rebuild_quads_if_needed<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        plot: Rect,
        view_bounds: DataRect,
        view_bounds_y2: Option<DataRect>,
        view_bounds_y3: Option<DataRect>,
        view_bounds_y4: Option<DataRect>,
        hidden: &HashSet<SeriesId>,
    ) -> Vec<PlotQuad> {
        let model_revision = self.model.revision(cx.app).unwrap_or(0);
        let Ok(model) = self.model.read(cx.app, |_app, m| m.clone()) else {
            return Vec::new();
        };

        self.layer.paint_quads(
            cx,
            &model,
            PlotPaintArgs {
                model_revision,
                plot,
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
                y2_scale: self.y2_scale,
                y3_scale: self.y3_scale,
                y4_scale: self.y4_scale,
                style: self.style,
                hidden,
            },
        )
    }
}

impl<H: UiHost, L: PlotLayer + 'static> Widget<H> for PlotCanvas<L> {
    fn event(&mut self, cx: &mut fret_ui::retained_bridge::EventCx<'_, H>, event: &Event) {
        // Axis enablement is derived from the model (series -> axis assignment), so make sure
        // we don't accidentally interpret "right axis series" using the primary Y transform.
        self.ensure_required_axes_enabled(cx.app);

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

                    let (
                        y_axis_gap,
                        y_axis_right_gap,
                        y_axis_right2_gap,
                        y_axis_right3_gap,
                        x_axis_gap,
                    ) = self.axis_gaps();
                    let layout = PlotLayout::from_bounds(
                        cx.bounds,
                        self.style.padding,
                        y_axis_gap,
                        y_axis_right_gap,
                        y_axis_right2_gap,
                        y_axis_right3_gap,
                        x_axis_gap,
                    );
                    let Some(region) = layout.hit_test_region(pos) else {
                        return;
                    };

                    let toggle_pan = modifiers.shift && !modifiers.ctrl;
                    let toggle_zoom = modifiers.ctrl && !modifiers.shift;
                    let toggle_both = !toggle_pan && !toggle_zoom;

                    match region {
                        PlotRegion::XAxis => {
                            if toggle_both || toggle_pan {
                                self.lock_x.pan = !self.lock_x.pan;
                            }
                            if toggle_both || toggle_zoom {
                                self.lock_x.zoom = !self.lock_x.zoom;
                            }
                        }
                        PlotRegion::YAxis(axis) => {
                            let lock = match axis {
                                YAxis::Left => &mut self.lock_y,
                                YAxis::Right => &mut self.lock_y2,
                                YAxis::Right2 => &mut self.lock_y3,
                                YAxis::Right3 => &mut self.lock_y4,
                            };
                            if toggle_both || toggle_pan {
                                lock.pan = !lock.pan;
                            }
                            if toggle_both || toggle_zoom {
                                lock.zoom = !lock.zoom;
                            }
                        }
                        PlotRegion::Plot => {
                            if toggle_both || toggle_pan {
                                self.lock_x.pan = !self.lock_x.pan;
                                self.lock_y.pan = !self.lock_y.pan;
                                if self.show_y2_axis {
                                    self.lock_y2.pan = !self.lock_y2.pan;
                                }
                                if self.show_y3_axis {
                                    self.lock_y3.pan = !self.lock_y3.pan;
                                }
                                if self.show_y4_axis {
                                    self.lock_y4.pan = !self.lock_y4.pan;
                                }
                            }
                            if toggle_both || toggle_zoom {
                                self.lock_x.zoom = !self.lock_x.zoom;
                                self.lock_y.zoom = !self.lock_y.zoom;
                                if self.show_y2_axis {
                                    self.lock_y2.zoom = !self.lock_y2.zoom;
                                }
                                if self.show_y3_axis {
                                    self.lock_y3.zoom = !self.lock_y3.zoom;
                                }
                                if self.show_y4_axis {
                                    self.lock_y4.zoom = !self.lock_y4.zoom;
                                }
                            }
                        }
                    }

                    self.hover = None;
                    self.cursor_px = None;
                    self.pan_button = None;
                    self.pan_target = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                    self.query_drag_button = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;

                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                } else if plain && *key == KeyCode::KeyR {
                    let _ = self.update_plot_state(cx.app, |s| {
                        s.view_is_auto = true;
                        s.view_bounds = None;
                        s.view_y2_is_auto = true;
                        s.view_bounds_y2 = None;
                        s.view_y3_is_auto = true;
                        s.view_bounds_y3 = None;
                        s.view_y4_is_auto = true;
                        s.view_bounds_y4 = None;
                        s.linked_cursor_x = None;
                        s.hidden_series.clear();
                        s.pinned_series = None;
                        s.query = None;
                    });
                    self.hover = None;
                    self.cursor_px = None;
                    self.pan_button = None;
                    self.pan_target = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                    self.query_drag_button = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                } else if plain && *key == KeyCode::KeyA {
                    let _ = self.update_plot_state(cx.app, |s| {
                        s.hidden_series.clear();
                        s.pinned_series = None;
                    });
                    self.hover = None;
                    self.cursor_px = None;
                    self.legend_hover = None;
                    self.pan_button = None;
                    self.pan_target = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                    self.query_drag_button = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                } else if plain && *key == KeyCode::KeyQ {
                    let query = self.read_plot_state(cx.app).query;
                    if query.is_some() {
                        let _ = self.update_plot_state(cx.app, |s| {
                            s.query = None;
                        });
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                } else if *key == KeyCode::Escape {
                    let state = self.read_plot_state(cx.app);
                    let has_active_drag = self.box_zoom_start.is_some()
                        || self.pan_button.is_some()
                        || self.query_drag_start.is_some();

                    if has_active_drag {
                        self.pan_button = None;
                        self.pan_target = None;
                        self.pan_start_pos = None;
                        self.pan_last_pos = None;
                        self.box_zoom_start = None;
                        self.box_zoom_current = None;
                        self.box_zoom_button = None;
                        self.box_zoom_required_mods = None;
                        self.query_drag_button = None;
                        self.query_drag_start = None;
                        self.query_drag_current = None;
                        self.hover = None;
                        if cx.captured == Some(cx.node) {
                            cx.release_pointer_capture();
                        }
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    } else if state.pinned_series.is_some() {
                        let _ = self.update_plot_state(cx.app, |s| {
                            s.pinned_series = None;
                        });
                        self.legend_hover = None;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    } else if state.query.is_some() {
                        let _ = self.update_plot_state(cx.app, |s| {
                            s.query = None;
                        });
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                }
            }
            Event::Pointer(PointerEvent::Down {
                position,
                button,
                modifiers,
                ..
            }) => {
                self.last_pointer_pos = Some(*position);
                let (
                    y_axis_gap,
                    y_axis_right_gap,
                    y_axis_right2_gap,
                    y_axis_right3_gap,
                    x_axis_gap,
                ) = self.axis_gaps();
                let layout = PlotLayout::from_bounds(
                    cx.bounds,
                    self.style.padding,
                    y_axis_gap,
                    y_axis_right_gap,
                    y_axis_right2_gap,
                    y_axis_right3_gap,
                    x_axis_gap,
                );
                if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                    return;
                }

                // Axis lock UI: Ctrl+Click on an axis region toggles pan+zoom lock.
                if modifiers.ctrl
                    && *button == MouseButton::Left
                    && let Some(region) = layout.hit_test_region(*position)
                    && region != PlotRegion::Plot
                {
                    match region {
                        PlotRegion::XAxis => {
                            self.lock_x.pan = !self.lock_x.pan;
                            self.lock_x.zoom = !self.lock_x.zoom;
                        }
                        PlotRegion::YAxis(axis) => {
                            let lock = match axis {
                                YAxis::Left => &mut self.lock_y,
                                YAxis::Right => &mut self.lock_y2,
                                YAxis::Right2 => &mut self.lock_y3,
                                YAxis::Right3 => &mut self.lock_y4,
                            };
                            lock.pan = !lock.pan;
                            lock.zoom = !lock.zoom;
                        }
                        PlotRegion::Plot => {}
                    }

                    self.hover = None;
                    self.cursor_px = None;
                    self.legend_hover = None;
                    self.pan_button = None;
                    self.pan_target = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                    self.query_drag_button = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;

                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if *button == MouseButton::Left
                    && let Some((_legend, rows)) = self.legend_layout(layout)
                    && let Some(series_index) = rows
                        .iter()
                        .enumerate()
                        .find(|(_i, r)| contains_point(**r, *position))
                        .map(|(i, _r)| i)
                {
                    let row = rows[series_index];
                    let swatch = Self::legend_swatch_column(row);
                    let Some(entry) = self.legend_entries.get(series_index).cloned() else {
                        return;
                    };
                    let id = entry.id;
                    let state = self.read_plot_state(cx.app);
                    let mut next_hidden = state.hidden_series;
                    let mut next_pinned = state.pinned_series;

                    // Legend interaction policy:
                    // - Shift+Click: solo the series (or restore all if already solo)
                    // - Click swatch column: toggle visibility
                    // - Click label area: pin/unpin tooltip + emphasis to this series
                    if modifiers.shift {
                        let ids: Vec<SeriesId> = self.legend_entries.iter().map(|e| e.id).collect();
                        let visible_count =
                            ids.iter().filter(|sid| !next_hidden.contains(sid)).count();
                        let is_solo = visible_count == 1 && !next_hidden.contains(&id);
                        if is_solo {
                            next_hidden.clear();
                        } else {
                            next_hidden = ids.into_iter().filter(|sid| *sid != id).collect();
                        }
                        next_hidden.remove(&id);
                    } else if contains_point(swatch, *position) {
                        let total = self.legend_entries.len();
                        let hidden_count = self
                            .legend_entries
                            .iter()
                            .filter(|e| next_hidden.contains(&e.id))
                            .count();
                        let visible_count = total.saturating_sub(hidden_count);

                        let is_hidden = next_hidden.contains(&id);
                        if !is_hidden && visible_count <= 1 {
                            // Never hide the last visible series.
                        } else if is_hidden {
                            next_hidden.remove(&id);
                        } else {
                            next_hidden.insert(id);
                        }
                    } else if next_pinned == Some(id) {
                        next_pinned = None;
                    } else {
                        next_pinned = Some(id);
                        next_hidden.remove(&id);
                    }

                    let _ = self.update_plot_state(cx.app, |s| {
                        s.hidden_series = next_hidden;
                        s.pinned_series = next_pinned;
                    });

                    self.hover = None;
                    self.cursor_px = None;
                    self.legend_hover = Some(id);
                    self.pan_button = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                    self.query_drag_button = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let region = layout.hit_test_region(*position);
                if region.is_none() {
                    return;
                }

                // ImPlot-compatible box zoom cancel: a distinct button press cancels an active box
                // selection (default: LMB cancels RMB selection).
                if region == Some(PlotRegion::Plot)
                    && self.box_zoom_start.is_some()
                    && let Some(cancel) = self.input_map.box_zoom_cancel
                    && cancel.matches(*button, *modifiers)
                    && self
                        .box_zoom_button
                        .is_some_and(|active| active != cancel.button)
                {
                    self.pan_button = None;
                    self.pan_target = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;
                    self.query_drag_button = None;
                    self.hover = None;
                    self.cursor_px = None;
                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let start_query = self
                    .input_map
                    .query_drag
                    .is_some_and(|ch| ch.matches(*button, *modifiers));
                let start_box_primary = self.input_map.box_zoom.matches(*button, *modifiers);
                let start_box_alt = self
                    .input_map
                    .box_zoom_alt
                    .is_some_and(|ch| ch.matches(*button, *modifiers));
                let start_pan = self.input_map.pan.matches(*button, *modifiers);

                if !start_query && !start_box_primary && !start_box_alt && !start_pan {
                    return;
                }

                if start_query || start_box_primary || start_box_alt {
                    if region != Some(PlotRegion::Plot) {
                        return;
                    }
                } else if start_pan && region.is_none() {
                    return;
                }

                self.cursor_px = (region == Some(PlotRegion::Plot))
                    .then(|| local_from_absolute(layout.plot.origin, *position));
                self.hover = None;

                if start_query {
                    let local = local_from_absolute(layout.plot.origin, *position);
                    self.query_drag_start = Some(local);
                    self.query_drag_current = Some(local);
                    self.query_drag_button = Some(*button);
                    self.pan_button = None;
                    self.pan_target = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                } else if start_box_primary || start_box_alt {
                    let local = local_from_absolute(layout.plot.origin, *position);
                    self.box_zoom_start = Some(local);
                    self.box_zoom_current = Some(local);
                    self.box_zoom_button = Some(*button);
                    self.box_zoom_required_mods = Some(if start_box_primary {
                        self.input_map.box_zoom.modifiers
                    } else {
                        self.input_map
                            .box_zoom_alt
                            .unwrap_or(self.input_map.box_zoom)
                            .modifiers
                    });
                    self.pan_button = None;
                    self.pan_target = None;
                    self.pan_start_pos = None;
                    self.pan_last_pos = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;
                    self.query_drag_button = None;
                } else {
                    self.pan_button = Some(*button);
                    self.pan_target = region;
                    self.pan_start_pos = Some(*position);
                    self.pan_last_pos = None;
                    self.box_zoom_start = None;
                    self.box_zoom_current = None;
                    self.box_zoom_button = None;
                    self.box_zoom_required_mods = None;
                    self.query_drag_start = None;
                    self.query_drag_current = None;
                    self.query_drag_button = None;
                }

                cx.request_focus(cx.node);
                cx.capture_pointer(cx.node);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            Event::Pointer(PointerEvent::Up {
                position,
                button,
                modifiers,
                click_count,
                ..
            }) => {
                self.last_pointer_pos = Some(*position);
                if self.pan_button == Some(*button)
                    || self.box_zoom_button == Some(*button)
                    || self.query_drag_button == Some(*button)
                    || self.input_map.fit.matches(*button, *modifiers)
                {
                    if self.input_map.fit.matches(*button, *modifiers)
                        && *click_count == 2
                        && self.pan_last_pos.is_none()
                        && self.box_zoom_start.is_none()
                        && self.query_drag_start.is_none()
                    {
                        let (
                            y_axis_gap,
                            y_axis_right_gap,
                            y_axis_right2_gap,
                            y_axis_right3_gap,
                            x_axis_gap,
                        ) = self.axis_gaps();
                        let layout = PlotLayout::from_bounds(
                            cx.bounds,
                            self.style.padding,
                            y_axis_gap,
                            y_axis_right_gap,
                            y_axis_right2_gap,
                            y_axis_right3_gap,
                            x_axis_gap,
                        );
                        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
                            let region = layout.hit_test_region(*position);
                            if let Some(region) = region {
                                let (fit, fit_y2, fit_y3, fit_y4) =
                                    self.fit_view_to_data_now(cx.app);
                                let show_y2_axis = self.show_y2_axis;
                                let show_y3_axis = self.show_y3_axis;
                                let show_y4_axis = self.show_y4_axis;

                                let state = self.read_plot_state(cx.app);
                                let current = self.current_view_bounds(cx.app, &state);
                                let current_y2 =
                                    self.current_view_bounds_y2(cx.app, &state, current);
                                let current_y3 =
                                    self.current_view_bounds_y3(cx.app, &state, current);
                                let current_y4 =
                                    self.current_view_bounds_y4(cx.app, &state, current);

                                let lock_x_zoom = self.lock_x.zoom;
                                let lock_y1_zoom = self.lock_y.zoom;
                                let lock_y2_zoom = self.lock_y2.zoom;
                                let lock_y3_zoom = self.lock_y3.zoom;
                                let lock_y4_zoom = self.lock_y4.zoom;

                                let x_scale = self.x_scale;
                                let y_scale = self.y_scale;
                                let y2_scale = self.y2_scale;
                                let y3_scale = self.y3_scale;
                                let y4_scale = self.y4_scale;

                                let x_constraints = self.x_constraints;
                                let y_constraints = self.y_constraints;
                                let y2_constraints = self.y2_constraints;
                                let y3_constraints = self.y3_constraints;
                                let y4_constraints = self.y4_constraints;

                                let _ = self.update_plot_state(cx.app, |s| match region {
                                    PlotRegion::Plot => {
                                        s.view_is_auto = false;
                                        s.view_bounds = Some(fit);
                                        if show_y2_axis {
                                            s.view_y2_is_auto = false;
                                            s.view_bounds_y2 = fit_y2;
                                        }
                                        if show_y3_axis {
                                            s.view_y3_is_auto = false;
                                            s.view_bounds_y3 = fit_y3;
                                        }
                                        if show_y4_axis {
                                            s.view_y4_is_auto = false;
                                            s.view_bounds_y4 = fit_y4;
                                        }
                                    }
                                    PlotRegion::XAxis => {
                                        if lock_x_zoom {
                                            return;
                                        }

                                        let mut next = current;
                                        next.x_min = fit.x_min;
                                        next.x_max = fit.x_max;
                                        next = constrain_view_bounds_scaled(
                                            next,
                                            x_scale,
                                            y_scale,
                                            x_constraints,
                                            y_constraints,
                                        );

                                        let next_y2 = current_y2.map(|mut vb| {
                                            vb.x_min = fit.x_min;
                                            vb.x_max = fit.x_max;
                                            constrain_view_bounds_scaled(
                                                vb,
                                                x_scale,
                                                y2_scale,
                                                x_constraints,
                                                y2_constraints,
                                            )
                                        });
                                        let next_y3 = current_y3.map(|mut vb| {
                                            vb.x_min = fit.x_min;
                                            vb.x_max = fit.x_max;
                                            constrain_view_bounds_scaled(
                                                vb,
                                                x_scale,
                                                y3_scale,
                                                x_constraints,
                                                y3_constraints,
                                            )
                                        });
                                        let next_y4 = current_y4.map(|mut vb| {
                                            vb.x_min = fit.x_min;
                                            vb.x_max = fit.x_max;
                                            constrain_view_bounds_scaled(
                                                vb,
                                                x_scale,
                                                y4_scale,
                                                x_constraints,
                                                y4_constraints,
                                            )
                                        });

                                        s.view_is_auto = false;
                                        s.view_bounds = Some(next);
                                        if show_y2_axis {
                                            s.view_y2_is_auto = false;
                                            s.view_bounds_y2 = next_y2;
                                        }
                                        if show_y3_axis {
                                            s.view_y3_is_auto = false;
                                            s.view_bounds_y3 = next_y3;
                                        }
                                        if show_y4_axis {
                                            s.view_y4_is_auto = false;
                                            s.view_bounds_y4 = next_y4;
                                        }
                                    }
                                    PlotRegion::YAxis(axis) => match axis {
                                        YAxis::Left => {
                                            if lock_y1_zoom {
                                                return;
                                            }

                                            let mut next = current;
                                            next.y_min = fit.y_min;
                                            next.y_max = fit.y_max;
                                            next = constrain_view_bounds_scaled(
                                                next,
                                                x_scale,
                                                y_scale,
                                                x_constraints,
                                                y_constraints,
                                            );
                                            s.view_is_auto = false;
                                            s.view_bounds = Some(next);
                                        }
                                        YAxis::Right => {
                                            if lock_y2_zoom {
                                                return;
                                            }
                                            let Some(fit_axis) = fit_y2 else {
                                                return;
                                            };
                                            let Some(mut next) = current_y2 else {
                                                return;
                                            };
                                            next.y_min = fit_axis.y_min;
                                            next.y_max = fit_axis.y_max;
                                            next = constrain_view_bounds_scaled(
                                                next,
                                                x_scale,
                                                y2_scale,
                                                x_constraints,
                                                y2_constraints,
                                            );
                                            s.view_y2_is_auto = false;
                                            s.view_bounds_y2 = Some(next);
                                        }
                                        YAxis::Right2 => {
                                            if lock_y3_zoom {
                                                return;
                                            }
                                            let Some(fit_axis) = fit_y3 else {
                                                return;
                                            };
                                            let Some(mut next) = current_y3 else {
                                                return;
                                            };
                                            next.y_min = fit_axis.y_min;
                                            next.y_max = fit_axis.y_max;
                                            next = constrain_view_bounds_scaled(
                                                next,
                                                x_scale,
                                                y3_scale,
                                                x_constraints,
                                                y3_constraints,
                                            );
                                            s.view_y3_is_auto = false;
                                            s.view_bounds_y3 = Some(next);
                                        }
                                        YAxis::Right3 => {
                                            if lock_y4_zoom {
                                                return;
                                            }
                                            let Some(fit_axis) = fit_y4 else {
                                                return;
                                            };
                                            let Some(mut next) = current_y4 else {
                                                return;
                                            };
                                            next.y_min = fit_axis.y_min;
                                            next.y_max = fit_axis.y_max;
                                            next = constrain_view_bounds_scaled(
                                                next,
                                                x_scale,
                                                y4_scale,
                                                x_constraints,
                                                y4_constraints,
                                            );
                                            s.view_y4_is_auto = false;
                                            s.view_bounds_y4 = Some(next);
                                        }
                                    },
                                });

                                self.hover = None;
                                self.cursor_px = None;
                                self.legend_hover = None;
                                self.pan_button = None;
                                self.pan_target = None;
                                self.pan_start_pos = None;
                                self.pan_last_pos = None;
                                self.box_zoom_start = None;
                                self.box_zoom_current = None;
                                self.box_zoom_button = None;
                                self.box_zoom_required_mods = None;
                                self.query_drag_button = None;
                                self.query_drag_start = None;
                                self.query_drag_current = None;
                                if cx.captured == Some(cx.node) {
                                    cx.release_pointer_capture();
                                }
                                cx.invalidate_self(Invalidation::Paint);
                                cx.request_redraw();
                                cx.stop_propagation();
                                return;
                            }
                        }
                    }

                    if self.query_drag_start.is_some() && self.query_drag_button == Some(*button) {
                        if cx.captured == Some(cx.node) {
                            cx.release_pointer_capture();
                        }

                        let (
                            y_axis_gap,
                            y_axis_right_gap,
                            y_axis_right2_gap,
                            y_axis_right3_gap,
                            x_axis_gap,
                        ) = self.axis_gaps();
                        let layout = PlotLayout::from_bounds(
                            cx.bounds,
                            self.style.padding,
                            y_axis_gap,
                            y_axis_right_gap,
                            y_axis_right2_gap,
                            y_axis_right3_gap,
                            x_axis_gap,
                        );
                        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
                            let start = self
                                .query_drag_start
                                .unwrap_or(Point::new(Px(0.0), Px(0.0)));
                            let end = self
                                .query_drag_current
                                .unwrap_or(Point::new(Px(0.0), Px(0.0)));

                            let w = (start.x.0 - end.x.0).abs();
                            let h = (start.y.0 - end.y.0).abs();

                            if w >= 4.0 && h >= 4.0 {
                                let state = self.read_plot_state(cx.app);
                                let view_bounds = self.current_view_bounds(cx.app, &state);
                                if let Some(next) = query_rect_from_plot_points_raw(
                                    view_bounds,
                                    layout.plot.size,
                                    start,
                                    end,
                                    self.x_scale,
                                    self.y_scale,
                                ) {
                                    let _ = self.update_plot_state(cx.app, |s| {
                                        s.query = Some(next);
                                    });
                                }
                            }
                        }

                        self.query_drag_button = None;
                        self.query_drag_start = None;
                        self.query_drag_current = None;
                        self.pan_last_pos = None;
                        self.hover = None;

                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    } else if self.box_zoom_start.is_some() && self.box_zoom_button == Some(*button)
                    {
                        if cx.captured == Some(cx.node) {
                            cx.release_pointer_capture();
                        }

                        let (
                            y_axis_gap,
                            y_axis_right_gap,
                            y_axis_right2_gap,
                            y_axis_right3_gap,
                            x_axis_gap,
                        ) = self.axis_gaps();
                        let layout = PlotLayout::from_bounds(
                            cx.bounds,
                            self.style.padding,
                            y_axis_gap,
                            y_axis_right_gap,
                            y_axis_right2_gap,
                            y_axis_right3_gap,
                            x_axis_gap,
                        );
                        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
                            let start = self.box_zoom_start.unwrap_or(Point::new(Px(0.0), Px(0.0)));
                            let end = self
                                .box_zoom_current
                                .unwrap_or(Point::new(Px(0.0), Px(0.0)));
                            let required =
                                self.box_zoom_required_mods.unwrap_or(ModifiersMask::NONE);
                            let (start, end) = Self::apply_box_select_modifiers(
                                layout.plot.size,
                                start,
                                end,
                                *modifiers,
                                self.input_map.box_zoom_expand_x,
                                self.input_map.box_zoom_expand_y,
                                required,
                            );

                            let w = (start.x.0 - end.x.0).abs();
                            let h = (start.y.0 - end.y.0).abs();

                            if w >= 4.0 && h >= 4.0 {
                                let all_locked = self.lock_x.zoom
                                    && self.lock_y.zoom
                                    && (!self.show_y2_axis || self.lock_y2.zoom)
                                    && (!self.show_y3_axis || self.lock_y3.zoom)
                                    && (!self.show_y4_axis || self.lock_y4.zoom);
                                if all_locked {
                                    // Axis locks prevent any view change; keep auto-fit state intact.
                                    // The selection rectangle is still useful feedback for users.
                                } else {
                                    let state = self.read_plot_state(cx.app);
                                    let view_bounds = self.current_view_bounds(cx.app, &state);
                                    let view_bounds_y2 =
                                        self.current_view_bounds_y2(cx.app, &state, view_bounds);
                                    let view_bounds_y3 =
                                        self.current_view_bounds_y3(cx.app, &state, view_bounds);
                                    let view_bounds_y4 =
                                        self.current_view_bounds_y4(cx.app, &state, view_bounds);
                                    if let Some(mut next) = data_rect_from_plot_points_scaled(
                                        view_bounds,
                                        layout.plot.size,
                                        start,
                                        end,
                                        self.x_scale,
                                        self.y_scale,
                                    ) {
                                        let mut next_y2 = (!self.lock_y2.zoom)
                                            .then(|| {
                                                view_bounds_y2.and_then(|vb| {
                                                    data_rect_from_plot_points_scaled(
                                                        vb,
                                                        layout.plot.size,
                                                        start,
                                                        end,
                                                        self.x_scale,
                                                        self.y2_scale,
                                                    )
                                                })
                                            })
                                            .flatten();
                                        let mut next_y3 = (!self.lock_y3.zoom)
                                            .then(|| {
                                                view_bounds_y3.and_then(|vb| {
                                                    data_rect_from_plot_points_scaled(
                                                        vb,
                                                        layout.plot.size,
                                                        start,
                                                        end,
                                                        self.x_scale,
                                                        self.y3_scale,
                                                    )
                                                })
                                            })
                                            .flatten();
                                        let mut next_y4 = (!self.lock_y4.zoom)
                                            .then(|| {
                                                view_bounds_y4.and_then(|vb| {
                                                    data_rect_from_plot_points_scaled(
                                                        vb,
                                                        layout.plot.size,
                                                        start,
                                                        end,
                                                        self.x_scale,
                                                        self.y4_scale,
                                                    )
                                                })
                                            })
                                            .flatten();
                                        let data_bounds = self.read_data_bounds(cx.app);
                                        if self.style.clamp_to_data_bounds {
                                            next = clamp_view_to_data_scaled(
                                                next,
                                                data_bounds,
                                                self.style.overscroll_fraction,
                                                self.x_scale,
                                                self.y_scale,
                                            );
                                            if let (Some(candidate), Some(bounds_y2)) =
                                                (next_y2.as_mut(), self.read_data_bounds_y2(cx.app))
                                            {
                                                *candidate = clamp_view_to_data_scaled(
                                                    *candidate,
                                                    bounds_y2,
                                                    self.style.overscroll_fraction,
                                                    self.x_scale,
                                                    self.y2_scale,
                                                );
                                            }
                                            if let (Some(candidate), Some(bounds_y3)) =
                                                (next_y3.as_mut(), self.read_data_bounds_y3(cx.app))
                                            {
                                                *candidate = clamp_view_to_data_scaled(
                                                    *candidate,
                                                    bounds_y3,
                                                    self.style.overscroll_fraction,
                                                    self.x_scale,
                                                    self.y3_scale,
                                                );
                                            }
                                            if let (Some(candidate), Some(bounds_y4)) =
                                                (next_y4.as_mut(), self.read_data_bounds_y4(cx.app))
                                            {
                                                *candidate = clamp_view_to_data_scaled(
                                                    *candidate,
                                                    bounds_y4,
                                                    self.style.overscroll_fraction,
                                                    self.x_scale,
                                                    self.y4_scale,
                                                );
                                            }
                                        }

                                        next = apply_axis_locks(
                                            view_bounds,
                                            next,
                                            self.lock_x.zoom,
                                            self.lock_y.zoom,
                                        );
                                        if let Some(vb_y2) = view_bounds_y2 {
                                            if let Some(candidate) = next_y2.as_mut() {
                                                *candidate = apply_axis_locks(
                                                    vb_y2,
                                                    *candidate,
                                                    self.lock_x.zoom,
                                                    self.lock_y2.zoom,
                                                );
                                            }
                                        }
                                        if let Some(vb_y3) = view_bounds_y3 {
                                            if let Some(candidate) = next_y3.as_mut() {
                                                *candidate = apply_axis_locks(
                                                    vb_y3,
                                                    *candidate,
                                                    self.lock_x.zoom,
                                                    self.lock_y3.zoom,
                                                );
                                            }
                                        }
                                        if let Some(vb_y4) = view_bounds_y4 {
                                            if let Some(candidate) = next_y4.as_mut() {
                                                *candidate = apply_axis_locks(
                                                    vb_y4,
                                                    *candidate,
                                                    self.lock_x.zoom,
                                                    self.lock_y4.zoom,
                                                );
                                            }
                                        }

                                        next = constrain_view_bounds_scaled(
                                            next,
                                            self.x_scale,
                                            self.y_scale,
                                            self.x_constraints,
                                            self.y_constraints,
                                        );
                                        if let Some(candidate) = next_y2.as_mut() {
                                            *candidate = constrain_view_bounds_scaled(
                                                *candidate,
                                                self.x_scale,
                                                self.y2_scale,
                                                self.x_constraints,
                                                self.y2_constraints,
                                            );
                                        }
                                        if let Some(candidate) = next_y3.as_mut() {
                                            *candidate = constrain_view_bounds_scaled(
                                                *candidate,
                                                self.x_scale,
                                                self.y3_scale,
                                                self.x_constraints,
                                                self.y3_constraints,
                                            );
                                        }
                                        if let Some(candidate) = next_y4.as_mut() {
                                            *candidate = constrain_view_bounds_scaled(
                                                *candidate,
                                                self.x_scale,
                                                self.y4_scale,
                                                self.x_constraints,
                                                self.y4_constraints,
                                            );
                                        }

                                        let primary_changed = next != view_bounds;
                                        let y2_changed = next_y2
                                            .zip(view_bounds_y2)
                                            .map(|(next, prev)| next != prev)
                                            .unwrap_or(
                                                next_y2.is_some() && view_bounds_y2.is_none(),
                                            );
                                        let y3_changed = next_y3
                                            .zip(view_bounds_y3)
                                            .map(|(next, prev)| next != prev)
                                            .unwrap_or(
                                                next_y3.is_some() && view_bounds_y3.is_none(),
                                            );
                                        let y4_changed = next_y4
                                            .zip(view_bounds_y4)
                                            .map(|(next, prev)| next != prev)
                                            .unwrap_or(
                                                next_y4.is_some() && view_bounds_y4.is_none(),
                                            );
                                        let show_y2_axis = self.show_y2_axis;
                                        let lock_y2_axis = self.lock_y2.zoom;
                                        let show_y3_axis = self.show_y3_axis;
                                        let lock_y3_axis = self.lock_y3.zoom;
                                        let show_y4_axis = self.show_y4_axis;
                                        let lock_y4_axis = self.lock_y4.zoom;
                                        let _ = self.update_plot_state(cx.app, |s| {
                                            if primary_changed {
                                                s.view_is_auto = false;
                                                s.view_bounds = Some(next);
                                            }
                                            if show_y2_axis
                                                && !lock_y2_axis
                                                && y2_changed
                                                && next_y2.is_some()
                                            {
                                                s.view_y2_is_auto = false;
                                                s.view_bounds_y2 = next_y2;
                                            }
                                            if show_y3_axis
                                                && !lock_y3_axis
                                                && y3_changed
                                                && next_y3.is_some()
                                            {
                                                s.view_y3_is_auto = false;
                                                s.view_bounds_y3 = next_y3;
                                            }
                                            if show_y4_axis
                                                && !lock_y4_axis
                                                && y4_changed
                                                && next_y4.is_some()
                                            {
                                                s.view_y4_is_auto = false;
                                                s.view_bounds_y4 = next_y4;
                                            }
                                        });
                                    }
                                }
                            }
                        }

                        self.box_zoom_start = None;
                        self.box_zoom_current = None;
                        self.box_zoom_button = None;
                        self.box_zoom_required_mods = None;
                        self.pan_last_pos = None;
                        self.hover = None;

                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    } else if self.pan_button == Some(*button)
                        && self.pan_start_pos.take().is_some()
                    {
                        if cx.captured == Some(cx.node) {
                            cx.release_pointer_capture();
                        }
                        self.pan_button = None;
                        self.pan_target = None;
                        self.pan_last_pos = None;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                }
            }
            Event::Pointer(PointerEvent::Wheel {
                position,
                delta,
                modifiers,
                ..
            }) => {
                self.last_pointer_pos = Some(*position);
                let (
                    y_axis_gap,
                    y_axis_right_gap,
                    y_axis_right2_gap,
                    y_axis_right3_gap,
                    x_axis_gap,
                ) = self.axis_gaps();
                let layout = PlotLayout::from_bounds(
                    cx.bounds,
                    self.style.padding,
                    y_axis_gap,
                    y_axis_right_gap,
                    y_axis_right2_gap,
                    y_axis_right3_gap,
                    x_axis_gap,
                );
                if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                    return;
                }

                let Some(region) = layout.hit_test_region(*position) else {
                    return;
                };
                if self.box_zoom_start.is_some() || self.query_drag_start.is_some() {
                    return;
                }

                if let Some(required) = self.input_map.wheel_zoom_mod
                    && !required.is_pressed(*modifiers)
                {
                    return;
                }

                let delta_y = delta.y.0;
                if !delta_y.is_finite() {
                    return;
                }

                let zoom = clamp_zoom_factors(2.0_f32.powf(delta_y * 0.0025));
                let mut zoom_x = zoom;
                let mut zoom_y1 = zoom;
                let mut zoom_y2 = zoom;
                let mut zoom_y3 = zoom;
                let mut zoom_y4 = zoom;

                match region {
                    PlotRegion::Plot => {
                        if modifiers.shift {
                            zoom_y1 = 1.0;
                            zoom_y2 = 1.0;
                            zoom_y3 = 1.0;
                            zoom_y4 = 1.0;
                        } else if modifiers.ctrl {
                            zoom_x = 1.0;
                        }
                    }
                    PlotRegion::XAxis => {
                        zoom_y1 = 1.0;
                        zoom_y2 = 1.0;
                        zoom_y3 = 1.0;
                        zoom_y4 = 1.0;
                    }
                    PlotRegion::YAxis(axis) => {
                        zoom_x = 1.0;
                        zoom_y1 = 1.0;
                        zoom_y2 = 1.0;
                        zoom_y3 = 1.0;
                        zoom_y4 = 1.0;
                        match axis {
                            YAxis::Left => zoom_y1 = zoom,
                            YAxis::Right => zoom_y2 = zoom,
                            YAxis::Right2 => zoom_y3 = zoom,
                            YAxis::Right3 => zoom_y4 = zoom,
                        }
                    }
                }

                if self.lock_x.zoom {
                    zoom_x = 1.0;
                }
                if self.lock_y.zoom {
                    zoom_y1 = 1.0;
                }
                if self.lock_y2.zoom {
                    zoom_y2 = 1.0;
                }
                if self.lock_y3.zoom {
                    zoom_y3 = 1.0;
                }
                if self.lock_y4.zoom {
                    zoom_y4 = 1.0;
                }

                let no_right_zoom = (!self.show_y2_axis || zoom_y2 == 1.0)
                    && (!self.show_y3_axis || zoom_y3 == 1.0)
                    && (!self.show_y4_axis || zoom_y4 == 1.0);
                if zoom_x == 1.0 && zoom_y1 == 1.0 && no_right_zoom {
                    return;
                }

                let state = self.read_plot_state(cx.app);
                let view_bounds = self.current_view_bounds(cx.app, &state);
                let view_bounds_y2 = self.current_view_bounds_y2(cx.app, &state, view_bounds);
                let view_bounds_y3 = self.current_view_bounds_y3(cx.app, &state, view_bounds);
                let view_bounds_y4 = self.current_view_bounds_y4(cx.app, &state, view_bounds);
                let local = local_from_absolute(layout.plot.origin, *position);
                let Some(next) = zoom_view_at_px_scaled(
                    view_bounds,
                    layout.plot.size,
                    local,
                    zoom_x,
                    zoom_y1,
                    self.x_scale,
                    self.y_scale,
                ) else {
                    return;
                };
                let mut next_y2 = (!self.lock_y2.zoom)
                    .then(|| {
                        view_bounds_y2.and_then(|vb| {
                            zoom_view_at_px_scaled(
                                vb,
                                layout.plot.size,
                                local,
                                zoom_x,
                                zoom_y2,
                                self.x_scale,
                                self.y2_scale,
                            )
                        })
                    })
                    .flatten();
                let mut next_y3 = (!self.lock_y3.zoom)
                    .then(|| {
                        view_bounds_y3.and_then(|vb| {
                            zoom_view_at_px_scaled(
                                vb,
                                layout.plot.size,
                                local,
                                zoom_x,
                                zoom_y3,
                                self.x_scale,
                                self.y3_scale,
                            )
                        })
                    })
                    .flatten();
                let mut next_y4 = (!self.lock_y4.zoom)
                    .then(|| {
                        view_bounds_y4.and_then(|vb| {
                            zoom_view_at_px_scaled(
                                vb,
                                layout.plot.size,
                                local,
                                zoom_x,
                                zoom_y4,
                                self.x_scale,
                                self.y4_scale,
                            )
                        })
                    })
                    .flatten();
                let data_bounds = self.read_data_bounds(cx.app);
                let mut next = if self.style.clamp_to_data_bounds {
                    clamp_view_to_data_scaled(
                        next,
                        data_bounds,
                        self.style.overscroll_fraction,
                        self.x_scale,
                        self.y_scale,
                    )
                } else {
                    next
                };
                if self.style.clamp_to_data_bounds {
                    if let (Some(candidate), Some(bounds_y2)) =
                        (next_y2.as_mut(), self.read_data_bounds_y2(cx.app))
                    {
                        *candidate = clamp_view_to_data_scaled(
                            *candidate,
                            bounds_y2,
                            self.style.overscroll_fraction,
                            self.x_scale,
                            self.y2_scale,
                        );
                    }
                    if let (Some(candidate), Some(bounds_y3)) =
                        (next_y3.as_mut(), self.read_data_bounds_y3(cx.app))
                    {
                        *candidate = clamp_view_to_data_scaled(
                            *candidate,
                            bounds_y3,
                            self.style.overscroll_fraction,
                            self.x_scale,
                            self.y3_scale,
                        );
                    }
                    if let (Some(candidate), Some(bounds_y4)) =
                        (next_y4.as_mut(), self.read_data_bounds_y4(cx.app))
                    {
                        *candidate = clamp_view_to_data_scaled(
                            *candidate,
                            bounds_y4,
                            self.style.overscroll_fraction,
                            self.x_scale,
                            self.y4_scale,
                        );
                    }
                }

                next = apply_axis_locks(view_bounds, next, self.lock_x.zoom, self.lock_y.zoom);
                if let (Some(vb_y2), Some(candidate)) = (view_bounds_y2, next_y2.as_mut()) {
                    *candidate =
                        apply_axis_locks(vb_y2, *candidate, self.lock_x.zoom, self.lock_y2.zoom);
                }
                if let (Some(vb_y3), Some(candidate)) = (view_bounds_y3, next_y3.as_mut()) {
                    *candidate =
                        apply_axis_locks(vb_y3, *candidate, self.lock_x.zoom, self.lock_y3.zoom);
                }
                if let (Some(vb_y4), Some(candidate)) = (view_bounds_y4, next_y4.as_mut()) {
                    *candidate =
                        apply_axis_locks(vb_y4, *candidate, self.lock_x.zoom, self.lock_y4.zoom);
                }

                next = constrain_view_bounds_scaled(
                    next,
                    self.x_scale,
                    self.y_scale,
                    self.x_constraints,
                    self.y_constraints,
                );
                if let Some(candidate) = next_y2.as_mut() {
                    *candidate = constrain_view_bounds_scaled(
                        *candidate,
                        self.x_scale,
                        self.y2_scale,
                        self.x_constraints,
                        self.y2_constraints,
                    );
                }
                if let Some(candidate) = next_y3.as_mut() {
                    *candidate = constrain_view_bounds_scaled(
                        *candidate,
                        self.x_scale,
                        self.y3_scale,
                        self.x_constraints,
                        self.y3_constraints,
                    );
                }
                if let Some(candidate) = next_y4.as_mut() {
                    *candidate = constrain_view_bounds_scaled(
                        *candidate,
                        self.x_scale,
                        self.y4_scale,
                        self.x_constraints,
                        self.y4_constraints,
                    );
                }

                let primary_changed = next != view_bounds;
                let y2_changed = next_y2
                    .zip(view_bounds_y2)
                    .map(|(next, prev)| next != prev)
                    .unwrap_or(next_y2.is_some() && view_bounds_y2.is_none());
                let y3_changed = next_y3
                    .zip(view_bounds_y3)
                    .map(|(next, prev)| next != prev)
                    .unwrap_or(next_y3.is_some() && view_bounds_y3.is_none());
                let y4_changed = next_y4
                    .zip(view_bounds_y4)
                    .map(|(next, prev)| next != prev)
                    .unwrap_or(next_y4.is_some() && view_bounds_y4.is_none());
                let show_y2_axis = self.show_y2_axis;
                let lock_y2_axis = self.lock_y2.zoom;
                let show_y3_axis = self.show_y3_axis;
                let lock_y3_axis = self.lock_y3.zoom;
                let show_y4_axis = self.show_y4_axis;
                let lock_y4_axis = self.lock_y4.zoom;
                let _ = self.update_plot_state(cx.app, |s| {
                    if primary_changed {
                        s.view_is_auto = false;
                        s.view_bounds = Some(next);
                    }
                    if show_y2_axis && !lock_y2_axis && y2_changed && next_y2.is_some() {
                        s.view_y2_is_auto = false;
                        s.view_bounds_y2 = next_y2;
                    }
                    if show_y3_axis && !lock_y3_axis && y3_changed && next_y3.is_some() {
                        s.view_y3_is_auto = false;
                        s.view_bounds_y3 = next_y3;
                    }
                    if show_y4_axis && !lock_y4_axis && y4_changed && next_y4.is_some() {
                        s.view_y4_is_auto = false;
                        s.view_bounds_y4 = next_y4;
                    }
                });
                cx.request_focus(cx.node);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            Event::Pointer(PointerEvent::Move { position, .. }) => {
                self.last_pointer_pos = Some(*position);
                let (
                    y_axis_gap,
                    y_axis_right_gap,
                    y_axis_right2_gap,
                    y_axis_right3_gap,
                    x_axis_gap,
                ) = self.axis_gaps();
                let layout = PlotLayout::from_bounds(
                    cx.bounds,
                    self.style.padding,
                    y_axis_gap,
                    y_axis_right_gap,
                    y_axis_right2_gap,
                    y_axis_right3_gap,
                    x_axis_gap,
                );
                if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                    return;
                }

                let pan_active = self.pan_button.is_some()
                    && self.pan_target.is_some()
                    && (self.pan_start_pos.is_some() || self.pan_last_pos.is_some());
                if self.box_zoom_start.is_none() && !pan_active {
                    if let Some((legend, rows)) = self.legend_layout(layout)
                        && contains_point(legend, *position)
                    {
                        let cursor_changed = self.cursor_px.take().is_some();

                        let series_index = rows
                            .iter()
                            .enumerate()
                            .find(|(_i, r)| contains_point(**r, *position))
                            .map(|(i, _r)| i);

                        let hovered_id =
                            series_index.and_then(|i| self.legend_entries.get(i).map(|e| e.id));

                        if self.legend_hover != hovered_id {
                            self.legend_hover = hovered_id;
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }

                        if self.hover.is_some() {
                            self.hover = None;
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }

                        if cursor_changed {
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }

                        cx.stop_propagation();
                        return;
                    }

                    if self.legend_hover.take().is_some() {
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                }

                if self.query_drag_start.is_some() {
                    let local = local_from_absolute(layout.plot.origin, *position);
                    self.cursor_px = Some(local);
                    self.query_drag_current = Some(local);
                    self.hover = None;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if self.box_zoom_start.is_some() {
                    let local = local_from_absolute(layout.plot.origin, *position);
                    self.cursor_px = Some(local);
                    self.box_zoom_current = Some(local);
                    self.hover = None;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if let Some(start) = self.pan_start_pos
                    && self.pan_button.is_some()
                    && let Some(target) = self.pan_target
                {
                    let last = self.pan_last_pos.unwrap_or(start);
                    self.cursor_px = None;
                    let dx_px_raw = position.x.0 - last.x.0;
                    let dy_px_raw = position.y.0 - last.y.0;

                    if dx_px_raw == 0.0 && dy_px_raw == 0.0 {
                        cx.stop_propagation();
                        return;
                    }

                    let mut dx_px = if self.lock_x.pan { 0.0 } else { dx_px_raw };
                    let mut dy_px_y1 = if self.lock_y.pan { 0.0 } else { dy_px_raw };
                    let mut dy_px_y2 = if self.lock_y2.pan { 0.0 } else { dy_px_raw };
                    let mut dy_px_y3 = if self.lock_y3.pan { 0.0 } else { dy_px_raw };
                    let mut dy_px_y4 = if self.lock_y4.pan { 0.0 } else { dy_px_raw };

                    match target {
                        PlotRegion::Plot => {}
                        PlotRegion::XAxis => {
                            dy_px_y1 = 0.0;
                            dy_px_y2 = 0.0;
                            dy_px_y3 = 0.0;
                            dy_px_y4 = 0.0;
                        }
                        PlotRegion::YAxis(axis) => {
                            dx_px = 0.0;
                            dy_px_y1 = 0.0;
                            dy_px_y2 = 0.0;
                            dy_px_y3 = 0.0;
                            dy_px_y4 = 0.0;
                            match axis {
                                YAxis::Left => {
                                    dy_px_y1 = if self.lock_y.pan { 0.0 } else { dy_px_raw }
                                }
                                YAxis::Right => {
                                    dy_px_y2 = if self.lock_y2.pan { 0.0 } else { dy_px_raw }
                                }
                                YAxis::Right2 => {
                                    dy_px_y3 = if self.lock_y3.pan { 0.0 } else { dy_px_raw }
                                }
                                YAxis::Right3 => {
                                    dy_px_y4 = if self.lock_y4.pan { 0.0 } else { dy_px_raw }
                                }
                            }
                        }
                    }

                    let no_right_pan = (!self.show_y2_axis || dy_px_y2 == 0.0)
                        && (!self.show_y3_axis || dy_px_y3 == 0.0)
                        && (!self.show_y4_axis || dy_px_y4 == 0.0);
                    if dx_px == 0.0 && dy_px_y1 == 0.0 && no_right_pan {
                        self.pan_last_pos = Some(*position);
                        cx.stop_propagation();
                        return;
                    }

                    let state = self.read_plot_state(cx.app);
                    let view_bounds = self.current_view_bounds(cx.app, &state);
                    let view_bounds_y2 = self.current_view_bounds_y2(cx.app, &state, view_bounds);
                    let view_bounds_y3 = self.current_view_bounds_y3(cx.app, &state, view_bounds);
                    let view_bounds_y4 = self.current_view_bounds_y4(cx.app, &state, view_bounds);
                    let Some(next) = pan_view_by_px_scaled(
                        view_bounds,
                        layout.plot.size,
                        dx_px,
                        dy_px_y1,
                        self.x_scale,
                        self.y_scale,
                    ) else {
                        return;
                    };
                    let mut next_y2 = (!self.lock_y2.pan)
                        .then(|| {
                            view_bounds_y2.and_then(|vb| {
                                pan_view_by_px_scaled(
                                    vb,
                                    layout.plot.size,
                                    dx_px,
                                    dy_px_y2,
                                    self.x_scale,
                                    self.y2_scale,
                                )
                            })
                        })
                        .flatten();
                    let mut next_y3 = (!self.lock_y3.pan)
                        .then(|| {
                            view_bounds_y3.and_then(|vb| {
                                pan_view_by_px_scaled(
                                    vb,
                                    layout.plot.size,
                                    dx_px,
                                    dy_px_y3,
                                    self.x_scale,
                                    self.y3_scale,
                                )
                            })
                        })
                        .flatten();
                    let mut next_y4 = (!self.lock_y4.pan)
                        .then(|| {
                            view_bounds_y4.and_then(|vb| {
                                pan_view_by_px_scaled(
                                    vb,
                                    layout.plot.size,
                                    dx_px,
                                    dy_px_y4,
                                    self.x_scale,
                                    self.y4_scale,
                                )
                            })
                        })
                        .flatten();
                    let data_bounds = self.read_data_bounds(cx.app);
                    let mut next = if self.style.clamp_to_data_bounds {
                        clamp_view_to_data_scaled(
                            next,
                            data_bounds,
                            self.style.overscroll_fraction,
                            self.x_scale,
                            self.y_scale,
                        )
                    } else {
                        next
                    };
                    if self.style.clamp_to_data_bounds {
                        if let (Some(candidate), Some(bounds_y2)) =
                            (next_y2.as_mut(), self.read_data_bounds_y2(cx.app))
                        {
                            *candidate = clamp_view_to_data_scaled(
                                *candidate,
                                bounds_y2,
                                self.style.overscroll_fraction,
                                self.x_scale,
                                self.y2_scale,
                            );
                        }
                        if let (Some(candidate), Some(bounds_y3)) =
                            (next_y3.as_mut(), self.read_data_bounds_y3(cx.app))
                        {
                            *candidate = clamp_view_to_data_scaled(
                                *candidate,
                                bounds_y3,
                                self.style.overscroll_fraction,
                                self.x_scale,
                                self.y3_scale,
                            );
                        }
                        if let (Some(candidate), Some(bounds_y4)) =
                            (next_y4.as_mut(), self.read_data_bounds_y4(cx.app))
                        {
                            *candidate = clamp_view_to_data_scaled(
                                *candidate,
                                bounds_y4,
                                self.style.overscroll_fraction,
                                self.x_scale,
                                self.y4_scale,
                            );
                        }
                    }

                    next = apply_axis_locks(view_bounds, next, self.lock_x.pan, self.lock_y.pan);
                    if let (Some(vb_y2), Some(candidate)) = (view_bounds_y2, next_y2.as_mut()) {
                        *candidate =
                            apply_axis_locks(vb_y2, *candidate, self.lock_x.pan, self.lock_y2.pan);
                    }
                    if let (Some(vb_y3), Some(candidate)) = (view_bounds_y3, next_y3.as_mut()) {
                        *candidate =
                            apply_axis_locks(vb_y3, *candidate, self.lock_x.pan, self.lock_y3.pan);
                    }
                    if let (Some(vb_y4), Some(candidate)) = (view_bounds_y4, next_y4.as_mut()) {
                        *candidate =
                            apply_axis_locks(vb_y4, *candidate, self.lock_x.pan, self.lock_y4.pan);
                    }

                    next = constrain_view_bounds_scaled(
                        next,
                        self.x_scale,
                        self.y_scale,
                        self.x_constraints,
                        self.y_constraints,
                    );
                    if let Some(candidate) = next_y2.as_mut() {
                        *candidate = constrain_view_bounds_scaled(
                            *candidate,
                            self.x_scale,
                            self.y2_scale,
                            self.x_constraints,
                            self.y2_constraints,
                        );
                    }
                    if let Some(candidate) = next_y3.as_mut() {
                        *candidate = constrain_view_bounds_scaled(
                            *candidate,
                            self.x_scale,
                            self.y3_scale,
                            self.x_constraints,
                            self.y3_constraints,
                        );
                    }
                    if let Some(candidate) = next_y4.as_mut() {
                        *candidate = constrain_view_bounds_scaled(
                            *candidate,
                            self.x_scale,
                            self.y4_scale,
                            self.x_constraints,
                            self.y4_constraints,
                        );
                    }

                    let primary_changed = next != view_bounds;
                    let y2_changed = next_y2
                        .zip(view_bounds_y2)
                        .map(|(next, prev)| next != prev)
                        .unwrap_or(next_y2.is_some() && view_bounds_y2.is_none());
                    let y3_changed = next_y3
                        .zip(view_bounds_y3)
                        .map(|(next, prev)| next != prev)
                        .unwrap_or(next_y3.is_some() && view_bounds_y3.is_none());
                    let y4_changed = next_y4
                        .zip(view_bounds_y4)
                        .map(|(next, prev)| next != prev)
                        .unwrap_or(next_y4.is_some() && view_bounds_y4.is_none());
                    let show_y2_axis = self.show_y2_axis;
                    let lock_y2_axis = self.lock_y2.pan;
                    let show_y3_axis = self.show_y3_axis;
                    let lock_y3_axis = self.lock_y3.pan;
                    let show_y4_axis = self.show_y4_axis;
                    let lock_y4_axis = self.lock_y4.pan;
                    let _ = self.update_plot_state(cx.app, |s| {
                        if primary_changed {
                            s.view_is_auto = false;
                            s.view_bounds = Some(next);
                        }
                        if show_y2_axis && !lock_y2_axis && y2_changed && next_y2.is_some() {
                            s.view_y2_is_auto = false;
                            s.view_bounds_y2 = next_y2;
                        }
                        if show_y3_axis && !lock_y3_axis && y3_changed && next_y3.is_some() {
                            s.view_y3_is_auto = false;
                            s.view_bounds_y3 = next_y3;
                        }
                        if show_y4_axis && !lock_y4_axis && y4_changed && next_y4.is_some() {
                            s.view_y4_is_auto = false;
                            s.view_bounds_y4 = next_y4;
                        }
                    });
                    self.pan_last_pos = Some(*position);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let inside = contains_point(layout.plot, *position);

                let prev_cursor = self.cursor_px;
                self.cursor_px = inside.then(|| local_from_absolute(layout.plot.origin, *position));
                if prev_cursor != self.cursor_px {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }

                let next_hover = if inside {
                    let model_revision = self.model.revision(cx.app).unwrap_or(0);
                    let scale_factor = self.last_scale_factor;

                    let state = self.read_plot_state(cx.app);
                    let view_bounds = self.current_view_bounds(cx.app, &state);
                    let view_bounds_y2 = self.current_view_bounds_y2(cx.app, &state, view_bounds);
                    let view_bounds_y3 = self.current_view_bounds_y3(cx.app, &state, view_bounds);
                    let view_bounds_y4 = self.current_view_bounds_y4(cx.app, &state, view_bounds);
                    let hidden = &state.hidden_series;
                    let pinned = state.pinned_series;

                    let local = local_from_absolute(layout.plot.origin, *position);

                    self.model
                        .read(cx.app, |_app, m| m.clone())
                        .ok()
                        .and_then(|model| {
                            self.layer.hit_test(
                                &model,
                                PlotHitTestArgs {
                                    model_revision,
                                    plot_size: layout.plot.size,
                                    view_bounds,
                                    view_bounds_y2,
                                    view_bounds_y3,
                                    view_bounds_y4,
                                    x_scale: self.x_scale,
                                    y_scale: self.y_scale,
                                    y2_scale: self.y2_scale,
                                    y3_scale: self.y3_scale,
                                    y4_scale: self.y4_scale,
                                    scale_factor,
                                    local,
                                    style: self.style,
                                    hover_threshold: self.style.hover_threshold,
                                    hidden,
                                    pinned,
                                },
                            )
                        })
                } else {
                    None
                };

                if self.hover != next_hover {
                    self.hover = next_hover;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }

                // Publish interaction output eagerly so linked-plot coordinators can react to
                // pointer movement without waiting for the next paint.
                let state = self.read_plot_state(cx.app);
                let view_bounds = self.current_view_bounds(cx.app, &state);
                let view_bounds_y2 = self.current_view_bounds_y2(cx.app, &state, view_bounds);
                let view_bounds_y3 = self.current_view_bounds_y3(cx.app, &state, view_bounds);
                let view_bounds_y4 = self.current_view_bounds_y4(cx.app, &state, view_bounds);
                self.publish_current_output_snapshot(
                    cx.app,
                    layout,
                    &state,
                    view_bounds,
                    view_bounds_y2,
                    view_bounds_y3,
                    view_bounds_y4,
                );
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.model, Invalidation::Paint);
        if let Some(state) = &self.plot_state_model {
            cx.observe_model(state, Invalidation::Paint);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.model, Invalidation::Paint);
        if let Some(state) = &self.plot_state_model {
            cx.observe_model(state, Invalidation::Paint);
        }
        cx.observe_global::<TextFontStackKey>(Invalidation::Paint);
        self.last_scale_factor = cx.scale_factor;

        self.ensure_required_axes_enabled(cx.app);

        let theme = cx.theme().snapshot();
        let font_stack_key = cx
            .app
            .global::<TextFontStackKey>()
            .map(|k| k.0)
            .unwrap_or(0);
        let background = self
            .style
            .background
            .unwrap_or(theme.colors.panel_background);
        let border = self.style.border.unwrap_or(theme.colors.panel_border);

        let axis_color = self.style.axis_color.unwrap_or(theme.colors.panel_border);
        let grid_color = self.style.grid_color.unwrap_or(Color {
            a: 0.35,
            ..theme.colors.panel_border
        });
        let label_color = self.style.label_color.unwrap_or(theme.colors.text_muted);
        let crosshair_color = self.style.crosshair_color.unwrap_or(Color {
            a: 0.65,
            ..theme.colors.accent
        });
        let selection_border = crosshair_color;
        let selection_fill = Color {
            a: (crosshair_color.a * 0.18).clamp(0.06, 0.22),
            ..crosshair_color
        };
        let tooltip_background = self
            .style
            .tooltip_background
            .unwrap_or(theme.colors.menu_background);
        let tooltip_border = self
            .style
            .tooltip_border
            .unwrap_or(theme.colors.menu_border);
        let tooltip_text_color = self
            .style
            .tooltip_text_color
            .unwrap_or(theme.colors.text_primary);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background,
            border: fret_core::Edges::all(self.style.border_width),
            border_color: border,
            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        let min_axis = self.style.axis_gap;
        self.x_axis_thickness = Px(self.x_axis_thickness.0.max(min_axis.0));
        self.y_axis_thickness = Px(self.y_axis_thickness.0.max(min_axis.0));
        if self.show_y2_axis {
            self.y_axis_right_thickness = Px(self.y_axis_right_thickness.0.max(min_axis.0));
        }
        if self.show_y3_axis {
            self.y_axis_right2_thickness = Px(self.y_axis_right2_thickness.0.max(min_axis.0));
        }
        if self.show_y4_axis {
            self.y_axis_right3_thickness = Px(self.y_axis_right3_thickness.0.max(min_axis.0));
        }

        // Layout can depend on text metrics (axis thickness). Converge in up to two passes.
        let mut layout = PlotLayout::from_bounds(
            cx.bounds,
            self.style.padding,
            self.y_axis_thickness,
            self.y_axis_right_thickness,
            self.y_axis_right2_thickness,
            self.y_axis_right3_thickness,
            self.x_axis_thickness,
        );
        let state = self.read_plot_state(cx.app);
        let view_bounds = self.current_view_bounds(cx.app, &state);
        let view_bounds_y2 = self.current_view_bounds_y2(cx.app, &state, view_bounds);
        let view_bounds_y3 = self.current_view_bounds_y3(cx.app, &state, view_bounds);
        let view_bounds_y4 = self.current_view_bounds_y4(cx.app, &state, view_bounds);

        // Axis labels can expand axis thickness; keep plot-local interaction points stable by
        // shifting stored coordinates when the plot origin moves.
        for _ in 0..2 {
            let changed = self.rebuild_axis_labels_if_needed(
                cx,
                layout,
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                theme.revision,
                font_stack_key,
            );
            if !changed {
                break;
            }

            let next_layout = PlotLayout::from_bounds(
                cx.bounds,
                self.style.padding,
                self.y_axis_thickness,
                self.y_axis_right_thickness,
                self.y_axis_right2_thickness,
                self.y_axis_right3_thickness,
                self.x_axis_thickness,
            );

            let dx = next_layout.plot.origin.x.0 - layout.plot.origin.x.0;
            let dy = next_layout.plot.origin.y.0 - layout.plot.origin.y.0;
            if dx != 0.0 || dy != 0.0 {
                let delta = Point::new(Px(dx), Px(dy));

                let shift = |p: &mut Point| {
                    p.x.0 -= delta.x.0;
                    p.y.0 -= delta.y.0;
                };

                if let Some(p) = self.cursor_px.as_mut() {
                    shift(p);
                }
                if let Some(p) = self.box_zoom_start.as_mut() {
                    shift(p);
                }
                if let Some(p) = self.box_zoom_current.as_mut() {
                    shift(p);
                }
                if let Some(p) = self.query_drag_start.as_mut() {
                    shift(p);
                }
                if let Some(p) = self.query_drag_current.as_mut() {
                    shift(p);
                }

                self.hover = None;
            }

            layout = next_layout;
        }

        self.publish_current_output_snapshot(
            cx.app,
            layout,
            &state,
            view_bounds,
            view_bounds_y2,
            view_bounds_y3,
            view_bounds_y4,
        );
        self.rebuild_legend_if_needed(cx, theme.revision, font_stack_key);

        // Grid + series + hover are clipped to the plot area.
        cx.scene.push(SceneOp::PushClipRect { rect: layout.plot });

        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
            // Grid: align to axis ticks so labels and grid are consistent (ImPlot-style).
            let x_ticks = &self.axis_ticks_x;
            let y_ticks = &self.axis_ticks_y;

            let transform = PlotTransform {
                viewport: layout.plot,
                data: view_bounds,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
            };

            for v in x_ticks.iter().copied() {
                let Some(x) = transform.data_x_to_px(v) else {
                    continue;
                };
                let x = Px(x.0.round());
                let background =
                    if self.x_scale == AxisScale::Log10 && log10_decade_exponent(v).is_none() {
                        dim_color(grid_color, 0.45)
                    } else {
                        grid_color
                    };
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(x, layout.plot.origin.y),
                        Size::new(Px(1.0), layout.plot.size.height),
                    ),
                    background,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            for v in y_ticks.iter().copied() {
                let Some(y) = transform.data_y_to_px(v) else {
                    continue;
                };
                let y = Px(y.0.round());
                let background =
                    if self.y_scale == AxisScale::Log10 && log10_decade_exponent(v).is_none() {
                        dim_color(grid_color, 0.45)
                    } else {
                        grid_color
                    };
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: Rect::new(
                        Point::new(layout.plot.origin.x, y),
                        Size::new(layout.plot.size.width, Px(1.0)),
                    ),
                    background,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            let emphasized = self.style.emphasize_hovered_series;
            let dim_alpha = self.style.dimmed_series_alpha;
            let series_meta: Vec<SeriesMeta> = self
                .model
                .read(cx.app, |_app, m| L::series_meta(m))
                .unwrap_or_default();
            let pinned = state
                .pinned_series
                .filter(|id| series_meta.iter().any(|s| s.id == *id));
            let hidden = &state.hidden_series;

            let emphasized_series = pinned
                .or(self.hover.map(|h| h.series_id))
                .or(self.legend_hover);

            for quad in self.rebuild_quads_if_needed(
                cx,
                layout.plot,
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                hidden,
            ) {
                cx.scene.push(SceneOp::Quad {
                    order: quad.order,
                    rect: offset_rect(quad.rect_local, layout.plot.origin),
                    background: quad.background,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            for (series_id, path, color) in self.rebuild_paths_if_needed(
                cx,
                layout.plot,
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                hidden,
            ) {
                let color = if emphasized {
                    if let Some(emphasized) = emphasized_series
                        && emphasized != series_id
                    {
                        dim_color(color, dim_alpha)
                    } else {
                        color
                    }
                } else {
                    color
                };
                cx.scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: layout.plot.origin,
                    path,
                    color,
                });
            }

            if let Some(cursor) = self.cursor_px {
                let x = Px((layout.plot.origin.x.0 + cursor.x.0).round());
                let y = Px((layout.plot.origin.y.0 + cursor.y.0).round());
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(3),
                    rect: Rect::new(
                        Point::new(x, layout.plot.origin.y),
                        Size::new(Px(1.0), layout.plot.size.height),
                    ),
                    background: crosshair_color,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });

                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(3),
                    rect: Rect::new(
                        Point::new(layout.plot.origin.x, y),
                        Size::new(layout.plot.size.width, Px(1.0)),
                    ),
                    background: crosshair_color,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }
            // Linked cursor (typically driven by `LinkedPlotGroup`).
            if self.cursor_px.is_none()
                && let Some(linked_x) = state.linked_cursor_x
                && linked_x.is_finite()
            {
                let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size);
                let transform = PlotTransform {
                    viewport: local_viewport,
                    data: view_bounds,
                    x_scale: self.x_scale,
                    y_scale: self.y_scale,
                };
                if let Some(px) = transform.data_x_to_px(linked_x) {
                    let x0 = px.0.clamp(0.0, layout.plot.size.width.0);
                    let x = Px((layout.plot.origin.x.0 + x0).round());
                    let linked_color = Color {
                        a: (crosshair_color.a * 0.55).clamp(0.05, 1.0),
                        ..crosshair_color
                    };
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(3),
                        rect: Rect::new(
                            Point::new(x, layout.plot.origin.y),
                            Size::new(Px(1.0), layout.plot.size.height),
                        ),
                        background: linked_color,
                        border: fret_core::Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: fret_core::Corners::all(Px(0.0)),
                    });
                }
            }

            if let Some(hover) = self.hover {
                let hx = Px((layout.plot.origin.x.0 + hover.plot_px.x.0).round());
                let hy = Px((layout.plot.origin.y.0 + hover.plot_px.y.0).round());

                let dot_size = Px(6.0);
                let dot_origin =
                    Point::new(Px(hx.0 - dot_size.0 * 0.5), Px(hy.0 - dot_size.0 * 0.5));
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect: Rect::new(dot_origin, Size::new(dot_size, dot_size)),
                    background: crosshair_color,
                    border: fret_core::Edges::all(Px(1.0)),
                    border_color: tooltip_border,
                    corner_radii: fret_core::Corners::all(Px(dot_size.0 * 0.5)),
                });
            }

            if let Some(query) = state.query {
                let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size);
                let transform = PlotTransform {
                    viewport: local_viewport,
                    data: view_bounds,
                    x_scale: self.x_scale,
                    y_scale: self.y_scale,
                };

                let a = transform.data_to_px(DataPoint {
                    x: query.x_min,
                    y: query.y_min,
                });
                let b = transform.data_to_px(DataPoint {
                    x: query.x_max,
                    y: query.y_max,
                });

                let x0 = a.x.0.min(b.x.0).clamp(0.0, layout.plot.size.width.0);
                let x1 = a.x.0.max(b.x.0).clamp(0.0, layout.plot.size.width.0);
                let y0 = a.y.0.min(b.y.0).clamp(0.0, layout.plot.size.height.0);
                let y1 = a.y.0.max(b.y.0).clamp(0.0, layout.plot.size.height.0);
                let w = x1 - x0;
                let h = y1 - y0;
                if w >= 1.0 && h >= 1.0 {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(5),
                        rect: Rect::new(
                            Point::new(
                                Px(layout.plot.origin.x.0 + x0),
                                Px(layout.plot.origin.y.0 + y0),
                            ),
                            Size::new(Px(w), Px(h)),
                        ),
                        background: selection_fill,
                        border: fret_core::Edges::all(Px(1.0)),
                        border_color: selection_border,
                        corner_radii: fret_core::Corners::all(Px(0.0)),
                    });
                }
            }

            if let (Some(start), Some(end)) = (self.query_drag_start, self.query_drag_current) {
                let x0 = start.x.0.min(end.x.0).clamp(0.0, layout.plot.size.width.0);
                let x1 = start.x.0.max(end.x.0).clamp(0.0, layout.plot.size.width.0);
                let y0 = start.y.0.min(end.y.0).clamp(0.0, layout.plot.size.height.0);
                let y1 = start.y.0.max(end.y.0).clamp(0.0, layout.plot.size.height.0);
                let w = x1 - x0;
                let h = y1 - y0;
                if w >= 1.0 && h >= 1.0 {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(5),
                        rect: Rect::new(
                            Point::new(
                                Px(layout.plot.origin.x.0 + x0),
                                Px(layout.plot.origin.y.0 + y0),
                            ),
                            Size::new(Px(w), Px(h)),
                        ),
                        background: selection_fill,
                        border: fret_core::Edges::all(Px(1.0)),
                        border_color: selection_border,
                        corner_radii: fret_core::Corners::all(Px(0.0)),
                    });
                }
            }

            if let (Some(start), Some(end)) = (self.box_zoom_start, self.box_zoom_current) {
                let x0 = start.x.0.min(end.x.0).clamp(0.0, layout.plot.size.width.0);
                let x1 = start.x.0.max(end.x.0).clamp(0.0, layout.plot.size.width.0);
                let y0 = start.y.0.min(end.y.0).clamp(0.0, layout.plot.size.height.0);
                let y1 = start.y.0.max(end.y.0).clamp(0.0, layout.plot.size.height.0);
                let w = x1 - x0;
                let h = y1 - y0;
                if w >= 1.0 && h >= 1.0 {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(5),
                        rect: Rect::new(
                            Point::new(
                                Px(layout.plot.origin.x.0 + x0),
                                Px(layout.plot.origin.y.0 + y0),
                            ),
                            Size::new(Px(w), Px(h)),
                        ),
                        background: selection_fill,
                        border: fret_core::Edges::all(Px(1.0)),
                        border_color: selection_border,
                        corner_radii: fret_core::Corners::all(Px(0.0)),
                    });
                }
            }
        }

        cx.scene.push(SceneOp::PopClip);

        // Legend (P0: shown when there is more than one series; can be moved to overlays later).
        if let Some((rect, rows)) = self.legend_layout(layout) {
            let series_overrides: Vec<Option<Color>> = self
                .model
                .read(cx.app, |_app, m| {
                    L::series_meta(m)
                        .into_iter()
                        .map(|s| s.stroke_color)
                        .collect()
                })
                .unwrap_or_default();
            let series_count = self.legend_entries.len();

            let pad = Px(8.0);
            let gap = Px(8.0);
            let swatch_w = Px(14.0);
            let swatch_h = Px(self.style.stroke_width.0.clamp(2.0, 6.0));
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(6),
                rect,
                background: tooltip_background,
                border: fret_core::Edges::all(Px(1.0)),
                border_color: tooltip_border,
                corner_radii: fret_core::Corners::all(Px(6.0)),
            });

            for (i, entry) in self.legend_entries.iter().enumerate() {
                let row = rows.get(i).copied().unwrap_or(rect);
                let row_h = row.size.height;

                let hovered_row = self.legend_hover == Some(entry.id);
                let pinned_row = state.pinned_series == Some(entry.id);
                if hovered_row || pinned_row {
                    let a = if pinned_row { 0.16 } else { 0.10 };
                    let highlight = Color {
                        a,
                        ..crosshair_color
                    };
                    let inset_x = Px(2.0);
                    let highlight_rect = Rect::new(
                        Point::new(Px(row.origin.x.0 + inset_x.0), row.origin.y),
                        Size::new(
                            Px((row.size.width.0 - inset_x.0 * 2.0).max(0.0)),
                            row.size.height,
                        ),
                    );
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(6),
                        rect: highlight_rect,
                        background: highlight,
                        border: fret_core::Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: fret_core::Corners::all(Px(4.0)),
                    });
                }

                let override_color = series_overrides.get(i).copied().flatten();
                let color = resolve_series_color(i, self.style, series_count, override_color);

                let visible = !state.hidden_series.contains(&entry.id);
                let swatch_color = if visible {
                    color
                } else {
                    Color {
                        a: (color.a * 0.20).clamp(0.05, 0.35),
                        ..color
                    }
                };
                let text_color = if visible {
                    tooltip_text_color
                } else {
                    Color {
                        a: (tooltip_text_color.a * 0.55).clamp(0.25, 0.85),
                        ..tooltip_text_color
                    }
                };

                let row_mid = row.origin.y.0 + row_h.0 * 0.5;
                let swatch_x = Px(row.origin.x.0 + pad.0);
                let swatch_y = Px(row_mid - swatch_h.0 * 0.5);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(7),
                    rect: Rect::new(
                        Point::new(swatch_x, swatch_y),
                        Size::new(swatch_w, swatch_h),
                    ),
                    background: swatch_color,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });

                let text_x = Px(swatch_x.0 + swatch_w.0 + gap.0);
                let text_top = row.origin.y.0 + (row_h.0 - entry.text.metrics.size.height.0) * 0.5;
                let origin = Point::new(text_x, Px(text_top + entry.text.metrics.baseline.0));
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(7),
                    origin,
                    text: entry.text.blob,
                    color: text_color,
                });
            }
        }

        // Axes.
        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
            // Y axis line.
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(10),
                rect: Rect::new(
                    layout.plot.origin,
                    Size::new(Px(1.0), layout.plot.size.height),
                ),
                background: axis_color,
                border: fret_core::Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });

            // Right-side Y axis line (Y2/Y3/Y4 share the plot edge).
            let any_right_axis = self.show_y2_axis || self.show_y3_axis || self.show_y4_axis;
            let any_right_width = layout.y_axis_right.size.width.0 > 0.0
                || layout.y_axis_right2.size.width.0 > 0.0
                || layout.y_axis_right3.size.width.0 > 0.0;
            if any_right_axis && any_right_width {
                let x = Px(layout.plot.origin.x.0 + layout.plot.size.width.0 - 1.0);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(10),
                    rect: Rect::new(
                        Point::new(x, layout.plot.origin.y),
                        Size::new(Px(1.0), layout.plot.size.height),
                    ),
                    background: axis_color,
                    border: fret_core::Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            // X axis line.
            let y = Px(layout.plot.origin.y.0 + layout.plot.size.height.0 - 1.0);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(10),
                rect: Rect::new(
                    Point::new(layout.plot.origin.x, y),
                    Size::new(layout.plot.size.width, Px(1.0)),
                ),
                background: axis_color,
                border: fret_core::Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }

        // Axis labels: tick density adapts to viewport + label spacing.
        let x_ticks = &self.axis_ticks_x;
        let y_ticks = &self.axis_ticks_y;
        let y2_ticks = &self.axis_ticks_y2;
        let y3_ticks = &self.axis_ticks_y3;
        let y4_ticks = &self.axis_ticks_y4;

        let transform_y1 = PlotTransform {
            viewport: layout.plot,
            data: view_bounds,
            x_scale: self.x_scale,
            y_scale: self.y_scale,
        };
        let transform_y2 = view_bounds_y2.map(|b| PlotTransform {
            viewport: layout.plot,
            data: b,
            x_scale: self.x_scale,
            y_scale: self.y2_scale,
        });
        let transform_y3 = view_bounds_y3.map(|b| PlotTransform {
            viewport: layout.plot,
            data: b,
            x_scale: self.x_scale,
            y_scale: self.y3_scale,
        });
        let transform_y4 = view_bounds_y4.map(|b| PlotTransform {
            viewport: layout.plot,
            data: b,
            x_scale: self.x_scale,
            y_scale: self.y4_scale,
        });

        for (i, label) in self.axis_labels_x.iter().enumerate() {
            if label.metrics.size.width.0 <= 0.0 {
                continue;
            }
            let Some(v) = x_ticks.get(i).copied() else {
                continue;
            };
            let Some(x) = transform_y1.data_x_to_px(v) else {
                continue;
            };
            let x = Px(x.0.round());

            let top = layout.x_axis.origin.y.0 + 2.0;
            let origin = Point::new(
                Px(x.0 - (label.metrics.size.width.0 * 0.5)),
                Px(top + label.metrics.baseline.0),
            );

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(11),
                origin,
                text: label.blob,
                color: label_color,
            });
        }

        for (i, label) in self.axis_labels_y.iter().enumerate() {
            if label.metrics.size.width.0 <= 0.0 {
                continue;
            }
            let Some(v) = y_ticks.get(i).copied() else {
                continue;
            };
            let Some(y) = transform_y1.data_y_to_px(v) else {
                continue;
            };
            let y = Px(y.0.round());

            let origin_x = layout.y_axis_left.origin.x.0 + layout.y_axis_left.size.width.0
                - label.metrics.size.width.0
                - 4.0;
            let top = y.0 - (label.metrics.size.height.0 * 0.5);
            let origin = Point::new(
                Px(origin_x.max(layout.y_axis_left.origin.x.0)),
                Px(top + label.metrics.baseline.0),
            );

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(11),
                origin,
                text: label.blob,
                color: label_color,
            });
        }

        if self.show_y2_axis {
            if let Some(transform_y2) = transform_y2 {
                for (i, label) in self.axis_labels_y2.iter().enumerate() {
                    if label.metrics.size.width.0 <= 0.0 {
                        continue;
                    }
                    let Some(v) = y2_ticks.get(i).copied() else {
                        continue;
                    };
                    let Some(y) = transform_y2.data_y_to_px(v) else {
                        continue;
                    };
                    let y = Px(y.0.round());

                    let origin_x = layout.y_axis_right.origin.x.0 + 4.0;
                    let top = y.0 - (label.metrics.size.height.0 * 0.5);
                    let origin = Point::new(Px(origin_x), Px(top + label.metrics.baseline.0));

                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(11),
                        origin,
                        text: label.blob,
                        color: label_color,
                    });
                }
            }
        }

        if self.show_y3_axis {
            if let Some(transform_y3) = transform_y3 {
                for (i, label) in self.axis_labels_y3.iter().enumerate() {
                    if label.metrics.size.width.0 <= 0.0 {
                        continue;
                    }
                    let Some(v) = y3_ticks.get(i).copied() else {
                        continue;
                    };
                    let Some(y) = transform_y3.data_y_to_px(v) else {
                        continue;
                    };
                    let y = Px(y.0.round());

                    let origin_x = layout.y_axis_right2.origin.x.0 + 4.0;
                    let top = y.0 - (label.metrics.size.height.0 * 0.5);
                    let origin = Point::new(Px(origin_x), Px(top + label.metrics.baseline.0));

                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(11),
                        origin,
                        text: label.blob,
                        color: label_color,
                    });
                }
            }
        }

        if self.show_y4_axis {
            if let Some(transform_y4) = transform_y4 {
                for (i, label) in self.axis_labels_y4.iter().enumerate() {
                    if label.metrics.size.width.0 <= 0.0 {
                        continue;
                    }
                    let Some(v) = y4_ticks.get(i).copied() else {
                        continue;
                    };
                    let Some(y) = transform_y4.data_y_to_px(v) else {
                        continue;
                    };
                    let y = Px(y.0.round());

                    let origin_x = layout.y_axis_right3.origin.x.0 + 4.0;
                    let top = y.0 - (label.metrics.size.height.0 * 0.5);
                    let origin = Point::new(Px(origin_x), Px(top + label.metrics.baseline.0));

                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(11),
                        origin,
                        text: label.blob,
                        color: label_color,
                    });
                }
            }
        }

        // Tooltip (P0: drawn in the same scene; can be moved to overlays later).
        //
        // Behavior:
        // - Selection tooltips are always shown while dragging (query/box-zoom).
        // - Series tooltips are shown only when hovering near a series.
        // - Mouse coordinate readout is controlled via `LinePlotStyle.mouse_readout`.
        let x_span = (view_bounds.x_max - view_bounds.x_min).abs();
        let y_span = (view_bounds.y_max - view_bounds.y_min).abs();

        let selection_tooltip =
            if let (Some(start), Some(end)) = (self.query_drag_start, self.query_drag_current) {
                query_rect_from_plot_points_raw(
                    view_bounds,
                    layout.plot.size,
                    start,
                    end,
                    self.x_scale,
                    self.y_scale,
                )
                .map(|rect| {
                    let x0 = self.tooltip_x_labels.format(rect.x_min, x_span);
                    let x1 = self.tooltip_x_labels.format(rect.x_max, x_span);
                    let y0 = self.tooltip_y_labels.format(rect.y_min, y_span);
                    let y1 = self.tooltip_y_labels.format(rect.y_max, y_span);
                    let text = format!("query\nx=[{x0}, {x1}]\ny=[{y0}, {y1}]");
                    (end, text)
                })
            } else if let (Some(start), Some(end)) = (self.box_zoom_start, self.box_zoom_current) {
                query_rect_from_plot_points_raw(
                    view_bounds,
                    layout.plot.size,
                    start,
                    end,
                    self.x_scale,
                    self.y_scale,
                )
                .map(|rect| {
                    let x0 = self.tooltip_x_labels.format(rect.x_min, x_span);
                    let x1 = self.tooltip_x_labels.format(rect.x_max, x_span);
                    let y0 = self.tooltip_y_labels.format(rect.y_min, y_span);
                    let y1 = self.tooltip_y_labels.format(rect.y_max, y_span);
                    let text = format!("zoom\nx=[{x0}, {x1}]\ny=[{y0}, {y1}]");
                    (end, text)
                })
            } else {
                None
            };

        let cursor_px = self.cursor_px;
        let cursor_data = cursor_px.and_then(|cursor_px| {
            if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                return None;
            }
            let transform = PlotTransform {
                viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size),
                data: view_bounds,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
            };
            let data = transform.px_to_data(cursor_px);
            (data.x.is_finite() && data.y.is_finite()).then_some(data)
        });

        let linked_x = state.linked_cursor_x.filter(|x| x.is_finite());
        let linked_overlay_active =
            linked_x.is_some() && self.style.linked_cursor_readout == MouseReadoutMode::Overlay;

        if linked_overlay_active {
            let linked_x = linked_x.expect("checked above");

            let hidden = &state.hidden_series;
            let readout_args = PlotCursorReadoutArgs {
                x: linked_x,
                plot_size: layout.plot.size,
                view_bounds,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
                scale_factor: cx.scale_factor,
                hidden,
            };
            let mut readout_rows = self
                .model
                .read(cx.app, |_app, m| L::cursor_readout(m, readout_args))
                .unwrap_or_default();
            apply_readout_policy(
                &mut readout_rows,
                state.pinned_series,
                self.legend_hover,
                self.style.linked_cursor_readout_policy,
            );

            let x_text = self.tooltip_x_labels.format(linked_x, x_span);
            let mut text = format!("x={x_text}");
            for row in readout_rows {
                let y_text = row
                    .y
                    .filter(|y| y.is_finite())
                    .map(|y| match row.y_axis {
                        YAxis::Right if self.show_y2_axis => {
                            let span = view_bounds_y2
                                .map(|b| (b.y_max - b.y_min).abs())
                                .unwrap_or(y_span);
                            self.y2_axis_labels.format(y, span)
                        }
                        YAxis::Right2 if self.show_y3_axis => {
                            let span = view_bounds_y3
                                .map(|b| (b.y_max - b.y_min).abs())
                                .unwrap_or(y_span);
                            self.y3_axis_labels.format(y, span)
                        }
                        YAxis::Right3 if self.show_y4_axis => {
                            let span = view_bounds_y4
                                .map(|b| (b.y_max - b.y_min).abs())
                                .unwrap_or(y_span);
                            self.y4_axis_labels.format(y, span)
                        }
                        _ => self.tooltip_y_labels.format(y, y_span),
                    })
                    .unwrap_or_else(|| "NA".to_string());
                text.push_str(&format!("\n{}: y={y_text}", row.label));
            }

            let font_size = cx
                .theme()
                .metric_by_key("font.size")
                .unwrap_or(cx.theme().metrics.font_size);
            let style = TextStyle {
                font: FontId::default(),
                size: Px((font_size.0 * 0.90).max(10.0)),
                weight: FontWeight::NORMAL,
                line_height: None,
                letter_spacing_em: None,
            };
            let constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            };

            let mut key = 0u64;
            key = Self::hash_u64(key, theme.revision);
            key = Self::hash_u64(key, font_stack_key);
            key = Self::hash_u64(key, u64::from(cx.scale_factor.to_bits()));
            for b in text.as_bytes() {
                key = Self::hash_u64(key, u64::from(*b));
            }
            key = Self::hash_u64(key, Self::text_style_key(&style));

            let needs = self
                .linked_cursor_readout_text
                .as_ref()
                .is_none_or(|t| t.key != key);
            if needs {
                if let Some(prev) = self.linked_cursor_readout_text.take() {
                    cx.services.text().release(prev.blob);
                }
                let prepared = self.prepare_text(cx.services, &text, &style, constraints);
                self.linked_cursor_readout_text = Some(PreparedText {
                    blob: prepared.blob,
                    metrics: prepared.metrics,
                    key,
                });
            }

            if let Some(tt) = self.linked_cursor_readout_text {
                let pad = Px(6.0);
                let margin = Px(6.0);
                let w = Px(tt.metrics.size.width.0 + pad.0 * 2.0);
                let h = Px(tt.metrics.size.height.0 + pad.0 * 2.0);
                let Some(rect) = overlay_rect_in_plot(
                    layout.plot,
                    Size::new(w, h),
                    self.style.linked_cursor_readout_anchor,
                    margin,
                ) else {
                    return;
                };
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(12),
                    rect,
                    background: tooltip_background,
                    border: fret_core::Edges::all(Px(1.0)),
                    border_color: tooltip_border,
                    corner_radii: fret_core::Corners::all(Px(6.0)),
                });

                let origin = Point::new(
                    Px(rect.origin.x.0 + pad.0),
                    Px(rect.origin.y.0 + pad.0 + tt.metrics.baseline.0),
                );
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(13),
                    origin,
                    text: tt.blob,
                    color: tooltip_text_color,
                });
            }
        }

        if self.style.mouse_readout == MouseReadoutMode::Overlay && !linked_overlay_active {
            if let Some(cursor_data) = cursor_data {
                let x_text = self.tooltip_x_labels.format(cursor_data.x, x_span);
                let y_text = self.tooltip_y_labels.format(cursor_data.y, y_span);
                let text = format!("x={x_text}  y={y_text}");

                let font_size = cx
                    .theme()
                    .metric_by_key("font.size")
                    .unwrap_or(cx.theme().metrics.font_size);
                let style = TextStyle {
                    font: FontId::default(),
                    size: Px((font_size.0 * 0.90).max(10.0)),
                    weight: FontWeight::NORMAL,
                    line_height: None,
                    letter_spacing_em: None,
                };
                let constraints = TextConstraints {
                    max_width: None,
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor: cx.scale_factor,
                };

                let mut key = 0u64;
                key = Self::hash_u64(key, theme.revision);
                key = Self::hash_u64(key, font_stack_key);
                key = Self::hash_u64(key, u64::from(cx.scale_factor.to_bits()));
                for b in text.as_bytes() {
                    key = Self::hash_u64(key, u64::from(*b));
                }
                key = Self::hash_u64(key, Self::text_style_key(&style));

                let needs = self
                    .mouse_readout_text
                    .as_ref()
                    .is_none_or(|t| t.key != key);
                if needs {
                    if let Some(prev) = self.mouse_readout_text.take() {
                        cx.services.text().release(prev.blob);
                    }
                    let prepared = self.prepare_text(cx.services, &text, &style, constraints);
                    self.mouse_readout_text = Some(PreparedText {
                        blob: prepared.blob,
                        metrics: prepared.metrics,
                        key,
                    });
                }

                if let Some(tt) = self.mouse_readout_text {
                    let margin = Px(6.0);
                    let w = Px(tt.metrics.size.width.0);
                    let h = Px(tt.metrics.size.height.0);
                    let Some(rect) = overlay_rect_in_plot(
                        layout.plot,
                        Size::new(w, h),
                        self.style.mouse_readout_anchor,
                        margin,
                    ) else {
                        return;
                    };
                    let origin =
                        Point::new(rect.origin.x, Px(rect.origin.y.0 + tt.metrics.baseline.0));
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(12),
                        origin,
                        text: tt.blob,
                        color: label_color,
                    });
                }
            }
        }

        let tooltip = selection_tooltip.or_else(|| {
            self.hover
                .map(|hover| {
                    let (series_count, series_label, y_axis) = self
                        .model
                        .read(cx.app, |_app, m| {
                            let series_count = L::series_meta(m).len();
                            let series_label = L::series_label(m, hover.series_id);
                            let y_axis = L::series_y_axis(m, hover.series_id);
                            (series_count, series_label, y_axis)
                        })
                        .unwrap_or((0, None, YAxis::Left));

                    let x_text = self.tooltip_x_labels.format(hover.data.x, x_span);
                    let y_text = if y_axis == YAxis::Right && self.show_y2_axis {
                        let span = view_bounds_y2
                            .map(|b| (b.y_max - b.y_min).abs())
                            .unwrap_or(y_span);
                        self.y2_axis_labels.format(hover.data.y, span)
                    } else if y_axis == YAxis::Right2 && self.show_y3_axis {
                        let span = view_bounds_y3
                            .map(|b| (b.y_max - b.y_min).abs())
                            .unwrap_or(y_span);
                        self.y3_axis_labels.format(hover.data.y, span)
                    } else if y_axis == YAxis::Right3 && self.show_y4_axis {
                        let span = view_bounds_y4
                            .map(|b| (b.y_max - b.y_min).abs())
                            .unwrap_or(y_span);
                        self.y4_axis_labels.format(hover.data.y, span)
                    } else {
                        self.tooltip_y_labels.format(hover.data.y, y_span)
                    };
                    let text = if series_count > 1 {
                        if let Some(label) = series_label.as_deref() {
                            format!("{label}  x={x_text}  y={y_text}")
                        } else {
                            format!("s={}  x={x_text}  y={y_text}", hover.series_id.0)
                        }
                    } else {
                        format!("x={x_text}  y={y_text}")
                    };
                    let text = if series_count == 0
                        && let Some(label) = series_label.as_deref()
                    {
                        format!("{label}  {text}")
                    } else {
                        text
                    };
                    let text = if let Some(v) = hover.value
                        && v.is_finite()
                    {
                        let v_text = if v.abs() < 10_000.0 {
                            format!("{v:.4}")
                        } else {
                            format!("{v:.4e}")
                        };
                        format!("{text}  value={v_text}")
                    } else {
                        text
                    };
                    (hover.plot_px, text)
                })
                .or_else(|| {
                    if self.style.mouse_readout != MouseReadoutMode::Tooltip {
                        return None;
                    }

                    let cursor_px = cursor_px?;
                    let cursor_data = cursor_data?;

                    let hidden = &state.hidden_series;
                    let readout_args = PlotCursorReadoutArgs {
                        x: cursor_data.x,
                        plot_size: layout.plot.size,
                        view_bounds,
                        x_scale: self.x_scale,
                        y_scale: self.y_scale,
                        scale_factor: cx.scale_factor,
                        hidden,
                    };
                    let mut readout_rows = self
                        .model
                        .read(cx.app, |_app, m| L::cursor_readout(m, readout_args))
                        .unwrap_or_default();

                    if let Some(pinned) = state.pinned_series {
                        readout_rows.retain(|r| r.series_id == pinned);
                    }

                    let x_text = self.tooltip_x_labels.format(cursor_data.x, x_span);
                    let y_text = self.tooltip_y_labels.format(cursor_data.y, y_span);
                    let mut text = format!("x={x_text}  y={y_text}");
                    for row in readout_rows {
                        let y_text = row
                            .y
                            .filter(|y| y.is_finite())
                            .map(|y| match row.y_axis {
                                YAxis::Right if self.show_y2_axis => {
                                    let span = view_bounds_y2
                                        .map(|b| (b.y_max - b.y_min).abs())
                                        .unwrap_or(y_span);
                                    self.y2_axis_labels.format(y, span)
                                }
                                YAxis::Right2 if self.show_y3_axis => {
                                    let span = view_bounds_y3
                                        .map(|b| (b.y_max - b.y_min).abs())
                                        .unwrap_or(y_span);
                                    self.y3_axis_labels.format(y, span)
                                }
                                YAxis::Right3 if self.show_y4_axis => {
                                    let span = view_bounds_y4
                                        .map(|b| (b.y_max - b.y_min).abs())
                                        .unwrap_or(y_span);
                                    self.y4_axis_labels.format(y, span)
                                }
                                _ => self.tooltip_y_labels.format(y, y_span),
                            })
                            .unwrap_or_else(|| "NA".to_string());
                        text.push_str(&format!("\n{}: y={y_text}", row.label));
                    }

                    if let Some(query) = state.query {
                        let x0 = self.tooltip_x_labels.format(query.x_min, x_span);
                        let x1 = self.tooltip_x_labels.format(query.x_max, x_span);
                        let y0 = self.tooltip_y_labels.format(query.y_min, y_span);
                        let y1 = self.tooltip_y_labels.format(query.y_max, y_span);
                        text.push_str(&format!("\nquery: x=[{x0}, {x1}]  y=[{y0}, {y1}]"));
                    }

                    Some((cursor_px, text))
                })
                .or_else(|| {
                    let linked_x = state.linked_cursor_x?;
                    if self.style.linked_cursor_readout != MouseReadoutMode::Tooltip {
                        return None;
                    }
                    if !linked_x.is_finite() {
                        return None;
                    }

                    let transform = PlotTransform {
                        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size),
                        data: view_bounds,
                        x_scale: self.x_scale,
                        y_scale: self.y_scale,
                    };
                    let Some(linked_x_px) = transform.data_x_to_px(linked_x) else {
                        return None;
                    };

                    let anchor_local = Point::new(
                        Px(linked_x_px.0.clamp(0.0, layout.plot.size.width.0)),
                        Px(0.0),
                    );

                    let hidden = &state.hidden_series;
                    let readout_args = PlotCursorReadoutArgs {
                        x: linked_x,
                        plot_size: layout.plot.size,
                        view_bounds,
                        x_scale: self.x_scale,
                        y_scale: self.y_scale,
                        scale_factor: cx.scale_factor,
                        hidden,
                    };
                    let mut readout_rows = self
                        .model
                        .read(cx.app, |_app, m| L::cursor_readout(m, readout_args))
                        .unwrap_or_default();
                    apply_readout_policy(
                        &mut readout_rows,
                        state.pinned_series,
                        self.legend_hover,
                        self.style.linked_cursor_readout_policy,
                    );

                    let x_text = self.tooltip_x_labels.format(linked_x, x_span);
                    let mut text = format!("x={x_text}");
                    for row in readout_rows {
                        let y_text = row
                            .y
                            .filter(|y| y.is_finite())
                            .map(|y| match row.y_axis {
                                YAxis::Right if self.show_y2_axis => {
                                    let span = view_bounds_y2
                                        .map(|b| (b.y_max - b.y_min).abs())
                                        .unwrap_or(y_span);
                                    self.y2_axis_labels.format(y, span)
                                }
                                YAxis::Right2 if self.show_y3_axis => {
                                    let span = view_bounds_y3
                                        .map(|b| (b.y_max - b.y_min).abs())
                                        .unwrap_or(y_span);
                                    self.y3_axis_labels.format(y, span)
                                }
                                YAxis::Right3 if self.show_y4_axis => {
                                    let span = view_bounds_y4
                                        .map(|b| (b.y_max - b.y_min).abs())
                                        .unwrap_or(y_span);
                                    self.y4_axis_labels.format(y, span)
                                }
                                _ => self.tooltip_y_labels.format(y, y_span),
                            })
                            .unwrap_or_else(|| "NA".to_string());
                        text.push_str(&format!("\n{}: y={y_text}", row.label));
                    }

                    if let Some(query) = state.query {
                        let x0 = self.tooltip_x_labels.format(query.x_min, x_span);
                        let x1 = self.tooltip_x_labels.format(query.x_max, x_span);
                        let y0 = self.tooltip_y_labels.format(query.y_min, y_span);
                        let y1 = self.tooltip_y_labels.format(query.y_max, y_span);
                        text.push_str(&format!("\nquery: x=[{x0}, {x1}]  y=[{y0}, {y1}]"));
                    }

                    Some((anchor_local, text))
                })
        });

        if let Some((anchor_local, text)) = tooltip {
            let font_size = cx
                .theme()
                .metric_by_key("font.size")
                .unwrap_or(cx.theme().metrics.font_size);
            let style = TextStyle {
                font: FontId::default(),
                size: Px((font_size.0 * 0.90).max(10.0)),
                weight: FontWeight::NORMAL,
                line_height: None,
                letter_spacing_em: None,
            };
            let constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            };

            let mut key = 0u64;
            key = Self::hash_u64(key, theme.revision);
            key = Self::hash_u64(key, font_stack_key);
            key = Self::hash_u64(key, u64::from(cx.scale_factor.to_bits()));
            for b in text.as_bytes() {
                key = Self::hash_u64(key, u64::from(*b));
            }
            key = Self::hash_u64(key, Self::text_style_key(&style));

            let needs = self.tooltip_text.as_ref().is_none_or(|t| t.key != key);
            if needs {
                if let Some(prev) = self.tooltip_text.take() {
                    cx.services.text().release(prev.blob);
                }
                let prepared = self.prepare_text(cx.services, &text, &style, constraints);
                self.tooltip_text = Some(PreparedText {
                    blob: prepared.blob,
                    metrics: prepared.metrics,
                    key,
                });
            }

            if let Some(tt) = self.tooltip_text {
                let anchor = Point::new(
                    Px(layout.plot.origin.x.0 + anchor_local.x.0),
                    Px(layout.plot.origin.y.0 + anchor_local.y.0),
                );
                let pad = Px(6.0);
                let gap = Px(10.0);
                let w = Px(tt.metrics.size.width.0 + pad.0 * 2.0);
                let h = Px(tt.metrics.size.height.0 + pad.0 * 2.0);

                let mut x = Px(anchor.x.0 + gap.0);
                let mut y = Px(anchor.y.0 + gap.0);
                if x.0 + w.0 > cx.bounds.origin.x.0 + cx.bounds.size.width.0 {
                    x = Px(anchor.x.0 - gap.0 - w.0);
                }
                if y.0 + h.0 > cx.bounds.origin.y.0 + cx.bounds.size.height.0 {
                    y = Px(anchor.y.0 - gap.0 - h.0);
                }
                x = Px(x.0.max(cx.bounds.origin.x.0));
                y = Px(y.0.max(cx.bounds.origin.y.0));

                let rect = Rect::new(Point::new(x, y), Size::new(w, h));
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(20),
                    rect,
                    background: tooltip_background,
                    border: fret_core::Edges::all(Px(1.0)),
                    border_color: tooltip_border,
                    corner_radii: fret_core::Corners::all(Px(6.0)),
                });

                let origin = Point::new(
                    Px(rect.origin.x.0 + pad.0),
                    Px(rect.origin.y.0 + pad.0 + tt.metrics.baseline.0),
                );
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(21),
                    origin,
                    text: tt.blob,
                    color: tooltip_text_color,
                });
            }
        }
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Viewport);
        cx.set_label("Plot");
    }

    fn cleanup_resources(&mut self, services: &mut dyn UiServices) {
        self.layer.cleanup_resources(services);
        self.clear_axis_label_cache(services);
        self.clear_legend_cache(services);
        if let Some(t) = self.tooltip_text.take() {
            services.text().release(t.blob);
        }
        if let Some(t) = self.mouse_readout_text.take() {
            services.text().release(t.blob);
        }
        if let Some(t) = self.linked_cursor_readout_text.take() {
            services.text().release(t.blob);
        }
    }
}
