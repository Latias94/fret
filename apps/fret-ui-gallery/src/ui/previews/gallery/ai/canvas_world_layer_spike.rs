use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_canvas_world_layer_spike(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
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
    use fret_ui::element::{CanvasCachePolicy, Length, PointerRegionProps};
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::declarative::style as decl_style;
    use fret_ui_kit::{ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, Radius, Space};
    use fret_ui_shadcn as shadcn;
    use fret_ui_shadcn::ButtonVariant;

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Canvas world layer (spike)"),
                cx.text("Goal: nodes as element subtrees under a pan/zoom view transform."),
            ]
        },
    );

    let view: fret_runtime::Model<PanZoom2D> =
        use_controllable_model(cx, None, PanZoom2D::default).model();
    let scale_mode: fret_runtime::Model<CanvasWorldScaleMode> =
        use_controllable_model(cx, None, CanvasWorldScaleMode::default).model();
    let overlay_clicks: fret_runtime::Model<u64> =
        use_controllable_model(cx, None, || 0u64).model();
    let node_clicks: fret_runtime::Model<u64> = use_controllable_model(cx, None, || 0u64).model();
    let bounds_store: fret_runtime::Model<CanvasWorldBoundsStore> =
        use_controllable_model(cx, None, CanvasWorldBoundsStore::default).model();
    let selected_count: fret_runtime::Model<u64> =
        use_controllable_model(cx, None, || 0u64).model();
    let marquee_blocked_count: fret_runtime::Model<u64> =
        use_controllable_model(cx, None, || 0u64).model();
    let node_a_canvas_pos: fret_runtime::Model<Point> =
        use_controllable_model(cx, None, || Point::new(Px(80.0), Px(80.0))).model();
    let node_b_canvas_pos: fret_runtime::Model<Point> =
        use_controllable_model(cx, None, || Point::new(Px(420.0), Px(260.0))).model();
    let connect_drag_started_count: fret_runtime::Model<u64> =
        use_controllable_model(cx, None, || 0u64).model();
    let connect_drag_up_count: fret_runtime::Model<u64> =
        use_controllable_model(cx, None, || 0u64).model();
    let connect_drag_hit_count: fret_runtime::Model<u64> =
        use_controllable_model(cx, None, || 0u64).model();
    let node_a_screen_bounds: fret_runtime::Model<Option<Rect>> =
        use_controllable_model(cx, None, || None).model();
    let node_b_screen_bounds: fret_runtime::Model<Option<Rect>> =
        use_controllable_model(cx, None, || None).model();
    let node_dragged_count: fret_runtime::Model<u64> =
        use_controllable_model(cx, None, || 0u64).model();
    let connections: fret_runtime::Model<Vec<(u64, u64)>> =
        use_controllable_model(cx, None, Vec::<(u64, u64)>::new).model();
    let reset_epoch: fret_runtime::Model<u64> = use_controllable_model(cx, None, || 0u64).model();

    let node_a_canvas_pos_for_world = node_a_canvas_pos.clone();
    let node_b_canvas_pos_for_world = node_b_canvas_pos.clone();
    let node_dragged_count_for_world = node_dragged_count.clone();

    #[derive(Default)]
    struct InitState {
        done: bool,
    }

    let needs_init = cx.with_state(InitState::default, |st| !st.done);
    if needs_init {
        cx.with_state(InitState::default, |st| st.done = true);

        let _ = cx
            .app
            .models_mut()
            .update(&view, |v| *v = PanZoom2D::default());
        let _ = cx
            .app
            .models_mut()
            .update(&scale_mode, |m| *m = CanvasWorldScaleMode::ScaleWithZoom);
        let _ = cx.app.models_mut().update(&overlay_clicks, |v| *v = 0);
        let _ = cx.app.models_mut().update(&node_clicks, |v| *v = 0);
        let _ = cx.app.models_mut().update(&selected_count, |v| *v = 0);
        let _ = cx
            .app
            .models_mut()
            .update(&marquee_blocked_count, |v| *v = 0);
        let _ = cx
            .app
            .models_mut()
            .update(&node_a_canvas_pos, |p| *p = Point::new(Px(80.0), Px(80.0)));
        let _ = cx.app.models_mut().update(&node_b_canvas_pos, |p| {
            *p = Point::new(Px(420.0), Px(260.0))
        });
        let _ = cx
            .app
            .models_mut()
            .update(&connect_drag_started_count, |v| *v = 0);
        let _ = cx
            .app
            .models_mut()
            .update(&connect_drag_up_count, |v| *v = 0);
        let _ = cx
            .app
            .models_mut()
            .update(&connect_drag_hit_count, |v| *v = 0);
        let _ = cx
            .app
            .models_mut()
            .update(&node_a_screen_bounds, |v| *v = None);
        let _ = cx
            .app
            .models_mut()
            .update(&node_b_screen_bounds, |v| *v = None);
        let _ = cx.app.models_mut().update(&node_dragged_count, |v| *v = 0);
        let _ = cx.app.models_mut().update(&connections, |v| v.clear());
        let _ = cx.app.models_mut().update(&reset_epoch, |v| *v = 0);

        cx.request_frame();
    }

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

    let stage_props = decl_style::container_props(
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
    );

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

    let paint = {
        let bg = theme.color_required("background");
        let grid = theme.color_required("border");
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
                        background: Paint::Solid(bg),
                        border: Edges::all(Px(0.0)),
                        border_paint: Paint::Solid(fret_core::scene::Color::TRANSPARENT),
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
                            background: Paint::Solid(grid),
                            border: Edges::all(Px(0.0)),
                            border_paint: Paint::Solid(fret_core::scene::Color::TRANSPARENT),
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
                            background: Paint::Solid(grid),
                            border: Edges::all(Px(0.0)),
                            border_paint: Paint::Solid(fret_core::scene::Color::TRANSPARENT),
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
    let blocked_for_marquee_start_filter = marquee_blocked_count.clone();
    let marquee_start_filter: OnCanvasMarqueeStart = Arc::new(move |host, action_cx, down| {
        let bounds = host.bounds();
        let view = host
            .models_mut()
            .read(&view_for_marquee_start_filter, |v| *v)
            .ok()
            .unwrap_or_default();
        let p_canvas = view.screen_to_canvas(bounds, down.position);

        let hit_node = host
            .models_mut()
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
            .unwrap_or(false);

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
    let node_b_screen_bounds_for_world = node_b_screen_bounds.clone();
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
                from_canvas: Point,
                start_screen: Point,
                current_screen: Point,
                view_at_start: PanZoom2D,
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
                .unwrap_or(Point::new(Px(80.0), Px(80.0)));
            let b_canvas = cx
                .get_model_copied(&node_b_canvas_pos_for_world, fret_ui::Invalidation::Layout)
                .unwrap_or(Point::new(Px(420.0), Px(260.0)));

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
                        let from_local = canvas_point_to_local(&world_cx, drag.from_canvas);
                        let to_local = match world_cx.scale_mode {
                            CanvasWorldScaleMode::ScaleWithZoom => {
                                let c = drag
                                    .view_at_start
                                    .screen_to_canvas(surface_bounds, drag.current_screen);
                                canvas_point_to_local(&world_cx, c)
                            }
                            CanvasWorldScaleMode::SemanticZoom => Point::new(
                                Px(drag.current_screen.x.0 - surface_bounds.origin.x.0),
                                Px(drag.current_screen.y.0 - surface_bounds.origin.y.0),
                            ),
                        };

                        vec![
                            ui_ai::WorkflowConnection::new(from_local, to_local)
                                .stroke_width(Px(2.0))
                                .test_id("ui-ai-cwl-connection-preview")
                                .refine_layout(connection_layer_layout.clone())
                                .into_element(cx),
                        ]
                    }
                }
            };

            let connect_drag_state_for_node_a = connect_drag_state.clone();
            let connect_drag_state_for_node_b = connect_drag_state.clone();
            let connections_for_node_a = connections_c.clone();
            let connections_for_node_b = connections_c.clone();
            let node_b_screen_bounds_for_node_a = node_b_screen_bounds_for_world.clone();
            let connect_drag_started_for_node_a = connect_drag_started_for_world.clone();
            let connect_drag_up_for_node_a = connect_drag_up_for_world.clone();
            let connect_drag_hit_for_node_a = connect_drag_hit_for_world.clone();

            let node_a_item = canvas_world_bounds_item(
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
                            .variant(ButtonVariant::Secondary)
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
                        top: Some(Px(0.0)),
                        left: Some(Px(0.0)),
                        right: Some(Px(0.0)),
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
                        top: Some(Px(0.0)),
                        bottom: Some(Px(0.0)),
                        right: Some(Px(0.0)),
                        ..Default::default()
                    };
                    source_handle_props.layout.size.width = Length::Px(Px(20.0));
                    source_handle_props.layout.size.height = Length::Fill;

                    let connect_down_state = connect_drag_state_for_node_a.clone();
                    let connect_move_state = connect_drag_state_for_node_a.clone();
                    let connect_up_state = connect_drag_state_for_node_a.clone();
                    let connect_bounds_store_for_down = bounds_store_for_node_a.clone();
                    let connect_connections = connections_for_node_a.clone();
                    let connect_target_screen_bounds = node_b_screen_bounds_for_node_a.clone();
                    let connect_started_count = connect_drag_started_for_node_a.clone();
                    let connect_up_count = connect_drag_up_for_node_a.clone();
                    let connect_hit_count = connect_drag_hit_for_node_a.clone();

                    let on_connect_down: fret_ui::action::OnPointerDown =
                        Arc::new(move |host, action_cx, down| {
                            if down.button != fret_core::MouseButton::Left {
                                return false;
                            }

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

                            host.capture_pointer();
                            let _ = host
                                .models_mut()
                                .update(&connect_started_count, |v| *v = v.saturating_add(1));
                            let _ = host.models_mut().update(&connect_down_state, |st| {
                                *st = Some(ConnectDragState {
                                    pointer_id: down.pointer_id,
                                    from_key: 1,
                                    from_canvas,
                                    start_screen: down.position,
                                    current_screen: down.position,
                                    view_at_start: default_view,
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
                            if !drag.active {
                                let dx = drag.current_screen.x.0 - drag.start_screen.x.0;
                                let dy = drag.current_screen.y.0 - drag.start_screen.y.0;
                                let dist_sq = dx * dx + dy * dy;
                                if dist_sq >= drag_min_px.0 * drag_min_px.0 {
                                    drag.active = true;
                                }
                            }

                            let _ = host
                                .models_mut()
                                .update(&connect_move_state, |st| *st = Some(drag));
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let on_connect_up: fret_ui::action::OnPointerUp =
                        Arc::new(move |host, action_cx, up| {
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
                            if up.button != fret_core::MouseButton::Left {
                                return false;
                            }

                            host.release_pointer_capture();
                            let _ = host
                                .models_mut()
                                .update(&connect_up_count, |v| *v = v.saturating_add(1));

                            if drag.active {
                                let target = host
                                    .models_mut()
                                    .read(&connect_target_screen_bounds, |st| *st)
                                    .ok()
                                    .flatten();

                                if let Some(target) = target {
                                    let slop = 12.0f32;
                                    let x0 = target.origin.x.0 - slop;
                                    let y0 = target.origin.y.0 - slop;
                                    let x1 = target.origin.x.0 + target.size.width.0 + slop;
                                    let y1 = target.origin.y.0 + target.size.height.0 + slop;
                                    let px = up.position.x.0;
                                    let py = up.position.y.0;
                                    let hit_target = px >= x0 && px <= x1 && py >= y0 && py <= y1;

                                    if hit_target {
                                        let _ = host.models_mut().update(&connect_hit_count, |v| {
                                            *v = v.saturating_add(1)
                                        });
                                        let _ = host.models_mut().update(&connect_connections, |v| {
                                            let key = (drag.from_key, 2u64);
                                            if !v.contains(&key) {
                                                v.push(key);
                                            }
                                        });
                                    }
                                }
                            }

                            let _ = host.models_mut().update(&connect_up_state, |st| *st = None);

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
                                .role(fret_core::SemanticsRole::Group)
                                .test_id("ui-ai-cwl-node-a-source-handle"),
                        );

                    let mut wrapper = fret_ui::element::ContainerProps::default();
                    wrapper.layout.position = fret_ui::element::PositionStyle::Absolute;
                    wrapper.layout.inset.left = Some(a_left);
                    wrapper.layout.inset.top = Some(a_top);
                    wrapper.layout.size.width = Length::Px(Px(260.0));
                    vec![cx.container(wrapper, move |_cx| [node_a, source_handle, handle])]
                },
            );

            let node_b_item = canvas_world_bounds_item(
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
                        top: Some(Px(0.0)),
                        left: Some(Px(0.0)),
                        right: Some(Px(0.0)),
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
                        top: Some(Px(0.0)),
                        bottom: Some(Px(0.0)),
                        left: Some(Px(0.0)),
                        ..Default::default()
                    };
                    target_handle_props.layout.size.width = Length::Px(Px(20.0));
                    target_handle_props.layout.size.height = Length::Fill;

                    let connect_drop_state = connect_drag_state_for_node_b.clone();
                    let connect_drop_connections = connections_for_node_b.clone();
                    let on_connect_drop: fret_ui::action::OnPointerUp =
                        Arc::new(move |host, action_cx, up| {
                            if up.button != fret_core::MouseButton::Left {
                                return false;
                            }

                            let drag = host
                                .models_mut()
                                .read(&connect_drop_state, |st| *st)
                                .ok()
                                .flatten();
                            let Some(drag) = drag else {
                                return false;
                            };
                            if up.pointer_id != drag.pointer_id {
                                return false;
                            }
                            if !drag.active {
                                return false;
                            }

                            let _ = host.models_mut().update(&connect_drop_connections, |v| {
                                let key = (drag.from_key, 2u64);
                                if !v.contains(&key) {
                                    v.push(key);
                                }
                            });
                            let _ = host
                                .models_mut()
                                .update(&connect_drop_state, |st| *st = None);
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let target_handle = cx
                        .pointer_region(target_handle_props, move |cx| {
                            cx.pointer_region_on_pointer_up(on_connect_drop.clone());
                            std::iter::empty()
                        })
                        .attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Group)
                                .test_id("ui-ai-cwl-node-b-target-handle"),
                        );

                    let mut wrapper = fret_ui::element::ContainerProps::default();
                    wrapper.layout.position = fret_ui::element::PositionStyle::Absolute;
                    wrapper.layout.inset.left = Some(b_left);
                    wrapper.layout.inset.top = Some(b_top);
                    wrapper.layout.size.width = Length::Px(Px(260.0));
                    vec![cx.container(wrapper, move |_cx| [node_b, target_handle, handle])]
                },
            );

            let connect_drag_overlay = {
                let overlay_enabled = cx
                    .read_model_ref(&connect_drag_state, fret_ui::Invalidation::Layout, |st| {
                        st.is_some()
                    })
                    .unwrap_or(false);

                let connect_overlay_move_state = connect_drag_state.clone();
                let on_connect_overlay_move: fret_ui::action::OnPointerMove =
                    Arc::new(move |host, action_cx, mv| {
                        let mut drag = host
                            .models_mut()
                            .read(&connect_overlay_move_state, |st| *st)
                            .ok()
                            .flatten();
                        let Some(mut drag) = drag.take() else {
                            return false;
                        };
                        if mv.pointer_id != drag.pointer_id {
                            return false;
                        }
                        if !mv.buttons.left {
                            let _ = host
                                .models_mut()
                                .update(&connect_overlay_move_state, |st| *st = None);
                            host.request_redraw(action_cx.window);
                            return true;
                        }

                        drag.current_screen = mv.position;
                        let _ = host
                            .models_mut()
                            .update(&connect_overlay_move_state, |st| *st = Some(drag));
                        host.request_redraw(action_cx.window);
                        true
                    });

                let connect_overlay_up_state = connect_drag_state.clone();
                let connect_overlay_connections = connections_c.clone();
                let connect_overlay_target_screen_bounds = node_b_screen_bounds_for_world.clone();
                let connect_overlay_up_count = connect_drag_up_for_world.clone();
                let connect_overlay_hit_count = connect_drag_hit_for_world.clone();
                let on_connect_overlay_up: fret_ui::action::OnPointerUp =
                    Arc::new(move |host, action_cx, up| {
                        if up.button != fret_core::MouseButton::Left {
                            return false;
                        }

                        let drag = host
                            .models_mut()
                            .read(&connect_overlay_up_state, |st| *st)
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
                            .update(&connect_overlay_up_count, |v| *v = v.saturating_add(1));

                        if drag.active {
                            let target = host
                                .models_mut()
                                .read(&connect_overlay_target_screen_bounds, |st| *st)
                                .ok()
                                .flatten();

                            if let Some(target) = target {
                                let slop = 12.0f32;
                                let x0 = target.origin.x.0 - slop;
                                let y0 = target.origin.y.0 - slop;
                                let x1 = target.origin.x.0 + target.size.width.0 + slop;
                                let y1 = target.origin.y.0 + target.size.height.0 + slop;
                                let px = up.position.x.0;
                                let py = up.position.y.0;
                                let hit_target = px >= x0 && px <= x1 && py >= y0 && py <= y1;

                                if hit_target {
                                    let _ = host.models_mut().update(&connect_overlay_hit_count, |v| {
                                        *v = v.saturating_add(1)
                                    });
                                    let _ = host.models_mut().update(&connect_overlay_connections, |v| {
                                        let key = (drag.from_key, 2u64);
                                        if !v.contains(&key) {
                                            v.push(key);
                                        }
                                    });
                                }
                            }
                        }

                        let _ = host
                            .models_mut()
                            .update(&connect_overlay_up_state, |st| *st = None);
                        host.request_redraw(action_cx.window);
                        true
                    });

                let mut props = PointerRegionProps::default();
                props.enabled = overlay_enabled;
                props.layout.position = fret_ui::element::PositionStyle::Absolute;
                props.layout.inset = fret_ui::element::InsetStyle {
                    top: Some(Px(0.0)),
                    left: Some(Px(0.0)),
                    right: Some(Px(0.0)),
                    bottom: Some(Px(0.0)),
                };
                props.layout.size.width = Length::Fill;
                props.layout.size.height = Length::Fill;

                cx.pointer_region(props, move |cx| {
                    cx.pointer_region_on_pointer_move(on_connect_overlay_move.clone());
                    cx.pointer_region_on_pointer_up(on_connect_overlay_up.clone());
                    std::iter::empty()
                })
            };

            connections_layer
                .into_iter()
                .chain(preview_layer)
                .chain([node_a_item, node_b_item, connect_drag_overlay])
                .collect::<Vec<_>>()
        },
        move |cx, world_cx| {
            #[derive(Default)]
            struct BoundsStabilityState {
                last: Option<(Rect, Rect)>,
                stable_frames: u32,
            }

            let mut overlay_region = CanvasInputExemptRegionProps::default();
            overlay_region.pointer_region.layout.position =
                fret_ui::element::PositionStyle::Absolute;
            overlay_region.pointer_region.layout.inset = fret_ui::element::InsetStyle {
                top: Some(Px(12.0)),
                left: Some(Px(12.0)),
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
                .variant(ButtonVariant::Secondary)
                .on_activate(on_mode_scale)
                .into_element(cx);

            let mode_semantic = shadcn::Button::new("Mode: Semantic zoom")
                .test_id("ui-ai-cwl-mode-semantic-zoom")
                .variant(ButtonVariant::Secondary)
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
                .variant(ButtonVariant::Secondary)
                .on_activate(on_commit_connection)
                .into_element(cx);

            let overlay = shadcn::Button::new(format!("Overlay clicks: {overlay_clicks_value}"))
                .test_id("ui-ai-cwl-overlay-click")
                .variant(ButtonVariant::Outline)
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

            let layout_settled = cx.with_state(BoundsStabilityState::default, |st| {
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
                    .update(&reset_node_a, |p| *p = Point::new(Px(80.0), Px(80.0)));
                let _ = host
                    .models_mut()
                    .update(&reset_node_b, |p| *p = Point::new(Px(420.0), Px(260.0)));
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
                .variant(ButtonVariant::Secondary)
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
                .variant(ButtonVariant::Secondary)
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
            anchor_layout.inset.right = Some(Px(12.0));
            anchor_layout.inset.bottom = Some(Px(12.0));
            anchor_layout.size.width = fret_ui::element::Length::Px(Px(20.0));
            anchor_layout.size.height = fret_ui::element::Length::Px(Px(20.0));
            let anchor = cx
                .container(
                    fret_ui::element::ContainerProps {
                        layout: anchor_layout,
                        ..Default::default()
                    },
                    |_cx| std::iter::empty(),
                )
                .attach_semantics(
                    SemanticsDecoration::default()
                        .role(fret_core::SemanticsRole::Group)
                        .test_id("ui-ai-cwl-marquee-anchor"),
                );

            vec![
                anchor,
                canvas_input_exempt_region(cx, overlay_region, move |cx| {
                    let blocked = (marquee_blocked_count_value > 0).then(|| {
                        cx.text("Marquee blocked (node hit)").attach_semantics(
                            SemanticsDecoration::default()
                                .role(fret_core::SemanticsRole::Text)
                                .test_id("ui-ai-cwl-marquee-blocked"),
                        )
                    });

                    let bounds_ready = (bounds_count >= 2).then(|| {
                        cx.text(format!("Bounds items: {bounds_count}"))
                            .attach_semantics(
                                SemanticsDecoration::default()
                                    .role(fret_core::SemanticsRole::Text)
                                    .test_id("ui-ai-cwl-bounds-ready"),
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
                        cx.text(format!("Selected: {selected_count_value}")),
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
                    .chain(connections_committed)
                    .chain(layout_settled)
                    .chain(reset_done)
                    .chain(bounds_ready)
                    .chain(node_dragged)
                    .chain(blocked)
                    .chain(connect_started)
                    .chain(connect_up)
                    .chain(connect_hit)
                    .chain(target_bounds_ready)
                    .collect::<Vec<_>>();

                    vec![stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().min_w_0())
                            .gap(Space::N2),
                        move |_cx| items,
                    )]
                }),
            ]
        },
    )
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-ai-canvas-world-layer-spike-root"),
    );

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| vec![header, cx.container(stage_props, move |_cx| vec![world])],
    )]
}
