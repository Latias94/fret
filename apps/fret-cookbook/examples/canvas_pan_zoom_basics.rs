use std::sync::Arc;

use fret::prelude::*;
use fret_canvas::ui::{
    PanZoomCanvasSurfacePanelProps, PanZoomInputPreset, pan_zoom_canvas_surface_panel,
};
use fret_canvas::view::{PanZoom2D, visible_canvas_rect};
use fret_core::scene::Paint;
use fret_core::{
    Corners, CursorIcon, DrawOrder, Edges, MouseButton, Point, PointerId, Px, Rect, SceneOp, Size,
};
use fret_runtime::DefaultAction;
use fret_ui::action::{OnPointerDown, OnPointerMove, OnPointerUp};
use fret_ui::canvas::CanvasPainter;
use fret_ui::element::SemanticsDecoration;

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

#[derive(Debug, Clone)]
enum Msg {
    ResetView,
    ResetNode,
}

struct CanvasPanZoomBasicsState {
    view: Model<PanZoom2D>,
    node_origin: Model<Point>,
    node_drag: Model<Option<NodeDragState>>,
    node_drag_count: Model<u64>,
}

struct CanvasPanZoomBasicsProgram;

impl MvuProgram for CanvasPanZoomBasicsProgram {
    type State = CanvasPanZoomBasicsState;
    type Message = Msg;

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        Self::State {
            view: app.models_mut().insert(PanZoom2D::default()),
            node_origin: app.models_mut().insert(Point::new(Px(120.0), Px(120.0))),
            node_drag: app.models_mut().insert(None),
            node_drag_count: app.models_mut().insert(0),
        }
    }

    fn update(app: &mut App, st: &mut Self::State, msg: Self::Message) {
        match msg {
            Msg::ResetView => {
                let _ = app
                    .models_mut()
                    .update(&st.view, |v| *v = PanZoom2D::default());
            }
            Msg::ResetNode => {
                let _ = app
                    .models_mut()
                    .update(&st.node_origin, |p| *p = Point::new(Px(120.0), Px(120.0)));
                let _ = app.models_mut().update(&st.node_drag_count, |n| *n = 0);
            }
        }
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        st: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();

        let view_value = cx.watch_model(&st.view).paint().copied_or_default();
        let node_origin = cx.watch_model(&st.node_origin).paint().copied_or_default();
        let node_drag_count = cx
            .watch_model(&st.node_drag_count)
            .paint()
            .copied_or_default();

        let reset_view = msg.cmd(Msg::ResetView);
        let reset_node = msg.cmd(Msg::ResetNode);

        let zoom_badge = shadcn::Badge::new(format!("Zoom: {:.2}", view_value.zoom))
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Meter)
                    .test_id(TEST_ID_ZOOM)
                    .numeric_value(view_value.zoom as f64)
                    .numeric_range(0.05, 64.0),
            );

        let pan_x = shadcn::Badge::new(format!("Pan X: {:.0}", view_value.pan.x.0))
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Meter)
                    .test_id(TEST_ID_PAN_X)
                    .numeric_value(view_value.pan.x.0 as f64),
            );
        let pan_y = shadcn::Badge::new(format!("Pan Y: {:.0}", view_value.pan.y.0))
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Meter)
                    .test_id(TEST_ID_PAN_Y)
                    .numeric_value(view_value.pan.y.0 as f64),
            );

        let drag_badge = shadcn::Badge::new(format!("Node drags: {node_drag_count}"))
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Meter)
                    .test_id(TEST_ID_NODE_DRAGS)
                    .numeric_value(node_drag_count as f64),
            );

        let toolbar = ui::h_flex(cx, |cx| {
            [
                shadcn::Button::new("Reset view")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(reset_view)
                    .into_element(cx)
                    .test_id(TEST_ID_RESET_VIEW),
                shadcn::Button::new("Reset node")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(reset_node)
                    .into_element(cx)
                    .test_id(TEST_ID_RESET_NODE),
                zoom_badge,
                pan_x,
                pan_y,
                drag_badge,
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let hint = shadcn::Alert::new([
            shadcn::AlertTitle::new("Interactions").into_element(cx),
            shadcn::AlertDescription::new(
                "Middle-drag pans. Wheel zooms. Left-drag the rectangle to move it in canvas space.",
            )
            .into_element(cx),
        ])
        .ui()
        .into_element(cx);

        let canvas = {
            let view_model = st.view.clone();
            let node_origin_model = st.node_origin.clone();
            let drag_model = st.node_drag.clone();
            let drag_count_model = st.node_drag_count.clone();

            let view_model_down = view_model.clone();
            let node_origin_model_down = node_origin_model.clone();
            let drag_model_down = drag_model.clone();
            let on_pointer_down: OnPointerDown = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      down: fret_ui::action::PointerDownCx| {
                    if down.button != MouseButton::Left {
                        return false;
                    }

                    let bounds = host.bounds();
                    let view = host
                        .models_mut()
                        .read(&view_model_down, |v| *v)
                        .ok()
                        .unwrap_or_default();
                    let origin = host
                        .models_mut()
                        .read(&node_origin_model_down, |p| *p)
                        .ok()
                        .unwrap_or(Point::new(Px(0.0), Px(0.0)));

                    let canvas_pos = view.screen_to_canvas(bounds, down.position);
                    let r = node_rect(origin);
                    let inside = canvas_pos.x.0 >= r.origin.x.0
                        && canvas_pos.y.0 >= r.origin.y.0
                        && canvas_pos.x.0 <= r.origin.x.0 + r.size.width.0
                        && canvas_pos.y.0 <= r.origin.y.0 + r.size.height.0;
                    if !inside {
                        return false;
                    }

                    host.prevent_default(DefaultAction::FocusOnPointerDown);
                    host.capture_pointer();
                    host.set_cursor_icon(CursorIcon::Pointer);

                    let _ = host.models_mut().update(&drag_model_down, |st| {
                        *st = Some(NodeDragState {
                            pointer_id: down.pointer_id,
                            start_canvas: canvas_pos,
                            origin_at_start: origin,
                        });
                    });
                    host.request_redraw(action_cx.window);
                    true
                },
            );

            let view_model_move = view_model.clone();
            let node_origin_model_move = node_origin_model.clone();
            let drag_model_move = drag_model.clone();
            let on_pointer_move: OnPointerMove = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      mv: fret_ui::action::PointerMoveCx| {
                    let bounds = host.bounds();
                    let view = host
                        .models_mut()
                        .read(&view_model_move, |v| *v)
                        .ok()
                        .unwrap_or_default();

                    let drag = host
                        .models_mut()
                        .read(&drag_model_move, |st| *st)
                        .ok()
                        .flatten();
                    if let Some(drag) = drag {
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
                        let _ = host
                            .models_mut()
                            .update(&node_origin_model_move, |p| *p = next);

                        host.request_redraw(action_cx.window);
                        host.set_cursor_icon(CursorIcon::Pointer);
                        return true;
                    }

                    // Hover cursor hint (do not consume the event).
                    let origin = host
                        .models_mut()
                        .read(&node_origin_model_move, |p| *p)
                        .ok()
                        .unwrap_or(Point::new(Px(0.0), Px(0.0)));
                    let canvas_pos = view.screen_to_canvas(bounds, mv.position);
                    let r = node_rect(origin);
                    let inside = canvas_pos.x.0 >= r.origin.x.0
                        && canvas_pos.y.0 >= r.origin.y.0
                        && canvas_pos.x.0 <= r.origin.x.0 + r.size.width.0
                        && canvas_pos.y.0 <= r.origin.y.0 + r.size.height.0;
                    if inside {
                        host.set_cursor_icon(CursorIcon::Pointer);
                    }
                    false
                },
            );

            let drag_model_up = drag_model.clone();
            let drag_count_model_up = drag_count_model.clone();
            let on_pointer_up: OnPointerUp = Arc::new(
                move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      up: fret_ui::action::PointerUpCx| {
                    if up.button != MouseButton::Left {
                        return false;
                    }

                    let drag = host
                        .models_mut()
                        .read(&drag_model_up, |st| *st)
                        .ok()
                        .flatten();
                    let Some(drag) = drag else {
                        return false;
                    };
                    if drag.pointer_id != up.pointer_id {
                        return false;
                    }

                    host.release_pointer_capture();
                    let _ = host.models_mut().update(&drag_model_up, |st| *st = None);
                    let _ = host.models_mut().update(&drag_count_model_up, |n| {
                        *n = n.saturating_add(1);
                    });
                    host.request_redraw(action_cx.window);
                    true
                },
            );

            let mut props = PanZoomCanvasSurfacePanelProps::default();
            props.preset = PanZoomInputPreset::DesktopCanvasCad;
            props.view = Some(st.view.clone());
            props.default_view = PanZoom2D::default();
            props.pan_button = MouseButton::Middle;
            props.on_pointer_down = Some(on_pointer_down);
            props.on_pointer_move = Some(on_pointer_move);
            props.on_pointer_up = Some(on_pointer_up);
            props.canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

            let bg = theme.color_token("card");
            let grid = theme.color_token("border");
            let node_fill = theme.color_token("primary");
            let node_border = theme.color_token("primary-foreground");

            let paint =
                move |p: &mut CanvasPainter<'_>,
                      paint_cx: fret_canvas::ui::PanZoomCanvasPaintCx| {
                    let bounds = p.bounds();
                    let Some(transform) = paint_cx.view.render_transform(bounds) else {
                        return;
                    };

                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect: bounds,
                        background: Paint::Solid(bg).into(),
                        border: Edges::all(Px(0.0)),
                        border_paint: Paint::Solid(fret_core::scene::Color::TRANSPARENT).into(),
                        corner_radii: Corners::all(Px(0.0)),
                    });

                    let vis = visible_canvas_rect(bounds, paint_cx.view);
                    let step = 80.0f32;
                    let min_x = (vis.origin.x.0 / step).floor() as i32 - 2;
                    let max_x = ((vis.origin.x.0 + vis.size.width.0) / step).ceil() as i32 + 2;
                    let min_y = (vis.origin.y.0 / step).floor() as i32 - 2;
                    let max_y = ((vis.origin.y.0 + vis.size.height.0) / step).ceil() as i32 + 2;
                    let line_w = fret_canvas::scale::constant_pixel_stroke_width(
                        Px(1.0),
                        paint_cx.view.zoom,
                    );

                    p.with_clip_rect(bounds, |p| {
                        p.with_transform(transform, |p| {
                            for x in min_x..=max_x {
                                let ox = x as f32 * step;
                                let rect = Rect::new(
                                    Point::new(Px(ox), Px(min_y as f32 * step)),
                                    Size::new(line_w, Px((max_y - min_y) as f32 * step)),
                                );
                                p.scene().push(SceneOp::Quad {
                                    order: DrawOrder(1),
                                    rect,
                                    background: Paint::Solid(grid).into(),
                                    border: Edges::all(Px(0.0)),
                                    border_paint: Paint::Solid(
                                        fret_core::scene::Color::TRANSPARENT,
                                    )
                                    .into(),
                                    corner_radii: Corners::all(Px(0.0)),
                                });
                            }
                            for y in min_y..=max_y {
                                let oy = y as f32 * step;
                                let rect = Rect::new(
                                    Point::new(Px(min_x as f32 * step), Px(oy)),
                                    Size::new(Px((max_x - min_x) as f32 * step), line_w),
                                );
                                p.scene().push(SceneOp::Quad {
                                    order: DrawOrder(1),
                                    rect,
                                    background: Paint::Solid(grid).into(),
                                    border: Edges::all(Px(0.0)),
                                    border_paint: Paint::Solid(
                                        fret_core::scene::Color::TRANSPARENT,
                                    )
                                    .into(),
                                    corner_radii: Corners::all(Px(0.0)),
                                });
                            }

                            let node = node_rect(node_origin);
                            let border_w = fret_canvas::scale::constant_pixel_stroke_width(
                                Px(2.0),
                                paint_cx.view.zoom,
                            );
                            p.scene().push(SceneOp::Quad {
                                order: DrawOrder(10),
                                rect: node,
                                background: Paint::Solid(node_fill).into(),
                                border: Edges::all(border_w),
                                border_paint: Paint::Solid(node_border).into(),
                                corner_radii: Corners::all(Px(10.0)),
                            });
                        });
                    });
                };

            pan_zoom_canvas_surface_panel(cx, props, paint)
                .test_id(TEST_ID_CANVAS)
                .into()
        };

        let card = shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Canvas pan/zoom basics").into_element(cx),
                shadcn::CardDescription::new(
                    "Uses fret-canvas pan/zoom wiring + a tiny app-owned drag tool for one item.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([ui::v_flex(cx, |_cx| [toolbar, hint, canvas])
                .gap(Space::N3)
                .w_full()
                .into_element(cx)])
            .into_element(cx),
        ])
        .ui()
        .w_full()
        .max_w(Px(980.0))
        .into_element(cx);

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-canvas-pan-zoom-basics")
        .window("cookbook-canvas-pan-zoom-basics", (1120.0, 780.0))
        .config_files(false)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_mvu::<CanvasPanZoomBasicsProgram>()
        .map_err(anyhow::Error::from)
}
