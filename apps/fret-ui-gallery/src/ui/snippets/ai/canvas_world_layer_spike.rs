pub const SOURCE: &str = include_str!("canvas_world_layer_spike.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    use std::sync::Arc;

    use fret_canvas::ui::{
        CanvasInputExemptRegionProps, CanvasMarqueeSelectionProps, CanvasWorldBoundsStore,
        CanvasWorldScaleMode, CanvasWorldSurfacePanelProps, OnCanvasMarqueeCommit,
        OnCanvasMarqueeStart, PanZoomInputPreset, canvas_input_exempt_region,
        canvas_world_bounds_item, canvas_world_fit_view_to_keys,
        canvas_world_surface_panel_with_marquee_selection, use_controllable_model,
    };
    use fret_canvas::view::{FitViewOptions2D, PanZoom2D, visible_canvas_rect};
    use fret_core::scene::Paint;
    use fret_core::{Corners, DrawOrder, Edges, Point, Px, Rect, SceneOp, Size};
    use fret_ui::action::OnActivate;
    use fret_ui::canvas::CanvasPainter;
    use fret_ui::element::{CanvasCachePolicy, Length, PointerRegionProps, SemanticsDecoration};
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct ConnectOverlayDragState {
        pointer_id: fret_core::PointerId,
        view_at_start: PanZoom2D,
        surface_bounds_at_start: Rect,
        scale_mode_at_start: CanvasWorldScaleMode,
    }

    #[derive(Clone)]
    struct LocalModels {
        view: fret_runtime::Model<PanZoom2D>,
        scale_mode: fret_runtime::Model<CanvasWorldScaleMode>,
        overlay_clicks: fret_runtime::Model<u64>,
        node_clicks: fret_runtime::Model<u64>,
        bounds_store: fret_runtime::Model<CanvasWorldBoundsStore>,
        selected_count: fret_runtime::Model<u64>,
        marquee_blocked_count: fret_runtime::Model<u64>,
        node_a_canvas_pos: fret_runtime::Model<Point>,
        node_b_canvas_pos: fret_runtime::Model<Point>,
        connect_drag_started_count: fret_runtime::Model<u64>,
        connect_drag_up_count: fret_runtime::Model<u64>,
        connect_drag_hit_count: fret_runtime::Model<u64>,
        connect_overlay_drag_state: fret_runtime::Model<Option<ConnectOverlayDragState>>,
        node_a_screen_bounds: fret_runtime::Model<Option<Rect>>,
        node_b_screen_bounds: fret_runtime::Model<Option<Rect>>,
        node_dragged_count: fret_runtime::Model<u64>,
        connections: fret_runtime::Model<Vec<(u64, u64)>>,
        reset_epoch: fret_runtime::Model<u64>,
    }

    let header = ui::v_flex(|cx| {
        vec![
            cx.text("Canvas world layer (spike)"),
            cx.text("Goal: nodes as element subtrees under a pan/zoom view transform."),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N2)
    .into_element(cx);

    let created = cx.slot_state(
        || false,
        |initialized| {
            if *initialized {
                false
            } else {
                *initialized = true;
                true
            }
        },
    );
    let models = LocalModels {
        view: cx.local_model_keyed("view", PanZoom2D::default),
        scale_mode: cx.local_model_keyed("scale_mode", || CanvasWorldScaleMode::ScaleWithZoom),
        overlay_clicks: cx.local_model_keyed("overlay_clicks", || 0u64),
        node_clicks: cx.local_model_keyed("node_clicks", || 0u64),
        bounds_store: cx.local_model_keyed("bounds_store", CanvasWorldBoundsStore::default),
        selected_count: cx.local_model_keyed("selected_count", || 0u64),
        marquee_blocked_count: cx.local_model_keyed("marquee_blocked_count", || 0u64),
        node_a_canvas_pos: cx
            .local_model_keyed("node_a_canvas_pos", || Point::new(Px(420.0), Px(80.0))),
        node_b_canvas_pos: cx
            .local_model_keyed("node_b_canvas_pos", || Point::new(Px(760.0), Px(260.0))),
        connect_drag_started_count: cx.local_model_keyed("connect_drag_started_count", || 0u64),
        connect_drag_up_count: cx.local_model_keyed("connect_drag_up_count", || 0u64),
        connect_drag_hit_count: cx.local_model_keyed("connect_drag_hit_count", || 0u64),
        connect_overlay_drag_state: cx.local_model_keyed("connect_overlay_drag_state", || {
            None::<ConnectOverlayDragState>
        }),
        node_a_screen_bounds: cx.local_model_keyed("node_a_screen_bounds", || None::<Rect>),
        node_b_screen_bounds: cx.local_model_keyed("node_b_screen_bounds", || None::<Rect>),
        node_dragged_count: cx.local_model_keyed("node_dragged_count", || 0u64),
        connections: cx.local_model_keyed("connections", Vec::<(u64, u64)>::new),
        reset_epoch: cx.local_model_keyed("reset_epoch", || 0u64),
    };

    if created {
        cx.request_frame();
    }

    let view = models.view.clone();
    let scale_mode = models.scale_mode.clone();
    let overlay_clicks = models.overlay_clicks.clone();
    let node_clicks = models.node_clicks.clone();
    let bounds_store = models.bounds_store.clone();
    let selected_count = models.selected_count.clone();
    let marquee_blocked_count = models.marquee_blocked_count.clone();
    let node_a_canvas_pos = models.node_a_canvas_pos.clone();
    let node_b_canvas_pos = models.node_b_canvas_pos.clone();
    let connect_drag_started_count = models.connect_drag_started_count.clone();
    let connect_drag_up_count = models.connect_drag_up_count.clone();
    let connect_drag_hit_count = models.connect_drag_hit_count.clone();
    let connect_overlay_drag_state = models.connect_overlay_drag_state.clone();
    let node_a_screen_bounds = models.node_a_screen_bounds.clone();
    let node_b_screen_bounds = models.node_b_screen_bounds.clone();
    let node_dragged_count = models.node_dragged_count.clone();
    let connections = models.connections.clone();
    let reset_epoch = models.reset_epoch.clone();

    let node_a_canvas_pos_for_world = node_a_canvas_pos.clone();
    let node_b_canvas_pos_for_world = node_b_canvas_pos.clone();
    let node_dragged_count_for_world = node_dragged_count.clone();

    let scale_mode_value = cx
        .get_model_copied(&scale_mode, fret_ui::Invalidation::Layout)
        .unwrap_or_default();
    let overlay_clicks_value = cx
        .get_model_copied(&overlay_clicks, fret_ui::Invalidation::Layout)
        .unwrap_or(0);
    let node_clicks_value = cx
        .get_model_copied(&node_clicks, fret_ui::Invalidation::Layout)
        .unwrap_or(0);
    let selected_count_value = cx
        .get_model_copied(&selected_count, fret_ui::Invalidation::Layout)
        .unwrap_or(0);
    let marquee_blocked_count_value = cx
        .get_model_copied(&marquee_blocked_count, fret_ui::Invalidation::Layout)
        .unwrap_or(0);
    let connect_drag_started_value = cx
        .get_model_copied(&connect_drag_started_count, fret_ui::Invalidation::Layout)
        .unwrap_or(0);
    let connect_drag_up_value = cx
        .get_model_copied(&connect_drag_up_count, fret_ui::Invalidation::Layout)
        .unwrap_or(0);
    let connect_drag_hit_value = cx
        .get_model_copied(&connect_drag_hit_count, fret_ui::Invalidation::Layout)
        .unwrap_or(0);
    let node_b_screen_bounds_value = cx
        .get_model_copied(&node_b_screen_bounds, fret_ui::Invalidation::Layout)
        .unwrap_or(None);
    let node_dragged_count_value = cx
        .get_model_copied(&node_dragged_count, fret_ui::Invalidation::Layout)
        .unwrap_or(0);

    let (bounds_count, bounds_union_canvas) = cx
        .read_model_ref(&bounds_store, fret_ui::Invalidation::Layout, |st| {
            let keys = [1u64, 2u64];
            (st.items.len(), st.union_canvas_bounds_for_keys(keys.iter()))
        })
        .unwrap_or((0, None));

    let stage_layout = LayoutRefinement::default()
        .w_full()
        .h_px(Px(420.0))
        .min_w_0()
        .relative()
        .overflow_hidden();

    let stage_props = cx.with_theme(|theme| {
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .rounded(Radius::Md)
                .border_1()
                .bg(ColorRef::Token {
                    key: "card",
                    fallback: ColorFallback::ThemePanelBackground,
                })
                .border_color(ColorRef::Token {
                    key: "border",
                    fallback: ColorFallback::ThemePanelBorder,
                })
                .p(Space::N2),
            stage_layout,
        )
    });

    let mut world_props = CanvasWorldSurfacePanelProps::default();
    world_props.pan_zoom.preset = PanZoomInputPreset::DesktopCanvasCad;
    world_props.pan_zoom.view = Some(view.clone());
    world_props.pan_zoom.default_view = PanZoom2D::default();
    world_props.scale_mode = scale_mode_value;
    world_props.pan_zoom.pointer_region = {
        let mut p = PointerRegionProps::default();
        p.layout.size.width = Length::Fill;
        p.layout.size.height = Length::Fill;
        p
    };
    world_props.pan_zoom.canvas.cache_policy = CanvasCachePolicy::smooth_default();

    let (bg, grid) = cx.with_theme(|theme| {
        (
            theme.color_required("background"),
            theme.color_required("border"),
        )
    });
    let paint = {
        move |p: &mut CanvasPainter<'_>, paint_cx: fret_canvas::ui::PanZoomCanvasPaintCx| {
            let bounds = p.bounds();
            let Some(transform) = paint_cx.view.render_transform(bounds) else {
                return;
            };

            // Grid in canvas space.
            let step = 80.0f32;
            let vis = visible_canvas_rect(bounds, paint_cx.view);
            let min_x = (vis.origin.x.0 / step).floor() as i32 - 2;
            let max_x = ((vis.origin.x.0 + vis.size.width.0) / step).ceil() as i32 + 2;
            let min_y = (vis.origin.y.0 / step).floor() as i32 - 2;
            let max_y = ((vis.origin.y.0 + vis.size.height.0) / step).ceil() as i32 + 2;

            p.with_clip_rect(bounds, |p| {
                p.with_transform(transform, |p| {
                    let rect = Rect::new(
                        Point::new(Px(-10_000.0), Px(-10_000.0)),
                        Size::new(Px(20_000.0), Px(20_000.0)),
                    );
                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect,
                        background: Paint::Solid(bg).into(),
                        border: Edges::all(Px(0.0)),
                        border_paint: Paint::Solid(fret_core::scene::Color::TRANSPARENT).into(),
                        corner_radii: Corners::all(Px(0.0)),
                    });

                    for x in min_x..=max_x {
                        let ox = x as f32 * step;
                        let line = Rect::new(
                            Point::new(Px(ox), Px(min_y as f32 * step)),
                            Size::new(Px(1.0), Px((max_y - min_y) as f32 * step)),
                        );
                        p.scene().push(SceneOp::Quad {
                            order: DrawOrder(1),
                            rect: line,
                            background: Paint::Solid(grid).into(),
                            border: Edges::all(Px(0.0)),
                            border_paint: Paint::Solid(fret_core::scene::Color::TRANSPARENT).into(),
                            corner_radii: Corners::all(Px(0.0)),
                        });
                    }

                    for y in min_y..=max_y {
                        let oy = y as f32 * step;
                        let line = Rect::new(
                            Point::new(Px(min_x as f32 * step), Px(oy)),
                            Size::new(Px((max_x - min_x) as f32 * step), Px(1.0)),
                        );
                        p.scene().push(SceneOp::Quad {
                            order: DrawOrder(1),
                            rect: line,
                            background: Paint::Solid(grid).into(),
                            border: Edges::all(Px(0.0)),
                            border_paint: Paint::Solid(fret_core::scene::Color::TRANSPARENT).into(),
                            corner_radii: Corners::all(Px(0.0)),
                        });
                    }
                });
            });
        }
    };

    let overlay_clicks_c = overlay_clicks.clone();
    let node_clicks_c = node_clicks.clone();
    let bounds_store_c = bounds_store.clone();
    let connections_c = connections.clone();

    let bounds_store_for_marquee = bounds_store.clone();
    let selected_count_for_marquee = selected_count.clone();
    let on_marquee_commit: OnCanvasMarqueeCommit = Arc::new(move |host, action_cx, commit| {
        let count = host
            .models_mut()
            .read(&bounds_store_for_marquee, |st| {
                let mut count = 0u64;
                for item in st.items.values() {
                    let (ax0, ay0, ax1, ay1) = (
                        item.canvas_bounds.origin.x.0,
                        item.canvas_bounds.origin.y.0,
                        item.canvas_bounds.origin.x.0 + item.canvas_bounds.size.width.0,
                        item.canvas_bounds.origin.y.0 + item.canvas_bounds.size.height.0,
                    );
                    let (bx0, by0, bx1, by1) = (
                        commit.rect_canvas.origin.x.0,
                        commit.rect_canvas.origin.y.0,
                        commit.rect_canvas.origin.x.0 + commit.rect_canvas.size.width.0,
                        commit.rect_canvas.origin.y.0 + commit.rect_canvas.size.height.0,
                    );
                    let intersects = ax0 < bx1 && ax1 > bx0 && ay0 < by1 && ay1 > by0;
                    if intersects {
                        count = count.saturating_add(1);
                    }
                }
                count
            })
            .ok()
            .unwrap_or(0);

        let _ = host
            .models_mut()
            .update(&selected_count_for_marquee, |v| *v = count);
        host.request_redraw(action_cx.window);
    });

    let view_for_marquee_start_filter = view.clone();
    let bounds_store_for_marquee_start_filter = bounds_store.clone();
    let node_a_screen_bounds_for_marquee_start_filter = node_a_screen_bounds.clone();
    let node_b_screen_bounds_for_marquee_start_filter = node_b_screen_bounds.clone();
    let blocked_for_marquee_start_filter = marquee_blocked_count.clone();
    let marquee_start_filter: OnCanvasMarqueeStart = Arc::new(move |host, action_cx, down| {
        let hit_node = {
            let a = host
                .models_mut()
                .read(&node_a_screen_bounds_for_marquee_start_filter, |st| *st)
                .ok()
                .flatten();
            let b = host
                .models_mut()
                .read(&node_b_screen_bounds_for_marquee_start_filter, |st| *st)
                .ok()
                .flatten();

            let hit_rect = |r: Rect, p: Point| {
                let x0 = r.origin.x.0;
                let y0 = r.origin.y.0;
                let x1 = x0 + r.size.width.0;
                let y1 = y0 + r.size.height.0;
                p.x.0 >= x0 && p.x.0 <= x1 && p.y.0 >= y0 && p.y.0 <= y1
            };

            a.is_some_and(|r| hit_rect(r, down.position))
                || b.is_some_and(|r| hit_rect(r, down.position))
        };

        let hit_node = if hit_node {
            true
        } else {
            let bounds = host.bounds();
            let view = host
                .models_mut()
                .read(&view_for_marquee_start_filter, |v| *v)
                .ok()
                .unwrap_or_default();
            let p_canvas = view.screen_to_canvas(bounds, down.position);

            host.models_mut()
                .read(&bounds_store_for_marquee_start_filter, |st| {
                    st.items.values().any(|item| {
                        let x0 = item.canvas_bounds.origin.x.0;
                        let y0 = item.canvas_bounds.origin.y.0;
                        let x1 = x0 + item.canvas_bounds.size.width.0;
                        let y1 = y0 + item.canvas_bounds.size.height.0;
                        p_canvas.x.0 >= x0
                            && p_canvas.x.0 <= x1
                            && p_canvas.y.0 >= y0
                            && p_canvas.y.0 <= y1
                    })
                })
                .ok()
                .unwrap_or(false)
        };

        if hit_node {
            let _ = host
                .models_mut()
                .update(&blocked_for_marquee_start_filter, |v| {
                    *v = v.saturating_add(1)
                });
            host.request_redraw(action_cx.window);
            return false;
        }

        true
    });

    let mut marquee = CanvasMarqueeSelectionProps::default();
    marquee.on_commit = Some(on_marquee_commit);
    marquee.start_filter = Some(marquee_start_filter);

    let node_a_screen_bounds_for_overlay = node_a_screen_bounds.clone();
    let node_b_screen_bounds_for_overlay = node_b_screen_bounds.clone();
    let connect_drag_started_for_world = connect_drag_started_count.clone();
    let connect_drag_up_for_world = connect_drag_up_count.clone();
    let connect_drag_hit_for_world = connect_drag_hit_count.clone();

    let world = canvas_world_surface_panel_with_marquee_selection(
        cx,
        world_props,
        marquee,
        paint,
        move |cx, world_cx| {
            #[derive(Debug, Clone, Copy, PartialEq)]
            struct NodeDragState {
                pointer_id: fret_core::PointerId,
                start_screen: Point,
                current_screen: Point,
                start_canvas: Point,
                view_at_start: PanZoom2D,
                modifiers: fret_core::Modifiers,
                active: bool,
            }

            #[derive(Debug, Clone, Copy, PartialEq)]
            struct ConnectDragState {
                pointer_id: fret_core::PointerId,
                from_key: u64,
                view_at_start: PanZoom2D,
                scale_mode_at_start: CanvasWorldScaleMode,
                start_screen: Point,
                current_screen: Point,
                from_canvas: Point,
                from_local: Point,
                current_local: Point,
                active: bool,
            }

            let connect_drag_state: fret_runtime::Model<Option<ConnectDragState>> =
                use_controllable_model(cx, None, || None).model();

            let on_node_activate: OnActivate = Arc::new(move |host, action_cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&node_clicks_c, |v| *v = v.saturating_add(1));
                host.request_redraw(action_cx.window);
            });

            let a_canvas = cx
                .get_model_copied(&node_a_canvas_pos_for_world, fret_ui::Invalidation::Layout)
                .unwrap_or(Point::new(Px(420.0), Px(80.0)));
            let b_canvas = cx
                .get_model_copied(&node_b_canvas_pos_for_world, fret_ui::Invalidation::Layout)
                .unwrap_or(Point::new(Px(760.0), Px(260.0)));

            let (a_left, a_top, b_left, b_top) = match world_cx.scale_mode {
                CanvasWorldScaleMode::ScaleWithZoom => {
                    (a_canvas.x, a_canvas.y, b_canvas.x, b_canvas.y)
                }
                CanvasWorldScaleMode::SemanticZoom => {
                    let a_screen = world_cx.canvas_to_screen(a_canvas);
                    let b_screen = world_cx.canvas_to_screen(b_canvas);
                    (
                        Px(a_screen.x.0 - world_cx.bounds.origin.x.0),
                        Px(a_screen.y.0 - world_cx.bounds.origin.y.0),
                        Px(b_screen.x.0 - world_cx.bounds.origin.x.0),
                        Px(b_screen.y.0 - world_cx.bounds.origin.y.0),
                    )
                }
            };

            let drag_min_px = Px(3.0);
            let drag_handle_height_px = Px(32.0);
            let surface_bounds = world_cx.bounds;
            let default_view = world_cx.view;
            let node_dragged_count_a = node_dragged_count_for_world.clone();
            let node_dragged_count_b = node_dragged_count_for_world.clone();

            let anchor_right_center = |r: Rect| -> Point {
                Point::new(
                    Px(r.origin.x.0 + r.size.width.0),
                    Px(r.origin.y.0 + r.size.height.0 * 0.5),
                )
            };
            let anchor_left_center = |r: Rect| -> Point {
                Point::new(r.origin.x, Px(r.origin.y.0 + r.size.height.0 * 0.5))
            };

            let canvas_point_to_local =
                |world_cx: &fret_canvas::ui::CanvasWorldPaintCx, canvas: Point| -> Point {
                    match world_cx.scale_mode {
                        CanvasWorldScaleMode::ScaleWithZoom => canvas,
                        CanvasWorldScaleMode::SemanticZoom => {
                            let screen = world_cx.canvas_to_screen(canvas);
                            Point::new(
                                Px(screen.x.0 - world_cx.bounds.origin.x.0),
                                Px(screen.y.0 - world_cx.bounds.origin.y.0),
                            )
                        }
                    }
                };

            let committed_connections: Vec<(u64, u64)> = cx
                .read_model_ref(&connections_c, fret_ui::Invalidation::Layout, |v| v.clone())
                .unwrap_or_default();

            let connection_layer_layout = LayoutRefinement::default()
                .absolute()
                .inset_px(Px(0.0))
                .w_full()
                .h_full()
                .min_w_0()
                .min_h_0();

            let bounds_store_for_connections_layer = bounds_store_c.clone();
            let bounds_store_for_node_a = bounds_store_c.clone();
            let bounds_store_for_node_b = bounds_store_c.clone();

            let connections_layer = {
                let (a_bounds, b_bounds) = cx
                    .read_model_ref(
                        &bounds_store_for_connections_layer,
                        fret_ui::Invalidation::Layout,
                        |st| {
                            (
                                st.items.get(&1).map(|i| i.canvas_bounds),
                                st.items.get(&2).map(|i| i.canvas_bounds),
                            )
                        },
                    )
                    .unwrap_or((None, None));

                let mut out: Vec<AnyElement> = Vec::new();

                for (from_key, to_key) in committed_connections.iter().copied() {
                    let (from_bounds, to_bounds) = match (from_key, to_key) {
                        (1, 2) => (a_bounds, b_bounds),
                        (2, 1) => (b_bounds, a_bounds),
                        _ => (None, None),
                    };

                    let Some(from_bounds) = from_bounds else {
                        continue;
                    };
                    let Some(to_bounds) = to_bounds else {
                        continue;
                    };

                    let from_canvas = anchor_right_center(from_bounds);
                    let to_canvas = anchor_left_center(to_bounds);

                    let from_local = canvas_point_to_local(&world_cx, from_canvas);
                    let to_local = canvas_point_to_local(&world_cx, to_canvas);

                    let conn = ui_ai::WorkflowConnection::new(from_local, to_local)
                        .stroke_width(Px(2.0))
                        .refine_layout(connection_layer_layout.clone());
                    out.push(conn.into_element(cx));
                }

                out
            };

            let preview_layer = {
                let drag: Option<ConnectDragState> = cx
                    .get_model_copied(&connect_drag_state, fret_ui::Invalidation::Layout)
                    .unwrap_or(None);
                match drag {
                    None => Vec::new(),
                    Some(drag) if !drag.active => Vec::new(),
                    Some(drag) => {
                        vec![
                            ui_ai::WorkflowConnection::new(drag.from_local, drag.current_local)
                                .stroke_width(Px(2.0))
                                .test_id("ui-ai-cwl-connection-preview")
                                .refine_layout(connection_layer_layout.clone())
                                .into_element(cx),
                        ]
                    }
                }
            };

            let connect_drag_state_for_node_a = connect_drag_state.clone();
            let connect_drag_started_for_node_a = connect_drag_started_for_world.clone();
            let connect_drag_up_for_node_a = connect_drag_up_for_world.clone();
            let connect_drag_hit_for_node_a = connect_drag_hit_for_world.clone();
            let bounds_store_for_connect_drop = bounds_store_c.clone();
            let connections_for_connect_drop = connections_c.clone();

            let node_a_item = {
                let item = canvas_world_bounds_item(
                    cx,
                    bounds_store_for_node_a.clone(),
                    1,
                    world_cx,
                    move |cx| {
                    let drag_state: fret_runtime::Model<Option<NodeDragState>> =
                        use_controllable_model(cx, None, || None).model();

                    let node_a = ui_ai::WorkflowNode::new([
                        ui_ai::WorkflowNodeHeader::new([
                            ui_ai::WorkflowNodeTitle::new("Node A").into_element(cx)
                        ])
                        .into_element(cx),
                        ui_ai::WorkflowNodeContent::new([
                            cx.text(format!("Clicks: {node_clicks_value}"))
                        ])
                        .into_element(cx),
                        ui_ai::WorkflowNodeFooter::new([shadcn::Button::new("Click node")
                            .test_id("ui-ai-cwl-node-click")
                            .on_activate(on_node_activate.clone())
                            .variant(shadcn::ButtonVariant::Secondary)
                            .into_element(cx)])
                        .into_element(cx),
                    ])
                    .handles(ui_ai::WorkflowNodeHandles {
                        source: true,
                        target: false,
                    })
                    .test_id("ui-ai-cwl-node-a")
                    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx);

                    let mut handle_props = PointerRegionProps::default();
                    handle_props.layout.position = fret_ui::element::PositionStyle::Absolute;
                    handle_props.layout.inset = fret_ui::element::InsetStyle {
                        top: Some(Px(0.0)).into(),
                        left: Some(Px(0.0)).into(),
                        right: Some(Px(0.0)).into(),
                        ..Default::default()
                    };
                    handle_props.layout.size.width = Length::Fill;
                    handle_props.layout.size.height = Length::Px(drag_handle_height_px);

                    let on_down_state = drag_state.clone();
                    let node_a_canvas_pos_for_down = node_a_canvas_pos_for_world.clone();
                    let on_down: fret_ui::action::OnPointerDown =
                        Arc::new(move |host, action_cx, down| {
                            if down.button != fret_core::MouseButton::Left {
                                return false;
                            }

                            host.capture_pointer();
                            let start_canvas = host
                                .models_mut()
                                .read(&node_a_canvas_pos_for_down, |p| *p)
                                .ok()
                                .unwrap_or(Point::new(Px(0.0), Px(0.0)));
                            let _ = host.models_mut().update(&on_down_state, |st| {
                                *st = Some(NodeDragState {
                                    pointer_id: down.pointer_id,
                                    start_screen: down.position,
                                    current_screen: down.position,
                                    start_canvas,
                                    view_at_start: default_view,
                                    modifiers: down.modifiers,
                                    active: false,
                                });
                            });
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let on_move_state = drag_state.clone();
                    let node_a_canvas_pos_for_move = node_a_canvas_pos_for_world.clone();
                    let on_move: fret_ui::action::OnPointerMove =
                        Arc::new(move |host, action_cx, mv| {
                            let mut drag = host
                                .models_mut()
                                .read(&on_move_state, |st| *st)
                                .ok()
                                .flatten();
                            let Some(mut drag) = drag.take() else {
                                return false;
                            };
                            if mv.pointer_id != drag.pointer_id {
                                return false;
                            }
                            if !mv.buttons.left {
                                host.release_pointer_capture();
                                let _ = host.models_mut().update(&on_move_state, |st| *st = None);
                                host.request_redraw(action_cx.window);
                                return true;
                            }

                            drag.current_screen = mv.position;
                            if !drag.active {
                                let dx = drag.current_screen.x.0 - drag.start_screen.x.0;
                                let dy = drag.current_screen.y.0 - drag.start_screen.y.0;
                                let dist_sq = dx * dx + dy * dy;
                                if dist_sq >= drag_min_px.0 * drag_min_px.0 {
                                    drag.active = true;
                                }
                            }

                            if drag.active {
                                let c0 = drag
                                    .view_at_start
                                    .screen_to_canvas(surface_bounds, drag.start_screen);
                                let c1 = drag
                                    .view_at_start
                                    .screen_to_canvas(surface_bounds, drag.current_screen);
                                let delta = Point::new(Px(c1.x.0 - c0.x.0), Px(c1.y.0 - c0.y.0));
                                let next = Point::new(
                                    Px(drag.start_canvas.x.0 + delta.x.0),
                                    Px(drag.start_canvas.y.0 + delta.y.0),
                                );
                                let _ = host
                                    .models_mut()
                                    .update(&node_a_canvas_pos_for_move, |p| *p = next);
                            }

                            let _ = host
                                .models_mut()
                                .update(&on_move_state, |st| *st = Some(drag));
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let on_up_state = drag_state.clone();
                    let node_dragged_count = node_dragged_count_a.clone();
                    let on_up: fret_ui::action::OnPointerUp =
                        Arc::new(move |host, action_cx, up| {
                            let drag = host
                                .models_mut()
                                .read(&on_up_state, |st| *st)
                                .ok()
                                .flatten();
                            let Some(drag) = drag else {
                                return false;
                            };
                            if up.pointer_id != drag.pointer_id {
                                return false;
                            }
                            if up.button != fret_core::MouseButton::Left {
                                return false;
                            }

                            host.release_pointer_capture();
                            let _ = host.models_mut().update(&on_up_state, |st| *st = None);
                            if drag.active {
                                let _ = host
                                    .models_mut()
                                    .update(&node_dragged_count, |v| *v = v.saturating_add(1));
                            }
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let handle = cx
                        .pointer_region(handle_props, move |cx| {
                            cx.pointer_region_on_pointer_down(on_down.clone());
                            cx.pointer_region_on_pointer_move(on_move.clone());
                            cx.pointer_region_on_pointer_up(on_up.clone());
                            std::iter::empty()
                        })
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Group)
                                .test_id("ui-ai-cwl-node-a-drag-handle"),
                        );

                    let mut source_handle_props = PointerRegionProps::default();
                    source_handle_props.layout.position = fret_ui::element::PositionStyle::Absolute;
                    source_handle_props.layout.inset = fret_ui::element::InsetStyle {
                        top: Some(Px(56.0)).into(),
                        right: Some(Px(0.0)).into(),
                        ..Default::default()
                    };
                    source_handle_props.layout.size.width = Length::Px(Px(20.0));
                    source_handle_props.layout.size.height = Length::Px(Px(20.0));

                    let connect_down_state = connect_drag_state_for_node_a.clone();
                    let connect_move_state = connect_drag_state_for_node_a.clone();
                    let connect_up_state = connect_drag_state_for_node_a.clone();

                    let connect_bounds_store_for_down = bounds_store_for_node_a.clone();
                    let connect_bounds_store_for_drop = bounds_store_for_connect_drop.clone();
                    let connect_connections_for_drop = connections_for_connect_drop.clone();

                    let connect_started_count = connect_drag_started_for_node_a.clone();
                    let connect_up_count = connect_drag_up_for_node_a.clone();
                    let connect_hit_count = connect_drag_hit_for_node_a.clone();

                    let connect_scale_mode_at_start = world_cx.scale_mode;

                    let on_connect_down: fret_ui::action::OnPointerDown =
                        Arc::new(move |host, action_cx, down| {
                            if down.button != fret_core::MouseButton::Left {
                                return false;
                            }

                            host.capture_pointer();

                            let from_canvas = host
                                .models_mut()
                                .read(&connect_bounds_store_for_down, |st| {
                                    st.items
                                        .get(&1u64)
                                        .map(|i| anchor_right_center(i.canvas_bounds))
                                })
                                .ok()
                                .flatten()
                                .unwrap_or(Point::new(Px(0.0), Px(0.0)));

                            let _ = host
                                .models_mut()
                                .update(&connect_started_count, |v| *v = v.saturating_add(1));

                            let from_local = match connect_scale_mode_at_start {
                                CanvasWorldScaleMode::ScaleWithZoom => from_canvas,
                                CanvasWorldScaleMode::SemanticZoom => {
                                    let screen =
                                        default_view.canvas_to_screen(surface_bounds, from_canvas);
                                    Point::new(
                                        Px(screen.x.0 - surface_bounds.origin.x.0),
                                        Px(screen.y.0 - surface_bounds.origin.y.0),
                                    )
                                }
                            };

                            let current_local = match connect_scale_mode_at_start {
                                CanvasWorldScaleMode::ScaleWithZoom => default_view
                                    .screen_to_canvas(surface_bounds, down.position),
                                CanvasWorldScaleMode::SemanticZoom => Point::new(
                                    Px(down.position.x.0 - surface_bounds.origin.x.0),
                                    Px(down.position.y.0 - surface_bounds.origin.y.0),
                                ),
                            };

                            let _ = host.models_mut().update(&connect_down_state, |st| {
                                *st = Some(ConnectDragState {
                                    pointer_id: down.pointer_id,
                                    from_key: 1,
                                    view_at_start: default_view,
                                    scale_mode_at_start: connect_scale_mode_at_start,
                                    start_screen: down.position,
                                    current_screen: down.position,
                                    from_canvas,
                                    from_local,
                                    current_local,
                                    active: true,
                                });
                            });
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let on_connect_move: fret_ui::action::OnPointerMove =
                        Arc::new(move |host, action_cx, mv| {
                            let mut drag = host
                                .models_mut()
                                .read(&connect_move_state, |st| *st)
                                .ok()
                                .flatten();
                            let Some(mut drag) = drag.take() else {
                                return false;
                            };
                            if mv.pointer_id != drag.pointer_id {
                                return false;
                            }
                            if !mv.buttons.left {
                                host.release_pointer_capture();
                                let _ = host
                                    .models_mut()
                                    .update(&connect_move_state, |st| *st = None);
                                host.request_redraw(action_cx.window);
                                return true;
                            }

                            drag.current_screen = mv.position;

                            drag.current_local = match drag.scale_mode_at_start {
                                CanvasWorldScaleMode::ScaleWithZoom => drag
                                    .view_at_start
                                    .screen_to_canvas(surface_bounds, drag.current_screen),
                                CanvasWorldScaleMode::SemanticZoom => Point::new(
                                    Px(drag.current_screen.x.0 - surface_bounds.origin.x.0),
                                    Px(drag.current_screen.y.0 - surface_bounds.origin.y.0),
                                ),
                            };
                            let _ = host
                                .models_mut()
                                .update(&connect_move_state, |st| *st = Some(drag));
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let on_connect_up: fret_ui::action::OnPointerUp =
                        Arc::new(move |host, action_cx, up| {
                            if up.button != fret_core::MouseButton::Left {
                                return false;
                            }

                            let drag = host
                                .models_mut()
                                .read(&connect_up_state, |st| *st)
                                .ok()
                                .flatten();
                            let Some(drag) = drag else {
                                return false;
                            };
                            if up.pointer_id != drag.pointer_id {
                                return false;
                            }

                            let _ = host
                                .models_mut()
                                .update(&connect_up_count, |v| *v = v.saturating_add(1));

                            if drag.active {
                                let target = host
                                    .models_mut()
                                    .read(&connect_bounds_store_for_drop, |st| {
                                        st.items.get(&2u64).map(|i| i.canvas_bounds)
                                    })
                                    .ok()
                                    .flatten();

                                if let Some(target_canvas) = target {
                                    let hit_slop_screen_px = 12.0f32;
                                    let p0_screen =
                                        drag.view_at_start.canvas_to_screen(surface_bounds, target_canvas.origin);
                                    let p1_screen = drag.view_at_start.canvas_to_screen(
                                        surface_bounds,
                                        Point::new(
                                            Px(target_canvas.origin.x.0 + target_canvas.size.width.0),
                                            Px(target_canvas.origin.y.0 + target_canvas.size.height.0),
                                        ),
                                    );

                                    let x0 = p0_screen.x.0.min(p1_screen.x.0) - hit_slop_screen_px;
                                    let y0 = p0_screen.y.0.min(p1_screen.y.0) - hit_slop_screen_px;
                                    let x1 = p0_screen.x.0.max(p1_screen.x.0) + hit_slop_screen_px;
                                    let y1 = p0_screen.y.0.max(p1_screen.y.0) + hit_slop_screen_px;

                                    let px = up.position.x.0;
                                    let py = up.position.y.0;
                                    let hit_target = px >= x0 && px <= x1 && py >= y0 && py <= y1;

                                    if hit_target {
                                        let _ = host
                                            .models_mut()
                                            .update(&connect_hit_count, |v| *v = v.saturating_add(1));
                                        let _ = host
                                            .models_mut()
                                            .update(&connect_connections_for_drop, |v| {
                                                let key = (drag.from_key, 2u64);
                                                if !v.contains(&key) {
                                                    v.push(key);
                                                }
                                            });
                                    }
                                }
                            }

                            host.release_pointer_capture();
                            let _ = host
                                .models_mut()
                                .update(&connect_up_state, |st| *st = None);
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let source_handle = cx
                        .pointer_region(source_handle_props, move |cx| {
                            cx.pointer_region_on_pointer_down(on_connect_down.clone());
                            cx.pointer_region_on_pointer_move(on_connect_move.clone());
                            cx.pointer_region_on_pointer_up(on_connect_up.clone());
                            std::iter::empty()
                        })
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Button)
                                .test_id("ui-ai-cwl-node-a-source-handle"),
                        );

                    let mut inner = fret_ui::element::ContainerProps::default();
                    inner.layout.size.width = Length::Px(Px(260.0));
                    vec![cx.container(inner, move |_cx| [node_a, handle, source_handle])]
                },
                );

                let mut wrapper = fret_ui::element::LayoutQueryRegionProps::default();
                wrapper.layout.position = fret_ui::element::PositionStyle::Absolute;
                wrapper.layout.inset.left = Some(a_left).into();
                wrapper.layout.inset.top = Some(a_top).into();
                wrapper.layout.size.width = Length::Px(Px(260.0));

                cx.layout_query_region(wrapper, move |_cx| [item])
            };

            let node_b_item = {
                let item = canvas_world_bounds_item(
                    cx,
                    bounds_store_for_node_b.clone(),
                    2,
                    world_cx,
                    move |cx| {
                    let drag_state: fret_runtime::Model<Option<NodeDragState>> =
                        use_controllable_model(cx, None, || None).model();

                    let node_b = ui_ai::WorkflowNode::new([
                        ui_ai::WorkflowNodeHeader::new([
                            ui_ai::WorkflowNodeTitle::new("Node B").into_element(cx)
                        ])
                        .into_element(cx),
                        ui_ai::WorkflowNodeContent::new([
                            cx.text("Try zooming/panning and click again.")
                        ])
                        .into_element(cx),
                    ])
                    .handles(ui_ai::WorkflowNodeHandles {
                        source: false,
                        target: true,
                    })
                    .test_id("ui-ai-cwl-node-b")
                    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx);

                    let mut handle_props = PointerRegionProps::default();
                    handle_props.layout.position = fret_ui::element::PositionStyle::Absolute;
                    handle_props.layout.inset = fret_ui::element::InsetStyle {
                        top: Some(Px(0.0)).into(),
                        left: Some(Px(0.0)).into(),
                        right: Some(Px(0.0)).into(),
                        ..Default::default()
                    };
                    handle_props.layout.size.width = Length::Fill;
                    handle_props.layout.size.height = Length::Px(drag_handle_height_px);

                    let on_down_state = drag_state.clone();
                    let node_b_canvas_pos_for_down = node_b_canvas_pos_for_world.clone();
                    let on_down: fret_ui::action::OnPointerDown =
                        Arc::new(move |host, action_cx, down| {
                            if down.button != fret_core::MouseButton::Left {
                                return false;
                            }

                            host.capture_pointer();
                            let start_canvas = host
                                .models_mut()
                                .read(&node_b_canvas_pos_for_down, |p| *p)
                                .ok()
                                .unwrap_or(Point::new(Px(0.0), Px(0.0)));
                            let _ = host.models_mut().update(&on_down_state, |st| {
                                *st = Some(NodeDragState {
                                    pointer_id: down.pointer_id,
                                    start_screen: down.position,
                                    current_screen: down.position,
                                    start_canvas,
                                    view_at_start: default_view,
                                    modifiers: down.modifiers,
                                    active: false,
                                });
                            });
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let on_move_state = drag_state.clone();
                    let node_b_canvas_pos_for_move = node_b_canvas_pos_for_world.clone();
                    let on_move: fret_ui::action::OnPointerMove =
                        Arc::new(move |host, action_cx, mv| {
                            let mut drag = host
                                .models_mut()
                                .read(&on_move_state, |st| *st)
                                .ok()
                                .flatten();
                            let Some(mut drag) = drag.take() else {
                                return false;
                            };
                            if mv.pointer_id != drag.pointer_id {
                                return false;
                            }
                            if !mv.buttons.left {
                                host.release_pointer_capture();
                                let _ = host.models_mut().update(&on_move_state, |st| *st = None);
                                host.request_redraw(action_cx.window);
                                return true;
                            }

                            drag.current_screen = mv.position;
                            if !drag.active {
                                let dx = drag.current_screen.x.0 - drag.start_screen.x.0;
                                let dy = drag.current_screen.y.0 - drag.start_screen.y.0;
                                let dist_sq = dx * dx + dy * dy;
                                if dist_sq >= drag_min_px.0 * drag_min_px.0 {
                                    drag.active = true;
                                }
                            }

                            if drag.active {
                                let c0 = drag
                                    .view_at_start
                                    .screen_to_canvas(surface_bounds, drag.start_screen);
                                let c1 = drag
                                    .view_at_start
                                    .screen_to_canvas(surface_bounds, drag.current_screen);
                                let delta = Point::new(Px(c1.x.0 - c0.x.0), Px(c1.y.0 - c0.y.0));
                                let next = Point::new(
                                    Px(drag.start_canvas.x.0 + delta.x.0),
                                    Px(drag.start_canvas.y.0 + delta.y.0),
                                );
                                let _ = host
                                    .models_mut()
                                    .update(&node_b_canvas_pos_for_move, |p| *p = next);
                            }

                            let _ = host
                                .models_mut()
                                .update(&on_move_state, |st| *st = Some(drag));
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let on_up_state = drag_state.clone();
                    let node_dragged_count = node_dragged_count_b.clone();
                    let on_up: fret_ui::action::OnPointerUp =
                        Arc::new(move |host, action_cx, up| {
                            let drag = host
                                .models_mut()
                                .read(&on_up_state, |st| *st)
                                .ok()
                                .flatten();
                            let Some(drag) = drag else {
                                return false;
                            };
                            if up.pointer_id != drag.pointer_id {
                                return false;
                            }
                            if up.button != fret_core::MouseButton::Left {
                                return false;
                            }

                            host.release_pointer_capture();
                            let _ = host.models_mut().update(&on_up_state, |st| *st = None);
                            if drag.active {
                                let _ = host
                                    .models_mut()
                                    .update(&node_dragged_count, |v| *v = v.saturating_add(1));
                            }
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let handle = cx
                        .pointer_region(handle_props, move |cx| {
                            cx.pointer_region_on_pointer_down(on_down.clone());
                            cx.pointer_region_on_pointer_move(on_move.clone());
                            cx.pointer_region_on_pointer_up(on_up.clone());
                            std::iter::empty()
                        })
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Group)
                                .test_id("ui-ai-cwl-node-b-drag-handle"),
                        );

                    let mut target_handle_props = PointerRegionProps::default();
                    target_handle_props.layout.position = fret_ui::element::PositionStyle::Absolute;
                    target_handle_props.layout.inset = fret_ui::element::InsetStyle {
                        top: Some(Px(56.0)).into(),
                        left: Some(Px(0.0)).into(),
                        ..Default::default()
                    };
                    target_handle_props.layout.size.width = Length::Px(Px(20.0));
                    target_handle_props.layout.size.height = Length::Px(Px(20.0));

                    let target_handle = cx
                        .pointer_region(target_handle_props, move |_cx| std::iter::empty())
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Button)
                                .test_id("ui-ai-cwl-node-b-target-handle"),
                        );

                    let mut inner = fret_ui::element::ContainerProps::default();
                    inner.layout.size.width = Length::Px(Px(260.0));
                    vec![cx.container(inner, move |_cx| [node_b, handle, target_handle])]
                },
                );

                let mut wrapper = fret_ui::element::LayoutQueryRegionProps::default();
                wrapper.layout.position = fret_ui::element::PositionStyle::Absolute;
                wrapper.layout.inset.left = Some(b_left).into();
                wrapper.layout.inset.top = Some(b_top).into();
                wrapper.layout.size.width = Length::Px(Px(260.0));

                cx.layout_query_region(wrapper, move |_cx| [item])
            };

            let mut layer = fret_ui::element::ContainerProps::default();
            layer.layout.size.width = Length::Fill;
            layer.layout.size.height = Length::Fill;
            layer.layout.position = fret_ui::element::PositionStyle::Relative;

            let children = connections_layer
                .into_iter()
                .chain(preview_layer)
                .chain([node_a_item, node_b_item])
                .collect::<Vec<_>>();
            vec![cx.container(layer, move |_cx| children)]
        },
        move |cx, world_cx| {
            #[derive(Default)]
            struct BoundsStabilityState {
                last: Option<(Rect, Rect)>,
                stable_frames: u32,
            }

            let connect_drag_state_for_overlay_down = connect_overlay_drag_state.clone();
            let connect_drag_state_for_overlay_move = connect_overlay_drag_state.clone();
            let connect_drag_state_for_overlay_up = connect_overlay_drag_state.clone();
            let connect_started_count_for_overlay = connect_drag_started_count.clone();
            let connect_up_count_for_overlay = connect_drag_up_count.clone();
            let connect_hit_count_for_overlay = connect_drag_hit_count.clone();
            let bounds_store_for_overlay = bounds_store.clone();
            let connections_for_overlay = connections.clone();
            let node_b_screen_bounds_for_connect = node_b_screen_bounds.clone();

            let overlay_surface_bounds = world_cx.bounds;
            let overlay_view = world_cx.view;
            let overlay_scale_mode = world_cx.scale_mode;

            let handle_width_screen_px = 20.0f32;
            let hit_slop_screen_px = 12.0f32;

            let mut connect_region_props = PointerRegionProps::default();
            connect_region_props.layout.position = fret_ui::element::PositionStyle::Absolute;
            connect_region_props.layout.inset = fret_ui::element::InsetStyle {
                top: Some(Px(0.0)).into(),
                left: Some(Px(0.0)).into(),
                ..Default::default()
            };

            let node_a_element = cx
                .read_model_ref(&bounds_store_for_overlay, fret_ui::Invalidation::Layout, |st| {
                    st.items.get(&1u64).map(|i| i.element)
                })
                .unwrap_or(None);
            let node_a_screen_bounds = node_a_element.and_then(|el| {
                cx.last_visual_bounds_for_element(el)
                    .or_else(|| cx.last_bounds_for_element(el))
            });

            let (connect_left_px, connect_top_px, connect_w_px, connect_h_px, connect_enabled) =
                if let Some(r) = node_a_screen_bounds {
                    let zoom = PanZoom2D::sanitize_zoom(overlay_view.zoom, 1.0);
                    let handle_width_screen_px = match overlay_scale_mode {
                        CanvasWorldScaleMode::ScaleWithZoom => handle_width_screen_px * zoom,
                        CanvasWorldScaleMode::SemanticZoom => handle_width_screen_px,
                    };

                    let sx0 = (r.origin.x.0 + r.size.width.0.max(0.0) - handle_width_screen_px)
                        .min(r.origin.x.0 + r.size.width.0.max(0.0));
                    let sy0 = r.origin.y.0;
                    let sx1 = r.origin.x.0 + r.size.width.0.max(0.0);
                    let sy1 = r.origin.y.0 + r.size.height.0.max(0.0);

                    let ix0 = sx0.max(overlay_surface_bounds.origin.x.0);
                    let iy0 = sy0.max(overlay_surface_bounds.origin.y.0);
                    let ix1 = sx1.min(
                        overlay_surface_bounds.origin.x.0 + overlay_surface_bounds.size.width.0,
                    );
                    let iy1 = sy1.min(
                        overlay_surface_bounds.origin.y.0 + overlay_surface_bounds.size.height.0,
                    );

                    if ix1 <= ix0 || iy1 <= iy0 {
                        (0.0, 0.0, 0.0, 0.0, false)
                    } else {
                        (
                            (ix0 - overlay_surface_bounds.origin.x.0).max(0.0),
                            (iy0 - overlay_surface_bounds.origin.y.0).max(0.0),
                            (ix1 - ix0).max(0.0),
                            (iy1 - iy0).max(0.0),
                            true,
                        )
                    }
                } else {
                    (0.0, 0.0, 0.0, 0.0, false)
                };

            connect_region_props.layout.inset.left = Some(Px(connect_left_px)).into();
            connect_region_props.layout.inset.top = Some(Px(connect_top_px)).into();
            connect_region_props.layout.size.width = Length::Px(Px(connect_w_px));
            connect_region_props.layout.size.height = Length::Px(Px(connect_h_px));
            connect_region_props.enabled = connect_enabled;

            let on_connect_down_overlay: fret_ui::action::OnPointerDown =
                Arc::new(move |host, action_cx, down| {
                    if down.button != fret_core::MouseButton::Left {
                        return false;
                    }

                    host.capture_pointer();
                    let _ = host
                        .models_mut()
                        .update(&connect_started_count_for_overlay, |v| *v = v.saturating_add(1));
                    let _ = host.models_mut().update(&connect_drag_state_for_overlay_down, |st| {
                        *st = Some(ConnectOverlayDragState {
                            pointer_id: down.pointer_id,
                            view_at_start: overlay_view,
                            surface_bounds_at_start: overlay_surface_bounds,
                            scale_mode_at_start: overlay_scale_mode,
                        });
                    });
                    host.request_redraw(action_cx.window);
                    true
                });

            let on_connect_move_overlay: fret_ui::action::OnPointerMove =
                Arc::new(move |host, action_cx, mv| {
                    let drag = host
                        .models_mut()
                        .read(&connect_drag_state_for_overlay_move, |st| *st)
                        .ok()
                        .flatten();
                    let Some(drag) = drag else {
                        return false;
                    };
                    if mv.pointer_id != drag.pointer_id {
                        return false;
                    }
                    if !mv.buttons.left {
                        host.release_pointer_capture();
                        let _ = host
                            .models_mut()
                            .update(&connect_drag_state_for_overlay_move, |st| *st = None);
                        host.request_redraw(action_cx.window);
                        return true;
                    }
                    host.request_redraw(action_cx.window);
                    true
                });

            let on_connect_up_overlay: fret_ui::action::OnPointerUp =
                Arc::new(move |host, action_cx, up| {
                    if up.button != fret_core::MouseButton::Left {
                        return false;
                    }

                    let drag = host
                        .models_mut()
                        .read(&connect_drag_state_for_overlay_up, |st| *st)
                        .ok()
                        .flatten();
                    let Some(drag) = drag else {
                        return false;
                    };
                    if up.pointer_id != drag.pointer_id {
                        return false;
                    }

                    let _ = host
                        .models_mut()
                        .update(&connect_up_count_for_overlay, |v| *v = v.saturating_add(1));

                    let node_b_screen = host
                        .models_mut()
                        .read(&node_b_screen_bounds_for_connect, |st| *st)
                        .ok()
                        .flatten();

                    if let Some(node_b_screen) = node_b_screen {
                        let zoom = PanZoom2D::sanitize_zoom(drag.view_at_start.zoom, 1.0);
                        let handle_width_screen = match drag.scale_mode_at_start {
                            CanvasWorldScaleMode::ScaleWithZoom => handle_width_screen_px * zoom,
                            CanvasWorldScaleMode::SemanticZoom => handle_width_screen_px,
                        };

                        let x0 = node_b_screen.origin.x.0 - hit_slop_screen_px;
                        let y0 = node_b_screen.origin.y.0 - hit_slop_screen_px;
                        let x1 = node_b_screen.origin.x.0 + handle_width_screen + hit_slop_screen_px;
                        let y1 = node_b_screen.origin.y.0
                            + node_b_screen.size.height.0
                            + hit_slop_screen_px;

                        let px = up.position.x.0;
                        let py = up.position.y.0;
                        let hit_target = px >= x0 && px <= x1 && py >= y0 && py <= y1;

                        if hit_target {
                            let _ = host.models_mut().update(&connect_hit_count_for_overlay, |v| {
                                *v = v.saturating_add(1)
                            });
                            let _ = host.models_mut().update(&connections_for_overlay, |v| {
                                let key = (1u64, 2u64);
                                if !v.contains(&key) {
                                    v.push(key);
                                }
                            });
                        }
                    }

                    host.release_pointer_capture();
                    let _ = host
                        .models_mut()
                        .update(&connect_drag_state_for_overlay_up, |st| *st = None);
                    host.request_redraw(action_cx.window);
                    true
                });

            let connect_region = cx.pointer_region(connect_region_props, move |cx| {
                cx.pointer_region_on_pointer_down(on_connect_down_overlay.clone());
                cx.pointer_region_on_pointer_move(on_connect_move_overlay.clone());
                cx.pointer_region_on_pointer_up(on_connect_up_overlay.clone());
                std::iter::empty()
            })
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(fret_core::SemanticsRole::Group)
                    .test_id("ui-ai-cwl-connect-overlay-source-region"),
            );

            let mut overlay_region = CanvasInputExemptRegionProps::default();
            overlay_region.pointer_region.layout.position =
                fret_ui::element::PositionStyle::Absolute;
            overlay_region.pointer_region.layout.inset = fret_ui::element::InsetStyle {
                top: Some(Px(12.0)).into(),
                left: Some(Px(12.0)).into(),
                ..Default::default()
            };

            let on_overlay_activate: OnActivate = Arc::new(move |host, action_cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&overlay_clicks_c, |v| *v = v.saturating_add(1));
                host.request_redraw(action_cx.window);
            });

            let scale_mode_scale = scale_mode.clone();
            let on_mode_scale: OnActivate = Arc::new(move |host, action_cx, _reason| {
                let _ = host.models_mut().update(&scale_mode_scale, |m| {
                    *m = CanvasWorldScaleMode::ScaleWithZoom
                });
                host.request_redraw(action_cx.window);
            });

            let scale_mode_semantic = scale_mode.clone();
            let on_mode_semantic: OnActivate = Arc::new(move |host, action_cx, _reason| {
                let _ = host.models_mut().update(&scale_mode_semantic, |m| {
                    *m = CanvasWorldScaleMode::SemanticZoom
                });
                host.request_redraw(action_cx.window);
            });

            let mode_scale = shadcn::Button::new("Mode: Scale-with-zoom")
                .test_id("ui-ai-cwl-mode-scale-with-zoom")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_activate(on_mode_scale)
                .into_element(cx);

            let mode_semantic = shadcn::Button::new("Mode: Semantic zoom")
                .test_id("ui-ai-cwl-mode-semantic-zoom")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_activate(on_mode_semantic)
                .into_element(cx);

            let commit_connections = connections.clone();
            let on_commit_connection: OnActivate = Arc::new(move |host, action_cx, _reason| {
                let _ = host.models_mut().update(&commit_connections, |v| {
                    let key = (1u64, 2u64);
                    if !v.contains(&key) {
                        v.push(key);
                    }
                });
                host.request_redraw(action_cx.window);
            });
            let commit_connection = shadcn::Button::new("Commit connection")
                .test_id("ui-ai-cwl-commit-connection")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_activate(on_commit_connection)
                .into_element(cx);

            let overlay = shadcn::Button::new(format!("Overlay clicks: {overlay_clicks_value}"))
                .test_id("ui-ai-cwl-overlay-click")
                .variant(shadcn::ButtonVariant::Outline)
                .on_activate(on_overlay_activate)
                .into_element(cx);

            let rect_approx_eq = |a: Rect, b: Rect| -> bool {
                let eps = 0.25;
                (a.origin.x.0 - b.origin.x.0).abs() <= eps
                    && (a.origin.y.0 - b.origin.y.0).abs() <= eps
                    && (a.size.width.0 - b.size.width.0).abs() <= eps
                    && (a.size.height.0 - b.size.height.0).abs() <= eps
            };

            let (a_element, b_element) = cx
                .read_model_ref(&bounds_store, fret_ui::Invalidation::Layout, |st| {
                    (
                        st.items.get(&1u64).map(|i| i.element),
                        st.items.get(&2u64).map(|i| i.element),
                    )
                })
                .unwrap_or((None, None));

            let a_screen = a_element.and_then(|el| {
                cx.last_visual_bounds_for_element(el)
                    .or_else(|| cx.last_bounds_for_element(el))
            });
            let b_screen = b_element.and_then(|el| {
                cx.last_visual_bounds_for_element(el)
                    .or_else(|| cx.last_bounds_for_element(el))
            });

            if let Some(a_screen) = a_screen {
                let prev = cx
                    .app
                    .models()
                    .read(&node_a_screen_bounds_for_overlay, |st| *st)
                    .ok()
                    .flatten();
                if prev.map_or(true, |p| !rect_approx_eq(p, a_screen)) {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&node_a_screen_bounds_for_overlay, |st| *st = Some(a_screen));
                }
            }
            if let Some(b_screen) = b_screen {
                let prev = cx
                    .app
                    .models()
                    .read(&node_b_screen_bounds_for_overlay, |st| *st)
                    .ok()
                    .flatten();
                if prev.map_or(true, |p| !rect_approx_eq(p, b_screen)) {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&node_b_screen_bounds_for_overlay, |st| *st = Some(b_screen));
                }
            }

            let layout_settled = cx.slot_state(BoundsStabilityState::default, |st| {
                let (Some(a), Some(b)) = (a_screen, b_screen) else {
                    st.last = None;
                    st.stable_frames = 0;
                    return false;
                };

                let approx_same = st.last.is_some_and(|(prev_a, prev_b)| {
                    let eps = 0.25;
                    let eq = |x: f32, y: f32| (x - y).abs() <= eps;
                    eq(prev_a.origin.x.0, a.origin.x.0)
                        && eq(prev_a.origin.y.0, a.origin.y.0)
                        && eq(prev_a.size.width.0, a.size.width.0)
                        && eq(prev_a.size.height.0, a.size.height.0)
                        && eq(prev_b.origin.x.0, b.origin.x.0)
                        && eq(prev_b.origin.y.0, b.origin.y.0)
                        && eq(prev_b.size.width.0, b.size.width.0)
                        && eq(prev_b.size.height.0, b.size.height.0)
                });

                if approx_same {
                    st.stable_frames = st.stable_frames.saturating_add(1);
                } else {
                    st.last = Some((a, b));
                    st.stable_frames = 0;
                }

                st.stable_frames >= 8
            });

            if !layout_settled {
                cx.request_frame();
            }

            let layout_settled = layout_settled.then(|| {
                cx.text("Layout settled").attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .test_id("ui-ai-cwl-layout-settled"),
                )
            });

            let reset_epoch_value = cx
                .get_model_copied(&reset_epoch, fret_ui::Invalidation::Layout)
                .unwrap_or(0);
            let reset_done = (reset_epoch_value > 0).then(|| {
                cx.text("Reset done").attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Text)
                        .test_id("ui-ai-cwl-reset-done"),
                )
            });

            let reset_view = view.clone();
            let reset_scale_mode = scale_mode.clone();
            let reset_node_a = node_a_canvas_pos.clone();
            let reset_node_b = node_b_canvas_pos.clone();
            let reset_overlay_clicks = overlay_clicks.clone();
            let reset_node_clicks = node_clicks.clone();
            let reset_selected = selected_count.clone();
            let reset_blocked = marquee_blocked_count.clone();
            let reset_dragged = node_dragged_count.clone();
            let reset_connections = connections.clone();
            let reset_epoch = reset_epoch.clone();
            let on_reset: OnActivate = Arc::new(move |host, action_cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&reset_view, |v| *v = PanZoom2D::default());
                let _ = host.models_mut().update(&reset_scale_mode, |m| {
                    *m = CanvasWorldScaleMode::ScaleWithZoom
                });
                let _ = host
                    .models_mut()
                    .update(&reset_node_a, |p| *p = Point::new(Px(420.0), Px(80.0)));
                let _ = host
                    .models_mut()
                    .update(&reset_node_b, |p| *p = Point::new(Px(760.0), Px(260.0)));
                let _ = host.models_mut().update(&reset_overlay_clicks, |v| *v = 0);
                let _ = host.models_mut().update(&reset_node_clicks, |v| *v = 0);
                let _ = host.models_mut().update(&reset_selected, |v| *v = 0);
                let _ = host.models_mut().update(&reset_blocked, |v| *v = 0);
                let _ = host.models_mut().update(&reset_dragged, |v| *v = 0);
                let _ = host.models_mut().update(&reset_connections, |v| v.clear());
                let _ = host
                    .models_mut()
                    .update(&reset_epoch, |v| *v = v.saturating_add(1));
                host.request_redraw(action_cx.window);
            });

            let reset = shadcn::Button::new("Reset")
                .test_id("ui-ai-cwl-reset")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_activate(on_reset)
                .into_element(cx);

            let fit_view_view = view.clone();
            let fit_view_store = bounds_store.clone();
            let fit_view_bounds = world_cx.bounds;
            let on_fit_view: OnActivate = Arc::new(move |host, action_cx, _reason| {
                let next: Option<PanZoom2D> = host
                    .models_mut()
                    .read(&fit_view_store, |st| {
                        canvas_world_fit_view_to_keys(
                            fit_view_bounds,
                            st,
                            [1u64, 2u64],
                            FitViewOptions2D::default(),
                        )
                    })
                    .ok()
                    .flatten();

                let Some(next) = next else {
                    return;
                };
                let _ = host.models_mut().update(&fit_view_view, |v| *v = next);
                host.request_redraw(action_cx.window);
            });

            let fit_view = shadcn::Button::new("Fit view")
                .test_id("ui-ai-cwl-fit-view")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_activate(on_fit_view)
                .into_element(cx);

            let connections_value: Vec<(u64, u64)> = cx
                .read_model_ref(&connections, fret_ui::Invalidation::Layout, |v| v.clone())
                .unwrap_or_default();
            let connections_committed = (!connections_value.is_empty()).then(|| {
                cx.text(format!("Connections: {}", connections_value.len()))
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .role(fret_core::SemanticsRole::Text)
                            .test_id("ui-ai-cwl-connection-committed"),
                    )
            });

            let bounds_text = match bounds_union_canvas {
                None => "Bounds: (unknown)".to_string(),
                Some(r) => format!(
                    "Bounds: {bounds_count} items; union canvas rect = ({:.1}, {:.1}) {:.1}×{:.1}",
                    r.origin.x.0, r.origin.y.0, r.size.width.0, r.size.height.0
                ),
            };

            let mut anchor_layout = fret_ui::element::LayoutStyle::default();
            anchor_layout.position = fret_ui::element::PositionStyle::Absolute;
            anchor_layout.inset.right = Some(Px(12.0)).into();
            anchor_layout.inset.bottom = Some(Px(12.0)).into();
            anchor_layout.size.width = fret_ui::element::Length::Px(Px(20.0));
            anchor_layout.size.height = fret_ui::element::Length::Px(Px(20.0));
            let anchor = cx
                .layout_query_region(
                    fret_ui::element::LayoutQueryRegionProps {
                        layout: anchor_layout,
                        name: Some("ui-ai-cwl.marquee-anchor".into()),
                    },
                    |_cx| std::iter::empty(),
                )
                .attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Group)
                        .test_id("ui-ai-cwl-marquee-anchor"),
                );

            let debug_drop_target_rect = cx
                .read_model_ref(
                    &bounds_store,
                    fret_ui::Invalidation::Layout,
                    |st| st.items.get(&2u64).map(|i| i.canvas_bounds),
                )
                .ok()
                .flatten()
                .map(|target_canvas| {
                    let p0_screen = overlay_view.canvas_to_screen(overlay_surface_bounds, target_canvas.origin);
                    let p1_screen = overlay_view.canvas_to_screen(
                        overlay_surface_bounds,
                        Point::new(
                            Px(target_canvas.origin.x.0 + target_canvas.size.width.0),
                            Px(target_canvas.origin.y.0 + target_canvas.size.height.0),
                        ),
                    );

                    let hit_slop_screen_px = 12.0f32;
                    let x0 = p0_screen.x.0.min(p1_screen.x.0) - hit_slop_screen_px;
                    let y0 = p0_screen.y.0.min(p1_screen.y.0) - hit_slop_screen_px;
                    let x1 = p0_screen.x.0.max(p1_screen.x.0) + hit_slop_screen_px;
                    let y1 = p0_screen.y.0.max(p1_screen.y.0) + hit_slop_screen_px;

                    let left = Px(x0 - overlay_surface_bounds.origin.x.0);
                    let top = Px(y0 - overlay_surface_bounds.origin.y.0);
                    let w = Px((x1 - x0).max(0.0));
                    let h = Px((y1 - y0).max(0.0));
                    (left, top, w, h)
                })
                .map(|(left, top, w, h)| {
                    let mut layout = fret_ui::element::LayoutStyle::default();
                    layout.position = fret_ui::element::PositionStyle::Absolute;
                    layout.inset.left = Some(left).into();
                    layout.inset.top = Some(top).into();
                    layout.size.width = Length::Px(w);
                    layout.size.height = Length::Px(h);
                    cx.container(
                        fret_ui::element::ContainerProps {
                            layout,
                            ..Default::default()
                        },
                        |_cx| std::iter::empty(),
                    )
                    .attach_semantics(
                        SemanticsDecoration::default()
                            .role(fret_core::SemanticsRole::Group)
                            .test_id("ui-ai-cwl-debug-drop-target-rect"),
                    )
                });

            let mut out: Vec<AnyElement> = vec![connect_region, anchor];
            if let Some(el) = debug_drop_target_rect {
                out.push(el);
            }
            out.push(canvas_input_exempt_region(cx, overlay_region, move |cx| {
                    let blocked = (marquee_blocked_count_value > 0).then(|| {
                        cx.text("Marquee blocked (node hit)").attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Text)
                                .test_id("ui-ai-cwl-marquee-blocked"),
                        )
                    });

                    let bounds_nodes_ready = cx
                        .read_model_ref(
                            &bounds_store,
                            fret_ui::Invalidation::Layout,
                            |st| st.items.contains_key(&1u64) && st.items.contains_key(&2u64),
                        )
                        .unwrap_or(false);
                    if !bounds_nodes_ready {
                        cx.request_frame();
                    }
                    let bounds_ready = bounds_nodes_ready.then(|| {
                        cx.text(format!("Bounds items: {bounds_count}"))
                            .attach_semantics(
                                SemanticsDecoration::default()
                                    .role(fret_core::SemanticsRole::Text)
                                    .test_id("ui-ai-cwl-bounds-ready"),
                            )
                    });

                    let connect_overlay_ready = connect_enabled.then(|| {
                        cx.text("Connect overlay source region ready").attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Text)
                                .test_id("ui-ai-cwl-connect-overlay-ready"),
                        )
                    });

                    let node_dragged = (node_dragged_count_value > 0).then(|| {
                        cx.text(format!("Node dragged count: {node_dragged_count_value}"))
                            .attach_semantics(
                                SemanticsDecoration::default()
                                    .role(fret_core::SemanticsRole::Text)
                                    .test_id("ui-ai-cwl-node-dragged"),
                            )
                    });

                    let selected_nonzero = (selected_count_value > 0).then(|| {
                        cx.text(format!("Selected nonzero: {selected_count_value}"))
                            .attach_semantics(
                                SemanticsDecoration::default()
                                    .role(fret_core::SemanticsRole::Text)
                                    .test_id("ui-ai-cwl-selected-nonzero"),
                            )
                    });

                    let connect_started = (connect_drag_started_value > 0).then(|| {
                        cx.text("Connect drag started").attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Text)
                                .test_id("ui-ai-cwl-connect-started"),
                        )
                    });
                    let connect_up = (connect_drag_up_value > 0).then(|| {
                        cx.text("Connect drag ended").attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Text)
                                .test_id("ui-ai-cwl-connect-up"),
                        )
                    });
                    let connect_hit = (connect_drag_hit_value > 0).then(|| {
                        cx.text("Connect drag hit target").attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Text)
                                .test_id("ui-ai-cwl-connect-hit"),
                        )
                    });
                    let target_bounds_ready = (node_b_screen_bounds_value.is_some()).then(|| {
                        cx.text("Target screen bounds ready").attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Text)
                                .test_id("ui-ai-cwl-target-screen-bounds-ready"),
                        )
                    });

                    let items = vec![
                        reset,
                        mode_scale,
                        mode_semantic,
                        commit_connection,
                        fit_view,
                        overlay,
                        cx.text(format!("Selected: {selected_count_value}"))
                            .attach_semantics(
                                SemanticsDecoration::default()
                                    .role(fret_core::SemanticsRole::Text)
                                    .test_id("ui-ai-cwl-selected-count"),
                            ),
                        cx.text(bounds_text),
                        cx.text(format!(
                            "Marquee blocked count: {marquee_blocked_count_value}"
                        )),
                        cx.text(format!(
                            "Connect drag stats: started={connect_drag_started_value} up={connect_drag_up_value} hit={connect_drag_hit_value}"
                        ))
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Text)
                                .test_id("ui-ai-cwl-connect-drag-stats"),
                        ),
                    ]
                    .into_iter()
                    .chain({
                        let view_value = cx
                            .get_model_copied(&view, fret_ui::Invalidation::Layout)
                            .unwrap_or_default();
                        let a = cx
                            .get_model_copied(&node_a_canvas_pos, fret_ui::Invalidation::Layout)
                            .unwrap_or(Point::new(Px(0.0), Px(0.0)));
                        let b = cx
                            .get_model_copied(&node_b_canvas_pos, fret_ui::Invalidation::Layout)
                            .unwrap_or(Point::new(Px(0.0), Px(0.0)));

                        let debug_view_text = format!(
                            "Debug view: pan=({:.1}, {:.1}) zoom={:.3}",
                            view_value.pan.x.0, view_value.pan.y.0, view_value.zoom
                        );
                        let debug_nodes_text = format!(
                            "Debug nodes (canvas): A=({:.1}, {:.1}) B=({:.1}, {:.1})",
                            a.x.0, a.y.0, b.x.0, b.y.0
                        );

                        [
                            cx.text(debug_view_text).attach_semantics(
                                SemanticsDecoration::default()
                                    .role(fret_core::SemanticsRole::Text)
                                    .test_id("ui-ai-cwl-debug-view"),
                            ),
                            cx.text(debug_nodes_text).attach_semantics(
                                SemanticsDecoration::default()
                                    .role(fret_core::SemanticsRole::Text)
                                    .test_id("ui-ai-cwl-debug-node-canvas-pos"),
                            ),
                        ]
                    })
                    .chain(connections_committed)
                    .chain(layout_settled)
                    .chain(reset_done)
                    .chain(bounds_ready)
                    .chain(connect_overlay_ready)
                    .chain(node_dragged)
                    .chain(selected_nonzero)
                    .chain(blocked)
                    .chain(connect_started)
                    .chain(connect_up)
                    .chain(connect_hit)
                    .chain(target_bounds_ready)
                    .collect::<Vec<_>>();

                    vec![ui::v_stack(move |_cx| items)
                            .layout(LayoutRefinement::default().min_w_0())
                            .gap(Space::N2).into_element(cx)]
                }));
            out
        },
    )
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-ai-canvas-world-layer-spike-root"),
    );

    ui::v_flex(move |cx| vec![header, cx.container(stage_props, move |_cx| vec![world])])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N4)
        .into_element(cx)
}
// endregion: example
