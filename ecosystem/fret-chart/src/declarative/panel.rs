use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use delinea::engine::model::{ChartPatch, PatchMode};
use delinea::engine::window::DataWindow;
use delinea::marks::{MarkKind, MarkPayloadRef, MarkTree};
use delinea::{Action, ChartEngine, WorkBudget};
use fret_core::{
    Color, Corners, DrawOrder, Edges, MouseButton, PathCommand, PathStyle, Point, Px, Rect, Size,
    StrokeStyle,
};
use fret_runtime::Model;
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::{AnyElement, CanvasProps, Length, PointerRegionProps};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::recipes::canvas_pan_zoom::{PanZoomCanvasPaintCx, PanZoomCanvasSurfacePanelProps};
use fret_ui_kit::recipes::canvas_tool_router::{
    CanvasToolDownResult, CanvasToolEntry, CanvasToolHandlers, CanvasToolId, CanvasToolRouterProps,
    OnCanvasToolPointerDown, OnCanvasToolPointerMove, OnCanvasToolPointerUp, OnCanvasToolWheel,
    canvas_tool_router_panel,
};

use crate::input_map::{ChartInputMap, ModifierKey};
use crate::retained::ChartStyle;
use crate::{DefaultTooltipFormatter, TooltipFormatter, TooltipTextLine};

use super::legend_overlay::{LegendOverlayState, LegendSeriesEntry, legend_overlay_tool};
use super::tooltip_overlay::{TooltipOverlayState, tooltip_overlay_tool};

#[derive(Debug, Default)]
struct NullTextMeasurer;

impl delinea::text::TextMeasurer for NullTextMeasurer {
    fn measure(
        &mut self,
        _text: delinea::ids::StringId,
        _style: delinea::text::TextStyleId,
    ) -> delinea::text::TextMetrics {
        delinea::text::TextMetrics::default()
    }
}

#[derive(Debug, Clone, Copy)]
struct ChartPanDrag {
    start_pos: Point,
    x_axis: delinea::AxisId,
    y_axis: delinea::AxisId,
    start_x: DataWindow,
    start_y: DataWindow,
}

fn default_chart_input_map_safe() -> ChartInputMap {
    let mut map = ChartInputMap::default();
    map.wheel_zoom_mod = Some(ModifierKey::Ctrl);
    map
}

fn primary_axes(engine: &ChartEngine) -> Option<(delinea::AxisId, delinea::AxisId)> {
    let model = engine.model();
    for id in &model.series_order {
        let s = model.series.get(id)?;
        if s.visible {
            return Some((s.x_axis, s.y_axis));
        }
    }
    None
}

fn fallback_window() -> DataWindow {
    DataWindow { min: 0.0, max: 1.0 }
}

fn window_for_axis_x(engine: &ChartEngine, axis: delinea::AxisId) -> DataWindow {
    engine
        .output()
        .axis_windows
        .get(&axis)
        .copied()
        .unwrap_or_else(fallback_window)
}

fn window_for_axis_y(engine: &ChartEngine, axis: delinea::AxisId) -> DataWindow {
    engine
        .output()
        .axis_windows
        .get(&axis)
        .copied()
        .unwrap_or_else(fallback_window)
}

fn paint_color(style: ChartStyle, paint: delinea::PaintId) -> Color {
    let palette = &style.series_palette;
    palette[(paint.0 as usize) % palette.len()]
}

fn series_color(style: ChartStyle, series: delinea::SeriesId) -> Color {
    let palette = &style.series_palette;
    palette[(series.0 as usize) % palette.len()]
}

fn ensure_engine_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<ChartEngine>>,
    spec: delinea::ChartSpec,
) -> Model<ChartEngine> {
    if let Some(model) = controlled {
        return model;
    }

    struct EngineState {
        model: Option<Model<ChartEngine>>,
    }
    impl Default for EngineState {
        fn default() -> Self {
            Self { model: None }
        }
    }

    let existing = cx.with_state(EngineState::default, |st| st.model.clone());
    if let Some(model) = existing {
        return model;
    }

    let mut spec = spec;
    spec.axis_pointer.get_or_insert_with(Default::default);
    let engine = ChartEngine::new(spec).expect("chart spec should be valid");
    let model = cx.app.models_mut().insert(engine);
    cx.with_state(EngineState::default, |st| st.model = Some(model.clone()));
    model
}

#[derive(Debug, Clone)]
struct MarksCache {
    rev: delinea::ids::Revision,
    marks: Arc<MarkTree>,
}

impl Default for MarksCache {
    fn default() -> Self {
        Self {
            rev: delinea::ids::Revision::default(),
            marks: Arc::new(MarkTree::default()),
        }
    }
}

#[derive(Clone)]
pub struct ChartCanvasPanelProps {
    pub pointer_region: PointerRegionProps,
    pub canvas: CanvasProps,

    /// When `None`, an internal engine model is created once from `spec`.
    pub engine: Option<Model<ChartEngine>>,
    pub spec: delinea::ChartSpec,

    /// Chart interaction mapping (ImPlot-aligned). Defaults to a "safe" wheel mapping
    /// (zoom requires Ctrl), because charts are often embedded inside scroll containers.
    pub input_map: ChartInputMap,

    pub style: ChartStyle,
}

impl ChartCanvasPanelProps {
    pub fn new(spec: delinea::ChartSpec) -> Self {
        Self {
            pointer_region: PointerRegionProps::default(),
            canvas: CanvasProps::default(),
            engine: None,
            spec,
            input_map: default_chart_input_map_safe(),
            style: ChartStyle::default(),
        }
    }
}

#[track_caller]
pub fn chart_canvas_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mut props: ChartCanvasPanelProps,
) -> AnyElement {
    props.pointer_region.layout.size.width = Length::Fill;
    props.pointer_region.layout.size.height = Length::Fill;
    props.canvas.layout.size.width = Length::Fill;
    props.canvas.layout.size.height = Length::Fill;

    let engine = ensure_engine_model(cx, props.engine.clone(), props.spec.clone());

    // Tool-local drag model.
    let pan_drag: Model<Option<ChartPanDrag>> = {
        struct PanDragState {
            model: Option<Model<Option<ChartPanDrag>>>,
        }
        impl Default for PanDragState {
            fn default() -> Self {
                Self { model: None }
            }
        }

        let existing = cx.with_state(PanDragState::default, |st| st.model.clone());
        if let Some(model) = existing {
            model
        } else {
            let model = cx.app.models_mut().insert(None::<ChartPanDrag>);
            cx.with_state(PanDragState::default, |st| st.model = Some(model.clone()));
            model
        }
    };

    let legend_state: Arc<Mutex<LegendOverlayState>> = cx.with_state(
        || Arc::new(Mutex::new(LegendOverlayState::default())),
        |st| st.clone(),
    );
    let tooltip_state: Arc<Mutex<TooltipOverlayState>> = cx.with_state(
        || Arc::new(Mutex::new(TooltipOverlayState::default())),
        |st| st.clone(),
    );

    // Step the engine during declarative render and cache the current marks snapshot.
    let bounds = cx.bounds;
    let mut unfinished = false;
    let mut marks_rev = delinea::ids::Revision::default();
    let mut output_marks: Arc<MarkTree> = Arc::new(MarkTree::default());
    let mut legend_series: Vec<LegendSeriesEntry> = Vec::new();
    let mut series_rank_by_id: BTreeMap<delinea::SeriesId, usize> = BTreeMap::default();
    let mut axis_pointer: Option<delinea::engine::AxisPointerOutput> = None;
    let mut tooltip_lines: Vec<TooltipTextLine> = Vec::new();

    let _ = engine.update(cx.app, |engine, _cx| {
        if engine.model().viewport != Some(bounds) {
            let _ = engine.apply_patch(
                ChartPatch {
                    viewport: Some(Some(bounds)),
                    ..ChartPatch::default()
                },
                PatchMode::Merge,
            );
        }

        let mut measurer = NullTextMeasurer::default();
        let start = Instant::now();
        let mut steps_ran = 0u32;
        let mut still_unfinished = true;
        while still_unfinished && steps_ran < 8 && start.elapsed() < Duration::from_millis(4) {
            let budget = WorkBudget::new(262_144, 0, 32);
            let step = engine.step(&mut measurer, budget);
            match step {
                Ok(step) => {
                    still_unfinished = step.unfinished;
                }
                Err(_) => {
                    still_unfinished = false;
                }
            }
            steps_ran = steps_ran.saturating_add(1);
        }

        unfinished = still_unfinished;
        marks_rev = engine.output().marks.revision;
        output_marks = Arc::new(engine.output().marks.clone());

        let model = engine.model();
        series_rank_by_id.clear();
        legend_series = model
            .series_in_order()
            .enumerate()
            .map(|(order, s)| LegendSeriesEntry {
                id: s.id,
                order,
                label: s
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("Series {}", s.id.0))
                    .into(),
                visible: s.visible,
            })
            .collect();
        for s in &legend_series {
            series_rank_by_id.insert(s.id, s.order);
        }

        axis_pointer = engine.output().axis_pointer.clone();
        tooltip_lines.clear();
        if let Some(axis_pointer) = axis_pointer.as_ref() {
            let formatter = DefaultTooltipFormatter::default();
            tooltip_lines =
                formatter.format_axis_pointer(engine, &engine.output().axis_windows, axis_pointer);
        }
    });

    if let Ok(mut st) = legend_state.lock() {
        st.sync_series(legend_series);
    }
    if let Ok(mut st) = tooltip_state.lock() {
        st.axis_pointer = axis_pointer;
        st.lines = tooltip_lines;
        st.series_rank_by_id = series_rank_by_id;
    }

    let cache = cx.with_state(MarksCache::default, |cache| {
        if cache.rev != marks_rev {
            *cache = MarksCache {
                rev: marks_rev,
                marks: output_marks.clone(),
            };
        }
        cache.marks.clone()
    });

    let style = props.style;
    let engine_c = engine.clone();
    let input_map = props.input_map;

    let pan_drag_down = pan_drag.clone();
    let on_pan_down: OnCanvasToolPointerDown = Arc::new(move |host, _action_cx, tool_cx, down| {
        if !input_map.pan.matches(down.button, down.modifiers) {
            return CanvasToolDownResult::unhandled();
        }
        if !tool_cx.bounds.contains(down.position) {
            return CanvasToolDownResult::unhandled();
        }

        let Some((x_axis, y_axis)) = host
            .models_mut()
            .read(&engine_c, |engine| primary_axes(engine))
            .ok()
            .flatten()
        else {
            return CanvasToolDownResult::unhandled();
        };

        let (start_x, start_y) = host
            .models_mut()
            .read(&engine_c, |engine| {
                (
                    window_for_axis_x(engine, x_axis),
                    window_for_axis_y(engine, y_axis),
                )
            })
            .ok()
            .unwrap_or((fallback_window(), fallback_window()));

        let _ = host.models_mut().update(&pan_drag_down, |st| {
            *st = Some(ChartPanDrag {
                start_pos: down.position,
                x_axis,
                y_axis,
                start_x,
                start_y,
            });
        });

        CanvasToolDownResult::activate_and_capture()
    });

    let pan_drag_move = pan_drag.clone();
    let engine_c = engine.clone();
    let on_pan_move: OnCanvasToolPointerMove = Arc::new(move |host, action_cx, tool_cx, mv| {
        let Some(drag) = host
            .models_mut()
            .read(&pan_drag_move, |st| *st)
            .ok()
            .flatten()
        else {
            return false;
        };

        let width = tool_cx.bounds.size.width.0;
        let height = tool_cx.bounds.size.height.0;
        if width <= 0.0 || height <= 0.0 {
            return false;
        }

        let dx = mv.position.x.0 - drag.start_pos.x.0;
        let dy = mv.position.y.0 - drag.start_pos.y.0;

        let _ = host.models_mut().update(&engine_c, |engine| {
            engine.apply_action(Action::PanDataWindowXFromBase {
                axis: drag.x_axis,
                base: drag.start_x,
                delta_px: dx,
                viewport_span_px: width,
            });
            engine.apply_action(Action::PanDataWindowYFromBase {
                axis: drag.y_axis,
                base: drag.start_y,
                delta_px: -dy,
                viewport_span_px: height,
            });
        });

        host.request_redraw(action_cx.window);
        true
    });

    let pan_drag_up = pan_drag.clone();
    let on_pan_up: OnCanvasToolPointerUp = Arc::new(move |host, _action_cx, _tool_cx, _up| {
        let _ = host.models_mut().update(&pan_drag_up, |st| *st = None);
        true
    });

    let engine_c = engine.clone();
    let on_hover_move: OnCanvasToolPointerMove = Arc::new(move |host, action_cx, _tool_cx, mv| {
        let _ = host.models_mut().update(&engine_c, |engine| {
            engine.apply_action(Action::HoverAt { point: mv.position });
        });
        host.request_redraw(action_cx.window);
        true
    });

    let engine_c = engine.clone();
    let input_map_c = input_map;
    let on_wheel_zoom: OnCanvasToolWheel = Arc::new(move |host, action_cx, tool_cx, wheel| {
        let delta_y = wheel.delta.y.0;
        if !delta_y.is_finite() {
            return false;
        }

        if let Some(required) = input_map_c.wheel_zoom_mod
            && !required.is_pressed(wheel.modifiers)
        {
            return false;
        }

        let width = tool_cx.bounds.size.width.0;
        let height = tool_cx.bounds.size.height.0;
        if width <= 0.0 || height <= 0.0 {
            return false;
        }

        let Some((x_axis, y_axis)) = host
            .models_mut()
            .read(&engine_c, |engine| primary_axes(engine))
            .ok()
            .flatten()
        else {
            return false;
        };

        let (base_x, base_y) = host
            .models_mut()
            .read(&engine_c, |engine| {
                (
                    window_for_axis_x(engine, x_axis),
                    window_for_axis_y(engine, y_axis),
                )
            })
            .ok()
            .unwrap_or((fallback_window(), fallback_window()));

        // Match ImPlot's default feel: zoom factor ~= 2^(delta_y * 0.0025)
        let log2_scale = delta_y * 0.0025;

        let local_x = (wheel.position.x.0 - tool_cx.bounds.origin.x.0).clamp(0.0, width);
        let local_y = (wheel.position.y.0 - tool_cx.bounds.origin.y.0).clamp(0.0, height);
        let center_x = local_x;
        let center_y_from_bottom = height - local_y;

        let _ = host.models_mut().update(&engine_c, |engine| {
            engine.apply_action(Action::ZoomDataWindowXFromBase {
                axis: x_axis,
                base: base_x,
                center_px: center_x,
                log2_scale,
                viewport_span_px: width,
            });
            engine.apply_action(Action::ZoomDataWindowYFromBase {
                axis: y_axis,
                base: base_y,
                center_px: center_y_from_bottom,
                log2_scale,
                viewport_span_px: height,
            });
        });

        host.request_redraw(action_cx.window);
        true
    });

    let legend_tool = legend_overlay_tool(engine.clone(), legend_state.clone(), style);
    let tooltip_tool = tooltip_overlay_tool(tooltip_state.clone(), style);

    let tools = vec![
        legend_tool,
        tooltip_tool,
        CanvasToolEntry {
            id: CanvasToolId::new(1),
            priority: 100,
            handlers: CanvasToolHandlers {
                on_pointer_down: Some(on_pan_down),
                on_pointer_move: Some(on_pan_move),
                on_pointer_up: Some(on_pan_up),
                ..Default::default()
            },
        },
        CanvasToolEntry {
            id: CanvasToolId::new(2),
            priority: 50,
            handlers: CanvasToolHandlers {
                on_wheel: Some(on_wheel_zoom),
                ..Default::default()
            },
        },
        CanvasToolEntry {
            id: CanvasToolId::new(3),
            priority: -10,
            handlers: CanvasToolHandlers {
                on_pointer_move: Some(on_hover_move),
                ..Default::default()
            },
        },
    ];

    let mut pan_zoom = PanZoomCanvasSurfacePanelProps::default();
    pan_zoom.pointer_region = props.pointer_region;
    pan_zoom.canvas = props.canvas;

    // Disable built-in infinite-canvas pan/zoom: chart interactions are routed via tools.
    pan_zoom.pan_button = MouseButton::Other(999);
    pan_zoom.min_zoom = 1.0;
    pan_zoom.max_zoom = 1.0;

    let router_props = CanvasToolRouterProps {
        pan_zoom,
        active_tool: None,
    };

    let marks = cache;
    let paint = move |painter: &mut CanvasPainter<'_>, paint_cx: PanZoomCanvasPaintCx| {
        if unfinished {
            painter.request_animation_frame();
        }

        let bounds = painter.bounds();

        // Basic background.
        if let Some(background) = style.background {
            painter.scene().push(fret_core::SceneOp::Quad {
                order: DrawOrder(style.draw_order.0.saturating_sub(1)),
                rect: bounds,
                background,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        }

        let viewport = bounds;
        painter.with_clip_rect(viewport, |painter| {
            let marks = &*marks;
            let arena = &marks.arena;

            for node in &marks.nodes {
                match (node.kind, &node.payload) {
                    (MarkKind::Polyline, MarkPayloadRef::Polyline(poly)) => {
                        let start = poly.points.start;
                        let end = poly.points.end;
                        if end <= start || end > arena.points.len() {
                            continue;
                        }

                        let mut commands: Vec<PathCommand> =
                            Vec::with_capacity((end - start).saturating_add(1));
                        for (i, p) in arena.points[start..end].iter().enumerate() {
                            if i == 0 {
                                commands.push(PathCommand::MoveTo(*p));
                            } else {
                                commands.push(PathCommand::LineTo(*p));
                            }
                        }
                        if commands.len() < 2 {
                            continue;
                        }

                        let stroke_width = poly
                            .stroke
                            .as_ref()
                            .map(|(_, s)| s.width)
                            .unwrap_or(style.stroke_width);
                        let stroke_color = if let Some((paint, _)) = &poly.stroke {
                            paint_color(style, *paint)
                        } else if let Some(series) = node.source_series {
                            series_color(style, series)
                        } else {
                            style.stroke_color
                        };

                        let key = node.id.0;
                        painter.path(
                            key,
                            DrawOrder(style.draw_order.0.saturating_add(node.order.0)),
                            Point::new(Px(0.0), Px(0.0)),
                            &commands,
                            PathStyle::Stroke(StrokeStyle {
                                width: stroke_width,
                            }),
                            stroke_color,
                            paint_cx.raster_scale_factor,
                        );
                    }
                    (MarkKind::Rect, MarkPayloadRef::Rect(rects)) => {
                        let start = rects.rects.start;
                        let end = rects.rects.end;
                        if end <= start || end > arena.rects.len() {
                            continue;
                        }

                        let stroke_width = rects
                            .stroke
                            .as_ref()
                            .map(|(_, s)| s.width)
                            .filter(|w| w.0.is_finite() && w.0 > 0.0)
                            .unwrap_or(Px(0.0));

                        for rect in &arena.rects[start..end] {
                            let mut background = Color::TRANSPARENT;
                            if let Some(paint) = rects.fill {
                                background = paint_color(style, paint);
                            } else if let Some(series) = node.source_series {
                                background = series_color(style, series);
                            }
                            background.a *= rects.opacity_mul.unwrap_or(1.0);

                            let border_color = if stroke_width.0 > 0.0 {
                                background
                            } else {
                                Color::TRANSPARENT
                            };

                            painter.scene().push(fret_core::SceneOp::Quad {
                                order: DrawOrder(style.draw_order.0.saturating_add(node.order.0)),
                                rect: *rect,
                                background,
                                border: Edges::all(stroke_width),
                                border_color,
                                corner_radii: Corners::all(Px(0.0)),
                            });
                        }
                    }
                    (MarkKind::Points, MarkPayloadRef::Points(points)) => {
                        let start = points.points.start;
                        let end = points.points.end;
                        if end <= start || end > arena.points.len() {
                            continue;
                        }

                        let base_point_r = style.scatter_point_radius.0.max(1.0);
                        let stroke_width = points
                            .stroke
                            .as_ref()
                            .map(|(_, s)| s.width)
                            .filter(|w| w.0.is_finite() && w.0 > 0.0)
                            .unwrap_or(Px(0.0));

                        for p in &arena.points[start..end] {
                            let radius_mul = points
                                .radius_mul
                                .filter(|v| v.is_finite() && *v > 0.0)
                                .unwrap_or(1.0);
                            let point_r = (base_point_r * radius_mul).max(1.0);

                            let mut fill = style.stroke_color;
                            if let Some(paint) = points.fill {
                                fill = paint_color(style, paint);
                                fill.a *= style.scatter_fill_alpha;
                            } else if let Some(series) = node.source_series {
                                fill = series_color(style, series);
                                fill.a *= style.scatter_fill_alpha;
                            }
                            fill.a *= points.opacity_mul.unwrap_or(1.0);

                            let border_color = if stroke_width.0 > 0.0 {
                                fill
                            } else {
                                Color::TRANSPARENT
                            };

                            painter.scene().push(fret_core::SceneOp::Quad {
                                order: DrawOrder(style.draw_order.0.saturating_add(node.order.0)),
                                rect: Rect::new(
                                    Point::new(Px(p.x.0 - point_r), Px(p.y.0 - point_r)),
                                    Size::new(Px(2.0 * point_r), Px(2.0 * point_r)),
                                ),
                                background: fill,
                                border: Edges::all(stroke_width),
                                border_color,
                                corner_radii: Corners::all(Px(point_r)),
                            });
                        }
                    }
                    _ => {}
                }
            }
        });
    };

    canvas_tool_router_panel(cx, router_props, tools, paint)
}
