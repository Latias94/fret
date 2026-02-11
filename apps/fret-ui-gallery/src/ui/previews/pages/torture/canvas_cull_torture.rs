use super::super::super::super::*;

pub(in crate::ui) fn preview_canvas_cull_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_canvas::ui::{
        PanZoomCanvasSurfacePanelProps, PanZoomInputPreset, pan_zoom_canvas_surface_panel,
    };
    use fret_canvas::view::{PanZoom2D, visible_canvas_rect};
    use fret_core::{
        Corners, DrawOrder, Edges, FontId, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
    };
    use fret_ui::canvas::CanvasTextConstraints;
    use fret_ui::element::{CanvasCachePolicy, Length};
    use std::cmp::Ordering;

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress a pan/zoom canvas scene with viewport-driven culling (candidate for prepaint-windowed cull windows)."),
                cx.text("Use scripted middle-drag + wheel steps to validate correctness and collect perf bundles."),
            ]
        },
    );

    let canvas =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let bg_even = theme.color_required("background");
            let bg_odd = theme.color_required("muted");
            let fg = theme.color_required("foreground");
            let grid = theme.color_required("border");

            let text_style = TextStyle {
                font: FontId::monospace(),
                size: Px(11.0),
                ..Default::default()
            };

            let mut props = PanZoomCanvasSurfacePanelProps::default();
            props.preset = PanZoomInputPreset::DesktopCanvasCad;
            props.pointer_region.layout.size.width = Length::Fill;
            props.pointer_region.layout.size.height = Length::Px(Px(520.0));
            props.canvas.cache_policy = CanvasCachePolicy::smooth_default();
            props.default_view = PanZoom2D {
                pan: fret_core::Point::new(Px(0.0), Px(0.0)),
                zoom: 1.0,
            };
            props.min_zoom = 0.05;
            props.max_zoom = 64.0;

            let cell_size = 48.0f32;
            let cell_pad = 3.0f32;
            let max_cells = 40_000i64;

            let canvas = pan_zoom_canvas_surface_panel(cx, props, move |painter, paint_cx| {
                let bounds = painter.bounds();

                let Some(transform) = paint_cx.view.render_transform(bounds) else {
                    return;
                };

                let vis = visible_canvas_rect(bounds, paint_cx.view);
                let min_x = vis.origin.x.0;
                let max_x = vis.origin.x.0 + vis.size.width.0;
                let min_y = vis.origin.y.0;
                let max_y = vis.origin.y.0 + vis.size.height.0;

                let start_x = (min_x / cell_size).floor() as i32 - 2;
                let end_x = (max_x / cell_size).ceil() as i32 + 2;
                let start_y = (min_y / cell_size).floor() as i32 - 2;
                let end_y = (max_y / cell_size).ceil() as i32 + 2;

                let x_count = (end_x - start_x + 1).max(0) as i64;
                let y_count = (end_y - start_y + 1).max(0) as i64;
                let estimated = x_count.saturating_mul(y_count);

                let stride = match estimated.cmp(&max_cells) {
                    Ordering::Less | Ordering::Equal => 1i32,
                    Ordering::Greater => {
                        ((estimated as f64 / max_cells as f64).ceil() as i32).max(1)
                    }
                };

                let clip = bounds;
                painter.with_clip_rect(clip, |painter| {
                    painter.with_transform(transform, |painter| {
                        let scope = painter.key_scope(&"ui-gallery-canvas-cull");

                        let mut y = start_y;
                        while y <= end_y {
                            let mut x = start_x;
                            while x <= end_x {
                                let ox = x as f32 * cell_size + cell_pad;
                                let oy = y as f32 * cell_size + cell_pad;
                                let size = cell_size - cell_pad * 2.0;
                                if size.is_finite() && size > 0.0 {
                                    let rect = fret_core::Rect::new(
                                        fret_core::Point::new(Px(ox), Px(oy)),
                                        fret_core::Size::new(Px(size), Px(size)),
                                    );

                                    let background =
                                        if ((x ^ y) & 1) == 0 { bg_even } else { bg_odd };
                                    painter.scene().push(fret_core::SceneOp::Quad {
                                        order: DrawOrder(0),
                                        rect,
                                        background: fret_core::Paint::Solid(background),
                                        border: Edges::all(Px(1.0)),
                                        border_paint: fret_core::Paint::Solid(grid),

                                        corner_radii: Corners::all(Px(4.0)),
                                    });

                                    if x == 0 && y == 0 {
                                        painter.scene().push(fret_core::SceneOp::Quad {
                                            order: DrawOrder(1),
                                            rect,
                                            background: fret_core::Paint::TRANSPARENT,

                                            border: Edges::all(Px(2.0)),
                                            border_paint: fret_core::Paint::Solid(fg),

                                            corner_radii: Corners::all(Px(4.0)),
                                        });
                                    }

                                    if (x % 20) == 0 && (y % 20) == 0 {
                                        let key: u64 = painter.child_key(scope, &(x, y)).into();
                                        let label = format!("{x},{y}");
                                        let origin = fret_core::Point::new(
                                            Px(rect.origin.x.0 + 6.0),
                                            Px(rect.origin.y.0 + 6.0),
                                        );
                                        let _ = painter.text(
                                            key,
                                            DrawOrder(2),
                                            origin,
                                            label,
                                            text_style.clone(),
                                            fg,
                                            CanvasTextConstraints {
                                                max_width: Some(Px(
                                                    (rect.size.width.0 - 12.0).max(0.0)
                                                )),
                                                wrap: TextWrap::None,
                                                overflow: TextOverflow::Clip,
                                            },
                                            painter.scale_factor(),
                                        );
                                    }
                                }

                                x = x.saturating_add(stride);
                            }
                            y = y.saturating_add(stride);
                        }
                    });
                });
            });

            vec![
                canvas.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Group)
                        .test_id("ui-gallery-canvas-cull-root"),
                ),
            ]
        });

    vec![header, canvas]
}
