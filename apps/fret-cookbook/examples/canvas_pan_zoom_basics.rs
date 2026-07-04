use fret::app::prelude::*;
use fret::app::{LocalState, RenderContextAccess as _};
use fret::canvas::{self, CanvasPaint, PanZoom2D, PanZoomCanvasPaintCx, Point, Rect, Size};
use fret::pointer::{CursorIcon, MouseButton, PointerDown, PointerId, PointerMove, PointerUp};
use fret::semantics::{SemanticsDecoration, SemanticsRole};
use fret::{shadcn, style::Space};

mod act {
    fret::actions!([
        ResetView = "cookbook.canvas_pan_zoom_basics.reset_view.v1",
        ResetNode = "cookbook.canvas_pan_zoom_basics.reset_node.v1"
    ]);
}

const TEST_ID_ROOT: &str = "cookbook.canvas_pan_zoom_basics.root";
const TEST_ID_CANVAS: &str = "cookbook.canvas_pan_zoom_basics.canvas";
const TEST_ID_RESET_VIEW: &str = "cookbook.canvas_pan_zoom_basics.reset_view";
const TEST_ID_RESET_NODE: &str = "cookbook.canvas_pan_zoom_basics.reset_node";
const TEST_ID_ZOOM: &str = "cookbook.canvas_pan_zoom_basics.zoom";
const TEST_ID_PAN_X: &str = "cookbook.canvas_pan_zoom_basics.pan_x";
const TEST_ID_PAN_Y: &str = "cookbook.canvas_pan_zoom_basics.pan_y";
const TEST_ID_NODE_DRAGS: &str = "cookbook.canvas_pan_zoom_basics.node_drags";

const NODE_SIZE: Size = Size {
    width: Px(220.0),
    height: Px(120.0),
};

fn node_rect(origin: Point) -> Rect {
    Rect::new(origin, NODE_SIZE)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct NodeDragState {
    pointer_id: PointerId,
    start_canvas: Point,
    origin_at_start: Point,
}

struct CanvasPanZoomBasicsView {
    view: LocalState<PanZoom2D>,
    node_origin: LocalState<Point>,
    node_drag: LocalState<Option<NodeDragState>>,
    node_drag_count: LocalState<u64>,
}

impl View for CanvasPanZoomBasicsView {
    fn init(app: &mut App, _window: WindowId) -> Self {
        Self {
            view: app.local_state(PanZoom2D::default()),
            node_origin: app.local_state(Point::new(Px(120.0), Px(120.0))),
            node_drag: app.local_state(None),
            node_drag_count: app.local_state(0),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = cx.theme_snapshot();

        let view_value = self.view.paint_value(cx);
        let node_origin = self.node_origin.paint_value(cx);
        let node_drag_count = self.node_drag_count.paint_value(cx);

        cx.actions()
            .local(&self.view)
            .set::<act::ResetView>(PanZoom2D::default());

        cx.actions()
            .locals_with((&self.node_origin, &self.node_drag, &self.node_drag_count))
            .on::<act::ResetNode>(|tx, (node_origin, node_drag, node_drag_count)| {
                let origin_updated = tx.set(&node_origin, Point::new(Px(120.0), Px(120.0)));
                let drag_updated = tx.set(&node_drag, None);
                let count_updated = tx.set(&node_drag_count, 0);
                origin_updated && drag_updated && count_updated
            });

        let zoom_badge = shadcn::Badge::new(format!("Zoom: {:.2}", view_value.zoom))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Meter)
                    .test_id(TEST_ID_ZOOM)
                    .numeric_value(view_value.zoom as f64)
                    .numeric_range(0.05, 64.0),
            );

        let pan_x = shadcn::Badge::new(format!("Pan X: {:.0}", view_value.pan.x.0))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Meter)
                    .test_id(TEST_ID_PAN_X)
                    .numeric_value(view_value.pan.x.0 as f64),
            );
        let pan_y = shadcn::Badge::new(format!("Pan Y: {:.0}", view_value.pan.y.0))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Meter)
                    .test_id(TEST_ID_PAN_Y)
                    .numeric_value(view_value.pan.y.0 as f64),
            );

        let drag_badge = shadcn::Badge::new(format!("Node drags: {node_drag_count}"))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Meter)
                    .test_id(TEST_ID_NODE_DRAGS)
                    .numeric_value(node_drag_count as f64),
            );

        let toolbar = ui::h_flex(|cx| {
            ui::children![
                cx;
                shadcn::Button::new("Reset view")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::ResetView)
                    .test_id(TEST_ID_RESET_VIEW),
                shadcn::Button::new("Reset node")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::ResetNode)
                    .test_id(TEST_ID_RESET_NODE),
                zoom_badge,
                pan_x,
                pan_y,
                drag_badge,
            ]
        })
        .gap(Space::N2)
        .items_center();

        let hint = shadcn::Alert::new(ui::children![
            cx;
            shadcn::AlertTitle::new("Interactions"),
            shadcn::AlertDescription::new(
                "Middle-drag pans. Wheel zooms. Left-drag the rectangle to move it in canvas space.",
            ),
        ])
        .ui();

        let canvas = {
            let view_model = self.view.clone();
            let node_origin_model = self.node_origin.clone();
            let drag_model = self.node_drag.clone();
            let drag_count_model = self.node_drag_count.clone();

            let view_model_down = view_model.clone();
            let node_origin_model_down = node_origin_model.clone();
            let drag_model_down = drag_model.clone();
            let on_pointer_down = move |cx: &mut fret::pointer::PointerActionCx<'_>,
                                        down: PointerDown| {
                if down.button != MouseButton::Left {
                    return false;
                }

                let bounds = cx.bounds();
                let view = cx.local_value_or(&view_model_down, PanZoom2D::default());
                let origin =
                    cx.local_value_or(&node_origin_model_down, Point::new(Px(0.0), Px(0.0)));

                let canvas_pos = view.screen_to_canvas(bounds, down.position);
                let r = node_rect(origin);
                let inside = canvas_pos.x.0 >= r.origin.x.0
                    && canvas_pos.y.0 >= r.origin.y.0
                    && canvas_pos.x.0 <= r.origin.x.0 + r.size.width.0
                    && canvas_pos.y.0 <= r.origin.y.0 + r.size.height.0;
                if !inside {
                    return false;
                }

                cx.prevent_focus_on_pointer_down();
                cx.capture_pointer();
                cx.set_cursor_icon(CursorIcon::Pointer);
                cx.set_local(
                    &drag_model_down,
                    Some(NodeDragState {
                        pointer_id: down.pointer_id,
                        start_canvas: canvas_pos,
                        origin_at_start: origin,
                    }),
                );
                true
            };

            let view_model_move = view_model.clone();
            let node_origin_model_move = node_origin_model.clone();
            let drag_model_move = drag_model.clone();
            let on_pointer_move = move |cx: &mut fret::pointer::PointerActionCx<'_>,
                                        mv: PointerMove| {
                let bounds = cx.bounds();
                let view = cx.local_value_or(&view_model_move, PanZoom2D::default());

                if let Some(drag) = cx.local_value_or(&drag_model_move, None) {
                    if drag.pointer_id != mv.pointer_id {
                        return false;
                    }

                    let canvas_pos = view.screen_to_canvas(bounds, mv.position);
                    let dx = canvas_pos.x.0 - drag.start_canvas.x.0;
                    let dy = canvas_pos.y.0 - drag.start_canvas.y.0;

                    let next = Point::new(
                        Px(drag.origin_at_start.x.0 + dx),
                        Px(drag.origin_at_start.y.0 + dy),
                    );
                    cx.set_local(&node_origin_model_move, next);
                    cx.set_cursor_icon(CursorIcon::Pointer);
                    return true;
                }

                // Hover cursor hint (do not consume the event).
                let origin =
                    cx.local_value_or(&node_origin_model_move, Point::new(Px(0.0), Px(0.0)));
                let canvas_pos = view.screen_to_canvas(bounds, mv.position);
                let r = node_rect(origin);
                let inside = canvas_pos.x.0 >= r.origin.x.0
                    && canvas_pos.y.0 >= r.origin.y.0
                    && canvas_pos.x.0 <= r.origin.x.0 + r.size.width.0
                    && canvas_pos.y.0 <= r.origin.y.0 + r.size.height.0;
                if inside {
                    cx.set_cursor_icon(CursorIcon::Pointer);
                }
                false
            };

            let drag_model_up = drag_model.clone();
            let drag_count_model_up = drag_count_model.clone();
            let on_pointer_up = move |cx: &mut fret::pointer::PointerActionCx<'_>,
                                      up: PointerUp| {
                if up.button != MouseButton::Left {
                    return false;
                }

                let Some(drag) = cx.local_value_or(&drag_model_up, None) else {
                    return false;
                };
                if drag.pointer_id != up.pointer_id {
                    return false;
                }

                cx.release_pointer_capture();
                cx.set_local(&drag_model_up, None);
                cx.update_local(&drag_count_model_up, |n| {
                    *n = n.saturating_add(1);
                });
                true
            };

            let bg = theme.color_token("card");
            let grid = theme.color_token("border");
            let node_fill = theme.color_token("primary");
            let node_border = theme.color_token("primary-foreground");

            let paint = move |p: &mut canvas::AppCanvasPainter<'_, '_>,
                              paint_cx: PanZoomCanvasPaintCx| {
                let bounds = p.bounds();
                let Some(transform) = paint_cx.view.render_transform(bounds) else {
                    return;
                };

                p.quad(
                    canvas::DrawOrder(0),
                    bounds,
                    CanvasPaint::Solid(bg),
                    canvas::Edges::all(Px(0.0)),
                    CanvasPaint::TRANSPARENT,
                    canvas::Corners::all(Px(0.0)),
                );

                let vis = canvas::visible_canvas_rect(bounds, paint_cx.view);
                let step = 80.0f32;
                let min_x = (vis.origin.x.0 / step).floor() as i32 - 2;
                let max_x = ((vis.origin.x.0 + vis.size.width.0) / step).ceil() as i32 + 2;
                let min_y = (vis.origin.y.0 / step).floor() as i32 - 2;
                let max_y = ((vis.origin.y.0 + vis.size.height.0) / step).ceil() as i32 + 2;
                let line_w = canvas::constant_pixel_stroke_width(Px(1.0), paint_cx.view.zoom);

                p.with_clip_rect(bounds, |p| {
                    p.with_transform(transform, |p| {
                        for x in min_x..=max_x {
                            let ox = x as f32 * step;
                            let rect = Rect::new(
                                Point::new(Px(ox), Px(min_y as f32 * step)),
                                Size::new(line_w, Px((max_y - min_y) as f32 * step)),
                            );
                            p.quad(
                                canvas::DrawOrder(1),
                                rect,
                                CanvasPaint::Solid(grid),
                                canvas::Edges::all(Px(0.0)),
                                CanvasPaint::TRANSPARENT,
                                canvas::Corners::all(Px(0.0)),
                            );
                        }
                        for y in min_y..=max_y {
                            let oy = y as f32 * step;
                            let rect = Rect::new(
                                Point::new(Px(min_x as f32 * step), Px(oy)),
                                Size::new(Px((max_x - min_x) as f32 * step), line_w),
                            );
                            p.quad(
                                canvas::DrawOrder(1),
                                rect,
                                CanvasPaint::Solid(grid),
                                canvas::Edges::all(Px(0.0)),
                                CanvasPaint::TRANSPARENT,
                                canvas::Corners::all(Px(0.0)),
                            );
                        }

                        let node = node_rect(node_origin);
                        let border_w =
                            canvas::constant_pixel_stroke_width(Px(2.0), paint_cx.view.zoom);
                        p.quad(
                            canvas::DrawOrder(10),
                            node,
                            CanvasPaint::Solid(node_fill),
                            canvas::Edges::all(border_w),
                            CanvasPaint::Solid(node_border),
                            canvas::Corners::all(Px(10.0)),
                        );
                    });
                });
            };

            canvas::PanZoomCanvas::new(&self.view)
                .default_view(PanZoom2D::default())
                .desktop_canvas_cad()
                .pan_button(MouseButton::Middle)
                .on_pointer_down(on_pointer_down)
                .on_pointer_move(on_pointer_move)
                .on_pointer_up(on_pointer_up)
                .into_element(cx, paint)
                .test_id(TEST_ID_CANVAS)
        };

        let card = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("Canvas pan/zoom basics"),
                        shadcn::card_description(
                            "Uses fret-canvas pan/zoom wiring + a tiny app-owned drag tool for one item.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::children![
                        cx;
                        ui::v_flex(|cx| ui::children![cx; toolbar, hint, canvas])
                            .gap(Space::N3)
                            .w_full(),
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(980.0));

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-canvas-pan-zoom-basics")
        .window("cookbook-canvas-pan-zoom-basics", (1120.0, 780.0))
        .config_files(false)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<CanvasPanZoomBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
