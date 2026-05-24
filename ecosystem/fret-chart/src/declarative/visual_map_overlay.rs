use std::sync::{Arc, Mutex};

use delinea::engine::window::DataWindow;
use delinea::{Action, ChartEngine, VisualMapId};
use fret_canvas::ui::{
    CanvasToolDownResult, CanvasToolEntry, CanvasToolHandlers, CanvasToolId,
    OnCanvasToolPointerDown, OnCanvasToolPointerMove, OnCanvasToolPointerUp, PanZoomCanvasPaintCx,
};
use fret_core::{Corners, DrawOrder, Edges, MouseButton, Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::canvas::CanvasPainter;

use super::panel::paint_color;
use crate::ChartStyle;
use crate::slider_logic::{SliderDragKind, slider_window_after_delta};
use crate::visual_map_logic::{
    visual_map_continuous_drag_start, visual_map_current_piece_mask, visual_map_domain_window,
    visual_map_piece_mask_after_click, visual_map_track_layouts, visual_map_y_at_value,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct VisualMapTrackState {
    pub(crate) id: VisualMapId,
    pub(crate) model: delinea::engine::model::VisualMapModel,
    pub(crate) track: Rect,
    pub(crate) current_window: DataWindow,
    pub(crate) current_piece_mask: u64,
}

#[derive(Debug, Clone, Copy)]
struct VisualMapDrag {
    visual_map: VisualMapId,
    kind: SliderDragKind,
    track: Rect,
    domain: DataWindow,
    start_window: DataWindow,
    start_value: f64,
}

#[derive(Debug, Default)]
pub(crate) struct VisualMapOverlayState {
    tracks: Vec<VisualMapTrackState>,
    drag: Option<VisualMapDrag>,
    piece_anchor: Option<(VisualMapId, u32)>,
}

impl VisualMapOverlayState {
    pub(crate) fn sync_tracks(&mut self, tracks: Vec<VisualMapTrackState>) {
        let has_track = |id: VisualMapId| tracks.iter().any(|track| track.id == id);

        if self.drag.is_some_and(|drag| !has_track(drag.visual_map)) {
            self.drag = None;
        }
        if self
            .piece_anchor
            .is_some_and(|(visual_map, _)| !has_track(visual_map))
        {
            self.piece_anchor = None;
        }

        if let Some(drag) = self.drag.as_mut()
            && let Some(track) = tracks.iter().find(|track| track.id == drag.visual_map)
        {
            drag.track = track.track;
            drag.domain = visual_map_domain_window(track.model);
        }

        self.tracks = tracks;
    }

    fn track_at(&self, position: Point) -> Option<VisualMapTrackState> {
        self.tracks
            .iter()
            .copied()
            .find(|track| track.track.contains(position))
    }
}

fn is_button_held(button: MouseButton, buttons: fret_core::MouseButtons) -> bool {
    match button {
        MouseButton::Left => buttons.left,
        MouseButton::Right => buttons.right,
        MouseButton::Middle => buttons.middle,
        _ => false,
    }
}

fn active_grid_for_chart(model: &delinea::engine::model::ChartModel) -> Option<delinea::GridId> {
    model
        .series_in_order()
        .find(|series| series.visible)
        .and_then(|primary| model.axes.get(&primary.x_axis).map(|axis| axis.grid))
}

fn visual_map_band_rect(
    model: &delinea::engine::model::ChartModel,
    bounds: Rect,
    style: ChartStyle,
) -> Option<Rect> {
    let active_grid = active_grid_for_chart(model)?;
    let has_visual_map = model.series_in_order().any(|series| {
        series.visible
            && model
                .axes
                .get(&series.x_axis)
                .is_some_and(|axis| axis.grid == active_grid)
            && model.visual_map_by_series.contains_key(&series.id)
    });
    if !has_visual_map {
        return None;
    }

    let axis_band_x = style.axis_band_x.0.max(0.0);
    let axis_band_y = style.axis_band_y.0.max(0.0);

    let mut x_top: Vec<delinea::AxisId> = Vec::new();
    let mut x_bottom: Vec<delinea::AxisId> = Vec::new();
    let mut y_left: Vec<delinea::AxisId> = Vec::new();
    let mut y_right: Vec<delinea::AxisId> = Vec::new();

    for (axis_id, axis) in &model.axes {
        if axis.grid != active_grid {
            continue;
        }

        match (axis.kind, axis.position) {
            (delinea::AxisKind::X, delinea::AxisPosition::Top) => x_top.push(*axis_id),
            (delinea::AxisKind::X, delinea::AxisPosition::Bottom) => x_bottom.push(*axis_id),
            (delinea::AxisKind::Y, delinea::AxisPosition::Left) => y_left.push(*axis_id),
            (delinea::AxisKind::Y, delinea::AxisPosition::Right) => y_right.push(*axis_id),
            _ => {}
        }
    }

    let mut inner = bounds;
    inner.origin.x.0 += style.padding.left.0;
    inner.origin.y.0 += style.padding.top.0;
    inner.size.width.0 =
        (inner.size.width.0 - style.padding.left.0 - style.padding.right.0).max(0.0);
    inner.size.height.0 =
        (inner.size.height.0 - style.padding.top.0 - style.padding.bottom.0).max(0.0);

    let left_total = axis_band_x * (y_left.len() as f32);
    let right_total = axis_band_x * (y_right.len() as f32);
    let top_total = axis_band_y * (x_top.len() as f32);
    let bottom_total = axis_band_y * (x_bottom.len() as f32);
    let plot_w =
        (inner.size.width.0 - left_total - right_total - style.visual_map_band_x.0.max(0.0))
            .max(0.0);
    let plot_h = (inner.size.height.0 - top_total - bottom_total).max(0.0);
    let plot = Rect::new(
        Point::new(
            Px(inner.origin.x.0 + left_total),
            Px(inner.origin.y.0 + top_total),
        ),
        Size::new(Px(plot_w), Px(plot_h)),
    );

    let x0 = plot.origin.x.0 + plot.size.width.0 + axis_band_x * (y_right.len() as f32);
    Some(Rect::new(
        Point::new(Px(x0), plot.origin.y),
        Size::new(Px(style.visual_map_band_x.0.max(0.0)), plot.size.height),
    ))
}

pub(crate) fn visual_map_tracks_for_engine(
    engine: &ChartEngine,
    bounds: Rect,
    style: ChartStyle,
) -> Vec<VisualMapTrackState> {
    let model = engine.model();
    let Some(band) = visual_map_band_rect(model, bounds, style) else {
        return Vec::new();
    };

    let maps: Vec<(VisualMapId, delinea::engine::model::VisualMapModel)> = model
        .visual_maps
        .iter()
        .map(|(id, vm)| (*id, *vm))
        .collect();
    let layouts = visual_map_track_layouts(
        Some(band),
        &maps,
        style.visual_map_item_gap,
        style.visual_map_padding,
    );

    layouts
        .into_iter()
        .map(|layout| {
            let current_window = {
                let domain = visual_map_domain_window(layout.model);
                let range = engine
                    .state()
                    .visual_map_range
                    .get(&layout.id)
                    .copied()
                    .flatten();
                match range {
                    Some(range) => DataWindow {
                        min: range.min,
                        max: range.max,
                    },
                    None => domain,
                }
            };
            let current_piece_mask = visual_map_current_piece_mask(
                layout.model,
                engine
                    .state()
                    .visual_map_piece_mask
                    .get(&layout.id)
                    .copied()
                    .flatten(),
            );

            VisualMapTrackState {
                id: layout.id,
                model: layout.model,
                track: layout.track,
                current_window,
                current_piece_mask,
            }
        })
        .collect()
}

pub(crate) fn visual_map_overlay_tool(
    engine: Model<ChartEngine>,
    state: Arc<Mutex<VisualMapOverlayState>>,
    style: ChartStyle,
) -> CanvasToolEntry {
    let down_state = state.clone();
    let engine_down = engine.clone();
    let on_pointer_down: OnCanvasToolPointerDown =
        Arc::new(move |host, action_cx, _tool_cx, down| {
            let Ok(mut state) = down_state.lock() else {
                return CanvasToolDownResult::unhandled();
            };
            if state.drag.is_some() {
                return CanvasToolDownResult::unhandled();
            }

            let Some(track) = state.track_at(down.position) else {
                return CanvasToolDownResult::unhandled();
            };
            if down.pointer_type != fret_core::PointerType::Mouse {
                return CanvasToolDownResult::unhandled();
            }

            let domain = visual_map_domain_window(track.model);
            let click_value =
                delinea::engine::axis::data_at_y_in_rect(domain, down.position.y.0, track.track);

            match track.model.mode {
                delinea::VisualMapMode::Piecewise
                    if down.button == MouseButton::Left || down.button == MouseButton::Right =>
                {
                    let current = track.current_piece_mask;
                    let wants_reset = (down.button == MouseButton::Right
                        && !down.modifiers.alt
                        && !down.modifiers.ctrl
                        && !down.modifiers.meta
                        && !down.modifiers.alt_gr)
                        || (down.button == MouseButton::Left && down.click_count == 2);
                    let update = visual_map_piece_mask_after_click(
                        track.id,
                        track.model,
                        click_value,
                        current,
                        state.piece_anchor,
                        down.modifiers.shift,
                        wants_reset,
                    );
                    let _ = host.models_mut().update(&engine_down, |engine| {
                        engine.apply_action(Action::SetVisualMapPieceMask {
                            visual_map: track.id,
                            mask: update.mask,
                        });
                    });
                    if let Some(track_state) = state
                        .tracks
                        .iter_mut()
                        .find(|track_state| track_state.id == track.id)
                    {
                        track_state.current_piece_mask =
                            visual_map_current_piece_mask(track.model, update.mask);
                    }
                    state.piece_anchor = update.anchor;
                    host.request_redraw(action_cx.window);
                    CanvasToolDownResult::handled()
                }
                delinea::VisualMapMode::Continuous if down.button == MouseButton::Left => {
                    let current_window = track.current_window;
                    let drag_start = visual_map_continuous_drag_start(
                        track.track,
                        domain,
                        current_window,
                        click_value,
                        down.position.y.0,
                        8.0,
                    );
                    let _ = host.models_mut().update(&engine_down, |engine| {
                        engine.apply_action(Action::SetVisualMapRange {
                            visual_map: track.id,
                            range: Some((drag_start.start_window.min, drag_start.start_window.max)),
                        });
                    });
                    state.drag = Some(VisualMapDrag {
                        visual_map: track.id,
                        kind: drag_start.kind,
                        track: track.track,
                        domain,
                        start_window: drag_start.start_window,
                        start_value: click_value,
                    });
                    if let Some(track_state) = state
                        .tracks
                        .iter_mut()
                        .find(|track_state| track_state.id == track.id)
                    {
                        track_state.current_window = drag_start.start_window;
                    }
                    host.request_redraw(action_cx.window);
                    CanvasToolDownResult {
                        handled: true,
                        activate: false,
                        capture: true,
                    }
                }
                _ => CanvasToolDownResult::unhandled(),
            }
        });

    let move_state = state.clone();
    let engine_move = engine.clone();
    let on_pointer_move: OnCanvasToolPointerMove =
        Arc::new(move |host, action_cx, _tool_cx, mv| {
            let Ok(mut state) = move_state.lock() else {
                return false;
            };
            let Some(drag) = state.drag else {
                return false;
            };
            if !state.tracks.iter().any(|track| track.id == drag.visual_map) {
                state.drag = None;
                return false;
            }
            if !is_button_held(MouseButton::Left, mv.buttons) {
                return false;
            }

            let current_value =
                delinea::engine::axis::data_at_y_in_rect(drag.domain, mv.position.y.0, drag.track);
            let delta_value = current_value - drag.start_value;
            let window =
                slider_window_after_delta(drag.domain, drag.start_window, delta_value, drag.kind);
            let _ = host.models_mut().update(&engine_move, |engine| {
                engine.apply_action(Action::SetVisualMapRange {
                    visual_map: drag.visual_map,
                    range: Some((window.min, window.max)),
                });
            });
            if let Some(track_state) = state
                .tracks
                .iter_mut()
                .find(|track_state| track_state.id == drag.visual_map)
            {
                track_state.current_window = window;
            }
            host.request_redraw(action_cx.window);
            true
        });

    let up_state = state.clone();
    let on_pointer_up: OnCanvasToolPointerUp = Arc::new(move |host, action_cx, _tool_cx, up| {
        let Ok(mut state) = up_state.lock() else {
            return false;
        };
        let Some(_drag) = state.drag else {
            return false;
        };
        if up.button != MouseButton::Left {
            return false;
        }

        state.drag = None;
        host.release_pointer_capture();
        host.request_redraw(action_cx.window);
        true
    });

    let paint_state = state.clone();
    let on_paint = Arc::new(
        move |painter: &mut CanvasPainter<'_>, _paint_cx: PanZoomCanvasPaintCx| {
            let Ok(state) = paint_state.lock() else {
                return;
            };

            if state.tracks.is_empty() {
                return;
            }

            let bounds = painter.bounds();
            if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
                return;
            }

            for (i, track) in state.tracks.iter().enumerate() {
                let track_order = DrawOrder(
                    style
                        .draw_order
                        .0
                        .saturating_add(8_600)
                        .saturating_add((i as u32).saturating_mul(20)),
                );
                painter.scene().push(fret_core::SceneOp::Quad {
                    order: track_order,
                    rect: track.track,
                    background: fret_core::Paint::Solid(style.visual_map_track_color).into(),
                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT.into(),
                    corner_radii: Corners::all(style.visual_map_corner_radius),
                });

                let buckets = track.model.buckets.max(1) as u32;
                let inset = 1.0f32;
                let ramp_rect = Rect::new(
                    Point::new(
                        Px(track.track.origin.x.0 + inset),
                        Px(track.track.origin.y.0 + inset),
                    ),
                    Size::new(
                        Px((track.track.size.width.0 - 2.0 * inset).max(1.0)),
                        Px((track.track.size.height.0 - 2.0 * inset).max(1.0)),
                    ),
                );
                let ramp_h = ramp_rect.size.height.0.max(1.0);
                let segment_h = (ramp_h / buckets as f32).max(1.0);

                match track.model.mode {
                    delinea::VisualMapMode::Continuous => {
                        let ramp_alpha = 0.35f32;
                        for bucket in 0..buckets {
                            let y1 = ramp_rect.origin.y.0 + ramp_h - (bucket as f32) * segment_h;
                            let y0 = (y1 - segment_h).max(ramp_rect.origin.y.0);
                            let h = (y1 - y0).max(1.0);

                            let mut c = paint_color(style, delinea::PaintId(bucket as u64));
                            c.a *= ramp_alpha;
                            painter.scene().push(fret_core::SceneOp::Quad {
                                order: DrawOrder(track_order.0.saturating_add(1)),
                                rect: Rect::new(
                                    Point::new(ramp_rect.origin.x, Px(y0)),
                                    Size::new(ramp_rect.size.width, Px(h)),
                                ),
                                background: fret_core::Paint::Solid(c).into(),
                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT.into(),
                                corner_radii: Corners::all(Px(0.0)),
                            });
                        }

                        let domain = visual_map_domain_window(track.model);
                        let window = track.current_window;
                        let y_min = visual_map_y_at_value(track.track, domain, window.min);
                        let y_max = visual_map_y_at_value(track.track, domain, window.max);
                        let top = y_max.min(y_min);
                        let bottom = y_max.max(y_min);

                        let win_rect = Rect::new(
                            Point::new(track.track.origin.x, Px(top)),
                            Size::new(track.track.size.width, Px((bottom - top).max(1.0))),
                        );
                        painter.scene().push(fret_core::SceneOp::Quad {
                            order: DrawOrder(track_order.0.saturating_add(2)),
                            rect: win_rect,
                            background: fret_core::Paint::Solid(style.visual_map_range_fill).into(),
                            border: Edges::all(style.selection_stroke_width),
                            border_paint: fret_core::Paint::Solid(style.visual_map_range_stroke)
                                .into(),
                            corner_radii: Corners::all(style.visual_map_corner_radius),
                        });

                        let handle_h = 2.0f32.max(style.selection_stroke_width.0);
                        let handle_color = style.visual_map_handle_color;
                        painter.scene().push(fret_core::SceneOp::Quad {
                            order: DrawOrder(track_order.0.saturating_add(3)),
                            rect: Rect::new(
                                Point::new(track.track.origin.x, Px(y_min - 0.5 * handle_h)),
                                Size::new(track.track.size.width, Px(handle_h)),
                            ),
                            background: fret_core::Paint::Solid(handle_color).into(),
                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT.into(),
                            corner_radii: Corners::all(Px(0.0)),
                        });
                        painter.scene().push(fret_core::SceneOp::Quad {
                            order: DrawOrder(track_order.0.saturating_add(4)),
                            rect: Rect::new(
                                Point::new(track.track.origin.x, Px(y_max - 0.5 * handle_h)),
                                Size::new(track.track.size.width, Px(handle_h)),
                            ),
                            background: fret_core::Paint::Solid(handle_color).into(),
                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT.into(),
                            corner_radii: Corners::all(Px(0.0)),
                        });
                    }
                    delinea::VisualMapMode::Piecewise => {
                        let mask = track.current_piece_mask;
                        let ramp_alpha_selected = 0.55f32;
                        let ramp_alpha_unselected = 0.12f32;
                        for bucket in 0..buckets {
                            let y1 = ramp_rect.origin.y.0 + ramp_h - (bucket as f32) * segment_h;
                            let y0 = (y1 - segment_h).max(ramp_rect.origin.y.0);
                            let h = (y1 - y0).max(1.0);

                            let selected = ((mask >> bucket) & 1) == 1;
                            let alpha = if selected {
                                ramp_alpha_selected
                            } else {
                                ramp_alpha_unselected
                            };

                            let mut c = paint_color(style, delinea::PaintId(bucket as u64));
                            c.a *= alpha;
                            painter.scene().push(fret_core::SceneOp::Quad {
                                order: DrawOrder(track_order.0.saturating_add(1)),
                                rect: Rect::new(
                                    Point::new(ramp_rect.origin.x, Px(y0)),
                                    Size::new(ramp_rect.size.width, Px(h)),
                                ),
                                background: fret_core::Paint::Solid(c).into(),
                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT.into(),
                                corner_radii: Corners::all(Px(0.0)),
                            });
                        }
                    }
                }
            }
        },
    );

    CanvasToolEntry {
        id: CanvasToolId::new(12),
        priority: 180,
        handlers: CanvasToolHandlers {
            on_pointer_down: Some(on_pointer_down),
            on_pointer_move: Some(on_pointer_move),
            on_pointer_up: Some(on_pointer_up),
            on_paint: Some(on_paint),
            ..Default::default()
        },
    }
}
