use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_canvas_world_layer_spike(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_canvas::ui::{
        CanvasInputExemptRegionProps, CanvasMarqueeSelectionProps, CanvasWorldBoundsStore,
        CanvasWorldScaleMode, CanvasWorldSurfacePanelProps, OnCanvasMarqueeCommit,
        PanZoomInputPreset, canvas_input_exempt_region, canvas_world_bounds_item,
        canvas_world_fit_view_to_keys, canvas_world_surface_panel_with_marquee_selection,
        use_controllable_model,
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

    let mut marquee = CanvasMarqueeSelectionProps::default();
    marquee.on_commit = Some(on_marquee_commit);

    let world = canvas_world_surface_panel_with_marquee_selection(
        cx,
        world_props,
        marquee,
        paint,
        move |cx, world_cx| {
            let abs = LayoutRefinement::default().absolute();

            let on_node_activate: OnActivate = Arc::new(move |host, action_cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&node_clicks_c, |v| *v = v.saturating_add(1));
                host.request_redraw(action_cx.window);
            });

            let a_canvas = Point::new(Px(80.0), Px(80.0));
            let b_canvas = Point::new(Px(420.0), Px(260.0));

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

            let node_a = ui_ai::WorkflowNode::new([
                ui_ai::WorkflowNodeHeader::new([
                    ui_ai::WorkflowNodeTitle::new("Node A").into_element(cx)
                ])
                .into_element(cx),
                ui_ai::WorkflowNodeContent::new([cx.text(format!("Clicks: {node_clicks_value}"))])
                    .into_element(cx),
                ui_ai::WorkflowNodeFooter::new([shadcn::Button::new("Click node")
                    .test_id("ui-ai-cwl-node-click")
                    .on_activate(on_node_activate.clone())
                    .variant(ButtonVariant::Secondary)
                    .into_element(cx)])
                .into_element(cx),
            ])
            .test_id("ui-ai-cwl-node-a")
            .refine_layout(abs.clone().left_px(a_left).top_px(a_top).w_px(Px(260.0)))
            .into_element(cx);

            let node_b = ui_ai::WorkflowNode::new([
                ui_ai::WorkflowNodeHeader::new([
                    ui_ai::WorkflowNodeTitle::new("Node B").into_element(cx)
                ])
                .into_element(cx),
                ui_ai::WorkflowNodeContent::new([cx.text("Try zooming/panning and click again.")])
                    .into_element(cx),
            ])
            .test_id("ui-ai-cwl-node-b")
            .refine_layout(abs.clone().left_px(b_left).top_px(b_top).w_px(Px(260.0)))
            .into_element(cx);

            vec![
                canvas_world_bounds_item(cx, bounds_store_c.clone(), 1, world_cx, move |_cx| {
                    vec![node_a]
                }),
                canvas_world_bounds_item(cx, bounds_store_c.clone(), 2, world_cx, move |_cx| {
                    vec![node_b]
                }),
            ]
        },
        move |cx, world_cx| {
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

            let overlay = shadcn::Button::new(format!("Overlay clicks: {overlay_clicks_value}"))
                .test_id("ui-ai-cwl-overlay-click")
                .variant(ButtonVariant::Outline)
                .on_activate(on_overlay_activate)
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
                    vec![
                        mode_scale,
                        mode_semantic,
                        fit_view,
                        overlay,
                        cx.text(format!("Selected: {selected_count_value}")),
                        cx.text(bounds_text),
                    ]
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
