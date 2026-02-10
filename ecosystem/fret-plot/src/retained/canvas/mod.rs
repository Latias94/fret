//! Retained plot canvas implementation.

mod cache;
mod constraints;
mod readout;
mod util;

use self::cache::LegendEntry;
pub use self::constraints::AxisConstraints;
use self::constraints::constrain_view_bounds_scaled;
use self::readout::apply_readout_policy;
pub(super) use self::util::contains_point;
use self::util::{dim_color, offset_rect, overlay_rect_in_plot};

use fret_canvas::cache::SceneOpCache;
use fret_canvas::diagnostics::{CanvasCacheKey, CanvasCacheStatsRegistry};
use fret_canvas::text::{PreparedText, TextCache};
use fret_core::geometry::{Point, Px, Rect, Size};
use fret_core::scene::{Color, DrawOrder, SceneOp};
use fret_core::{
    Event, FontId, FontWeight, KeyCode, MouseButton, PathId, PointerEvent, SemanticsRole,
    TextConstraints, TextOverflow, TextSlant, TextStyle, TextWrap, Transform2D, UiServices,
};
use fret_runtime::{Model, TextFontStackKey};
use fret_ui::Theme;
use fret_ui::UiHost;
use fret_ui::retained_bridge::{
    Invalidation, LayoutCx, PaintCx, SemanticsCx, UiTreeRetainedExt, Widget,
};
use slotmap::Key;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use super::YAxis;
use super::layers::resolve_series_color;
use super::layers::{
    PlotCursorReadoutArgs, PlotCursorReadoutRow, PlotHitTestArgs, PlotHover, PlotLayer,
    PlotPaintArgs, PlotQuad, PlotQuadsSceneCachePolicy, SeriesMeta,
};
use super::layout::{PlotLayout, PlotRegion};
use super::state::{
    PlotAxisLock, PlotAxisLocks, PlotDragOutput, PlotDragPhase, PlotHoverOutput, PlotImage,
    PlotImageLayer, PlotOutput, PlotOutputSnapshot, PlotState,
};
use super::style::{LinePlotStyle, MouseReadoutMode, SeriesTooltipMode};
use super::{AreaPlotModel, LinePlotModel};

use crate::cartesian::{AxisScale, DataPoint, DataRect, PlotTransform, PreparedPlotTransform};
use crate::input_map::{ModifierKey, ModifiersMask, PlotInputMap};
use crate::plot::axis::{AxisLabelFormat, AxisLabelFormatter, AxisTicks, axis_ticks_scaled};
use crate::plot::colormap::ColorMapId;
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

fn all_visible_axes_zoom_locked(
    show_y2_axis: bool,
    show_y3_axis: bool,
    show_y4_axis: bool,
    lock_x_zoom: bool,
    lock_y1_zoom: bool,
    lock_y2_zoom: bool,
    lock_y3_zoom: bool,
    lock_y4_zoom: bool,
) -> bool {
    lock_x_zoom
        && lock_y1_zoom
        && (!show_y2_axis || lock_y2_zoom)
        && (!show_y3_axis || lock_y3_zoom)
        && (!show_y4_axis || lock_y4_zoom)
}

fn fit_view_bounds_with_zoom_locks(
    current: DataRect,
    fit: DataRect,
    lock_x_zoom: bool,
    lock_y_zoom: bool,
    x_scale: AxisScale,
    y_scale: AxisScale,
    x_constraints: AxisConstraints,
    y_constraints: AxisConstraints,
) -> Option<DataRect> {
    if lock_x_zoom && lock_y_zoom {
        return None;
    }

    let next = apply_axis_locks(current, fit, lock_x_zoom, lock_y_zoom);
    Some(constrain_view_bounds_scaled(
        next,
        x_scale,
        y_scale,
        x_constraints,
        y_constraints,
    ))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WheelZoomMode {
    PlotAll,
    PlotXOnly,
    PlotYOnly,
    XAxis,
    YAxis(YAxis),
}

fn wheel_zoom_mode(
    input_map: PlotInputMap,
    region: PlotRegion,
    modifiers: fret_core::Modifiers,
) -> Option<WheelZoomMode> {
    if let Some(required) = input_map.wheel_zoom_mod
        && !required.is_pressed(modifiers)
    {
        return None;
    }

    match region {
        PlotRegion::Plot => {
            let x_only = input_map
                .wheel_zoom_x_only_mod
                .is_some_and(|m| m.is_pressed(modifiers));
            let y_only = input_map
                .wheel_zoom_y_only_mod
                .is_some_and(|m| m.is_pressed(modifiers));

            if x_only {
                return Some(WheelZoomMode::PlotXOnly);
            }
            if y_only {
                return Some(WheelZoomMode::PlotYOnly);
            }
            Some(WheelZoomMode::PlotAll)
        }
        PlotRegion::XAxis => Some(WheelZoomMode::XAxis),
        PlotRegion::YAxis(axis) => Some(WheelZoomMode::YAxis(axis)),
    }
}

fn paint_plot_images(
    scene: &mut fret_core::Scene,
    images: &[PlotImage],
    layer: PlotImageLayer,
    transform_y1: PlotTransform,
    transform_y2: Option<PlotTransform>,
    transform_y3: Option<PlotTransform>,
    transform_y4: Option<PlotTransform>,
) {
    if images.is_empty() {
        return;
    }

    let transform_for_axis = |axis: YAxis| -> Option<PlotTransform> {
        match axis {
            YAxis::Left => Some(transform_y1),
            YAxis::Right => transform_y2,
            YAxis::Right2 => transform_y3,
            YAxis::Right3 => transform_y4,
        }
    };

    for img in images {
        if img.layer != layer {
            continue;
        }

        let Some(transform) = transform_for_axis(img.axis) else {
            continue;
        };

        let rect = img.rect;
        if !rect.x_min.is_finite()
            || !rect.x_max.is_finite()
            || !rect.y_min.is_finite()
            || !rect.y_max.is_finite()
        {
            continue;
        }

        let a = transform.data_to_px(DataPoint {
            x: rect.x_min,
            y: rect.y_min,
        });
        let b = transform.data_to_px(DataPoint {
            x: rect.x_max,
            y: rect.y_max,
        });
        if !a.x.0.is_finite() || !a.y.0.is_finite() || !b.x.0.is_finite() || !b.y.0.is_finite() {
            continue;
        }

        let left = a.x.0.min(b.x.0);
        let right = a.x.0.max(b.x.0);
        let top = a.y.0.min(b.y.0);
        let bottom = a.y.0.max(b.y.0);
        let w = (right - left).max(0.0);
        let h = (bottom - top).max(0.0);
        if w <= 0.0 || h <= 0.0 {
            continue;
        }

        let opacity = img.opacity.clamp(0.0, 1.0);
        if opacity <= 0.0 {
            continue;
        }

        scene.push(SceneOp::ImageRegion {
            order: DrawOrder(1),
            rect: Rect::new(Point::new(Px(left), Px(top)), Size::new(Px(w), Px(h))),
            image: img.image,
            uv: img.uv,
            opacity,
        });
    }
}

fn log10_tick_label_or_empty(v: f64) -> String {
    let Some(exp) = log10_decade_exponent(v) else {
        return String::new();
    };
    format!("10^{exp}")
}

fn format_colorbar_value(v: f32) -> String {
    if !v.is_finite() {
        return "NA".to_string();
    }
    let a = v.abs();
    if a > 1.0e6 || (a > 0.0 && a < 1.0e-3) {
        return format!("{v:.3e}");
    }
    if a >= 1000.0 {
        return format!("{v:.0}");
    }
    if a >= 10.0 {
        return format!("{v:.2}");
    }
    format!("{v:.3}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DragRectHandle {
    Inside,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DragAxisConstraint {
    XOnly,
    YOnly,
}

#[derive(Debug, Clone, Copy)]
enum DragCapture {
    LineX {
        id: u64,
        button: MouseButton,
        offset_x: f64,
        current_x: f64,
    },
    LineY {
        id: u64,
        axis: YAxis,
        button: MouseButton,
        offset_y: f64,
        current_y: f64,
    },
    Point {
        id: u64,
        axis: YAxis,
        button: MouseButton,
        offset: DataPoint,
        start: DataPoint,
        constraint: Option<DragAxisConstraint>,
        current: DataPoint,
    },
    Rect {
        id: u64,
        axis: YAxis,
        button: MouseButton,
        handle: DragRectHandle,
        offset: DataPoint,
        start: DataRect,
        constraint: Option<DragAxisConstraint>,
        current: DataRect,
    },
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
    lock_x: PlotAxisLock,
    lock_y: PlotAxisLock,
    lock_y2: PlotAxisLock,
    lock_y3: PlotAxisLock,
    lock_y4: PlotAxisLock,
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
    drag_capture: Option<DragCapture>,
    drag_output: Option<PlotDragOutput>,
    axis_text_cache: TextCache,
    indicator_text_cache: TextCache,
    tooltip_text_cache: TextCache,
    readout_text_cache: TextCache,
    overlay_text_cache: TextCache,
    heatmap_text_cache: TextCache,
    debug_text_cache: TextCache,
    text_env_key: Option<u64>,
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
    axis_lock_indicator_x: Option<PreparedText>,
    axis_lock_indicator_y: Option<PreparedText>,
    axis_lock_indicator_y2: Option<PreparedText>,
    axis_lock_indicator_y3: Option<PreparedText>,
    axis_lock_indicator_y4: Option<PreparedText>,
    legend_text_cache: TextCache,
    legend_key: Option<u64>,
    legend_entries: Vec<LegendEntry>,
    tooltip_text: Option<PreparedText>,
    mouse_readout_text: Option<PreparedText>,
    linked_cursor_readout_text: Option<PreparedText>,
    overlays_text_key: Option<u64>,
    overlays_text: Vec<PreparedText>,

    heatmap_colorbar_text_key: Option<u64>,
    heatmap_colorbar_text: Vec<PreparedText>,
    heatmap_colorbar_gradient_cache: SceneOpCache<u64>,
    quads_scene_cache: SceneOpCache<u64>,

    #[cfg(debug_assertions)]
    debug_overlay: bool,
    #[cfg(debug_assertions)]
    debug_overlay_text: Option<PreparedText>,
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

#[cfg(test)]
mod wheel_zoom_policy_tests {
    use super::*;

    #[test]
    fn wheel_zoom_respects_wheel_zoom_mod_gate() {
        let mut map = PlotInputMap::default();
        map.wheel_zoom_mod = Some(ModifierKey::Shift);
        map.wheel_zoom_x_only_mod = None;
        map.wheel_zoom_y_only_mod = None;

        let region = PlotRegion::Plot;
        let mods = fret_core::Modifiers::default();
        assert_eq!(wheel_zoom_mode(map, region, mods), None);

        let mods = fret_core::Modifiers {
            shift: true,
            ..fret_core::Modifiers::default()
        };
        assert_eq!(
            wheel_zoom_mode(map, region, mods),
            Some(WheelZoomMode::PlotAll)
        );
    }

    #[test]
    fn wheel_zoom_plot_region_prefers_x_only_over_y_only() {
        let map = PlotInputMap::default();
        let region = PlotRegion::Plot;
        let mods = fret_core::Modifiers {
            shift: true,
            ctrl: true,
            ..fret_core::Modifiers::default()
        };
        assert_eq!(
            wheel_zoom_mode(map, region, mods),
            Some(WheelZoomMode::PlotXOnly)
        );
    }

    #[test]
    fn wheel_zoom_axis_region_routing_overrides_axis_only_modifiers() {
        let map = PlotInputMap::default();
        let mods = fret_core::Modifiers {
            shift: true,
            ctrl: true,
            ..fret_core::Modifiers::default()
        };

        assert_eq!(
            wheel_zoom_mode(map, PlotRegion::XAxis, mods),
            Some(WheelZoomMode::XAxis)
        );
        assert_eq!(
            wheel_zoom_mode(map, PlotRegion::YAxis(YAxis::Right2), mods),
            Some(WheelZoomMode::YAxis(YAxis::Right2))
        );
    }
}

#[cfg(test)]
mod fit_and_box_zoom_lock_tests {
    use super::*;

    #[test]
    fn all_visible_axes_zoom_locked_ignores_hidden_axes() {
        assert!(all_visible_axes_zoom_locked(
            false, false, false, true, true, false, false, false,
        ));
    }

    #[test]
    fn all_visible_axes_zoom_locked_requires_visible_axes_locked() {
        assert!(!all_visible_axes_zoom_locked(
            true, false, false, true, true, false, true, true,
        ));
    }

    #[test]
    fn fit_view_bounds_is_none_when_both_axes_locked() {
        let current = DataRect {
            x_min: 0.0,
            x_max: 10.0,
            y_min: 0.0,
            y_max: 10.0,
        };
        let fit = DataRect {
            x_min: -5.0,
            x_max: 5.0,
            y_min: -2.0,
            y_max: 2.0,
        };

        assert_eq!(
            fit_view_bounds_with_zoom_locks(
                current,
                fit,
                true,
                true,
                AxisScale::Linear,
                AxisScale::Linear,
                AxisConstraints::default(),
                AxisConstraints::default(),
            ),
            None
        );
    }

    #[test]
    fn fit_view_bounds_preserves_locked_axis_and_updates_unlocked_axis() {
        let current = DataRect {
            x_min: 0.0,
            x_max: 10.0,
            y_min: 0.0,
            y_max: 10.0,
        };
        let fit = DataRect {
            x_min: -5.0,
            x_max: 5.0,
            y_min: -2.0,
            y_max: 2.0,
        };

        let next = fit_view_bounds_with_zoom_locks(
            current,
            fit,
            true,
            false,
            AxisScale::Linear,
            AxisScale::Linear,
            AxisConstraints::default(),
            AxisConstraints::default(),
        )
        .expect("expected update");
        assert_eq!((next.x_min, next.x_max), (current.x_min, current.x_max));
        assert_eq!((next.y_min, next.y_max), (fit.y_min, fit.y_max));
    }
}

impl<L: PlotLayer + 'static> PlotCanvas<L> {
    pub(super) fn with_layer_mut(mut self, f: impl FnOnce(&mut L)) -> Self {
        f(&mut self.layer);
        self
    }

    fn view_interacting(&self) -> bool {
        self.pan_start_pos.is_some()
            || self.box_zoom_start.is_some()
            || self.query_drag_start.is_some()
            || self.drag_capture.is_some()
            || self.drag_output.is_some()
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
            lock_x: PlotAxisLock::default(),
            lock_y: PlotAxisLock::default(),
            lock_y2: PlotAxisLock::default(),
            lock_y3: PlotAxisLock::default(),
            lock_y4: PlotAxisLock::default(),
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
            drag_capture: None,
            drag_output: None,
            axis_text_cache: TextCache::default(),
            indicator_text_cache: TextCache::default(),
            tooltip_text_cache: TextCache::default(),
            readout_text_cache: TextCache::default(),
            overlay_text_cache: TextCache::default(),
            heatmap_text_cache: TextCache::default(),
            debug_text_cache: TextCache::default(),
            text_env_key: None,
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
            axis_lock_indicator_x: None,
            axis_lock_indicator_y: None,
            axis_lock_indicator_y2: None,
            axis_lock_indicator_y3: None,
            axis_lock_indicator_y4: None,
            legend_text_cache: TextCache::default(),
            legend_key: None,
            legend_entries: Vec::new(),
            tooltip_text: None,
            mouse_readout_text: None,
            linked_cursor_readout_text: None,
            overlays_text_key: None,
            overlays_text: Vec::new(),

            heatmap_colorbar_text_key: None,
            heatmap_colorbar_text: Vec::new(),
            heatmap_colorbar_gradient_cache: SceneOpCache::default(),
            quads_scene_cache: SceneOpCache::default(),

            #[cfg(debug_assertions)]
            debug_overlay: false,
            #[cfg(debug_assertions)]
            debug_overlay_text: None,
        }
    }

    pub fn style(mut self, style: LinePlotStyle) -> Self {
        self.style = style;
        self
    }

    /// Enable a small diagnostic overlay that displays per-frame draw stats (e.g. path count).
    ///
    /// This is intended for debugging rendering issues in demos and is disabled by default.
    pub fn debug_overlay(mut self, enabled: bool) -> Self {
        #[cfg(debug_assertions)]
        {
            self.debug_overlay = enabled;
        }
        #[cfg(not(debug_assertions))]
        {
            let _ = enabled;
        }
        self
    }

    pub fn heatmap_colormap(mut self, colormap: ColorMapId) -> Self {
        self.style.heatmap_colormap = colormap;
        self
    }

    pub fn heatmap_colorbar(mut self, enabled: bool) -> Self {
        self.style.heatmap_show_colorbar = enabled;
        self
    }

    pub fn heatmap_colorbar_width(mut self, width: Px) -> Self {
        self.style.heatmap_colorbar_width = width;
        self
    }

    pub fn heatmap_colorbar_padding(mut self, padding: Px) -> Self {
        self.style.heatmap_colorbar_padding = padding;
        self
    }

    pub fn heatmap_colorbar_steps(mut self, steps: usize) -> Self {
        self.style.heatmap_colorbar_steps = steps;
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
        self.lock_x = PlotAxisLock {
            pan: locked,
            zoom: locked,
        };
        self.plot_state.axis_locks.x = self.lock_x;
        self
    }

    pub fn y_axis_locked(mut self, locked: bool) -> Self {
        self.lock_y = PlotAxisLock {
            pan: locked,
            zoom: locked,
        };
        self.plot_state.axis_locks.y = self.lock_y;
        self
    }

    pub fn y2_axis_locked(mut self, locked: bool) -> Self {
        self.lock_y2 = PlotAxisLock {
            pan: locked,
            zoom: locked,
        };
        self.plot_state.axis_locks.y2 = self.lock_y2;
        self
    }

    pub fn y3_axis_locked(mut self, locked: bool) -> Self {
        self.lock_y3 = PlotAxisLock {
            pan: locked,
            zoom: locked,
        };
        self.plot_state.axis_locks.y3 = self.lock_y3;
        self
    }

    pub fn y4_axis_locked(mut self, locked: bool) -> Self {
        self.lock_y4 = PlotAxisLock {
            pan: locked,
            zoom: locked,
        };
        self.plot_state.axis_locks.y4 = self.lock_y4;
        self
    }

    pub fn x_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_x.pan = locked;
        self.plot_state.axis_locks.x = self.lock_x;
        self
    }

    pub fn x_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_x.zoom = locked;
        self.plot_state.axis_locks.x = self.lock_x;
        self
    }

    pub fn y_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_y.pan = locked;
        self.plot_state.axis_locks.y = self.lock_y;
        self
    }

    pub fn y_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_y.zoom = locked;
        self.plot_state.axis_locks.y = self.lock_y;
        self
    }

    pub fn y2_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_y2.pan = locked;
        self.plot_state.axis_locks.y2 = self.lock_y2;
        self
    }

    pub fn y2_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_y2.zoom = locked;
        self.plot_state.axis_locks.y2 = self.lock_y2;
        self
    }

    pub fn y3_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_y3.pan = locked;
        self.plot_state.axis_locks.y3 = self.lock_y3;
        self
    }

    pub fn y3_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_y3.zoom = locked;
        self.plot_state.axis_locks.y3 = self.lock_y3;
        self
    }

    pub fn y4_axis_pan_locked(mut self, locked: bool) -> Self {
        self.lock_y4.pan = locked;
        self.plot_state.axis_locks.y4 = self.lock_y4;
        self
    }

    pub fn y4_axis_zoom_locked(mut self, locked: bool) -> Self {
        self.lock_y4.zoom = locked;
        self.plot_state.axis_locks.y4 = self.lock_y4;
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

    fn resolve_style_from_theme(&self, theme: &Theme) -> LinePlotStyle {
        let default_style = LinePlotStyle::default();

        let series_palette = if self.style.series_palette == default_style.series_palette {
            crate::theme_tokens::resolve_series_palette(theme, default_style.series_palette)
        } else {
            self.style.series_palette
        };

        let stroke_color = if self.style.stroke_color == default_style.stroke_color
            && self.style.series_palette == default_style.series_palette
        {
            series_palette[0]
        } else {
            self.style.stroke_color
        };

        let border_width = if self.style.border_width == default_style.border_width {
            crate::theme_tokens::metric(theme, "fret.plot.border_width", "plot.border_width")
                .unwrap_or(default_style.border_width)
        } else {
            self.style.border_width
        };

        let padding = if self.style.padding == default_style.padding {
            crate::theme_tokens::metric(theme, "fret.plot.padding", "plot.padding")
                .unwrap_or(default_style.padding)
        } else {
            self.style.padding
        };

        let axis_gap = if self.style.axis_gap == default_style.axis_gap {
            crate::theme_tokens::metric(theme, "fret.plot.axis_gap", "plot.axis_gap")
                .unwrap_or(default_style.axis_gap)
        } else {
            self.style.axis_gap
        };

        let stroke_width = if self.style.stroke_width == default_style.stroke_width {
            crate::theme_tokens::metric(theme, "fret.plot.stroke_width", "plot.stroke_width")
                .unwrap_or(default_style.stroke_width)
        } else {
            self.style.stroke_width
        };

        let hover_threshold = if self.style.hover_threshold == default_style.hover_threshold {
            crate::theme_tokens::metric(theme, "fret.plot.hover_threshold", "plot.hover_threshold")
                .unwrap_or(default_style.hover_threshold)
        } else {
            self.style.hover_threshold
        };

        LinePlotStyle {
            series_palette,
            stroke_color,
            border_width,
            padding,
            axis_gap,
            stroke_width,
            hover_threshold,
            ..self.style
        }
    }

    fn axis_gaps_for_style(&self, axis_gap: Px) -> (Px, Px, Px, Px, Px) {
        let min = axis_gap.0.max(0.0);
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

    fn canvas_axis_locks(&self) -> PlotAxisLocks {
        PlotAxisLocks {
            x: self.lock_x,
            y: self.lock_y,
            y2: self.lock_y2,
            y3: self.lock_y3,
            y4: self.lock_y4,
        }
    }

    fn set_canvas_axis_locks(&mut self, locks: PlotAxisLocks) {
        self.lock_x = locks.x;
        self.lock_y = locks.y;
        self.lock_y2 = locks.y2;
        self.lock_y3 = locks.y3;
        self.lock_y4 = locks.y4;
    }

    fn sync_axis_locks<H: UiHost>(&mut self, app: &mut H) {
        let state = self.read_plot_state(app);
        let state_locks = state.axis_locks;
        let canvas_locks = self.canvas_axis_locks();

        if self.plot_state_model.is_some() {
            // Allow PlotCanvas builder configuration to provide initial locks when the caller
            // uses an external PlotState but has not set axis locks yet.
            if state_locks == PlotAxisLocks::default() && canvas_locks != PlotAxisLocks::default() {
                let _ = self.update_plot_state(app, |s| {
                    s.axis_locks = canvas_locks;
                });
            } else if state_locks != canvas_locks {
                self.set_canvas_axis_locks(state_locks);
            }
        } else if self.plot_state.axis_locks != canvas_locks {
            // Internal PlotState should always stay in sync with the widget-owned lock flags.
            self.plot_state.axis_locks = canvas_locks;
        }
    }

    fn persist_axis_locks<H: UiHost>(&mut self, app: &mut H) {
        let locks = self.canvas_axis_locks();
        let _ = self.update_plot_state(app, |s| {
            s.axis_locks = locks;
        });
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
                drag: self.drag_output,
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
        style: LinePlotStyle,
    ) -> Vec<(SeriesId, PathId, Color)> {
        let model_revision = self.model.revision(cx.app).unwrap_or(0);
        let Ok(model) = self.model.read(cx.app, |_app, m| m.clone()) else {
            return Vec::new();
        };

        // Plot paint layers operate in plot-local coordinates (origin at the plot top-left).
        // The caller applies `layout.plot.origin` when emitting `SceneOp`s.
        let plot_local = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);

        self.layer.paint_paths(
            cx,
            &model,
            PlotPaintArgs {
                model_revision,
                plot: plot_local,
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
                y2_scale: self.y2_scale,
                y3_scale: self.y3_scale,
                y4_scale: self.y4_scale,
                style,
                hidden,
                view_interacting: self.view_interacting(),
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
        style: LinePlotStyle,
    ) -> Vec<PlotQuad> {
        let model_revision = self.model.revision(cx.app).unwrap_or(0);
        let Ok(model) = self.model.read(cx.app, |_app, m| m.clone()) else {
            return Vec::new();
        };

        // Plot paint layers operate in plot-local coordinates (origin at the plot top-left).
        // The caller applies `layout.plot.origin` when emitting `SceneOp`s.
        let plot_local = Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size);

        self.layer.paint_quads(
            cx,
            &model,
            PlotPaintArgs {
                model_revision,
                plot: plot_local,
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
                y2_scale: self.y2_scale,
                y3_scale: self.y3_scale,
                y4_scale: self.y4_scale,
                style,
                hidden,
                view_interacting: self.view_interacting(),
            },
        )
    }
}

impl<H: UiHost, L: PlotLayer + 'static> Widget<H> for PlotCanvas<L> {
    fn event(&mut self, cx: &mut fret_ui::retained_bridge::EventCx<'_, H>, event: &Event) {
        // Axis enablement is derived from the model (series -> axis assignment), so make sure
        // we don't accidentally interpret "right axis series" using the primary Y transform.
        self.ensure_required_axes_enabled(cx.app);
        self.sync_axis_locks(cx.app);
        let resolved_style = self.resolve_style_from_theme(cx.theme());

        match event {
            Event::KeyDown { key, modifiers, .. } => {
                let plain = !modifiers.shift
                    && !modifiers.ctrl
                    && !modifiers.alt
                    && !modifiers.alt_gr
                    && !modifiers.meta;
                let lock_action = if let Some(chord) = self.input_map.axis_pan_lock_toggle
                    && chord.matches(*key, *modifiers)
                {
                    Some((true, false))
                } else if let Some(chord) = self.input_map.axis_zoom_lock_toggle
                    && chord.matches(*key, *modifiers)
                {
                    Some((false, true))
                } else if let Some(chord) = self.input_map.axis_lock_toggle
                    && chord.matches(*key, *modifiers)
                {
                    Some((true, true))
                } else {
                    None
                };

                if let Some((toggle_pan, toggle_zoom)) = lock_action {
                    let Some(pos) = self.last_pointer_pos else {
                        return;
                    };

                    let (
                        y_axis_gap,
                        y_axis_right_gap,
                        y_axis_right2_gap,
                        y_axis_right3_gap,
                        x_axis_gap,
                    ) = self.axis_gaps_for_style(resolved_style.axis_gap);
                    let layout = PlotLayout::from_bounds(
                        cx.bounds,
                        resolved_style.padding,
                        y_axis_gap,
                        y_axis_right_gap,
                        y_axis_right2_gap,
                        y_axis_right3_gap,
                        x_axis_gap,
                    );
                    let Some(region) = layout.hit_test_region(pos) else {
                        return;
                    };

                    match region {
                        PlotRegion::XAxis => {
                            if toggle_pan {
                                self.lock_x.pan = !self.lock_x.pan;
                            }
                            if toggle_zoom {
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
                            if toggle_pan {
                                lock.pan = !lock.pan;
                            }
                            if toggle_zoom {
                                lock.zoom = !lock.zoom;
                            }
                        }
                        PlotRegion::Plot => {
                            if toggle_pan {
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
                            if toggle_zoom {
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

                    self.persist_axis_locks(cx.app);

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
                    self.drag_capture = None;
                    self.drag_output = None;

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
                    self.drag_capture = None;
                    self.drag_output = None;
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
                    self.drag_capture = None;
                    self.drag_output = None;
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
                        || self.query_drag_start.is_some()
                        || self.drag_capture.is_some();

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
                        self.drag_capture = None;
                        self.drag_output = None;
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
                ) = self.axis_gaps_for_style(resolved_style.axis_gap);
                let layout = PlotLayout::from_bounds(
                    cx.bounds,
                    resolved_style.padding,
                    y_axis_gap,
                    y_axis_right_gap,
                    y_axis_right2_gap,
                    y_axis_right3_gap,
                    x_axis_gap,
                );
                if layout.plot.size.width.0 <= 0.0 || layout.plot.size.height.0 <= 0.0 {
                    return;
                }

                // Clear stale drag output on any new press.
                self.drag_output = None;

                // Axis lock UI: Ctrl+Click on an axis region toggles pan+zoom lock.
                if let Some(chord) = self.input_map.axis_lock_click
                    && chord.matches(*button, *modifiers)
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

                    self.persist_axis_locks(cx.app);

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
                    self.drag_capture = None;
                    self.drag_output = None;

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
                    self.drag_capture = None;
                    self.drag_output = None;
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

                // If the user clicks on a draggable overlay, prefer dragging it over panning.
                if start_pan && region == Some(PlotRegion::Plot) {
                    let state = self.read_plot_state(cx.app);
                    let view_bounds = self.current_view_bounds(cx.app, &state);
                    let view_bounds_y2 = self.current_view_bounds_y2(cx.app, &state, view_bounds);
                    let view_bounds_y3 = self.current_view_bounds_y3(cx.app, &state, view_bounds);
                    let view_bounds_y4 = self.current_view_bounds_y4(cx.app, &state, view_bounds);

                    let local = local_from_absolute(layout.plot.origin, *position);
                    let threshold = resolved_style.hover_threshold.0.max(1.0);

                    let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size);
                    let transform_x = PlotTransform {
                        viewport: local_viewport,
                        data: view_bounds,
                        x_scale: self.x_scale,
                        y_scale: self.y_scale,
                    };
                    let transform_for_y_axis = |axis: YAxis| -> Option<PlotTransform> {
                        match axis {
                            YAxis::Left => Some(PlotTransform {
                                viewport: local_viewport,
                                data: view_bounds,
                                x_scale: self.x_scale,
                                y_scale: self.y_scale,
                            }),
                            YAxis::Right if self.show_y2_axis => {
                                view_bounds_y2.map(|b| PlotTransform {
                                    viewport: local_viewport,
                                    data: b,
                                    x_scale: self.x_scale,
                                    y_scale: self.y2_scale,
                                })
                            }
                            YAxis::Right2 if self.show_y3_axis => {
                                view_bounds_y3.map(|b| PlotTransform {
                                    viewport: local_viewport,
                                    data: b,
                                    x_scale: self.x_scale,
                                    y_scale: self.y3_scale,
                                })
                            }
                            YAxis::Right3 if self.show_y4_axis => {
                                view_bounds_y4.map(|b| PlotTransform {
                                    viewport: local_viewport,
                                    data: b,
                                    x_scale: self.x_scale,
                                    y_scale: self.y4_scale,
                                })
                            }
                            _ => None,
                        }
                    };

                    let mut best: Option<(f32, DragCapture)> = None;
                    let overlays = &state.overlays;

                    // If the user clicks on a draggable point, prefer it over other drag handles.
                    for point in &overlays.drag_points {
                        if !point.point.x.is_finite() || !point.point.y.is_finite() {
                            continue;
                        }
                        let Some(transform) = transform_for_y_axis(point.axis) else {
                            continue;
                        };
                        let p_px = transform.data_to_px(point.point);
                        if !p_px.x.0.is_finite() || !p_px.y.0.is_finite() {
                            continue;
                        }
                        let hit_r = point.radius.0.max(threshold);
                        let dx = local.x.0 - p_px.x.0;
                        let dy = local.y.0 - p_px.y.0;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist > hit_r {
                            continue;
                        }
                        let data = transform.px_to_data(local);
                        if !data.x.is_finite() || !data.y.is_finite() {
                            continue;
                        }
                        let offset = DataPoint {
                            x: data.x - point.point.x,
                            y: data.y - point.point.y,
                        };
                        let candidate = DragCapture::Point {
                            id: point.id,
                            axis: point.axis,
                            button: *button,
                            offset,
                            start: point.point,
                            constraint: None,
                            current: point.point,
                        };
                        if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                            best = Some((dist, candidate));
                        }
                    }

                    if best.is_none() {
                        for line in &overlays.drag_lines_x {
                            if !line.x.is_finite() {
                                continue;
                            }
                            let Some(x_px) = transform_x.data_x_to_px(line.x) else {
                                continue;
                            };
                            let dist = (local.x.0 - x_px.0).abs();
                            if dist > threshold {
                                continue;
                            }
                            let data = transform_x.px_to_data(local);
                            if !data.x.is_finite() {
                                continue;
                            }
                            let offset_x = data.x - line.x;
                            let candidate = DragCapture::LineX {
                                id: line.id,
                                button: *button,
                                offset_x,
                                current_x: line.x,
                            };
                            if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                                best = Some((dist, candidate));
                            }
                        }

                        for line in &overlays.drag_lines_y {
                            if !line.y.is_finite() {
                                continue;
                            }
                            let Some(transform) = transform_for_y_axis(line.axis) else {
                                continue;
                            };
                            let Some(y_px) = transform.data_y_to_px(line.y) else {
                                continue;
                            };
                            let dist = (local.y.0 - y_px.0).abs();
                            if dist > threshold {
                                continue;
                            }
                            let data = transform.px_to_data(local);
                            if !data.y.is_finite() {
                                continue;
                            }
                            let offset_y = data.y - line.y;
                            let candidate = DragCapture::LineY {
                                id: line.id,
                                axis: line.axis,
                                button: *button,
                                offset_y,
                                current_y: line.y,
                            };
                            if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                                best = Some((dist, candidate));
                            }
                        }

                        for rect in &overlays.drag_rects {
                            let Some(transform) = transform_for_y_axis(rect.axis) else {
                                continue;
                            };
                            let a = transform.data_to_px(DataPoint {
                                x: rect.rect.x_min,
                                y: rect.rect.y_min,
                            });
                            let b = transform.data_to_px(DataPoint {
                                x: rect.rect.x_max,
                                y: rect.rect.y_max,
                            });
                            if !a.x.0.is_finite()
                                || !a.y.0.is_finite()
                                || !b.x.0.is_finite()
                                || !b.y.0.is_finite()
                            {
                                continue;
                            }
                            let left = a.x.0.min(b.x.0);
                            let right = a.x.0.max(b.x.0);
                            let top = a.y.0.min(b.y.0);
                            let bottom = a.y.0.max(b.y.0);

                            let inside = local.x.0 >= left
                                && local.x.0 <= right
                                && local.y.0 >= top
                                && local.y.0 <= bottom;
                            if !inside {
                                continue;
                            }

                            let dist_left = (local.x.0 - left).abs();
                            let dist_right = (local.x.0 - right).abs();
                            let dist_top = (local.y.0 - top).abs();
                            let dist_bottom = (local.y.0 - bottom).abs();

                            let mut handle = DragRectHandle::Inside;
                            let mut dist = 0.0f32;
                            let mut set = |d: f32, h: DragRectHandle| {
                                if d <= threshold && (handle == DragRectHandle::Inside || d < dist)
                                {
                                    handle = h;
                                    dist = d;
                                }
                            };
                            set(dist_left, DragRectHandle::Left);
                            set(dist_right, DragRectHandle::Right);
                            set(dist_top, DragRectHandle::Top);
                            set(dist_bottom, DragRectHandle::Bottom);

                            let data = transform.px_to_data(local);
                            if !data.x.is_finite() || !data.y.is_finite() {
                                continue;
                            }

                            let offset = match handle {
                                DragRectHandle::Inside => DataPoint {
                                    x: data.x - rect.rect.x_min,
                                    y: data.y - rect.rect.y_min,
                                },
                                DragRectHandle::Left => DataPoint {
                                    x: data.x - rect.rect.x_min,
                                    y: 0.0,
                                },
                                DragRectHandle::Right => DataPoint {
                                    x: data.x - rect.rect.x_max,
                                    y: 0.0,
                                },
                                DragRectHandle::Top => DataPoint {
                                    x: 0.0,
                                    y: data.y - rect.rect.y_max,
                                },
                                DragRectHandle::Bottom => DataPoint {
                                    x: 0.0,
                                    y: data.y - rect.rect.y_min,
                                },
                            };

                            let candidate = DragCapture::Rect {
                                id: rect.id,
                                axis: rect.axis,
                                button: *button,
                                handle,
                                offset,
                                start: rect.rect,
                                constraint: None,
                                current: rect.rect,
                            };
                            if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                                best = Some((dist, candidate));
                            }
                        }
                    }

                    if let Some((_dist, capture)) = best {
                        self.cursor_px = Some(local);
                        self.hover = None;
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

                        self.drag_output = Some(match capture {
                            DragCapture::LineX { id, current_x, .. } => PlotDragOutput::LineX {
                                id,
                                x: current_x,
                                phase: PlotDragPhase::Start,
                            },
                            DragCapture::LineY {
                                id,
                                axis,
                                current_y,
                                ..
                            } => PlotDragOutput::LineY {
                                id,
                                axis,
                                y: current_y,
                                phase: PlotDragPhase::Start,
                            },
                            DragCapture::Point {
                                id, axis, current, ..
                            } => PlotDragOutput::Point {
                                id,
                                axis,
                                point: current,
                                phase: PlotDragPhase::Start,
                            },
                            DragCapture::Rect {
                                id, axis, current, ..
                            } => PlotDragOutput::Rect {
                                id,
                                axis,
                                rect: current,
                                phase: PlotDragPhase::Start,
                            },
                        });
                        self.drag_capture = Some(capture);

                        cx.request_focus(cx.node);
                        cx.capture_pointer(cx.node);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }
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
                    self.drag_capture = None;
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
                    self.drag_capture = None;
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
                    self.drag_capture = None;
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

                if let Some(capture) = self.drag_capture
                    && match capture {
                        DragCapture::LineX { button: b, .. } => b == *button,
                        DragCapture::LineY { button: b, .. } => b == *button,
                        DragCapture::Point { button: b, .. } => b == *button,
                        DragCapture::Rect { button: b, .. } => b == *button,
                    }
                {
                    self.drag_capture = None;
                    self.drag_output = Some(match capture {
                        DragCapture::LineX { id, current_x, .. } => PlotDragOutput::LineX {
                            id,
                            x: current_x,
                            phase: PlotDragPhase::End,
                        },
                        DragCapture::LineY {
                            id,
                            axis,
                            current_y,
                            ..
                        } => PlotDragOutput::LineY {
                            id,
                            axis,
                            y: current_y,
                            phase: PlotDragPhase::End,
                        },
                        DragCapture::Point {
                            id, axis, current, ..
                        } => PlotDragOutput::Point {
                            id,
                            axis,
                            point: current,
                            phase: PlotDragPhase::End,
                        },
                        DragCapture::Rect {
                            id, axis, current, ..
                        } => PlotDragOutput::Rect {
                            id,
                            axis,
                            rect: current,
                            phase: PlotDragPhase::End,
                        },
                    });

                    if cx.captured == Some(cx.node) {
                        cx.release_pointer_capture();
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

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
                        ) = self.axis_gaps_for_style(resolved_style.axis_gap);
                        let layout = PlotLayout::from_bounds(
                            cx.bounds,
                            resolved_style.padding,
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
                                        let all_locked = all_visible_axes_zoom_locked(
                                            show_y2_axis,
                                            show_y3_axis,
                                            show_y4_axis,
                                            lock_x_zoom,
                                            lock_y1_zoom,
                                            lock_y2_zoom,
                                            lock_y3_zoom,
                                            lock_y4_zoom,
                                        );
                                        if all_locked {
                                            // Axis locks prevent any view change; keep auto-fit state intact.
                                            return;
                                        }

                                        if let Some(next) = fit_view_bounds_with_zoom_locks(
                                            current,
                                            fit,
                                            lock_x_zoom,
                                            lock_y1_zoom,
                                            x_scale,
                                            y_scale,
                                            x_constraints,
                                            y_constraints,
                                        ) {
                                            s.view_is_auto = false;
                                            s.view_bounds = Some(next);
                                        }

                                        if show_y2_axis
                                            && let (Some(current_axis), Some(fit_axis)) =
                                                (current_y2, fit_y2)
                                            && let Some(next) = fit_view_bounds_with_zoom_locks(
                                                current_axis,
                                                fit_axis,
                                                lock_x_zoom,
                                                lock_y2_zoom,
                                                x_scale,
                                                y2_scale,
                                                x_constraints,
                                                y2_constraints,
                                            )
                                        {
                                            s.view_y2_is_auto = false;
                                            s.view_bounds_y2 = Some(next);
                                        }
                                        if show_y3_axis
                                            && let (Some(current_axis), Some(fit_axis)) =
                                                (current_y3, fit_y3)
                                            && let Some(next) = fit_view_bounds_with_zoom_locks(
                                                current_axis,
                                                fit_axis,
                                                lock_x_zoom,
                                                lock_y3_zoom,
                                                x_scale,
                                                y3_scale,
                                                x_constraints,
                                                y3_constraints,
                                            )
                                        {
                                            s.view_y3_is_auto = false;
                                            s.view_bounds_y3 = Some(next);
                                        }
                                        if show_y4_axis
                                            && let (Some(current_axis), Some(fit_axis)) =
                                                (current_y4, fit_y4)
                                            && let Some(next) = fit_view_bounds_with_zoom_locks(
                                                current_axis,
                                                fit_axis,
                                                lock_x_zoom,
                                                lock_y4_zoom,
                                                x_scale,
                                                y4_scale,
                                                x_constraints,
                                                y4_constraints,
                                            )
                                        {
                                            s.view_y4_is_auto = false;
                                            s.view_bounds_y4 = Some(next);
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
                        ) = self.axis_gaps_for_style(resolved_style.axis_gap);
                        let layout = PlotLayout::from_bounds(
                            cx.bounds,
                            resolved_style.padding,
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
                        ) = self.axis_gaps_for_style(resolved_style.axis_gap);
                        let layout = PlotLayout::from_bounds(
                            cx.bounds,
                            resolved_style.padding,
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
                                let all_locked = all_visible_axes_zoom_locked(
                                    self.show_y2_axis,
                                    self.show_y3_axis,
                                    self.show_y4_axis,
                                    self.lock_x.zoom,
                                    self.lock_y.zoom,
                                    self.lock_y2.zoom,
                                    self.lock_y3.zoom,
                                    self.lock_y4.zoom,
                                );
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
                ) = self.axis_gaps_for_style(resolved_style.axis_gap);
                let layout = PlotLayout::from_bounds(
                    cx.bounds,
                    resolved_style.padding,
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

                let delta_y = delta.y.0;
                if !delta_y.is_finite() {
                    return;
                }

                let speed = self.input_map.wheel_zoom_log2_per_px;
                let speed = if speed.is_finite() { speed } else { 0.0025 };
                let zoom = clamp_zoom_factors(2.0_f32.powf(delta_y * speed));
                let mut zoom_x = zoom;
                let mut zoom_y1 = zoom;
                let mut zoom_y2 = zoom;
                let mut zoom_y3 = zoom;
                let mut zoom_y4 = zoom;

                let Some(mode) = wheel_zoom_mode(self.input_map, region, *modifiers) else {
                    return;
                };

                match mode {
                    WheelZoomMode::PlotAll => {}
                    WheelZoomMode::PlotXOnly => {
                        zoom_y1 = 1.0;
                        zoom_y2 = 1.0;
                        zoom_y3 = 1.0;
                        zoom_y4 = 1.0;
                    }
                    WheelZoomMode::PlotYOnly => {
                        zoom_x = 1.0;
                    }
                    WheelZoomMode::XAxis => {
                        zoom_y1 = 1.0;
                        zoom_y2 = 1.0;
                        zoom_y3 = 1.0;
                        zoom_y4 = 1.0;
                    }
                    WheelZoomMode::YAxis(axis) => {
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
            Event::Pointer(PointerEvent::Move {
                position,
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
                ) = self.axis_gaps_for_style(resolved_style.axis_gap);
                let layout = PlotLayout::from_bounds(
                    cx.bounds,
                    resolved_style.padding,
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

                if let Some(mut capture) = self.drag_capture {
                    let snap_to_nearest_tick = |value: f64, ticks: &[f64]| -> f64 {
                        let mut best: Option<(f64, f64)> = None;
                        for t in ticks {
                            if !t.is_finite() {
                                continue;
                            }
                            let dist = (value - t).abs();
                            if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                                best = Some((dist, *t));
                            }
                        }
                        best.map(|(_, t)| t).unwrap_or(value)
                    };
                    let snap_x = |x: f64| -> f64 { snap_to_nearest_tick(x, &self.axis_ticks_x) };
                    let snap_y = |axis: YAxis, y: f64| -> f64 {
                        match axis {
                            YAxis::Left => snap_to_nearest_tick(y, &self.axis_ticks_y),
                            YAxis::Right => snap_to_nearest_tick(y, &self.axis_ticks_y2),
                            YAxis::Right2 => snap_to_nearest_tick(y, &self.axis_ticks_y3),
                            YAxis::Right3 => snap_to_nearest_tick(y, &self.axis_ticks_y4),
                        }
                    };

                    let state = self.read_plot_state(cx.app);
                    let view_bounds = self.current_view_bounds(cx.app, &state);
                    let view_bounds_y2 = self.current_view_bounds_y2(cx.app, &state, view_bounds);
                    let view_bounds_y3 = self.current_view_bounds_y3(cx.app, &state, view_bounds);
                    let view_bounds_y4 = self.current_view_bounds_y4(cx.app, &state, view_bounds);

                    let local = local_from_absolute(layout.plot.origin, *position);
                    self.cursor_px = Some(local);
                    self.hover = None;

                    let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size);
                    let transform_x = PlotTransform {
                        viewport: local_viewport,
                        data: view_bounds,
                        x_scale: self.x_scale,
                        y_scale: self.y_scale,
                    };
                    let transform_for_y_axis = |axis: YAxis| -> Option<PlotTransform> {
                        match axis {
                            YAxis::Left => Some(PlotTransform {
                                viewport: local_viewport,
                                data: view_bounds,
                                x_scale: self.x_scale,
                                y_scale: self.y_scale,
                            }),
                            YAxis::Right if self.show_y2_axis => {
                                view_bounds_y2.map(|b| PlotTransform {
                                    viewport: local_viewport,
                                    data: b,
                                    x_scale: self.x_scale,
                                    y_scale: self.y2_scale,
                                })
                            }
                            YAxis::Right2 if self.show_y3_axis => {
                                view_bounds_y3.map(|b| PlotTransform {
                                    viewport: local_viewport,
                                    data: b,
                                    x_scale: self.x_scale,
                                    y_scale: self.y3_scale,
                                })
                            }
                            YAxis::Right3 if self.show_y4_axis => {
                                view_bounds_y4.map(|b| PlotTransform {
                                    viewport: local_viewport,
                                    data: b,
                                    x_scale: self.x_scale,
                                    y_scale: self.y4_scale,
                                })
                            }
                            _ => None,
                        }
                    };

                    match &mut capture {
                        DragCapture::LineX {
                            id,
                            offset_x,
                            current_x,
                            ..
                        } => {
                            let data = transform_x.px_to_data(local);
                            if data.x.is_finite() {
                                *current_x = data.x - *offset_x;
                                self.drag_output = Some(PlotDragOutput::LineX {
                                    id: *id,
                                    x: *current_x,
                                    phase: PlotDragPhase::Update,
                                });
                            }
                        }
                        DragCapture::LineY {
                            id,
                            axis,
                            offset_y,
                            current_y,
                            ..
                        } => {
                            if let Some(transform) = transform_for_y_axis(*axis) {
                                let data = transform.px_to_data(local);
                                if data.y.is_finite() {
                                    *current_y = data.y - *offset_y;
                                    self.drag_output = Some(PlotDragOutput::LineY {
                                        id: *id,
                                        axis: *axis,
                                        y: *current_y,
                                        phase: PlotDragPhase::Update,
                                    });
                                }
                            }
                        }
                        DragCapture::Point {
                            id,
                            axis,
                            offset,
                            start,
                            constraint,
                            current,
                            ..
                        } => {
                            if let Some(transform) = transform_for_y_axis(*axis) {
                                let data = transform.px_to_data(local);
                                if data.x.is_finite() && data.y.is_finite() {
                                    let mut next = DataPoint {
                                        x: data.x - offset.x,
                                        y: data.y - offset.y,
                                    };

                                    if modifiers.shift {
                                        let next_constraint = match *constraint {
                                            Some(c) => Some(c),
                                            None => {
                                                let dx = (next.x - start.x).abs();
                                                let dy = (next.y - start.y).abs();
                                                Some(if dx >= dy {
                                                    DragAxisConstraint::XOnly
                                                } else {
                                                    DragAxisConstraint::YOnly
                                                })
                                            }
                                        };
                                        if let Some(c) = next_constraint {
                                            match c {
                                                DragAxisConstraint::XOnly => next.y = start.y,
                                                DragAxisConstraint::YOnly => next.x = start.x,
                                            }
                                        }
                                        *constraint = next_constraint;
                                    } else {
                                        *constraint = None;
                                    }

                                    if modifiers.alt || modifiers.alt_gr {
                                        next.x = snap_x(next.x);
                                        next.y = snap_y(*axis, next.y);
                                    }
                                    if next.x.is_finite() && next.y.is_finite() {
                                        *current = next;
                                        self.drag_output = Some(PlotDragOutput::Point {
                                            id: *id,
                                            axis: *axis,
                                            point: next,
                                            phase: PlotDragPhase::Update,
                                        });
                                    }
                                }
                            }
                        }
                        DragCapture::Rect {
                            id,
                            axis,
                            handle,
                            offset,
                            start,
                            constraint,
                            current,
                            ..
                        } => {
                            if let Some(transform) = transform_for_y_axis(*axis) {
                                let data = transform.px_to_data(local);
                                if data.x.is_finite() && data.y.is_finite() {
                                    let mut next = *current;
                                    match handle {
                                        DragRectHandle::Inside => {
                                            let w = start.width();
                                            let h = start.height();
                                            next.x_min = data.x - offset.x;
                                            next.x_max = next.x_min + w;
                                            next.y_min = data.y - offset.y;
                                            next.y_max = next.y_min + h;
                                        }
                                        DragRectHandle::Left => {
                                            next.x_min = data.x - offset.x;
                                            if next.x_min > next.x_max {
                                                next.x_min = next.x_max;
                                            }
                                        }
                                        DragRectHandle::Right => {
                                            next.x_max = data.x - offset.x;
                                            if next.x_max < next.x_min {
                                                next.x_max = next.x_min;
                                            }
                                        }
                                        DragRectHandle::Top => {
                                            next.y_max = data.y - offset.y;
                                            if next.y_max < next.y_min {
                                                next.y_max = next.y_min;
                                            }
                                        }
                                        DragRectHandle::Bottom => {
                                            next.y_min = data.y - offset.y;
                                            if next.y_min > next.y_max {
                                                next.y_min = next.y_max;
                                            }
                                        }
                                    }

                                    if matches!(handle, DragRectHandle::Inside) && modifiers.shift {
                                        let next_constraint = match *constraint {
                                            Some(c) => Some(c),
                                            None => {
                                                let dx = (next.x_min - start.x_min).abs();
                                                let dy = (next.y_min - start.y_min).abs();
                                                Some(if dx >= dy {
                                                    DragAxisConstraint::XOnly
                                                } else {
                                                    DragAxisConstraint::YOnly
                                                })
                                            }
                                        };
                                        if let Some(c) = next_constraint {
                                            match c {
                                                DragAxisConstraint::XOnly => {
                                                    next.y_min = start.y_min;
                                                    next.y_max = start.y_max;
                                                }
                                                DragAxisConstraint::YOnly => {
                                                    next.x_min = start.x_min;
                                                    next.x_max = start.x_max;
                                                }
                                            }
                                        }
                                        *constraint = next_constraint;
                                    } else {
                                        *constraint = None;
                                    }

                                    if modifiers.alt || modifiers.alt_gr {
                                        match handle {
                                            DragRectHandle::Inside => {
                                                let w = start.width();
                                                let h = start.height();
                                                next.x_min = snap_x(next.x_min);
                                                next.y_min = snap_y(*axis, next.y_min);
                                                next.x_max = next.x_min + w;
                                                next.y_max = next.y_min + h;
                                            }
                                            DragRectHandle::Left => {
                                                next.x_min = snap_x(next.x_min);
                                                if next.x_min > next.x_max {
                                                    next.x_min = next.x_max;
                                                }
                                            }
                                            DragRectHandle::Right => {
                                                next.x_max = snap_x(next.x_max);
                                                if next.x_max < next.x_min {
                                                    next.x_max = next.x_min;
                                                }
                                            }
                                            DragRectHandle::Top => {
                                                next.y_max = snap_y(*axis, next.y_max);
                                                if next.y_max < next.y_min {
                                                    next.y_max = next.y_min;
                                                }
                                            }
                                            DragRectHandle::Bottom => {
                                                next.y_min = snap_y(*axis, next.y_min);
                                                if next.y_min > next.y_max {
                                                    next.y_min = next.y_max;
                                                }
                                            }
                                        }
                                    }
                                    *current = next;
                                    self.drag_output = Some(PlotDragOutput::Rect {
                                        id: *id,
                                        axis: *axis,
                                        rect: next,
                                        phase: PlotDragPhase::Update,
                                    });
                                }
                            }
                        }
                    }

                    self.drag_capture = Some(capture);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
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
        self.sync_axis_locks(cx.app);

        let default_style = LinePlotStyle::default();
        let font_stack_key = cx
            .app
            .global::<TextFontStackKey>()
            .map(|k| k.0)
            .unwrap_or(0);
        let (
            theme_revision,
            theme_font_size,
            resolved_series_palette,
            background,
            border,
            axis_color,
            grid_color,
            label_color,
            crosshair_color,
            selection_border,
            selection_fill,
            tooltip_background,
            tooltip_border,
            tooltip_text_color,
            resolved_stroke_color,
            resolved_border_width,
            resolved_padding,
            resolved_axis_gap,
            resolved_stroke_width,
            resolved_hover_threshold,
            resolved_heatmap_colorbar_width,
            resolved_heatmap_colorbar_padding,
        ) = {
            let theme = cx.theme();
            let resolved_series_palette =
                if self.style.series_palette == default_style.series_palette {
                    crate::theme_tokens::resolve_series_palette(theme, default_style.series_palette)
                } else {
                    self.style.series_palette
                };

            let resolved_stroke_color = if self.style.stroke_color == default_style.stroke_color
                && self.style.series_palette == default_style.series_palette
            {
                resolved_series_palette[0]
            } else {
                self.style.stroke_color
            };

            let resolved_border_width = if self.style.border_width == default_style.border_width {
                crate::theme_tokens::metric(theme, "fret.plot.border_width", "plot.border_width")
                    .unwrap_or(default_style.border_width)
            } else {
                self.style.border_width
            };

            let resolved_padding = if self.style.padding == default_style.padding {
                crate::theme_tokens::metric(theme, "fret.plot.padding", "plot.padding")
                    .unwrap_or(default_style.padding)
            } else {
                self.style.padding
            };

            let resolved_axis_gap = if self.style.axis_gap == default_style.axis_gap {
                crate::theme_tokens::metric(theme, "fret.plot.axis_gap", "plot.axis_gap")
                    .unwrap_or(default_style.axis_gap)
            } else {
                self.style.axis_gap
            };

            let resolved_stroke_width = if self.style.stroke_width == default_style.stroke_width {
                crate::theme_tokens::metric(theme, "fret.plot.stroke_width", "plot.stroke_width")
                    .unwrap_or(default_style.stroke_width)
            } else {
                self.style.stroke_width
            };

            let resolved_hover_threshold =
                if self.style.hover_threshold == default_style.hover_threshold {
                    crate::theme_tokens::metric(
                        theme,
                        "fret.plot.hover_threshold",
                        "plot.hover_threshold",
                    )
                    .unwrap_or(default_style.hover_threshold)
                } else {
                    self.style.hover_threshold
                };

            let resolved_heatmap_colorbar_width =
                if self.style.heatmap_colorbar_width == default_style.heatmap_colorbar_width {
                    crate::theme_tokens::metric(
                        theme,
                        "fret.plot.heatmap.colorbar.width",
                        "plot.heatmap.colorbar.width",
                    )
                    .unwrap_or(default_style.heatmap_colorbar_width)
                } else {
                    self.style.heatmap_colorbar_width
                };

            let resolved_heatmap_colorbar_padding =
                if self.style.heatmap_colorbar_padding == default_style.heatmap_colorbar_padding {
                    crate::theme_tokens::metric(
                        theme,
                        "fret.plot.heatmap.colorbar.padding",
                        "plot.heatmap.colorbar.padding",
                    )
                    .unwrap_or(default_style.heatmap_colorbar_padding)
                } else {
                    self.style.heatmap_colorbar_padding
                };

            let background = self.style.background.unwrap_or_else(|| {
                crate::theme_tokens::color(theme, "fret.plot.background", "plot.background")
                    .unwrap_or_else(|| theme.color_required("card"))
            });
            let border = self.style.border.unwrap_or_else(|| {
                crate::theme_tokens::color(theme, "fret.plot.border", "plot.border")
                    .unwrap_or_else(|| theme.color_required("border"))
            });

            let axis_color = self.style.axis_color.unwrap_or_else(|| {
                crate::theme_tokens::color(theme, "fret.plot.axis", "plot.axis")
                    .unwrap_or_else(|| theme.color_required("border"))
            });
            let grid_color = self.style.grid_color.unwrap_or_else(|| {
                crate::theme_tokens::color(theme, "fret.plot.grid", "plot.grid").unwrap_or_else(
                    || Color {
                        a: 0.35,
                        ..theme.color_required("border")
                    },
                )
            });
            let label_color = self.style.label_color.unwrap_or_else(|| {
                crate::theme_tokens::color(theme, "fret.plot.label", "plot.label")
                    .unwrap_or_else(|| theme.color_required("muted-foreground"))
            });
            let crosshair_color = self.style.crosshair_color.unwrap_or_else(|| {
                crate::theme_tokens::color(theme, "fret.plot.crosshair", "plot.crosshair")
                    .unwrap_or_else(|| Color {
                        a: 0.65,
                        ..theme.color_required("primary")
                    })
            });

            let selection_border = crate::theme_tokens::color(
                theme,
                "fret.plot.selection.stroke",
                "plot.selection.stroke",
            )
            .unwrap_or(crosshair_color);
            let selection_fill = crate::theme_tokens::color(
                theme,
                "fret.plot.selection.fill",
                "plot.selection.fill",
            )
            .unwrap_or(Color {
                a: (selection_border.a * 0.18).clamp(0.06, 0.22),
                ..selection_border
            });

            let tooltip_background = self.style.tooltip_background.unwrap_or_else(|| {
                crate::theme_tokens::color(
                    theme,
                    "fret.plot.tooltip.background",
                    "plot.tooltip.background",
                )
                .unwrap_or_else(|| theme.color_required("popover"))
            });
            let tooltip_border = self.style.tooltip_border.unwrap_or_else(|| {
                crate::theme_tokens::color(theme, "fret.plot.tooltip.border", "plot.tooltip.border")
                    .unwrap_or_else(|| theme.color_required("popover.border"))
            });
            let tooltip_text_color = self.style.tooltip_text_color.unwrap_or_else(|| {
                crate::theme_tokens::color(theme, "fret.plot.tooltip.text", "plot.tooltip.text")
                    .unwrap_or_else(|| theme.color_required("popover-foreground"))
            });

            let theme_font_size = theme.metric_required("metric.font.size");

            (
                theme.revision(),
                theme_font_size,
                resolved_series_palette,
                background,
                border,
                axis_color,
                grid_color,
                label_color,
                crosshair_color,
                selection_border,
                selection_fill,
                tooltip_background,
                tooltip_border,
                tooltip_text_color,
                resolved_stroke_color,
                resolved_border_width,
                resolved_padding,
                resolved_axis_gap,
                resolved_stroke_width,
                resolved_hover_threshold,
                resolved_heatmap_colorbar_width,
                resolved_heatmap_colorbar_padding,
            )
        };

        let mut text_env_key = 0u64;
        text_env_key = Self::hash_u64(text_env_key, theme_revision);
        text_env_key = Self::hash_u64(text_env_key, font_stack_key);
        text_env_key = Self::hash_u64(text_env_key, u64::from(cx.scale_factor.to_bits()));
        if self.text_env_key != Some(text_env_key) {
            self.text_env_key = Some(text_env_key);

            self.axis_text_cache.clear(cx.services);
            self.legend_text_cache.clear(cx.services);
            self.indicator_text_cache.clear(cx.services);
            self.tooltip_text_cache.clear(cx.services);
            self.readout_text_cache.clear(cx.services);
            self.overlay_text_cache.clear(cx.services);
            self.heatmap_text_cache.clear(cx.services);
            self.debug_text_cache.clear(cx.services);

            self.axis_label_key = None;
            self.axis_labels_x.clear();
            self.axis_labels_y.clear();
            self.axis_labels_y2.clear();
            self.axis_labels_y3.clear();
            self.axis_labels_y4.clear();
            self.axis_lock_indicator_x = None;
            self.axis_lock_indicator_y = None;
            self.axis_lock_indicator_y2 = None;
            self.axis_lock_indicator_y3 = None;
            self.axis_lock_indicator_y4 = None;

            self.legend_key = None;
            self.legend_entries.clear();

            self.tooltip_text = None;
            self.mouse_readout_text = None;
            self.linked_cursor_readout_text = None;
            self.overlays_text_key = None;
            self.overlays_text.clear();

            self.heatmap_colorbar_text_key = None;
            self.heatmap_colorbar_text.clear();
            self.heatmap_colorbar_gradient_cache.clear();
            self.quads_scene_cache.clear();

            #[cfg(debug_assertions)]
            {
                self.debug_overlay_text = None;
            }
        }

        self.axis_text_cache.begin_frame();
        self.legend_text_cache.begin_frame();
        self.indicator_text_cache.begin_frame();
        self.tooltip_text_cache.begin_frame();
        self.readout_text_cache.begin_frame();
        self.overlay_text_cache.begin_frame();
        self.heatmap_text_cache.begin_frame();
        self.debug_text_cache.begin_frame();

        self.indicator_text_cache.prune(cx.services, 60, 64);
        self.tooltip_text_cache.prune(cx.services, 60, 256);
        self.readout_text_cache.prune(cx.services, 60, 256);
        self.overlay_text_cache.prune(cx.services, 60, 512);
        self.heatmap_text_cache.prune(cx.services, 60, 64);
        self.debug_text_cache.prune(cx.services, 4, 16);

        let resolved_style = LinePlotStyle {
            series_palette: resolved_series_palette,
            stroke_color: resolved_stroke_color,
            border_width: resolved_border_width,
            padding: resolved_padding,
            axis_gap: resolved_axis_gap,
            stroke_width: resolved_stroke_width,
            hover_threshold: resolved_hover_threshold,
            heatmap_colorbar_width: resolved_heatmap_colorbar_width,
            heatmap_colorbar_padding: resolved_heatmap_colorbar_padding,
            ..self.style
        };

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: fret_core::Paint::Solid(background),
            border: fret_core::Edges::all(resolved_style.border_width),
            border_paint: fret_core::Paint::Solid(border),

            corner_radii: fret_core::Corners::all(Px(0.0)),
        });

        let min_axis = resolved_style.axis_gap;
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
            resolved_style.padding,
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
                theme_revision,
                font_stack_key,
            );
            if !changed {
                break;
            }

            let next_layout = PlotLayout::from_bounds(
                cx.bounds,
                resolved_style.padding,
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
        self.rebuild_legend_if_needed(cx, theme_revision, font_stack_key);

        // Grid + series + hover are clipped to the plot area.
        cx.scene.push(SceneOp::PushClipRect { rect: layout.plot });

        if layout.plot.size.width.0 > 0.0 && layout.plot.size.height.0 > 0.0 {
            // Plot-space images (caller-owned overlays). These are rendered in plot coordinates and
            // clipped to the plot viewport.
            let plot_images = &state.overlays.images;
            let image_transform_y1 = PlotTransform {
                viewport: layout.plot,
                data: view_bounds,
                x_scale: self.x_scale,
                y_scale: self.y_scale,
            };
            let image_transform_y2 =
                view_bounds_y2
                    .filter(|_| self.show_y2_axis)
                    .map(|b| PlotTransform {
                        viewport: layout.plot,
                        data: b,
                        x_scale: self.x_scale,
                        y_scale: self.y2_scale,
                    });
            let image_transform_y3 =
                view_bounds_y3
                    .filter(|_| self.show_y3_axis)
                    .map(|b| PlotTransform {
                        viewport: layout.plot,
                        data: b,
                        x_scale: self.x_scale,
                        y_scale: self.y3_scale,
                    });
            let image_transform_y4 =
                view_bounds_y4
                    .filter(|_| self.show_y4_axis)
                    .map(|b| PlotTransform {
                        viewport: layout.plot,
                        data: b,
                        x_scale: self.x_scale,
                        y_scale: self.y4_scale,
                    });

            paint_plot_images(
                cx.scene,
                plot_images,
                PlotImageLayer::BelowGrid,
                image_transform_y1,
                image_transform_y2,
                image_transform_y3,
                image_transform_y4,
            );

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
                    background: fret_core::Paint::Solid(background),
                    border: fret_core::Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT,

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
                    background: fret_core::Paint::Solid(background),
                    border: fret_core::Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT,

                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });
            }

            paint_plot_images(
                cx.scene,
                plot_images,
                PlotImageLayer::AboveGrid,
                image_transform_y1,
                image_transform_y2,
                image_transform_y3,
                image_transform_y4,
            );

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

            let mut emphasized_path: Option<(PathId, Color)> = None;

            let quads_emitted_by_layer = self
                .model
                .read(cx.app, |_app, m| m.clone())
                .ok()
                .is_some_and(|model| {
                    let model_revision = self.model.revision(cx.app).unwrap_or(0);
                    let plot_local = Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size);

                    self.layer.paint_quads_scene_ops_tiled(
                        cx,
                        &model,
                        PlotPaintArgs {
                            model_revision,
                            plot: plot_local,
                            view_bounds,
                            view_bounds_y2,
                            view_bounds_y3,
                            view_bounds_y4,
                            x_scale: self.x_scale,
                            y_scale: self.y_scale,
                            y2_scale: self.y2_scale,
                            y3_scale: self.y3_scale,
                            y4_scale: self.y4_scale,
                            style: resolved_style,
                            hidden,
                            view_interacting: self.view_interacting(),
                        },
                        layout.plot.origin,
                    )
                });

            if !quads_emitted_by_layer {
                match self.layer.quads_scene_cache_policy() {
                    PlotQuadsSceneCachePolicy::Disabled => {
                        for quad in self.rebuild_quads_if_needed(
                            cx,
                            layout.plot,
                            view_bounds,
                            view_bounds_y2,
                            view_bounds_y3,
                            view_bounds_y4,
                            hidden,
                            resolved_style,
                        ) {
                            cx.scene.push(SceneOp::Quad {
                                order: quad.order,
                                rect: offset_rect(quad.rect_local, layout.plot.origin),
                                background: fret_core::Paint::Solid(quad.background),

                                border: fret_core::Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT,

                                corner_radii: fret_core::Corners::all(Px(0.0)),
                            });
                        }
                    }
                    PlotQuadsSceneCachePolicy::Enabled => {
                        let mut key = 0u64;
                        key = Self::hash_u64(key, theme_revision);
                        key = Self::hash_u64(key, self.model.revision(cx.app).unwrap_or(0));
                        key = Self::hash_u64(key, u64::from(resolved_style.heatmap_colormap.key()));
                        key = Self::hash_f32_bits(key, layout.plot.size.width.0);
                        key = Self::hash_f32_bits(key, layout.plot.size.height.0);
                        key = Self::hash_f64_bits(key, view_bounds.x_min);
                        key = Self::hash_f64_bits(key, view_bounds.x_max);
                        key = Self::hash_f64_bits(key, view_bounds.y_min);
                        key = Self::hash_f64_bits(key, view_bounds.y_max);
                        key = Self::hash_u64(key, self.x_scale.key());
                        key = Self::hash_u64(key, self.y_scale.key());

                        if self.quads_scene_cache.try_replay_with(
                            key,
                            cx.scene,
                            layout.plot.origin,
                            |_ops| {},
                        ) {
                            // Cache hit.
                        } else {
                            let quads = self.rebuild_quads_if_needed(
                                cx,
                                layout.plot,
                                view_bounds,
                                view_bounds_y2,
                                view_bounds_y3,
                                view_bounds_y4,
                                hidden,
                                resolved_style,
                            );

                            let mut ops: Vec<SceneOp> = Vec::with_capacity(quads.len());
                            for quad in quads {
                                ops.push(SceneOp::Quad {
                                    order: quad.order,
                                    rect: quad.rect_local,
                                    background: fret_core::Paint::Solid(quad.background),

                                    border: fret_core::Edges::all(Px(0.0)),
                                    border_paint: fret_core::Paint::TRANSPARENT,

                                    corner_radii: fret_core::Corners::all(Px(0.0)),
                                });
                            }

                            #[cfg(debug_assertions)]
                            {
                                debug_assert!(
                                    ops.iter().all(|op| {
                                        !matches!(
                                            op,
                                            SceneOp::Text { .. }
                                                | SceneOp::Path { .. }
                                                | SceneOp::SvgMaskIcon { .. }
                                                | SceneOp::SvgImage { .. }
                                        )
                                    }),
                                    "Cached plot quad scene ops must not include hosted resources without touching their caches on replay"
                                );
                            }

                            cx.scene.replay_ops_translated(&ops, layout.plot.origin);
                            self.quads_scene_cache.store_ops(key, ops);
                        }
                    }
                }
            }

            let mut debug_paths_pushed: u32 = 0;

            let paths = self.rebuild_paths_if_needed(
                cx,
                layout.plot,
                view_bounds,
                view_bounds_y2,
                view_bounds_y3,
                view_bounds_y4,
                hidden,
                resolved_style,
            );
            let debug_paths_prepared = paths.len().min(u32::MAX as usize) as u32;

            for (series_id, path, color) in paths {
                if emphasized
                    && let Some(emphasized) = emphasized_series
                    && emphasized == series_id
                {
                    emphasized_path = Some((path, color));
                    continue;
                }

                let color = if emphasized && emphasized_series.is_some() {
                    dim_color(color, dim_alpha)
                } else {
                    color
                };
                cx.scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: layout.plot.origin,
                    path,
                    color,
                });
                debug_paths_pushed = debug_paths_pushed.saturating_add(1);
            }

            if let Some((path, color)) = emphasized_path {
                cx.scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: layout.plot.origin,
                    path,
                    color,
                });
                debug_paths_pushed = debug_paths_pushed.saturating_add(1);
            }

            // Heatmap colorbar (when the current plot layer supports it).
            if resolved_style.heatmap_show_colorbar
                && let Ok(model) = self.model.read(cx.app, |_app, m| m.clone())
                && let Some((vmin, vmax)) = L::heatmap_value_range(&model)
                && vmin.is_finite()
                && vmax.is_finite()
                && vmax > vmin
            {
                let padding = resolved_style.heatmap_colorbar_padding.0.max(0.0);
                let bar_w = resolved_style.heatmap_colorbar_width.0.max(1.0);
                let steps = resolved_style.heatmap_colorbar_steps.clamp(8, 512);

                let plot_w = layout.plot.size.width.0;
                let plot_h = layout.plot.size.height.0;
                let bar_h = (plot_h - padding * 2.0).max(0.0);
                if plot_w > 0.0 && plot_h > 0.0 && bar_h >= 24.0 {
                    let text_style = TextStyle {
                        font: FontId::default(),
                        size: Px((theme_font_size.0 * 0.85).max(9.0)),
                        weight: FontWeight::NORMAL,
                        slant: TextSlant::Normal,
                        line_height: None,
                        letter_spacing_em: None,
                    };
                    let text_constraints = TextConstraints {
                        max_width: None,
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        scale_factor: cx.scale_factor,
                    };

                    let max_label = format_colorbar_value(vmax);
                    let min_label = format_colorbar_value(vmin);

                    let mut key = 0u64;
                    key = Self::hash_u64(key, theme_revision);
                    key = Self::hash_u64(key, font_stack_key);
                    key = Self::hash_u64(key, u64::from(cx.scale_factor.to_bits()));
                    key = Self::hash_u64(key, u64::from(vmin.to_bits()));
                    key = Self::hash_u64(key, u64::from(vmax.to_bits()));
                    key = Self::hash_u64(key, u64::from(resolved_style.heatmap_colormap.key()));
                    key = Self::hash_u64(key, u64::from(steps as u32));
                    key = Self::hash_u64(key, u64::from(bar_w.to_bits()));
                    key = Self::hash_u64(key, Self::text_style_key(&text_style));

                    self.heatmap_colorbar_text_key = Some(key);
                    self.heatmap_colorbar_text.clear();
                    self.heatmap_colorbar_text.reserve(2);
                    self.heatmap_colorbar_text
                        .push(self.heatmap_text_cache.prepare(
                            cx.services,
                            max_label.as_str(),
                            &text_style,
                            text_constraints,
                        ));
                    self.heatmap_colorbar_text
                        .push(self.heatmap_text_cache.prepare(
                            cx.services,
                            min_label.as_str(),
                            &text_style,
                            text_constraints,
                        ));

                    let max_text = self.heatmap_colorbar_text.get(0).copied();
                    let min_text = self.heatmap_colorbar_text.get(1).copied();

                    let label_gap = 6.0_f32;
                    let label_w = max_text
                        .map(|t| t.metrics.size.width.0)
                        .unwrap_or(0.0)
                        .max(min_text.map(|t| t.metrics.size.width.0).unwrap_or(0.0));

                    let panel_w = (bar_w + label_gap + label_w).max(bar_w);
                    let panel_left = (plot_w - padding - panel_w).max(padding);
                    let panel_top = padding;

                    let bar_left = panel_left;
                    let label_x = bar_left + bar_w + label_gap;
                    let bar_top = panel_top;

                    let panel_rect = Rect::new(
                        Point::new(
                            Px(layout.plot.origin.x.0 + panel_left),
                            Px(layout.plot.origin.y.0 + panel_top),
                        ),
                        Size::new(Px(panel_w), Px(bar_h)),
                    );
                    let panel_radius = cx.theme().metric_required("metric.radius.sm");
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(3),
                        rect: panel_rect,
                        background: fret_core::Paint::Solid(Color {
                            a: 0.88,
                            ..tooltip_background
                        }),
                        border: fret_core::Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(tooltip_border),

                        corner_radii: fret_core::Corners::all(panel_radius),
                    });

                    let colormap = resolved_style.heatmap_colormap;
                    cx.scene.with_transform(
                        Transform2D::translation(layout.plot.origin),
                        |scene| {
                            let mut gradient_key = 0u64;
                            gradient_key = Self::hash_u64(gradient_key, u64::from(colormap.key()));
                            gradient_key = Self::hash_u64(gradient_key, u64::from(steps as u32));
                            gradient_key = Self::hash_f32_bits(gradient_key, bar_left);
                            gradient_key = Self::hash_f32_bits(gradient_key, bar_top);
                            gradient_key = Self::hash_f32_bits(gradient_key, bar_w);
                            gradient_key = Self::hash_f32_bits(gradient_key, bar_h);

                            self.heatmap_colorbar_gradient_cache.replay_or_record_with(
                                gradient_key,
                                scene,
                                Point::new(Px(0.0), Px(0.0)),
                                |_ops| {},
                                |scene| {
                                    #[cfg(debug_assertions)]
                                    let start = scene.ops_len();
                                    for i in 0..steps {
                                        let t0 = (i as f32) / (steps as f32);
                                        let t1 = ((i + 1) as f32) / (steps as f32);
                                        let t = (t0 + t1) * 0.5;
                                        let color = crate::plot::colormap::sample(colormap, t);

                                        let y0 = bar_top + (1.0 - t1) * bar_h;
                                        let h = (t1 - t0) * bar_h;
                                        scene.push(SceneOp::Quad {
                                            order: DrawOrder(4),
                                            rect: Rect::new(
                                                Point::new(Px(bar_left), Px(y0)),
                                                Size::new(Px(bar_w), Px(h.max(1.0))),
                                            ),
                                            background: fret_core::Paint::Solid(color),

                                            border: fret_core::Edges::all(Px(0.0)),
                                            border_paint: fret_core::Paint::TRANSPARENT,

                                            corner_radii: fret_core::Corners::all(Px(0.0)),
                                        });
                                    }
                                    #[cfg(debug_assertions)]
                                    {
                                        let end = scene.ops_len();
                                        debug_assert!(
                                            scene.ops()[start..end].iter().all(|op| {
                                                !matches!(
                                                    op,
                                                    SceneOp::Text { .. }
                                                        | SceneOp::Path { .. }
                                                        | SceneOp::SvgMaskIcon { .. }
                                                        | SceneOp::SvgImage { .. }
                                                )
                                            }),
                                            "Cached colorbar gradient ops must not include hosted resources without touching their caches on replay"
                                        );
                                    }
                                },
                            );

                            scene.push(SceneOp::Quad {
                                order: DrawOrder(5),
                                rect: Rect::new(
                                    Point::new(Px(bar_left), Px(bar_top)),
                                    Size::new(Px(bar_w), Px(bar_h)),
                                ),
                                background: fret_core::Paint::TRANSPARENT,

                                border: fret_core::Edges::all(Px(1.0)),
                                border_paint: fret_core::Paint::Solid(tooltip_border),

                                corner_radii: fret_core::Corners::all(Px(0.0)),
                            });
                        },
                    );

                    let text_color = tooltip_text_color;
                    let text_margin = 2.0_f32;

                    if let Some(t) = max_text {
                        let origin = Point::new(
                            Px(layout.plot.origin.x.0 + label_x),
                            Px(layout.plot.origin.y.0
                                + bar_top
                                + text_margin
                                + t.metrics.baseline.0),
                        );
                        cx.scene.push(SceneOp::Text {
                            order: DrawOrder(6),
                            origin,
                            text: t.blob,
                            color: text_color,
                        });
                    }
                    if let Some(t) = min_text {
                        let baseline = layout.plot.origin.y.0 + bar_top + bar_h
                            - text_margin
                            - (t.metrics.size.height.0 - t.metrics.baseline.0);
                        let origin = Point::new(Px(layout.plot.origin.x.0 + label_x), Px(baseline));
                        cx.scene.push(SceneOp::Text {
                            order: DrawOrder(6),
                            origin,
                            text: t.blob,
                            color: text_color,
                        });
                    }
                }
            }

            // Infinite reference lines (caller-owned overlays).
            let overlays = &state.overlays;
            if !overlays.inf_lines_x.is_empty()
                || !overlays.inf_lines_y.is_empty()
                || !overlays.drag_lines_x.is_empty()
                || !overlays.drag_lines_y.is_empty()
                || !overlays.drag_points.is_empty()
                || !overlays.drag_rects.is_empty()
                || !overlays.tags_x.is_empty()
                || !overlays.tags_y.is_empty()
                || !overlays.text.is_empty()
            {
                let default_color = Color {
                    a: (crosshair_color.a * 0.45).clamp(0.05, 1.0),
                    ..crosshair_color
                };

                let theme = cx.theme();
                let annotation_background = crate::theme_tokens::color(
                    theme,
                    "fret.plot.annotation.background",
                    "plot.annotation.background",
                )
                .unwrap_or(tooltip_background);
                let annotation_border = crate::theme_tokens::color(
                    theme,
                    "fret.plot.annotation.border",
                    "plot.annotation.border",
                )
                .unwrap_or(tooltip_border);
                let annotation_text = crate::theme_tokens::color(
                    theme,
                    "fret.plot.annotation.text",
                    "plot.annotation.text",
                )
                .unwrap_or(tooltip_text_color);
                let annotation_stroke = crate::theme_tokens::color(
                    theme,
                    "fret.plot.annotation.stroke",
                    "plot.annotation.stroke",
                )
                .unwrap_or(crosshair_color);
                let annotation_padding = crate::theme_tokens::metric(
                    theme,
                    "fret.plot.annotation.padding",
                    "plot.annotation.padding",
                )
                .unwrap_or_else(|| theme.metric_required("metric.padding.sm"));
                let annotation_radius = crate::theme_tokens::metric(
                    theme,
                    "fret.plot.annotation.radius",
                    "plot.annotation.radius",
                )
                .unwrap_or_else(|| theme.metric_required("metric.radius.sm"));

                let margin = Px(6.0);
                let marker_len = Px(8.0);

                let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size);
                let transform_x: Option<PreparedPlotTransform> = PlotTransform {
                    viewport: local_viewport,
                    data: view_bounds,
                    x_scale: self.x_scale,
                    y_scale: self.y_scale,
                }
                .prepare();

                let transform_for_y_axis = |axis: YAxis| -> Option<PreparedPlotTransform> {
                    match axis {
                        YAxis::Left => PlotTransform {
                            viewport: local_viewport,
                            data: view_bounds,
                            x_scale: self.x_scale,
                            y_scale: self.y_scale,
                        }
                        .prepare(),
                        YAxis::Right if self.show_y2_axis => view_bounds_y2.and_then(|b| {
                            PlotTransform {
                                viewport: local_viewport,
                                data: b,
                                x_scale: self.x_scale,
                                y_scale: self.y2_scale,
                            }
                            .prepare()
                        }),
                        YAxis::Right2 if self.show_y3_axis => view_bounds_y3.and_then(|b| {
                            PlotTransform {
                                viewport: local_viewport,
                                data: b,
                                x_scale: self.x_scale,
                                y_scale: self.y3_scale,
                            }
                            .prepare()
                        }),
                        YAxis::Right3 if self.show_y4_axis => view_bounds_y4.and_then(|b| {
                            PlotTransform {
                                viewport: local_viewport,
                                data: b,
                                x_scale: self.x_scale,
                                y_scale: self.y4_scale,
                            }
                            .prepare()
                        }),
                        _ => None,
                    }
                };

                if !overlays.inf_lines_x.is_empty()
                    && let Some(transform_x) = transform_x
                {
                    for line in &overlays.inf_lines_x {
                        if !line.x.is_finite() {
                            continue;
                        }
                        let Some(x_px) = transform_x.data_x_to_px(line.x) else {
                            continue;
                        };

                        let w = line.width.0.max(1.0).min(layout.plot.size.width.0.max(1.0));
                        let left = (x_px.0 - w * 0.5).clamp(0.0, layout.plot.size.width.0 - w);
                        let x = Px((layout.plot.origin.x.0 + left).round());

                        cx.scene.push(SceneOp::Quad {
                            order: DrawOrder(3),
                            rect: Rect::new(
                                Point::new(x, layout.plot.origin.y),
                                Size::new(Px(w), layout.plot.size.height),
                            ),
                            background: fret_core::Paint::Solid(
                                line.color.unwrap_or(default_color),
                            ),

                            border: fret_core::Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT,

                            corner_radii: fret_core::Corners::all(Px(0.0)),
                        });
                    }
                }

                if !overlays.inf_lines_y.is_empty() {
                    for line in &overlays.inf_lines_y {
                        if !line.y.is_finite() {
                            continue;
                        }
                        let Some(transform) = transform_for_y_axis(line.axis) else {
                            continue;
                        };
                        let Some(y_px) = transform.data_y_to_px(line.y) else {
                            continue;
                        };

                        let h = line
                            .width
                            .0
                            .max(1.0)
                            .min(layout.plot.size.height.0.max(1.0));
                        let top = (y_px.0 - h * 0.5).clamp(0.0, layout.plot.size.height.0 - h);
                        let y = Px((layout.plot.origin.y.0 + top).round());

                        cx.scene.push(SceneOp::Quad {
                            order: DrawOrder(3),
                            rect: Rect::new(
                                Point::new(layout.plot.origin.x, y),
                                Size::new(layout.plot.size.width, Px(h)),
                            ),
                            background: fret_core::Paint::Solid(
                                line.color.unwrap_or(default_color),
                            ),

                            border: fret_core::Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT,

                            corner_radii: fret_core::Corners::all(Px(0.0)),
                        });
                    }
                }

                if !overlays.drag_rects.is_empty() {
                    for rect in &overlays.drag_rects {
                        let Some(transform) = transform_for_y_axis(rect.axis) else {
                            continue;
                        };

                        let mut current = rect.rect;
                        if let Some(DragCapture::Rect {
                            id,
                            current: dragged,
                            ..
                        }) = self.drag_capture
                            && id == rect.id
                        {
                            current = dragged;
                        }

                        if !current.x_min.is_finite()
                            || !current.x_max.is_finite()
                            || !current.y_min.is_finite()
                            || !current.y_max.is_finite()
                        {
                            continue;
                        }

                        let a = transform.data_to_px(DataPoint {
                            x: current.x_min,
                            y: current.y_min,
                        });
                        let b = transform.data_to_px(DataPoint {
                            x: current.x_max,
                            y: current.y_max,
                        });
                        if !a.x.0.is_finite()
                            || !a.y.0.is_finite()
                            || !b.x.0.is_finite()
                            || !b.y.0.is_finite()
                        {
                            continue;
                        }

                        let left = a.x.0.min(b.x.0).clamp(0.0, layout.plot.size.width.0);
                        let right = a.x.0.max(b.x.0).clamp(0.0, layout.plot.size.width.0);
                        let top = a.y.0.min(b.y.0).clamp(0.0, layout.plot.size.height.0);
                        let bottom = a.y.0.max(b.y.0).clamp(0.0, layout.plot.size.height.0);

                        let w = (right - left).max(0.0);
                        let h = (bottom - top).max(0.0);
                        if w <= 0.0 || h <= 0.0 {
                            continue;
                        }

                        let color = rect.color.unwrap_or(annotation_stroke);
                        let fill = rect.fill.unwrap_or(Color { a: 0.12, ..color });
                        let border_w = rect.border_width.0.max(1.0);

                        let abs_rect = Rect::new(
                            Point::new(
                                Px((layout.plot.origin.x.0 + left).round()),
                                Px((layout.plot.origin.y.0 + top).round()),
                            ),
                            Size::new(Px(w), Px(h)),
                        );

                        cx.scene.push(SceneOp::Quad {
                            order: DrawOrder(3),
                            rect: abs_rect,
                            background: fret_core::Paint::Solid(fill),

                            border: fret_core::Edges::all(Px(border_w)),
                            border_paint: fret_core::Paint::Solid(color),

                            corner_radii: fret_core::Corners::all(Px(0.0)),
                        });
                    }
                }

                if !overlays.drag_lines_x.is_empty()
                    && let Some(transform_x) = transform_x
                {
                    for line in &overlays.drag_lines_x {
                        let mut x = line.x;
                        if let Some(DragCapture::LineX { id, current_x, .. }) = self.drag_capture
                            && id == line.id
                        {
                            x = current_x;
                        }

                        if !x.is_finite() {
                            continue;
                        }
                        let Some(x_px) = transform_x.data_x_to_px(x) else {
                            continue;
                        };

                        let w = line.width.0.max(1.0).min(layout.plot.size.width.0.max(1.0));
                        let left = (x_px.0 - w * 0.5).clamp(0.0, layout.plot.size.width.0 - w);
                        let x_abs = Px((layout.plot.origin.x.0 + left).round());

                        cx.scene.push(SceneOp::Quad {
                            order: DrawOrder(3),
                            rect: Rect::new(
                                Point::new(x_abs, layout.plot.origin.y),
                                Size::new(Px(w), layout.plot.size.height),
                            ),
                            background: fret_core::Paint::Solid(
                                line.color.unwrap_or(annotation_stroke),
                            ),

                            border: fret_core::Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT,

                            corner_radii: fret_core::Corners::all(Px(0.0)),
                        });
                    }
                }

                if !overlays.drag_lines_y.is_empty() {
                    for line in &overlays.drag_lines_y {
                        let Some(transform) = transform_for_y_axis(line.axis) else {
                            continue;
                        };

                        let mut y = line.y;
                        if let Some(DragCapture::LineY { id, current_y, .. }) = self.drag_capture
                            && id == line.id
                        {
                            y = current_y;
                        }

                        if !y.is_finite() {
                            continue;
                        }
                        let Some(y_px) = transform.data_y_to_px(y) else {
                            continue;
                        };

                        let h = line
                            .width
                            .0
                            .max(1.0)
                            .min(layout.plot.size.height.0.max(1.0));
                        let top = (y_px.0 - h * 0.5).clamp(0.0, layout.plot.size.height.0 - h);
                        let y_abs = Px((layout.plot.origin.y.0 + top).round());

                        cx.scene.push(SceneOp::Quad {
                            order: DrawOrder(3),
                            rect: Rect::new(
                                Point::new(layout.plot.origin.x, y_abs),
                                Size::new(layout.plot.size.width, Px(h)),
                            ),
                            background: fret_core::Paint::Solid(
                                line.color.unwrap_or(annotation_stroke),
                            ),

                            border: fret_core::Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT,

                            corner_radii: fret_core::Corners::all(Px(0.0)),
                        });
                    }
                }

                if !overlays.drag_points.is_empty() {
                    for point in &overlays.drag_points {
                        let Some(transform) = transform_for_y_axis(point.axis) else {
                            continue;
                        };

                        let mut current = point.point;
                        if let Some(DragCapture::Point {
                            id,
                            current: dragged,
                            ..
                        }) = self.drag_capture
                            && id == point.id
                        {
                            current = dragged;
                        }

                        if !current.x.is_finite() || !current.y.is_finite() {
                            continue;
                        }

                        let p_px = transform.data_to_px(current);
                        if !p_px.x.0.is_finite() || !p_px.y.0.is_finite() {
                            continue;
                        }

                        let color = point.color.unwrap_or(annotation_stroke);
                        let r = point.radius.0.max(2.0);
                        let d = (r * 2.0).max(1.0);

                        let max_left = (layout.plot.size.width.0 - d).max(0.0);
                        let max_top = (layout.plot.size.height.0 - d).max(0.0);
                        let left = (p_px.x.0 - r).clamp(0.0, max_left);
                        let top = (p_px.y.0 - r).clamp(0.0, max_top);
                        let abs_rect = Rect::new(
                            Point::new(
                                Px((layout.plot.origin.x.0 + left).round()),
                                Px((layout.plot.origin.y.0 + top).round()),
                            ),
                            Size::new(Px(d), Px(d)),
                        );

                        cx.scene.push(SceneOp::Quad {
                            order: DrawOrder(3),
                            rect: abs_rect,
                            background: fret_core::Paint::Solid(color),

                            border: fret_core::Edges::all(Px(1.0)),
                            border_paint: fret_core::Paint::Solid(annotation_border),

                            corner_radii: fret_core::Corners::all(Px(r)),
                        });
                    }
                }

                if !overlays.tags_x.is_empty()
                    || !overlays.tags_y.is_empty()
                    || !overlays.text.is_empty()
                    || !overlays.drag_lines_x.is_empty()
                    || !overlays.drag_lines_y.is_empty()
                    || !overlays.drag_points.is_empty()
                    || !overlays.drag_rects.is_empty()
                {
                    #[derive(Debug, Clone)]
                    enum OverlayPlacement {
                        TagX {
                            x: Px,
                            color: Color,
                        },
                        TagY {
                            y: Px,
                            right: bool,
                            color: Color,
                        },
                        Text {
                            origin: Point,
                            color: Color,
                            background: Option<Color>,
                            border: Option<Color>,
                            padding: Px,
                            corner_radius: Px,
                        },
                    }

                    #[derive(Debug, Clone)]
                    struct OverlayDraft {
                        text: String,
                        placement: OverlayPlacement,
                    }

                    let x_span = (view_bounds.x_max - view_bounds.x_min).abs();
                    let y_span = (view_bounds.y_max - view_bounds.y_min).abs();

                    let mut drafts: Vec<OverlayDraft> = Vec::new();

                    for line in &overlays.drag_lines_x {
                        let Some(transform_x) = transform_x else {
                            break;
                        };
                        let mut x = line.x;
                        if let Some(DragCapture::LineX { id, current_x, .. }) = self.drag_capture
                            && id == line.id
                        {
                            x = current_x;
                        }

                        if !x.is_finite() {
                            continue;
                        }
                        let x_value = x;
                        let Some(x_px) = transform_x.data_x_to_px(x) else {
                            continue;
                        };
                        let x = Px((layout.plot.origin.x.0 + x_px.0).round());

                        let value = line
                            .show_value
                            .then(|| self.tooltip_x_labels.format(x_value, x_span))
                            .unwrap_or_default();
                        let text = match (&line.label, line.show_value) {
                            (Some(label), true) => format!("{label}: {value}"),
                            (Some(label), false) => label.clone(),
                            (None, true) => value,
                            (None, false) => String::new(),
                        };
                        if text.is_empty() {
                            continue;
                        }

                        drafts.push(OverlayDraft {
                            text,
                            placement: OverlayPlacement::TagX {
                                x,
                                color: line.color.unwrap_or(annotation_stroke),
                            },
                        });
                    }

                    for line in &overlays.drag_lines_y {
                        let Some(transform) = transform_for_y_axis(line.axis) else {
                            continue;
                        };

                        let mut y = line.y;
                        if let Some(DragCapture::LineY { id, current_y, .. }) = self.drag_capture
                            && id == line.id
                        {
                            y = current_y;
                        }

                        if !y.is_finite() {
                            continue;
                        }
                        let y_value = y;
                        let Some(y_px) = transform.data_y_to_px(y) else {
                            continue;
                        };
                        let y = Px((layout.plot.origin.y.0 + y_px.0).round());

                        let (span, labels) = match line.axis {
                            YAxis::Right if self.show_y2_axis => (
                                view_bounds_y2
                                    .map(|b| (b.y_max - b.y_min).abs())
                                    .unwrap_or(y_span),
                                &self.y2_axis_labels,
                            ),
                            YAxis::Right2 if self.show_y3_axis => (
                                view_bounds_y3
                                    .map(|b| (b.y_max - b.y_min).abs())
                                    .unwrap_or(y_span),
                                &self.y3_axis_labels,
                            ),
                            YAxis::Right3 if self.show_y4_axis => (
                                view_bounds_y4
                                    .map(|b| (b.y_max - b.y_min).abs())
                                    .unwrap_or(y_span),
                                &self.y4_axis_labels,
                            ),
                            _ => (y_span, &self.tooltip_y_labels),
                        };

                        let value = line
                            .show_value
                            .then(|| labels.format(y_value, span))
                            .unwrap_or_default();
                        let text = match (&line.label, line.show_value) {
                            (Some(label), true) => format!("{label}: {value}"),
                            (Some(label), false) => label.clone(),
                            (None, true) => value,
                            (None, false) => String::new(),
                        };
                        if text.is_empty() {
                            continue;
                        }

                        drafts.push(OverlayDraft {
                            text,
                            placement: OverlayPlacement::TagY {
                                y,
                                right: line.axis != YAxis::Left,
                                color: line.color.unwrap_or(annotation_stroke),
                            },
                        });
                    }

                    for point in &overlays.drag_points {
                        let Some(transform) = transform_for_y_axis(point.axis) else {
                            continue;
                        };

                        let mut current = point.point;
                        if let Some(DragCapture::Point {
                            id,
                            current: dragged,
                            ..
                        }) = self.drag_capture
                            && id == point.id
                        {
                            current = dragged;
                        }

                        if !current.x.is_finite() || !current.y.is_finite() {
                            continue;
                        }

                        let p_px = transform.data_to_px(current);
                        if !p_px.x.0.is_finite() || !p_px.y.0.is_finite() {
                            continue;
                        }

                        let x_value = current.x;
                        let y_value = current.y;
                        let Some(transform_x) = transform_x else {
                            continue;
                        };
                        let Some(x_px) = transform_x.data_x_to_px(x_value) else {
                            continue;
                        };

                        let (span, labels) = match point.axis {
                            YAxis::Right if self.show_y2_axis => (
                                view_bounds_y2
                                    .map(|b| (b.y_max - b.y_min).abs())
                                    .unwrap_or(y_span),
                                &self.y2_axis_labels,
                            ),
                            YAxis::Right2 if self.show_y3_axis => (
                                view_bounds_y3
                                    .map(|b| (b.y_max - b.y_min).abs())
                                    .unwrap_or(y_span),
                                &self.y3_axis_labels,
                            ),
                            YAxis::Right3 if self.show_y4_axis => (
                                view_bounds_y4
                                    .map(|b| (b.y_max - b.y_min).abs())
                                    .unwrap_or(y_span),
                                &self.y4_axis_labels,
                            ),
                            _ => (y_span, &self.tooltip_y_labels),
                        };

                        let value = point.show_value.then(|| {
                            let x = self.tooltip_x_labels.format(x_value, x_span);
                            let y = labels.format(y_value, span);
                            format!("({x}, {y})")
                        });
                        let text = match (&point.label, value) {
                            (Some(label), Some(value)) => format!("{label}: {value}"),
                            (Some(label), None) => label.clone(),
                            (None, Some(value)) => value,
                            (None, None) => String::new(),
                        };
                        if text.is_empty() {
                            continue;
                        }

                        let margin = Px(8.0);
                        let origin = Point::new(
                            Px((layout.plot.origin.x.0 + x_px.0 + margin.0).round()),
                            Px((layout.plot.origin.y.0 + p_px.y.0 - margin.0).round()),
                        );

                        drafts.push(OverlayDraft {
                            text,
                            placement: OverlayPlacement::Text {
                                origin,
                                color: annotation_text,
                                background: Some(annotation_background),
                                border: Some(annotation_border),
                                padding: annotation_padding,
                                corner_radius: annotation_radius,
                            },
                        });
                    }

                    if let Some(transform_x) = transform_x {
                        for tag in &overlays.tags_x {
                            if !tag.x.is_finite() {
                                continue;
                            }
                            let Some(x_px) = transform_x.data_x_to_px(tag.x) else {
                                continue;
                            };
                            let x = Px((layout.plot.origin.x.0 + x_px.0).round());

                            let value = tag
                                .show_value
                                .then(|| self.tooltip_x_labels.format(tag.x, x_span))
                                .unwrap_or_default();
                            let text = match (&tag.label, tag.show_value) {
                                (Some(label), true) => format!("{label}: {value}"),
                                (Some(label), false) => label.clone(),
                                (None, true) => value,
                                (None, false) => String::new(),
                            };
                            if text.is_empty() {
                                continue;
                            }

                            drafts.push(OverlayDraft {
                                text,
                                placement: OverlayPlacement::TagX {
                                    x,
                                    color: tag.color.unwrap_or(annotation_stroke),
                                },
                            });
                        }
                    }

                    for tag in &overlays.tags_y {
                        if !tag.y.is_finite() {
                            continue;
                        }
                        let Some(transform) = transform_for_y_axis(tag.axis) else {
                            continue;
                        };
                        let Some(y_px) = transform.data_y_to_px(tag.y) else {
                            continue;
                        };
                        let y = Px((layout.plot.origin.y.0 + y_px.0).round());

                        let (span, labels) = match tag.axis {
                            YAxis::Right if self.show_y2_axis => (
                                view_bounds_y2
                                    .map(|b| (b.y_max - b.y_min).abs())
                                    .unwrap_or(y_span),
                                &self.y2_axis_labels,
                            ),
                            YAxis::Right2 if self.show_y3_axis => (
                                view_bounds_y3
                                    .map(|b| (b.y_max - b.y_min).abs())
                                    .unwrap_or(y_span),
                                &self.y3_axis_labels,
                            ),
                            YAxis::Right3 if self.show_y4_axis => (
                                view_bounds_y4
                                    .map(|b| (b.y_max - b.y_min).abs())
                                    .unwrap_or(y_span),
                                &self.y4_axis_labels,
                            ),
                            _ => (y_span, &self.tooltip_y_labels),
                        };

                        let value = tag
                            .show_value
                            .then(|| labels.format(tag.y, span))
                            .unwrap_or_default();
                        let text = match (&tag.label, tag.show_value) {
                            (Some(label), true) => format!("{label}: {value}"),
                            (Some(label), false) => label.clone(),
                            (None, true) => value,
                            (None, false) => String::new(),
                        };
                        if text.is_empty() {
                            continue;
                        }

                        drafts.push(OverlayDraft {
                            text,
                            placement: OverlayPlacement::TagY {
                                y,
                                right: tag.axis != YAxis::Left,
                                color: tag.color.unwrap_or(annotation_stroke),
                            },
                        });
                    }

                    for t in &overlays.text {
                        if !t.x.is_finite() || !t.y.is_finite() {
                            continue;
                        }
                        let Some(transform) = transform_for_y_axis(t.axis) else {
                            continue;
                        };
                        let Some(px_x) = transform.data_x_to_px(t.x) else {
                            continue;
                        };
                        let Some(px_y) = transform.data_y_to_px(t.y) else {
                            continue;
                        };
                        let origin = Point::new(
                            Px((layout.plot.origin.x.0 + px_x.0 + t.offset.x.0).round()),
                            Px((layout.plot.origin.y.0 + px_y.0 + t.offset.y.0).round()),
                        );

                        let padding = if t.background.is_some() && t.padding.0 <= 0.0 {
                            annotation_padding
                        } else {
                            t.padding
                        };
                        let corner_radius = if t.background.is_some() && t.corner_radius.0 <= 0.0 {
                            annotation_radius
                        } else {
                            t.corner_radius
                        };

                        drafts.push(OverlayDraft {
                            text: t.text.clone(),
                            placement: OverlayPlacement::Text {
                                origin,
                                color: t.color.unwrap_or(annotation_text),
                                background: t.background,
                                border: t.border,
                                padding,
                                corner_radius,
                            },
                        });
                    }

                    let overlay_font_size = Px((theme_font_size.0 * 0.90).max(10.0));
                    let overlay_style = TextStyle {
                        font: FontId::default(),
                        size: overlay_font_size,
                        weight: FontWeight::NORMAL,
                        slant: TextSlant::Normal,
                        line_height: None,
                        letter_spacing_em: None,
                    };
                    let overlay_constraints = TextConstraints {
                        max_width: None,
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        scale_factor: cx.scale_factor,
                    };

                    let mut overlay_key = 0u64;
                    overlay_key = Self::hash_u64(overlay_key, theme_revision);
                    overlay_key = Self::hash_u64(overlay_key, font_stack_key);
                    overlay_key = Self::hash_u64(overlay_key, u64::from(cx.scale_factor.to_bits()));
                    overlay_key = Self::hash_u64(overlay_key, Self::text_style_key(&overlay_style));
                    overlay_key = Self::hash_u64(overlay_key, drafts.len() as u64);
                    for d in &drafts {
                        for b in d.text.as_bytes() {
                            overlay_key = Self::hash_u64(overlay_key, u64::from(*b));
                        }
                    }

                    self.overlays_text_key = Some(overlay_key);
                    self.overlays_text.clear();
                    self.overlays_text.reserve(drafts.len());
                    for d in &drafts {
                        self.overlays_text.push(self.overlay_text_cache.prepare(
                            cx.services,
                            &d.text,
                            &overlay_style,
                            overlay_constraints,
                        ));
                    }

                    for (i, d) in drafts.iter().enumerate() {
                        let Some(text) = self.overlays_text.get(i).copied() else {
                            continue;
                        };

                        match &d.placement {
                            OverlayPlacement::TagX { x, color } => {
                                let pad = annotation_padding;
                                let w = Px(text.metrics.size.width.0 + pad.0 * 2.0);
                                let h = Px(text.metrics.size.height.0 + pad.0 * 2.0);

                                let left = (x.0 - w.0 * 0.5).clamp(
                                    layout.plot.origin.x.0,
                                    layout.plot.origin.x.0 + layout.plot.size.width.0 - w.0,
                                );
                                let top = (layout.plot.origin.y.0 + layout.plot.size.height.0
                                    - h.0
                                    - margin.0)
                                    .clamp(
                                        layout.plot.origin.y.0,
                                        layout.plot.origin.y.0 + layout.plot.size.height.0 - h.0,
                                    );
                                let rect =
                                    Rect::new(Point::new(Px(left), Px(top)), Size::new(w, h));

                                cx.scene.push(SceneOp::Quad {
                                    order: DrawOrder(3),
                                    rect,
                                    background: fret_core::Paint::Solid(annotation_background),

                                    border: fret_core::Edges::all(Px(1.0)),
                                    border_paint: fret_core::Paint::Solid(annotation_border),

                                    corner_radii: fret_core::Corners::all(annotation_radius),
                                });

                                let origin = Point::new(
                                    Px(rect.origin.x.0 + pad.0),
                                    Px(rect.origin.y.0 + pad.0 + text.metrics.baseline.0),
                                );
                                cx.scene.push(SceneOp::Text {
                                    order: DrawOrder(3),
                                    origin,
                                    text: text.blob,
                                    color: annotation_text,
                                });

                                let marker_w = Px(2.0);
                                let marker_h =
                                    Px(marker_len.0.min(layout.plot.size.height.0.max(0.0)));
                                let marker_left = (x.0 - marker_w.0 * 0.5).clamp(
                                    layout.plot.origin.x.0,
                                    layout.plot.origin.x.0 + layout.plot.size.width.0 - marker_w.0,
                                );
                                let marker_top = (layout.plot.origin.y.0
                                    + layout.plot.size.height.0
                                    - marker_h.0)
                                    .max(layout.plot.origin.y.0);
                                cx.scene.push(SceneOp::Quad {
                                    order: DrawOrder(3),
                                    rect: Rect::new(
                                        Point::new(Px(marker_left), Px(marker_top)),
                                        Size::new(marker_w, marker_h),
                                    ),
                                    background: fret_core::Paint::Solid(*color),

                                    border: fret_core::Edges::all(Px(0.0)),
                                    border_paint: fret_core::Paint::TRANSPARENT,

                                    corner_radii: fret_core::Corners::all(Px(0.0)),
                                });
                            }
                            OverlayPlacement::TagY { y, right, color } => {
                                let pad = annotation_padding;
                                let w = Px(text.metrics.size.width.0 + pad.0 * 2.0);
                                let h = Px(text.metrics.size.height.0 + pad.0 * 2.0);

                                let left = if *right {
                                    (layout.plot.origin.x.0 + layout.plot.size.width.0
                                        - w.0
                                        - margin.0)
                                        .max(layout.plot.origin.x.0)
                                } else {
                                    layout.plot.origin.x.0 + margin.0
                                };
                                let top = (y.0 - h.0 * 0.5).clamp(
                                    layout.plot.origin.y.0,
                                    layout.plot.origin.y.0 + layout.plot.size.height.0 - h.0,
                                );
                                let rect =
                                    Rect::new(Point::new(Px(left), Px(top)), Size::new(w, h));

                                cx.scene.push(SceneOp::Quad {
                                    order: DrawOrder(3),
                                    rect,
                                    background: fret_core::Paint::Solid(annotation_background),

                                    border: fret_core::Edges::all(Px(1.0)),
                                    border_paint: fret_core::Paint::Solid(annotation_border),

                                    corner_radii: fret_core::Corners::all(annotation_radius),
                                });

                                let origin = Point::new(
                                    Px(rect.origin.x.0 + pad.0),
                                    Px(rect.origin.y.0 + pad.0 + text.metrics.baseline.0),
                                );
                                cx.scene.push(SceneOp::Text {
                                    order: DrawOrder(3),
                                    origin,
                                    text: text.blob,
                                    color: annotation_text,
                                });

                                let marker_h = Px(2.0);
                                let marker_w =
                                    Px(marker_len.0.min(layout.plot.size.width.0.max(0.0)));
                                let marker_top = (y.0 - marker_h.0 * 0.5).clamp(
                                    layout.plot.origin.y.0,
                                    layout.plot.origin.y.0 + layout.plot.size.height.0 - marker_h.0,
                                );
                                let marker_left = if *right {
                                    (layout.plot.origin.x.0 + layout.plot.size.width.0 - marker_w.0)
                                        .max(layout.plot.origin.x.0)
                                } else {
                                    layout.plot.origin.x.0
                                };
                                cx.scene.push(SceneOp::Quad {
                                    order: DrawOrder(3),
                                    rect: Rect::new(
                                        Point::new(Px(marker_left), Px(marker_top)),
                                        Size::new(marker_w, marker_h),
                                    ),
                                    background: fret_core::Paint::Solid(*color),

                                    border: fret_core::Edges::all(Px(0.0)),
                                    border_paint: fret_core::Paint::TRANSPARENT,

                                    corner_radii: fret_core::Corners::all(Px(0.0)),
                                });
                            }
                            OverlayPlacement::Text {
                                origin,
                                color,
                                background,
                                border,
                                padding,
                                corner_radius,
                            } => {
                                let pad = *padding;
                                let w = Px(text.metrics.size.width.0 + pad.0 * 2.0);
                                let h = Px(text.metrics.size.height.0 + pad.0 * 2.0);

                                let left = origin.x.0.clamp(
                                    layout.plot.origin.x.0,
                                    layout.plot.origin.x.0 + layout.plot.size.width.0 - w.0,
                                );
                                let top = origin.y.0.clamp(
                                    layout.plot.origin.y.0,
                                    layout.plot.origin.y.0 + layout.plot.size.height.0 - h.0,
                                );
                                let rect =
                                    Rect::new(Point::new(Px(left), Px(top)), Size::new(w, h));

                                if let Some(bg) = *background {
                                    cx.scene.push(SceneOp::Quad {
                                        order: DrawOrder(3),
                                        rect,
                                        background: fret_core::Paint::Solid(bg),

                                        border: fret_core::Edges::all(Px(1.0)),
                                        border_paint: fret_core::Paint::Solid(
                                            border.unwrap_or(annotation_border),
                                        ),

                                        corner_radii: fret_core::Corners::all(*corner_radius),
                                    });
                                }

                                let text_origin = Point::new(
                                    Px(rect.origin.x.0 + pad.0),
                                    Px(rect.origin.y.0 + pad.0 + text.metrics.baseline.0),
                                );
                                cx.scene.push(SceneOp::Text {
                                    order: DrawOrder(3),
                                    origin: text_origin,
                                    text: text.blob,
                                    color: *color,
                                });
                            }
                        }
                    }
                }
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
                    background: fret_core::Paint::Solid(crosshair_color),

                    border: fret_core::Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT,

                    corner_radii: fret_core::Corners::all(Px(0.0)),
                });

                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(3),
                    rect: Rect::new(
                        Point::new(layout.plot.origin.x, y),
                        Size::new(layout.plot.size.width, Px(1.0)),
                    ),
                    background: fret_core::Paint::Solid(crosshair_color),

                    border: fret_core::Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT,

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
                        background: fret_core::Paint::Solid(linked_color),

                        border: fret_core::Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,

                        corner_radii: fret_core::Corners::all(Px(0.0)),
                    });
                }
            }

            if let Some(hover) = self.hover {
                let hx = Px((layout.plot.origin.x.0 + hover.plot_px.x.0).round());
                let hy = Px((layout.plot.origin.y.0 + hover.plot_px.y.0).round());

                let hover_color = self
                    .model
                    .read(cx.app, |_app, m| {
                        let meta = L::series_meta(m);
                        let series_count = meta.len().max(1);
                        let mut series_index = 0usize;
                        let mut override_color = None;
                        for (i, s) in meta.iter().enumerate() {
                            if s.id == hover.series_id {
                                series_index = i;
                                override_color = s.stroke_color;
                                break;
                            }
                        }
                        resolve_series_color(series_index, self.style, series_count, override_color)
                    })
                    .unwrap_or(crosshair_color);

                let outer_size = Px(10.0);
                let outer_origin =
                    Point::new(Px(hx.0 - outer_size.0 * 0.5), Px(hy.0 - outer_size.0 * 0.5));
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect: Rect::new(outer_origin, Size::new(outer_size, outer_size)),
                    background: fret_core::Paint::TRANSPARENT,

                    border: fret_core::Edges::all(Px(2.0)),
                    border_paint: fret_core::Paint::Solid(hover_color),

                    corner_radii: fret_core::Corners::all(Px(outer_size.0 * 0.5)),
                });

                let dot_size = Px(6.0);
                let dot_origin =
                    Point::new(Px(hx.0 - dot_size.0 * 0.5), Px(hy.0 - dot_size.0 * 0.5));
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(5),
                    rect: Rect::new(dot_origin, Size::new(dot_size, dot_size)),
                    background: fret_core::Paint::Solid(hover_color),

                    border: fret_core::Edges::all(Px(1.0)),
                    border_paint: fret_core::Paint::Solid(tooltip_border),

                    corner_radii: fret_core::Corners::all(Px(dot_size.0 * 0.5)),
                });
            }

            if self.hover.is_none()
                && self.style.series_tooltip == SeriesTooltipMode::NearestAtCursor
                && let Some(cursor_px) = self.cursor_px
            {
                let local_viewport = Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size);
                let cursor_data = PlotTransform {
                    viewport: local_viewport,
                    data: view_bounds,
                    x_scale: self.x_scale,
                    y_scale: self.y_scale,
                }
                .px_to_data(cursor_px);

                if cursor_data.x.is_finite() && cursor_data.y.is_finite() {
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
                    let readout_rows = self
                        .model
                        .read(cx.app, |_app, m| L::cursor_readout(m, readout_args))
                        .unwrap_or_default();

                    let pinned = state.pinned_series.filter(|id| !hidden.contains(id));
                    let legend_hover = self.legend_hover.filter(|id| !hidden.contains(id));

                    let mut best: Option<(f64, PlotCursorReadoutRow)> = None;
                    for row in readout_rows {
                        if let Some(pinned) = pinned {
                            if row.series_id != pinned {
                                continue;
                            }
                        } else if let Some(hovered) = legend_hover {
                            if row.series_id != hovered {
                                continue;
                            }
                        }

                        let Some(y) = row.y.filter(|y| y.is_finite()) else {
                            continue;
                        };
                        let dist = (cursor_data.y - y).abs();
                        if !dist.is_finite() {
                            continue;
                        }

                        if best.as_ref().is_none_or(|(d, _)| dist < *d) {
                            best = Some((dist, row));
                        }
                    }

                    if let Some((_dist, row)) = best
                        && let Some(y) = row.y.filter(|y| y.is_finite())
                    {
                        let local_viewport =
                            Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size);
                        let transform = match row.y_axis {
                            YAxis::Right if self.show_y2_axis => {
                                view_bounds_y2.map(|b| PlotTransform {
                                    viewport: local_viewport,
                                    data: b,
                                    x_scale: self.x_scale,
                                    y_scale: self.y2_scale,
                                })
                            }
                            YAxis::Right2 if self.show_y3_axis => {
                                view_bounds_y3.map(|b| PlotTransform {
                                    viewport: local_viewport,
                                    data: b,
                                    x_scale: self.x_scale,
                                    y_scale: self.y3_scale,
                                })
                            }
                            YAxis::Right3 if self.show_y4_axis => {
                                view_bounds_y4.map(|b| PlotTransform {
                                    viewport: local_viewport,
                                    data: b,
                                    x_scale: self.x_scale,
                                    y_scale: self.y4_scale,
                                })
                            }
                            _ => Some(PlotTransform {
                                viewport: local_viewport,
                                data: view_bounds,
                                x_scale: self.x_scale,
                                y_scale: self.y_scale,
                            }),
                        };

                        if let Some(transform) = transform {
                            let p = transform.data_to_px(DataPoint {
                                x: cursor_data.x,
                                y,
                            });
                            if p.x.0.is_finite()
                                && p.y.0.is_finite()
                                && (0.0..=layout.plot.size.width.0).contains(&p.x.0)
                                && (0.0..=layout.plot.size.height.0).contains(&p.y.0)
                            {
                                let series_color = self
                                    .model
                                    .read(cx.app, |_app, m| {
                                        let meta = L::series_meta(m);
                                        let series_count = meta.len().max(1);
                                        let mut series_index = 0usize;
                                        let mut override_color = None;
                                        for (i, s) in meta.iter().enumerate() {
                                            if s.id == row.series_id {
                                                series_index = i;
                                                override_color = s.stroke_color;
                                                break;
                                            }
                                        }
                                        resolve_series_color(
                                            series_index,
                                            self.style,
                                            series_count,
                                            override_color,
                                        )
                                    })
                                    .unwrap_or(crosshair_color);

                                let hx = Px((layout.plot.origin.x.0 + cursor_px.x.0).round());
                                let hy = Px((layout.plot.origin.y.0 + p.y.0).round());

                                let outer_size = Px(10.0);
                                let outer_origin = Point::new(
                                    Px(hx.0 - outer_size.0 * 0.5),
                                    Px(hy.0 - outer_size.0 * 0.5),
                                );
                                cx.scene.push(SceneOp::Quad {
                                    order: DrawOrder(4),
                                    rect: Rect::new(
                                        outer_origin,
                                        Size::new(outer_size, outer_size),
                                    ),
                                    background: fret_core::Paint::TRANSPARENT,

                                    border: fret_core::Edges::all(Px(2.0)),
                                    border_paint: fret_core::Paint::Solid(series_color),

                                    corner_radii: fret_core::Corners::all(Px(outer_size.0 * 0.5)),
                                });

                                let dot_size = Px(6.0);
                                let dot_origin = Point::new(
                                    Px(hx.0 - dot_size.0 * 0.5),
                                    Px(hy.0 - dot_size.0 * 0.5),
                                );
                                cx.scene.push(SceneOp::Quad {
                                    order: DrawOrder(5),
                                    rect: Rect::new(dot_origin, Size::new(dot_size, dot_size)),
                                    background: fret_core::Paint::Solid(series_color),

                                    border: fret_core::Edges::all(Px(1.0)),
                                    border_paint: fret_core::Paint::Solid(tooltip_border),

                                    corner_radii: fret_core::Corners::all(Px(dot_size.0 * 0.5)),
                                });
                            }
                        }
                    }
                }
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
                        background: fret_core::Paint::Solid(selection_fill),

                        border: fret_core::Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(selection_border),

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
                        background: fret_core::Paint::Solid(selection_fill),

                        border: fret_core::Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(selection_border),

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
                        background: fret_core::Paint::Solid(selection_fill),

                        border: fret_core::Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(selection_border),

                        corner_radii: fret_core::Corners::all(Px(0.0)),
                    });
                }
            }

            #[cfg(debug_assertions)]
            if self.debug_overlay {
                // Plot debug: sample a few points from the first series and draw them as quads.
                // This helps distinguish "path generation is wrong" from "path rendering is broken".
                //
                // We use a best-effort downcast to known model types without extending the
                // `PlotLayer` trait for debugging.
                let mut debug_sample_points: u32 = 0;
                let mut debug_model_kind: u32 = 0; // 0=unknown, 1=line, 2=area
                if let Ok(model_any) = self.model.read(cx.app, |_app, m| m.clone()) {
                    use std::any::Any;

                    let mut points: Vec<DataPoint> = Vec::new();
                    if let Some(model) = (&model_any as &dyn Any).downcast_ref::<LinePlotModel>() {
                        debug_model_kind = 1;
                        if let Some(series) = model.series.first() {
                            if let Some(slice) = series.data.as_slice() {
                                let stride =
                                    ((slice.len() + 15) / 16).max(1).min(slice.len().max(1));
                                for p in slice.iter().copied().step_by(stride).take(24) {
                                    points.push(p);
                                }
                            } else {
                                for i in (0..series.data.len()).step_by(64).take(24) {
                                    if let Some(p) = series.data.get(i) {
                                        points.push(p);
                                    }
                                }
                            }
                        }
                    } else if let Some(model) =
                        (&model_any as &dyn Any).downcast_ref::<AreaPlotModel>()
                    {
                        debug_model_kind = 2;
                        if let Some(series) = model.series.first() {
                            if let Some(slice) = series.data.as_slice() {
                                let stride =
                                    ((slice.len() + 15) / 16).max(1).min(slice.len().max(1));
                                for p in slice.iter().copied().step_by(stride).take(24) {
                                    points.push(p);
                                }
                            } else {
                                for i in (0..series.data.len()).step_by(64).take(24) {
                                    if let Some(p) = series.data.get(i) {
                                        points.push(p);
                                    }
                                }
                            }
                        }
                    }

                    if !points.is_empty() {
                        debug_sample_points = points.len().min(u32::MAX as usize) as u32;
                        let local_viewport =
                            Rect::new(Point::new(Px(0.0), Px(0.0)), layout.plot.size);
                        let transform = PlotTransform {
                            viewport: local_viewport,
                            data: view_bounds,
                            x_scale: self.x_scale,
                            y_scale: self.y_scale,
                        };

                        let r = Px(2.5);
                        for p in points {
                            if !p.x.is_finite() || !p.y.is_finite() {
                                continue;
                            }
                            let local = transform.data_to_px(p);
                            if !local.x.0.is_finite() || !local.y.0.is_finite() {
                                continue;
                            }
                            let x = Px((layout.plot.origin.x.0 + local.x.0).round());
                            let y = Px((layout.plot.origin.y.0 + local.y.0).round());
                            cx.scene.push(SceneOp::Quad {
                                order: DrawOrder(5),
                                rect: Rect::new(
                                    Point::new(Px(x.0 - r.0), Px(y.0 - r.0)),
                                    Size::new(Px(r.0 * 2.0), Px(r.0 * 2.0)),
                                ),
                                background: fret_core::Paint::Solid(Color {
                                    r: 1.0,
                                    g: 1.0,
                                    b: 1.0,
                                    a: 0.9,
                                }),
                                border: fret_core::Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT,

                                corner_radii: fret_core::Corners::all(r),
                            });
                        }
                    }
                }

                let text_style = TextStyle {
                    font: FontId::default(),
                    size: Px(11.0),
                    weight: FontWeight::NORMAL,
                    slant: TextSlant::Normal,
                    line_height: None,
                    letter_spacing_em: None,
                };
                let constraints = TextConstraints {
                    max_width: None,
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor: cx.scale_factor,
                };

                let label = format!(
                    "plot debug: model={} samples={} paths_prepared={} paths_pushed={} view=[{:.3},{:.3}]x[{:.3},{:.3}]",
                    debug_model_kind,
                    debug_sample_points,
                    debug_paths_prepared,
                    debug_paths_pushed,
                    view_bounds.x_min,
                    view_bounds.x_max,
                    view_bounds.y_min,
                    view_bounds.y_max
                );
                self.debug_overlay_text = Some(self.debug_text_cache.prepare(
                    cx.services,
                    &label,
                    &text_style,
                    constraints,
                ));

                if let Some(t) = self.debug_overlay_text {
                    let pad = 6.0f32;
                    let w = (t.metrics.size.width.0 + pad * 2.0).max(1.0);
                    let h = (t.metrics.size.height.0 + pad * 2.0).max(1.0);
                    let origin = Point::new(
                        Px(layout.plot.origin.x.0 + 8.0),
                        Px(layout.plot.origin.y.0 + 8.0),
                    );

                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(99),
                        rect: Rect::new(origin, Size::new(Px(w), Px(h))),
                        background: fret_core::Paint::Solid(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.65,
                        }),
                        border: fret_core::Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 0.18,
                        }),
                        corner_radii: fret_core::Corners::all(Px(4.0)),
                    });

                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(100),
                        origin: Point::new(
                            Px(origin.x.0 + pad),
                            Px(origin.y.0 + pad + t.metrics.baseline.0),
                        ),
                        text: t.blob,
                        color: Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        },
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
                background: fret_core::Paint::Solid(tooltip_background),

                border: fret_core::Edges::all(Px(1.0)),
                border_paint: fret_core::Paint::Solid(tooltip_border),

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
                        background: fret_core::Paint::Solid(highlight),

                        border: fret_core::Edges::all(Px(0.0)),
                        border_paint: fret_core::Paint::TRANSPARENT,

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
                    background: fret_core::Paint::Solid(swatch_color),

                    border: fret_core::Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT,

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
                background: fret_core::Paint::Solid(axis_color),

                border: fret_core::Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT,

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
                    background: fret_core::Paint::Solid(axis_color),

                    border: fret_core::Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT,

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
                background: fret_core::Paint::Solid(axis_color),

                border: fret_core::Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT,

                corner_radii: fret_core::Corners::all(Px(0.0)),
            });
        }

        // Axis labels: tick density adapts to viewport + label spacing.
        let x_ticks = &self.axis_ticks_x;
        let y_ticks = &self.axis_ticks_y;
        let y2_ticks = &self.axis_ticks_y2;
        let y3_ticks = &self.axis_ticks_y3;
        let y4_ticks = &self.axis_ticks_y4;

        let transform_y1: Option<PreparedPlotTransform> = PlotTransform {
            viewport: layout.plot,
            data: view_bounds,
            x_scale: self.x_scale,
            y_scale: self.y_scale,
        }
        .prepare();
        let transform_y2: Option<PreparedPlotTransform> = view_bounds_y2.and_then(|b| {
            PlotTransform {
                viewport: layout.plot,
                data: b,
                x_scale: self.x_scale,
                y_scale: self.y2_scale,
            }
            .prepare()
        });
        let transform_y3: Option<PreparedPlotTransform> = view_bounds_y3.and_then(|b| {
            PlotTransform {
                viewport: layout.plot,
                data: b,
                x_scale: self.x_scale,
                y_scale: self.y3_scale,
            }
            .prepare()
        });
        let transform_y4: Option<PreparedPlotTransform> = view_bounds_y4.and_then(|b| {
            PlotTransform {
                viewport: layout.plot,
                data: b,
                x_scale: self.x_scale,
                y_scale: self.y4_scale,
            }
            .prepare()
        });

        if let Some(transform_y1) = transform_y1 {
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

        // Axis lock indicators (P0: lightweight discoverability).
        let lock_indicator = |lock: PlotAxisLock| match (lock.pan, lock.zoom) {
            (false, false) => None,
            (true, true) => Some("L"),
            (true, false) => Some("P"),
            (false, true) => Some("Z"),
        };

        let font_size = cx
            .theme()
            .metric_by_key("font.size")
            .unwrap_or(cx.theme().metrics.font_size);
        let indicator_style = TextStyle {
            font: FontId::default(),
            size: Px((font_size.0 * 0.85).max(9.0)),
            weight: FontWeight::BOLD,
            slant: TextSlant::Normal,
            line_height: None,
            letter_spacing_em: None,
        };
        let indicator_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        self.axis_lock_indicator_x = lock_indicator(self.lock_x).map(|token| {
            self.indicator_text_cache.prepare(
                cx.services,
                token,
                &indicator_style,
                indicator_constraints,
            )
        });
        self.axis_lock_indicator_y = lock_indicator(self.lock_y).map(|token| {
            self.indicator_text_cache.prepare(
                cx.services,
                token,
                &indicator_style,
                indicator_constraints,
            )
        });
        self.axis_lock_indicator_y2 = self
            .show_y2_axis
            .then_some(self.lock_y2)
            .and_then(lock_indicator)
            .map(|token| {
                self.indicator_text_cache.prepare(
                    cx.services,
                    token,
                    &indicator_style,
                    indicator_constraints,
                )
            });
        self.axis_lock_indicator_y3 = self
            .show_y3_axis
            .then_some(self.lock_y3)
            .and_then(lock_indicator)
            .map(|token| {
                self.indicator_text_cache.prepare(
                    cx.services,
                    token,
                    &indicator_style,
                    indicator_constraints,
                )
            });
        self.axis_lock_indicator_y4 = self
            .show_y4_axis
            .then_some(self.lock_y4)
            .and_then(lock_indicator)
            .map(|token| {
                self.indicator_text_cache.prepare(
                    cx.services,
                    token,
                    &indicator_style,
                    indicator_constraints,
                )
            });

        let indicator_margin = Px(3.0);
        if let Some(t) = self.axis_lock_indicator_x {
            let rect = layout.x_axis;
            if rect.size.width.0 > 0.0 && rect.size.height.0 > 0.0 {
                let top = rect.origin.y.0 + rect.size.height.0
                    - indicator_margin.0
                    - t.metrics.size.height.0;
                let origin = Point::new(
                    Px(rect.origin.x.0 + indicator_margin.0),
                    Px(top.max(rect.origin.y.0) + t.metrics.baseline.0),
                );
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(11),
                    origin,
                    text: t.blob,
                    color: label_color,
                });
            }
        }

        if let Some(t) = self.axis_lock_indicator_y {
            let rect = layout.y_axis_left;
            if rect.size.width.0 > 0.0 && rect.size.height.0 > 0.0 {
                let top = rect.origin.y.0 + indicator_margin.0;
                let origin = Point::new(
                    Px(rect.origin.x.0 + indicator_margin.0),
                    Px(top + t.metrics.baseline.0),
                );
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(11),
                    origin,
                    text: t.blob,
                    color: label_color,
                });
            }
        }

        if self.show_y2_axis {
            if let Some(t) = self.axis_lock_indicator_y2 {
                let rect = layout.y_axis_right;
                if rect.size.width.0 > 0.0 && rect.size.height.0 > 0.0 {
                    let top = rect.origin.y.0 + indicator_margin.0;
                    let origin_x = rect.origin.x.0 + rect.size.width.0
                        - indicator_margin.0
                        - t.metrics.size.width.0;
                    let origin = Point::new(
                        Px(origin_x.max(rect.origin.x.0)),
                        Px(top + t.metrics.baseline.0),
                    );
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(11),
                        origin,
                        text: t.blob,
                        color: label_color,
                    });
                }
            }
        }

        if self.show_y3_axis {
            if let Some(t) = self.axis_lock_indicator_y3 {
                let rect = layout.y_axis_right2;
                if rect.size.width.0 > 0.0 && rect.size.height.0 > 0.0 {
                    let top = rect.origin.y.0 + indicator_margin.0;
                    let origin_x = rect.origin.x.0 + rect.size.width.0
                        - indicator_margin.0
                        - t.metrics.size.width.0;
                    let origin = Point::new(
                        Px(origin_x.max(rect.origin.x.0)),
                        Px(top + t.metrics.baseline.0),
                    );
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(11),
                        origin,
                        text: t.blob,
                        color: label_color,
                    });
                }
            }
        }

        if self.show_y4_axis {
            if let Some(t) = self.axis_lock_indicator_y4 {
                let rect = layout.y_axis_right3;
                if rect.size.width.0 > 0.0 && rect.size.height.0 > 0.0 {
                    let top = rect.origin.y.0 + indicator_margin.0;
                    let origin_x = rect.origin.x.0 + rect.size.width.0
                        - indicator_margin.0
                        - t.metrics.size.width.0;
                    let origin = Point::new(
                        Px(origin_x.max(rect.origin.x.0)),
                        Px(top + t.metrics.baseline.0),
                    );
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(11),
                        origin,
                        text: t.blob,
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
                slant: TextSlant::Normal,
                line_height: None,
                letter_spacing_em: None,
            };
            let constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            };

            self.linked_cursor_readout_text =
                Some(
                    self.readout_text_cache
                        .prepare(cx.services, &text, &style, constraints),
                );

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
                    background: fret_core::Paint::Solid(tooltip_background),

                    border: fret_core::Edges::all(Px(1.0)),
                    border_paint: fret_core::Paint::Solid(tooltip_border),

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
                    slant: TextSlant::Normal,
                    line_height: None,
                    letter_spacing_em: None,
                };
                let constraints = TextConstraints {
                    max_width: None,
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor: cx.scale_factor,
                };

                self.mouse_readout_text =
                    Some(
                        self.readout_text_cache
                            .prepare(cx.services, &text, &style, constraints),
                    );

                if let Some(tt) = self.mouse_readout_text {
                    let pad = Px(6.0);
                    let margin = Px(6.0);
                    let w = Px(tt.metrics.size.width.0 + pad.0 * 2.0);
                    let h = Px(tt.metrics.size.height.0 + pad.0 * 2.0);
                    let Some(rect) = overlay_rect_in_plot(
                        layout.plot,
                        Size::new(w, h),
                        self.style.mouse_readout_anchor,
                        margin,
                    ) else {
                        return;
                    };
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(12),
                        rect,
                        background: fret_core::Paint::Solid(tooltip_background),

                        border: fret_core::Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(tooltip_border),

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
                        color: label_color,
                    });
                }
            }
        }

        let tooltip = selection_tooltip
            .map(|(anchor, text)| (anchor, text, None))
            .or_else(|| {
                let format_y = |y: f64, axis: YAxis| {
                    if axis == YAxis::Right && self.show_y2_axis {
                        let span = view_bounds_y2
                            .map(|b| (b.y_max - b.y_min).abs())
                            .unwrap_or(y_span);
                        self.y2_axis_labels.format(y, span)
                    } else if axis == YAxis::Right2 && self.show_y3_axis {
                        let span = view_bounds_y3
                            .map(|b| (b.y_max - b.y_min).abs())
                            .unwrap_or(y_span);
                        self.y3_axis_labels.format(y, span)
                    } else if axis == YAxis::Right3 && self.show_y4_axis {
                        let span = view_bounds_y4
                            .map(|b| (b.y_max - b.y_min).abs())
                            .unwrap_or(y_span);
                        self.y4_axis_labels.format(y, span)
                    } else {
                        self.tooltip_y_labels.format(y, y_span)
                    }
                };

                self.hover
                    .and_then(|hover| {
                        if self.style.series_tooltip == SeriesTooltipMode::NearestAtCursor
                            && state.pinned_series.is_some()
                        {
                            return None;
                        }

                        let (series_count, series_label, y_axis) = self
                            .model
                            .read(cx.app, |_app, m| {
                                let series_count = L::series_meta(m).len();
                                let series_label = L::series_label(m, hover.series_id);
                                let y_axis = L::series_y_axis(m, hover.series_id);
                                (series_count, series_label, y_axis)
                            })
                            .unwrap_or((0, None, YAxis::Left));

                        let series_color = self
                            .model
                            .read(cx.app, |_app, m| {
                                let meta = L::series_meta(m);
                                let series_count = meta.len().max(1);
                                let mut series_index = 0usize;
                                let mut override_color = None;
                                for (i, s) in meta.iter().enumerate() {
                                    if s.id == hover.series_id {
                                        series_index = i;
                                        override_color = s.stroke_color;
                                        break;
                                    }
                                }
                                resolve_series_color(
                                    series_index,
                                    self.style,
                                    series_count,
                                    override_color,
                                )
                            })
                            .unwrap_or(crosshair_color);

                        let x_text = self.tooltip_x_labels.format(hover.data.x, x_span);
                        let y_text = format_y(hover.data.y, y_axis);
                        let line = format!("x={x_text}  y={y_text}");
                        let header = if series_count > 1 {
                            series_label
                                .as_deref()
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("s={}", hover.series_id.0))
                        } else {
                            String::new()
                        };
                        let header = if series_count == 0
                            && let Some(label) = series_label.as_deref()
                        {
                            label.to_string()
                        } else {
                            header
                        };

                        let header = if let Some(v) = hover.value
                            && v.is_finite()
                        {
                            let v_text = if v.abs() < 10_000.0 {
                                format!("{v:.4}")
                            } else {
                                format!("{v:.4e}")
                            };
                            if header.is_empty() {
                                format!("value={v_text}")
                            } else {
                                format!("{header}  value={v_text}")
                            }
                        } else {
                            header
                        };

                        let text = if header.is_empty() {
                            line
                        } else {
                            format!("{header}\n{line}")
                        };

                        Some((hover.plot_px, text, Some(series_color)))
                    })
                    .or_else(|| {
                        if self.style.series_tooltip != SeriesTooltipMode::NearestAtCursor {
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
                        let readout_rows = self
                            .model
                            .read(cx.app, |_app, m| L::cursor_readout(m, readout_args))
                            .unwrap_or_default();

                        let pinned = state.pinned_series.filter(|id| !hidden.contains(id));
                        let legend_hover = self.legend_hover.filter(|id| !hidden.contains(id));

                        let mut best: Option<(f64, PlotCursorReadoutRow)> = None;
                        for row in readout_rows {
                            if let Some(pinned) = pinned {
                                if row.series_id != pinned {
                                    continue;
                                }
                            } else if let Some(hovered) = legend_hover {
                                if row.series_id != hovered {
                                    continue;
                                }
                            }

                            let Some(y) = row.y.filter(|y| y.is_finite()) else {
                                continue;
                            };
                            let dist = (cursor_data.y - y).abs();
                            if !dist.is_finite() {
                                continue;
                            }

                            if best.as_ref().is_none_or(|(d, _)| dist < *d) {
                                best = Some((dist, row));
                            }
                        }

                        let (_dist, row) = best?;
                        let y = row.y?;
                        let x_text = self.tooltip_x_labels.format(cursor_data.x, x_span);
                        let y_text = format_y(y, row.y_axis);
                        let header = if !row.label.is_empty() {
                            row.label.to_string()
                        } else {
                            format!("s={}", row.series_id.0)
                        };
                        let line = format!("x={x_text}  y={y_text}");
                        let text = format!("{header}\n{line}");

                        let series_color = self
                            .model
                            .read(cx.app, |_app, m| {
                                let meta = L::series_meta(m);
                                let series_count = meta.len().max(1);
                                let mut series_index = 0usize;
                                let mut override_color = None;
                                for (i, s) in meta.iter().enumerate() {
                                    if s.id == row.series_id {
                                        series_index = i;
                                        override_color = s.stroke_color;
                                        break;
                                    }
                                }
                                resolve_series_color(
                                    series_index,
                                    self.style,
                                    series_count,
                                    override_color,
                                )
                            })
                            .unwrap_or(crosshair_color);

                        Some((cursor_px, text, Some(series_color)))
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
                                .map(|y| format_y(y, row.y_axis))
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

                        Some((cursor_px, text, None))
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
                                .map(|y| format_y(y, row.y_axis))
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

                        Some((anchor_local, text, None))
                    })
            });

        if let Some((anchor_local, text, swatch_color)) = tooltip {
            let font_size = theme_font_size;
            let style = TextStyle {
                font: FontId::default(),
                size: Px((font_size.0 * 0.90).max(10.0)),
                weight: FontWeight::NORMAL,
                slant: TextSlant::Normal,
                line_height: None,
                letter_spacing_em: None,
            };
            let constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            };

            self.tooltip_text =
                Some(
                    self.tooltip_text_cache
                        .prepare(cx.services, &text, &style, constraints),
                );

            if let Some(tt) = self.tooltip_text {
                let anchor = Point::new(
                    Px(layout.plot.origin.x.0 + anchor_local.x.0),
                    Px(layout.plot.origin.y.0 + anchor_local.y.0),
                );
                let pad = Px(6.0);
                let swatch_size = Px(10.0);
                let swatch_gap = Px(8.0);
                let swatch_extra = swatch_color
                    .map(|_| swatch_size.0 + swatch_gap.0)
                    .unwrap_or(0.0);
                let gap = Px(10.0);
                let w = Px(tt.metrics.size.width.0 + pad.0 * 2.0 + swatch_extra);
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
                    background: fret_core::Paint::Solid(tooltip_background),

                    border: fret_core::Edges::all(Px(1.0)),
                    border_paint: fret_core::Paint::Solid(tooltip_border),

                    corner_radii: fret_core::Corners::all(Px(6.0)),
                });

                if let Some(swatch_color) = swatch_color {
                    let swatch_top = rect.origin.y.0 + (rect.size.height.0 - swatch_size.0) * 0.5;
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(21),
                        rect: Rect::new(
                            Point::new(Px(rect.origin.x.0 + pad.0), Px(swatch_top)),
                            Size::new(swatch_size, swatch_size),
                        ),
                        background: fret_core::Paint::Solid(swatch_color),

                        border: fret_core::Edges::all(Px(1.0)),
                        border_paint: fret_core::Paint::Solid(tooltip_border),

                        corner_radii: fret_core::Corners::all(Px(2.0)),
                    });
                }

                let origin = Point::new(
                    Px(rect.origin.x.0 + pad.0 + swatch_extra),
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

        if let Some(window) = cx.window {
            let frame_id = cx.app.frame_id().0;
            let node = cx.node.data().as_ffi();
            let window = window.data().as_ffi();

            let axis_key = CanvasCacheKey {
                window,
                node,
                name: "fret-plot.canvas.text.axis",
            };
            let legend_key = CanvasCacheKey {
                window,
                node,
                name: "fret-plot.canvas.text.legend",
            };
            let indicator_key = CanvasCacheKey {
                window,
                node,
                name: "fret-plot.canvas.text.indicator",
            };
            let tooltip_key = CanvasCacheKey {
                window,
                node,
                name: "fret-plot.canvas.text.tooltip",
            };
            let readout_key = CanvasCacheKey {
                window,
                node,
                name: "fret-plot.canvas.text.readout",
            };
            let overlay_key = CanvasCacheKey {
                window,
                node,
                name: "fret-plot.canvas.text.overlay",
            };
            let heatmap_key = CanvasCacheKey {
                window,
                node,
                name: "fret-plot.canvas.text.heatmap",
            };
            let debug_key = CanvasCacheKey {
                window,
                node,
                name: "fret-plot.canvas.text.debug",
            };

            cx.app
                .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
                    registry.record_text_cache(
                        axis_key,
                        frame_id,
                        self.axis_text_cache.len(),
                        self.axis_text_cache.stats(),
                    );
                    registry.record_text_cache(
                        legend_key,
                        frame_id,
                        self.legend_text_cache.len(),
                        self.legend_text_cache.stats(),
                    );
                    registry.record_text_cache(
                        indicator_key,
                        frame_id,
                        self.indicator_text_cache.len(),
                        self.indicator_text_cache.stats(),
                    );
                    registry.record_text_cache(
                        tooltip_key,
                        frame_id,
                        self.tooltip_text_cache.len(),
                        self.tooltip_text_cache.stats(),
                    );
                    registry.record_text_cache(
                        readout_key,
                        frame_id,
                        self.readout_text_cache.len(),
                        self.readout_text_cache.stats(),
                    );
                    registry.record_text_cache(
                        overlay_key,
                        frame_id,
                        self.overlay_text_cache.len(),
                        self.overlay_text_cache.stats(),
                    );
                    registry.record_text_cache(
                        heatmap_key,
                        frame_id,
                        self.heatmap_text_cache.len(),
                        self.heatmap_text_cache.stats(),
                    );
                    registry.record_text_cache(
                        debug_key,
                        frame_id,
                        self.debug_text_cache.len(),
                        self.debug_text_cache.stats(),
                    );
                });
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

        self.indicator_text_cache.clear(services);
        self.tooltip_text_cache.clear(services);
        self.readout_text_cache.clear(services);
        self.overlay_text_cache.clear(services);
        self.heatmap_text_cache.clear(services);
        self.debug_text_cache.clear(services);

        self.axis_lock_indicator_x = None;
        self.axis_lock_indicator_y = None;
        self.axis_lock_indicator_y2 = None;
        self.axis_lock_indicator_y3 = None;
        self.axis_lock_indicator_y4 = None;
        self.tooltip_text = None;
        self.mouse_readout_text = None;
        self.linked_cursor_readout_text = None;
        self.overlays_text_key = None;
        self.overlays_text.clear();
        self.heatmap_colorbar_text_key = None;
        self.heatmap_colorbar_text.clear();
        self.heatmap_colorbar_gradient_cache.clear();
        self.quads_scene_cache.clear();

        #[cfg(debug_assertions)]
        {
            self.debug_overlay_text = None;
        }
    }
}
