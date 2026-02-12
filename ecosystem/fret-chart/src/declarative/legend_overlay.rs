use std::sync::{Arc, Mutex};

use delinea::{Action, ChartEngine};
use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, FontWeight, MouseButton, Point, Px, Rect, Size,
    TextMetrics, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::canvas::{CanvasPainter, CanvasTextConstraints};
use fret_ui_kit::recipes::canvas_pan_zoom::PanZoomCanvasPaintCx;
use fret_ui_kit::recipes::canvas_tool_router::{
    CanvasToolDownResult, CanvasToolEntry, CanvasToolHandlers, CanvasToolId,
    OnCanvasToolPointerDown, OnCanvasToolPointerMove, OnCanvasToolWheel,
};

use crate::retained::ChartStyle;

#[derive(Debug, Clone)]
pub(crate) struct LegendSeriesEntry {
    pub id: delinea::SeriesId,
    pub order: usize,
    pub label: Arc<str>,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LegendSelectorAction {
    All,
    None,
    Invert,
}

#[derive(Debug, Default)]
pub(crate) struct LegendOverlayState {
    pub series: Vec<LegendSeriesEntry>,
    panel_rect: Option<Rect>,
    last_pointer_pos: Option<Point>,
    item_rects: Vec<(delinea::SeriesId, Rect)>,
    selector_rects: Vec<(LegendSelectorAction, Rect)>,
    hover_series: Option<delinea::SeriesId>,
    hover_selector: Option<LegendSelectorAction>,
    pub anchor: Option<delinea::SeriesId>,
    scroll_y: Px,
    content_height: Px,
    view_height: Px,
}

impl LegendOverlayState {
    pub(crate) fn is_pointer_in_panel(&self) -> bool {
        let in_panel = self
            .panel_rect
            .is_some_and(|r| self.last_pointer_pos.is_some_and(|pos| r.contains(pos)));
        in_panel || self.hover_series.is_some() || self.hover_selector.is_some()
    }

    fn series_at(&self, pos: Point) -> Option<delinea::SeriesId> {
        self.item_rects
            .iter()
            .find_map(|(id, r)| r.contains(pos).then_some(*id))
    }

    fn selector_at(&self, pos: Point) -> Option<LegendSelectorAction> {
        self.selector_rects
            .iter()
            .find_map(|(action, r)| r.contains(pos).then_some(*action))
    }

    fn max_scroll_y(&self) -> Px {
        if self.content_height.0 <= self.view_height.0 {
            return Px(0.0);
        }
        Px(self.content_height.0 - self.view_height.0)
    }

    fn apply_wheel_scroll(&mut self, wheel_delta_y: Px) -> bool {
        let max_scroll = self.max_scroll_y();
        if max_scroll.0 <= 0.0 {
            return false;
        }

        let prev = self.scroll_y;
        let speed = 0.75f32;
        let next = (self.scroll_y.0 - wheel_delta_y.0 * speed).clamp(0.0, max_scroll.0);
        self.scroll_y = Px(next);
        self.scroll_y.0 != prev.0
    }

    pub fn sync_series(&mut self, series: Vec<LegendSeriesEntry>) {
        let has = |id: delinea::SeriesId| series.iter().any(|s| s.id == id);

        if self.hover_series.is_some_and(|id| !has(id)) {
            self.hover_series = None;
        }
        if self.anchor.is_some_and(|id| !has(id)) {
            self.anchor = None;
        }
        self.series = series;
    }
}

pub(crate) fn legend_overlay_tool(
    engine: Model<ChartEngine>,
    legend_state: Arc<Mutex<LegendOverlayState>>,
    style: ChartStyle,
) -> CanvasToolEntry {
    let legend_state_down = legend_state.clone();
    let engine_down = engine.clone();
    let on_pointer_down: OnCanvasToolPointerDown =
        Arc::new(move |host, action_cx, _tool_cx, down| {
            let Ok(mut st) = legend_state_down.lock() else {
                return CanvasToolDownResult::unhandled();
            };
            st.last_pointer_pos = Some(down.position);
            let selector = st.selector_at(down.position);
            let series = st.series_at(down.position);
            let in_panel = st.panel_rect.is_some_and(|r| r.contains(down.position));
            let anchor = st.anchor;
            drop(st);

            if down.button == MouseButton::Right && in_panel {
                let _ = host.models_mut().update(&engine_down, |engine| {
                    let updates = crate::legend_logic::legend_reset_updates(engine.model());
                    if !updates.is_empty() {
                        engine.apply_action(Action::SetSeriesVisibility { updates });
                    }
                });
                if let Ok(mut st) = legend_state_down.lock() {
                    st.anchor = None;
                }
                host.request_focus(action_cx.target);
                return CanvasToolDownResult::handled();
            }

            if down.button != MouseButton::Left {
                return CanvasToolDownResult::unhandled();
            }

            if let Some(action) = selector {
                let _ = host.models_mut().update(&engine_down, |engine| {
                    let model = engine.model();
                    let updates = match action {
                        LegendSelectorAction::All => {
                            crate::legend_logic::legend_select_all_updates(model)
                        }
                        LegendSelectorAction::None => {
                            crate::legend_logic::legend_select_none_updates(model)
                        }
                        LegendSelectorAction::Invert => {
                            crate::legend_logic::legend_invert_updates(model)
                        }
                    };
                    if !updates.is_empty() {
                        engine.apply_action(Action::SetSeriesVisibility { updates });
                    }
                });

                if let Ok(mut st) = legend_state_down.lock() {
                    st.anchor = None;
                    st.hover_series = None;
                    st.hover_selector = Some(action);
                }
                host.request_focus(action_cx.target);
                return CanvasToolDownResult::handled();
            }

            let Some(series) = series else {
                return CanvasToolDownResult::unhandled();
            };

            let mods = down.modifiers;
            let _ = host.models_mut().update(&engine_down, |engine| {
                let model = engine.model();
                if down.click_count >= 2 {
                    let updates = crate::legend_logic::legend_double_click_updates(model, series);
                    if !updates.is_empty() {
                        engine.apply_action(Action::SetSeriesVisibility { updates });
                    }
                    return;
                }

                if mods.shift
                    && let Some(anchor) = anchor
                {
                    let updates = crate::legend_logic::legend_shift_range_toggle_updates(
                        model, anchor, series,
                    );
                    if !updates.is_empty() {
                        engine.apply_action(Action::SetSeriesVisibility { updates });
                    }
                    return;
                }

                let visible = model.series.get(&series).map(|s| s.visible).unwrap_or(true);
                engine.apply_action(Action::SetSeriesVisible {
                    series,
                    visible: !visible,
                });
            });

            if let Ok(mut st) = legend_state_down.lock() {
                st.anchor = Some(series);
                st.hover_series = Some(series);
                st.hover_selector = None;
            }

            host.request_focus(action_cx.target);
            CanvasToolDownResult::handled()
        });

    let legend_state_move = legend_state.clone();
    let on_pointer_move: OnCanvasToolPointerMove =
        Arc::new(move |host, action_cx, _tool_cx, mv| {
            let Ok(mut st) = legend_state_move.lock() else {
                return false;
            };
            st.last_pointer_pos = Some(mv.position);

            let in_panel = st.panel_rect.is_some_and(|r| r.contains(mv.position));
            if !in_panel {
                let changed = st.hover_series.is_some() || st.hover_selector.is_some();
                st.hover_series = None;
                st.hover_selector = None;
                drop(st);

                host.set_cursor_icon(CursorIcon::Default);
                if changed {
                    host.request_redraw(action_cx.window);
                }
                return false;
            }

            let selector = st.selector_at(mv.position);
            let series = if selector.is_some() {
                None
            } else {
                st.series_at(mv.position)
            };
            st.hover_series = series;
            st.hover_selector = selector;
            drop(st);

            host.set_cursor_icon(CursorIcon::Pointer);
            host.request_redraw(action_cx.window);
            true
        });

    let legend_state_wheel = legend_state.clone();
    let on_wheel: OnCanvasToolWheel = Arc::new(move |host, action_cx, _tool_cx, wheel| {
        let Ok(mut st) = legend_state_wheel.lock() else {
            return false;
        };
        if !st.panel_rect.is_some_and(|r| r.contains(wheel.position)) {
            return false;
        }
        let changed = st.apply_wheel_scroll(wheel.delta.y);
        drop(st);
        if changed {
            host.request_redraw(action_cx.window);
        }
        changed
    });

    let legend_state_paint = legend_state.clone();
    let legend_style = style;
    let on_paint = Arc::new(
        move |painter: &mut CanvasPainter<'_>, paint_cx: PanZoomCanvasPaintCx| {
            let Ok(mut st) = legend_state_paint.lock() else {
                return;
            };

            st.item_rects.clear();
            st.selector_rects.clear();
            st.panel_rect = None;
            st.content_height = Px(0.0);
            st.view_height = Px(0.0);

            let plot = painter.bounds();
            if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
                return;
            }
            if st.series.is_empty() {
                return;
            }

            let text_style = TextStyle {
                size: Px(12.0),
                weight: FontWeight::NORMAL,
                ..TextStyle::default()
            };
            let selector_text_style = TextStyle {
                size: Px(11.0),
                weight: FontWeight::MEDIUM,
                ..TextStyle::default()
            };
            let constraints = CanvasTextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            };

            let scope = painter.key_scope(&"fret-chart.declarative.legend");

            struct PreparedSeries {
                id: delinea::SeriesId,
                order: usize,
                label: Arc<str>,
                visible: bool,
                key: u64,
                metrics: TextMetrics,
            }
            struct PreparedSelector {
                action: LegendSelectorAction,
                label: &'static str,
                key: u64,
                metrics: TextMetrics,
            }

            let mut max_text_w = 1.0f32;
            let mut row_h = 1.0f32;
            let mut prepared_series: Vec<PreparedSeries> = Vec::with_capacity(st.series.len());
            for s in &st.series {
                let key: u64 = painter
                    .child_key(scope, &("series", s.id.0, s.label.as_ref()))
                    .into();
                let metrics = painter.text(
                    key,
                    DrawOrder(0),
                    Point::new(Px(0.0), Px(0.0)),
                    s.label.clone(),
                    text_style.clone(),
                    Color::TRANSPARENT,
                    constraints,
                    paint_cx.raster_scale_factor,
                );
                max_text_w = max_text_w.max(metrics.size.width.0.max(1.0));
                row_h = row_h.max(metrics.size.height.0.max(1.0));
                prepared_series.push(PreparedSeries {
                    id: s.id,
                    order: s.order,
                    label: s.label.clone(),
                    visible: s.visible,
                    key,
                    metrics,
                });
            }

            let selector_gap = 8.0f32;
            let selector_labels: [(LegendSelectorAction, &'static str); 3] = [
                (LegendSelectorAction::All, "All"),
                (LegendSelectorAction::None, "None"),
                (LegendSelectorAction::Invert, "Invert"),
            ];
            let mut selector_total_w = 0.0f32;
            let mut selector_h = 0.0f32;
            let mut prepared_selectors: Vec<PreparedSelector> =
                Vec::with_capacity(selector_labels.len());
            for (action, label) in selector_labels {
                let key: u64 = painter.child_key(scope, &("selector", label)).into();
                let metrics = painter.text(
                    key,
                    DrawOrder(0),
                    Point::new(Px(0.0), Px(0.0)),
                    label,
                    selector_text_style.clone(),
                    Color::TRANSPARENT,
                    constraints,
                    paint_cx.raster_scale_factor,
                );
                selector_total_w += metrics.size.width.0.max(1.0);
                selector_h = selector_h.max(metrics.size.height.0.max(1.0));
                prepared_selectors.push(PreparedSelector {
                    action,
                    label,
                    key,
                    metrics,
                });
            }
            if !prepared_selectors.is_empty() {
                selector_total_w +=
                    selector_gap * (prepared_selectors.len().saturating_sub(1) as f32);
            }
            let selector_h = selector_h.max(1.0);
            let selector_row_h = (selector_h + 4.0).max(1.0);

            let pad = legend_style.legend_padding;
            let sw = legend_style.legend_swatch_size.0.max(1.0);
            let sw_gap = legend_style.legend_swatch_gap.0.max(0.0);
            let gap = legend_style.legend_item_gap.0.max(0.0);

            let row_h = row_h.max(sw);
            let legend_w = (pad.left.0 + sw + sw_gap + max_text_w + pad.right.0).max(1.0);

            let items_h =
                ((row_h + gap) * (prepared_series.len().saturating_sub(1) as f32) + row_h).max(1.0);
            let full_h = (pad.top.0 + selector_row_h + items_h + pad.bottom.0).max(1.0);

            let margin = 8.0f32;
            let min_h = (pad.top.0 + row_h + pad.bottom.0).max(1.0);
            let max_h = (plot.size.height.0 - 2.0 * margin).max(min_h);
            let legend_h = full_h.min(max_h);
            let view_h = (legend_h - selector_row_h - pad.top.0 - pad.bottom.0).max(1.0);
            st.content_height = Px(items_h);
            st.view_height = Px(view_h);
            st.scroll_y = Px(st.scroll_y.0.clamp(0.0, st.max_scroll_y().0));

            let x0 = (plot.origin.x.0 + plot.size.width.0 - legend_w - margin)
                .max(plot.origin.x.0 + margin);
            let y0 = plot.origin.y.0 + margin;

            let legend_rect = Rect::new(
                Point::new(Px(x0), Px(y0)),
                Size::new(Px(legend_w), Px(legend_h)),
            );
            st.panel_rect = Some(legend_rect);

            let legend_order = DrawOrder(legend_style.draw_order.0.saturating_add(8_900));
            painter.scene().push(fret_core::SceneOp::Quad {
                order: legend_order,
                rect: legend_rect,
                background: fret_core::Paint::Solid(legend_style.legend_background),

                border: Edges::all(legend_style.legend_border_width),
                border_paint: fret_core::Paint::Solid(legend_style.legend_border_color),

                corner_radii: Corners::all(legend_style.legend_corner_radius),
            });

            painter.with_clip_rect(legend_rect, |painter| {
                let selector_y = y0 + pad.top.0;
                let selector_x0 = x0 + legend_w - pad.right.0 - selector_total_w;
                let mut sx = selector_x0;
                for s in &prepared_selectors {
                    let w = s.metrics.size.width.0.max(1.0);
                    let rect = Rect::new(
                        Point::new(Px(sx), Px(selector_y)),
                        Size::new(Px(w), Px(selector_row_h)),
                    );
                    st.selector_rects.push((s.action, rect));

                    if st.hover_selector == Some(s.action) {
                        painter.scene().push(fret_core::SceneOp::Quad {
                            order: DrawOrder(legend_order.0.saturating_add(1)),
                            rect,
                            background: fret_core::Paint::Solid(
                                legend_style.legend_hover_background,
                            ),

                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT,

                            corner_radii: Corners::all(Px(4.0)),
                        });
                    }

                    let text_y =
                        selector_y + 0.5 * (selector_row_h - s.metrics.size.height.0.max(1.0));
                    let _ = painter.text(
                        s.key,
                        DrawOrder(legend_order.0.saturating_add(2)),
                        Point::new(Px(sx), Px(text_y)),
                        s.label,
                        selector_text_style.clone(),
                        legend_style.legend_text_color,
                        constraints,
                        paint_cx.raster_scale_factor,
                    );

                    sx += w + selector_gap;
                }

                let items_clip = Rect::new(
                    Point::new(Px(x0), Px(y0 + pad.top.0 + selector_row_h)),
                    Size::new(Px(legend_w), Px(view_h)),
                );
                painter.with_clip_rect(items_clip, |painter| {
                    let mut y = items_clip.origin.y.0 - st.scroll_y.0;
                    for (i, s) in prepared_series.iter().enumerate() {
                        let item_rect = Rect::new(
                            Point::new(Px(x0), Px(y)),
                            Size::new(Px(legend_w), Px(row_h)),
                        );
                        st.item_rects.push((s.id, item_rect));

                        if st.hover_series == Some(s.id) {
                            painter.scene().push(fret_core::SceneOp::Quad {
                                order: DrawOrder(legend_order.0.saturating_add(1 + i as u32 * 3)),
                                rect: item_rect,
                                background: fret_core::Paint::Solid(
                                    legend_style.legend_hover_background,
                                ),

                                border: Edges::all(Px(0.0)),
                                border_paint: fret_core::Paint::TRANSPARENT,

                                corner_radii: Corners::all(Px(0.0)),
                            });
                        }

                        let mut swatch = legend_style.series_palette
                            [s.order % legend_style.series_palette.len()];
                        swatch.a = if s.visible { 0.9 } else { 0.25 };
                        let sw_x = x0 + pad.left.0;
                        let sw_y = y + 0.5 * (row_h - sw);
                        painter.scene().push(fret_core::SceneOp::Quad {
                            order: DrawOrder(legend_order.0.saturating_add(2 + i as u32 * 3)),
                            rect: Rect::new(
                                Point::new(Px(sw_x), Px(sw_y)),
                                Size::new(Px(sw), Px(sw)),
                            ),
                            background: fret_core::Paint::Solid(swatch),

                            border: Edges::all(Px(0.0)),
                            border_paint: fret_core::Paint::TRANSPARENT,

                            corner_radii: Corners::all(Px(2.0)),
                        });

                        let text_x = sw_x + sw + sw_gap;
                        let text_y = y + 0.5 * (row_h - s.metrics.size.height.0.max(1.0));
                        let mut text_color = legend_style.legend_text_color;
                        if !s.visible {
                            text_color.a *= 0.55;
                        }
                        let _ = painter.text(
                            s.key,
                            DrawOrder(legend_order.0.saturating_add(3 + i as u32 * 3)),
                            Point::new(Px(text_x), Px(text_y)),
                            s.label.clone(),
                            text_style.clone(),
                            text_color,
                            constraints,
                            paint_cx.raster_scale_factor,
                        );

                        y += row_h + gap;
                    }
                });
            });
        },
    );

    CanvasToolEntry {
        id: CanvasToolId::new(10),
        priority: 200,
        handlers: CanvasToolHandlers {
            on_pointer_down: Some(on_pointer_down),
            on_pointer_move: Some(on_pointer_move),
            on_wheel: Some(on_wheel),
            on_paint: Some(on_paint),
            ..Default::default()
        },
    }
}
