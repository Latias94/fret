use std::sync::Arc;

use fret::component::prelude::*;
use fret::{FretApp, advanced::prelude::*, shadcn};
use fret_core::{CursorIcon, MouseButton, Point, PointerId, Px};
use fret_runtime::DefaultAction;
use fret_ui::Invalidation;
use fret_ui::action::UiPointerActionHost;
use fret_ui::element::{AnyElement, PointerRegionProps};

const TEST_ID_ROOT: &str = "cookbook.drag_basics.root";
const TEST_ID_DRAGGABLE: &str = "cookbook.drag_basics.draggable";
const TEST_ID_POS: &str = "cookbook.drag_basics.pos";
const TEST_ID_DRAG_COUNT: &str = "cookbook.drag_basics.drag_count";

#[derive(Debug, Clone, Copy)]
struct DragState {
    pointer: PointerId,
    start_local: Point,
    origin_at_start: Point,
}

struct DragBasicsView {
    origin: Model<Point>,
    drag: Model<Option<DragState>>,
    drag_count: Model<u32>,
}

impl View for DragBasicsView {
    fn init(app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self {
            origin: app.models_mut().insert(Point::new(Px(0.0), Px(0.0))),
            drag: app.models_mut().insert(None::<DragState>),
            drag_count: app.models_mut().insert(0),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();

        let origin = self
            .origin
            .layout(cx)
            .value_or(Point::new(Px(0.0), Px(0.0)));
        let drag_count = self.drag_count.layout(cx).value_or(0);

        let pos_label = format!("Offset: ({:.0}, {:.0})", origin.x.0, origin.y.0);

        let drag_model = self.drag.clone();
        let origin_model = self.origin.clone();

        let on_pointer_down: fret_ui::action::OnPointerDown =
            Arc::new(move |host: &mut dyn UiPointerActionHost, action_cx, down| {
                if down.button != MouseButton::Left {
                    return false;
                }

                host.prevent_default(DefaultAction::FocusOnPointerDown);
                host.capture_pointer();
                host.set_cursor_icon(CursorIcon::Pointer);

                let st = DragState {
                    pointer: down.pointer_id,
                    start_local: down.position_local,
                    origin_at_start: host
                        .models_mut()
                        .read(&origin_model, |p| *p)
                        .ok()
                        .unwrap_or_else(|| Point::new(Px(0.0), Px(0.0))),
                };

                let _ = host.models_mut().update(&drag_model, |v| *v = Some(st));
                host.invalidate(Invalidation::Paint);
                host.request_redraw(action_cx.window);
                true
            });

        let drag_model_move = self.drag.clone();
        let origin_model_move = self.origin.clone();
        let on_pointer_move: fret_ui::action::OnPointerMove =
            Arc::new(move |host: &mut dyn UiPointerActionHost, action_cx, mv| {
                let drag = host
                    .models_mut()
                    .read(&drag_model_move, |v| *v)
                    .ok()
                    .flatten();
                let Some(drag) = drag else {
                    return false;
                };
                if drag.pointer != mv.pointer_id {
                    return false;
                }

                let dx = mv.position_local.x.0 - drag.start_local.x.0;
                let dy = mv.position_local.y.0 - drag.start_local.y.0;
                let _ = host.models_mut().update(&origin_model_move, |p| {
                    let x = (drag.origin_at_start.x.0 + dx).clamp(0.0, 480.0);
                    let y = (drag.origin_at_start.y.0 + dy).clamp(0.0, 120.0);
                    *p = Point::new(Px(x), Px(y));
                });

                host.invalidate(Invalidation::Paint);
                host.request_redraw(action_cx.window);
                true
            });

        let drag_model_up = self.drag.clone();
        let drag_count_model_up = self.drag_count.clone();
        let on_pointer_up: fret_ui::action::OnPointerUp =
            Arc::new(move |host: &mut dyn UiPointerActionHost, action_cx, up| {
                if up.button != MouseButton::Left {
                    return false;
                }

                host.release_pointer_capture();
                host.set_cursor_icon(CursorIcon::Default);

                let _ = host.models_mut().update(&drag_model_up, |v| *v = None);
                let _ = host
                    .models_mut()
                    .update(&drag_count_model_up, |n| *n = n.saturating_add(1));
                host.invalidate(Invalidation::Paint);
                host.request_redraw(action_cx.window);
                true
            });

        let header = shadcn::card_header(|cx| {
            ui::children![cx;
                shadcn::card_title("Drag basics"),
                shadcn::card_description(
                    "A tiny pointer-capture drag example using a pointer region.",
                ),
            ]
        });

        let pos = shadcn::Badge::new(pos_label)
            .variant(shadcn::BadgeVariant::Secondary)
            .test_id(TEST_ID_POS);

        let drag_count_badge = shadcn::Badge::new(format!("Drags: {drag_count}"))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
                fret_ui::element::SemanticsDecoration::default()
                    .role(SemanticsRole::ProgressBar)
                    .test_id(TEST_ID_DRAG_COUNT)
                    .numeric_value(drag_count as f64)
                    .numeric_range(0.0, 1024.0),
            );

        let mut region = PointerRegionProps::default();
        region.layout.size.width = Length::Fill;
        region.layout.size.height = Length::Px(Px(240.0));

        let draggable = cx.pointer_region(region, |cx| {
            cx.pointer_region_on_pointer_down(on_pointer_down);
            cx.pointer_region_on_pointer_move(on_pointer_move);
            cx.pointer_region_on_pointer_up(on_pointer_up);

            let box_size = Px(72.0);
            let box_el = ui::v_flex(|cx| [cx.text("Drag")])
                .w_px(box_size)
                .h_px(box_size)
                .items_center()
                .justify_center()
                .bg(ColorRef::Color(theme.color_token("primary")))
                .rounded(Radius::Lg)
                .text_color(ColorRef::Color(theme.color_token("primary-foreground")))
                .test_id(TEST_ID_DRAGGABLE);

            let offset_x = Px(origin.x.0.clamp(0.0, 480.0));
            let offset_y = Px(origin.y.0.clamp(0.0, 120.0));

            let top_spacer = ui::container(|_cx| Vec::<AnyElement>::new()).h_px(offset_y);
            let left_spacer = ui::container(|_cx| Vec::<AnyElement>::new()).w_px(offset_x);

            let row = ui::h_flex(|cx| ui::children![cx; left_spacer, box_el]);
            let col = ui::v_flex(|cx| ui::children![cx; top_spacer, row]);

            let bounds = ui::container(|cx| ui::children![cx; col])
                .w_full()
                .h_full()
                .bg(ColorRef::Color(theme.color_token("muted")))
                .rounded(Radius::Lg)
                .into_element(cx);

            vec![bounds]
        });

        let card = shadcn::card(|cx| {
            ui::children![cx;
                header,
                shadcn::card_content(|cx| {
                    ui::children![cx;
                        ui::v_flex(|cx| ui::children![cx; pos, drag_count_badge, draggable])
                            .gap(Space::N3)
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(720.0));

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-drag-basics")
        .window("cookbook-drag-basics", (760.0, 520.0))
        .config_files(false)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<DragBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
