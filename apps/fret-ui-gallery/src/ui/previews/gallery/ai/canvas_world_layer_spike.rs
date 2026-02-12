use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_canvas_world_layer_spike(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_canvas::ui::{
        CanvasInputExemptRegionProps, CanvasWorldSurfacePanelProps, PanZoomInputPreset,
        canvas_input_exempt_region, canvas_world_surface_panel, use_controllable_model,
    };
    use fret_canvas::view::{PanZoom2D, visible_canvas_rect};
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
    let overlay_clicks: fret_runtime::Model<u64> =
        use_controllable_model(cx, None, || 0u64).model();
    let node_clicks: fret_runtime::Model<u64> = use_controllable_model(cx, None, || 0u64).model();

    let overlay_clicks_value = cx
        .get_model_copied(&overlay_clicks, fret_ui::Invalidation::Layout)
        .unwrap_or(0);
    let node_clicks_value = cx
        .get_model_copied(&node_clicks, fret_ui::Invalidation::Layout)
        .unwrap_or(0);

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
    let world = canvas_world_surface_panel(
        cx,
        world_props,
        paint,
        move |cx, _world_cx| {
            let abs = LayoutRefinement::default().absolute();

            let on_node_activate: OnActivate = Arc::new(move |host, action_cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&node_clicks_c, |v| *v = v.saturating_add(1));
                host.request_redraw(action_cx.window);
            });

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
            .refine_layout(
                abs.clone()
                    .left_px(Px(80.0))
                    .top_px(Px(80.0))
                    .w_px(Px(260.0)),
            )
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
            .refine_layout(
                abs.clone()
                    .left_px(Px(420.0))
                    .top_px(Px(260.0))
                    .w_px(Px(260.0)),
            )
            .into_element(cx);

            vec![node_a, node_b]
        },
        move |cx, _world_cx| {
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

            let overlay = shadcn::Button::new(format!("Overlay clicks: {overlay_clicks_value}"))
                .test_id("ui-ai-cwl-overlay-click")
                .variant(ButtonVariant::Outline)
                .on_activate(on_overlay_activate)
                .into_element(cx);

            vec![canvas_input_exempt_region(cx, overlay_region, move |_cx| {
                [overlay]
            })]
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
