use std::sync::Arc;

use fret::app::LocalState;
use fret::app::prelude::*;
use fret::icons::IconId;
use fret::shadcn::raw::{LayoutRefinement, icon};
use fret::style::{
    ChromeRefinement, ColorRef, MetricRef, Radius, Space, TextOverflow, TextWrap, Theme,
    ThemeSnapshot,
};
use fret_core::scene::DashPatternV1;
use fret_core::{
    AttributedText, Color, Corners, DecorationLineStyle, Px, StrikethroughStyle, TextAlign,
    TextPaintStyle, TextSpan,
};
use fret_ui::element::{HoverRegionProps, StyledTextProps};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{WidgetStateProperty, WidgetStates, typography};

mod act {
    fret::actions!([
        Add = "todo_demo.add.v1",
        ClearDone = "todo_demo.clear_done.v1",
        FilterAll = "todo_demo.filter_all.v1",
        FilterActive = "todo_demo.filter_active.v1",
        FilterCompleted = "todo_demo.filter_completed.v1"
    ]);

    fret::payload_actions!([
        Toggle(u64) = "todo_demo.toggle.v1",
        Remove(u64) = "todo_demo.remove.v2"
    ]);
}

const TEST_ID_ROOT: &str = "todo_demo.root";
const TEST_ID_DRAFT: &str = "todo_demo.draft";
const TEST_ID_ADD: &str = "todo_demo.add";
const TEST_ID_CLEAR_DONE: &str = "todo_demo.clear_done";
const TEST_ID_PROGRESS: &str = "todo_demo.progress";
const TEST_ID_ROWS: &str = "todo_demo.rows";
const TEST_ID_FILTER_ALL: &str = "todo_demo.filter.all";
const TEST_ID_FILTER_ACTIVE: &str = "todo_demo.filter.active";
const TEST_ID_FILTER_COMPLETED: &str = "todo_demo.filter.completed";
const TEST_ID_DONE_PREFIX: &str = "todo_demo.done.";
const TEST_ID_ROW_PREFIX: &str = "todo_demo.row.";
const TEST_ID_REMOVE_PREFIX: &str = "todo_demo.remove.";

#[derive(Clone)]
struct TodoRow {
    id: u64,
    done: bool,
    text: Arc<str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TodoFilter {
    All,
    Active,
    Completed,
}

impl TodoFilter {
    fn matches(self, done: bool) -> bool {
        match self {
            Self::All => true,
            Self::Active => !done,
            Self::Completed => done,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::All => "全部",
            Self::Active => "进行中",
            Self::Completed => "已完成",
        }
    }
}

struct TodoLocals {
    draft: LocalState<String>,
    filter: LocalState<TodoFilter>,
    next_id: LocalState<u64>,
    todos: LocalState<Vec<TodoRow>>,
}

impl TodoLocals {
    fn new(app: &mut App) -> Self {
        Self {
            draft: LocalState::from_model(app.models_mut().insert(String::new())),
            filter: LocalState::from_model(app.models_mut().insert(TodoFilter::All)),
            next_id: LocalState::from_model(app.models_mut().insert(4u64)),
            todos: LocalState::from_model(app.models_mut().insert(vec![
                TodoRow {
                    id: 1,
                    done: true,
                    text: Arc::from("学习 React Hooks"),
                },
                TodoRow {
                    id: 2,
                    done: true,
                    text: Arc::from("掌握 Tailwind CSS"),
                },
                TodoRow {
                    id: 3,
                    done: false,
                    text: Arc::from("构建现代化 Todo 应用"),
                },
            ])),
        }
    }

    fn bind_actions(&self, cx: &mut AppUi<'_, '_>) {
        cx.actions()
            .locals_with((&self.draft, &self.next_id, &self.todos))
            .on::<act::Add>(|tx, (draft, next_id, todos)| {
                let text = tx.value(&draft).trim().to_string();
                if text.is_empty() {
                    return false;
                }

                let id = tx.value(&next_id);
                let _ = tx.update(&next_id, |value| *value = value.saturating_add(1));

                if !tx.update(&todos, |rows| {
                    rows.push(TodoRow {
                        id,
                        done: false,
                        text: Arc::from(text),
                    });
                }) {
                    return false;
                }

                tx.set(&draft, String::new())
            });

        cx.actions()
            .locals_with(&self.todos)
            .on::<act::ClearDone>(|tx, todos| {
                tx.update_if(&todos, |rows| {
                    let before = rows.len();
                    rows.retain(|row| !row.done);
                    rows.len() != before
                })
            });

        cx.actions()
            .local(&self.filter)
            .set::<act::FilterAll>(TodoFilter::All);
        cx.actions()
            .local(&self.filter)
            .set::<act::FilterActive>(TodoFilter::Active);
        cx.actions()
            .local(&self.filter)
            .set::<act::FilterCompleted>(TodoFilter::Completed);

        cx.actions()
            .local(&self.todos)
            .payload_update_if::<act::Toggle>(|rows, id| {
                if let Some(row) = rows.iter_mut().find(|row| row.id == id) {
                    row.done = !row.done;
                    true
                } else {
                    false
                }
            });

        cx.actions()
            .local(&self.todos)
            .payload_update_if::<act::Remove>(|rows, id| {
                let before = rows.len();
                rows.retain(|row| row.id != id);
                rows.len() != before
            });
    }
}

struct TodoDemoView {
    locals: TodoLocals,
}

impl View for TodoDemoView {
    fn init(app: &mut App, _window: WindowId) -> Self {
        Self {
            locals: TodoLocals::new(app),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();
        let locals = &self.locals;
        locals.bind_actions(cx);

        let todos = locals.todos.layout_value(cx);
        let draft_value = locals.draft.layout_value(cx);
        let filter_value = locals.filter.layout_value(cx);

        let filtered_todos: Vec<TodoRow> = todos
            .iter()
            .filter(|row| filter_value.matches(row.done))
            .cloned()
            .collect();

        let done_count = todos.iter().filter(|row| row.done).count();
        let total_count = todos.len();
        let active_count = total_count.saturating_sub(done_count);
        let progress_pct = if total_count == 0 {
            0.0
        } else {
            (done_count as f32 / total_count as f32) * 100.0
        };
        let add_enabled = !draft_value.trim().is_empty();
        let has_completed = done_count > 0;
        let muted_foreground = theme.color_token("muted-foreground");
        let primary = theme.color_token("primary");
        let primary_foreground = theme.color_token("primary-foreground");
        let muted = theme.color_token("muted");
        let border = theme.color_token("border");
        let card = theme.color_token("card");
        let destructive = theme.color_token("destructive");
        let footer_bg = alpha(muted, 0.5);
        let success = Color::from_srgb_hex_rgb(0x22_c5_5e);
        let title_icon = ui::v_flex(|cx| {
            let icon_el = icon::icon_with(
                cx,
                IconId::new("lucide.list-todo"),
                Some(Px(20.0)),
                Some(ColorRef::Color(primary_foreground)),
            );
            ui::single(cx, icon_el)
        })
        .w_px(Px(44.0))
        .h_px(Px(44.0))
        .justify_center()
        .items_center()
        .bg(ColorRef::Color(primary))
        .rounded(Radius::Lg)
        .shadow_sm();

        let status_line = if total_count == 0 {
            ui::text("添加一个新任务开始吧")
                .text_sm()
                .text_color(ColorRef::Color(muted_foreground))
                .into_element(cx)
        } else if active_count == 0 {
            ui::h_flex(|cx| {
                ui::children![
                    cx;
                    icon::icon_with(
                        cx,
                        IconId::new("lucide.sparkles"),
                        Some(Px(14.0)),
                        Some(ColorRef::Color(success)),
                    ),
                    ui::text("太棒了！所有任务已完成")
                        .text_sm()
                        .text_color(ColorRef::Color(muted_foreground))
                        .into_element(cx),
                ]
            })
            .gap(Space::N1)
            .items_center()
            .into_element(cx)
        } else {
            ui::text(format!("还有 {active_count} 个未完成的任务"))
                .text_sm()
                .text_color(ColorRef::Color(muted_foreground))
                .into_element(cx)
        };

        let title_block = ui::v_flex(|cx| {
            ui::children![
                cx;
                ui::text("我的待办事项").text_base().font_semibold(),
                status_line,
            ]
        })
        .gap(Space::N1)
        .flex_1()
        .min_w_0();

        let header_top = ui::h_flex(|cx| ui::children![cx; title_icon, title_block])
            .gap(Space::N3)
            .items_center()
            .w_full();

        let progress_block = (total_count > 0).then(|| {
            ui::v_flex(|cx| {
                ui::children![
                    cx;
                    ui::h_flex(|cx| {
                        ui::children![
                            cx;
                            ui::text("完成进度")
                                .text_xs()
                                .text_color(ColorRef::Color(muted_foreground))
                                .into_element(cx),
                            ui::text(format!("{:.0}%", progress_pct))
                                .text_xs()
                                .text_color(ColorRef::Color(muted_foreground))
                                .into_element(cx),
                        ]
                    })
                    .items_center()
                    .justify_between()
                    .w_full()
                    .into_element(cx),
                    shadcn::Progress::from_value(progress_pct)
                        .a11y_label("Todo completion progress")
                        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx)
                        .test_id(TEST_ID_PROGRESS),
                ]
            })
            .gap(Space::N1p5)
            .w_full()
            .into_element(cx)
        });

        let header = ui::v_flex(|cx| {
            let mut children = vec![header_top.into_element(cx)];
            if let Some(progress_block) = progress_block {
                children.push(progress_block);
            }
            children
        })
        .gap(Space::N3)
        .w_full();

        let add_btn = shadcn::Button::new("")
            .size(shadcn::ButtonSize::Icon)
            .disabled(!add_enabled)
            .action(act::Add)
            .a11y_label("添加任务")
            .corner_radii_override(Corners::all(Px(14.0)))
            .refine_style(ChromeRefinement::default().shadow_sm())
            .children([icon::icon(cx, IconId::new("lucide.plus"))])
            .test_id(TEST_ID_ADD);

        let input = shadcn::Input::new(&locals.draft)
            .a11y_label("新任务")
            .placeholder("添加新任务...")
            .submit_action(act::Add)
            .corner_radii_override(Corners::all(Px(14.0)))
            .refine_style(ChromeRefinement::default().shadow_sm())
            .test_id(TEST_ID_DRAFT)
            .ui()
            .flex_1()
            .min_w_0();

        let input_row = ui::h_flex(|cx| ui::children![cx; input, add_btn])
            .gap(Space::N2)
            .items_center()
            .w_full();

        let rows_body = ui::v_flex(|cx| {
            if filtered_todos.is_empty() {
                let empty_label = match filter_value {
                    TodoFilter::All => "没有待办任务，享受休息吧！",
                    TodoFilter::Active => "没有进行中的任务",
                    TodoFilter::Completed => "没有已完成的任务",
                };

                let empty_icon = ui::v_flex(|cx| {
                    let icon_el = icon::icon_with(
                        cx,
                        IconId::new("lucide.check"),
                        Some(Px(22.0)),
                        Some(ColorRef::Color(muted_foreground)),
                    );
                    ui::single(cx, icon_el)
                })
                .w_px(Px(48.0))
                .h_px(Px(48.0))
                .justify_center()
                .items_center()
                .bg(ColorRef::Color(muted))
                .rounded(Radius::Full);

                return ui::children![
                    cx;
                    ui::v_flex(|cx| {
                        ui::children![
                            cx;
                            empty_icon,
                            ui::text(empty_label)
                                .text_sm()
                                .text_color(ColorRef::Color(muted_foreground)),
                        ]
                    })
                    .gap(Space::N3)
                    .justify_center()
                    .items_center()
                    .border_1()
                    .border_color(ColorRef::Color(theme.color_token("border")))
                    .style(ChromeRefinement::default().border_dash(DashPatternV1::new(
                        Px(4.0),
                        Px(4.0),
                        Px(0.0),
                    )))
                    .rounded(Radius::Lg)
                    .p(Space::N6)
                    .w_full()
                    .into_element(cx)
                ];
            }

            ui::for_each_keyed(
                cx,
                &filtered_todos,
                |row| row.id,
                |row| {
                    let theme = theme.clone();
                    todo_row(theme, row)
                },
            )
        })
        .gap(Space::N2)
        .w_full()
        .items_stretch();

        let rows = shadcn::ScrollArea::new([rows_body.into_element(cx)])
            .refine_layout(LayoutRefinement::default().w_full().max_h(Px(420.0)))
            .viewport_test_id(TEST_ID_ROWS)
            .into_element(cx);

        let filters = ui::h_flex(|cx| {
            ui::children![
                cx;
                filter_chip(TodoFilter::All, filter_value, act::FilterAll, TEST_ID_FILTER_ALL),
                filter_chip(
                    TodoFilter::Active,
                    filter_value,
                    act::FilterActive,
                    TEST_ID_FILTER_ACTIVE,
                ),
                filter_chip(
                    TodoFilter::Completed,
                    filter_value,
                    act::FilterCompleted,
                    TEST_ID_FILTER_COMPLETED,
                ),
            ]
        })
        .gap(Space::N1)
        .items_center()
        .wrap();

        let clear_done_style = shadcn::raw::button::ButtonStyle::default()
            .background(
                WidgetStateProperty::new(Some(ColorRef::Color(Color::TRANSPARENT)))
                    .when(
                        WidgetStates::HOVERED,
                        Some(ColorRef::Color(alpha(destructive, 0.10))),
                    )
                    .when(
                        WidgetStates::ACTIVE,
                        Some(ColorRef::Color(alpha(destructive, 0.16))),
                    ),
            )
            .foreground(
                WidgetStateProperty::new(Some(ColorRef::Color(muted_foreground)))
                    .when(WidgetStates::HOVERED, Some(ColorRef::Color(destructive)))
                    .when(WidgetStates::ACTIVE, Some(ColorRef::Color(destructive))),
            );

        let clear_done_btn = shadcn::Button::new("清除已完成")
            .variant(shadcn::ButtonVariant::Ghost)
            .size(shadcn::ButtonSize::Xs)
            .corner_radii_override(Corners::all(Px(9999.0)))
            .style(clear_done_style)
            .disabled(!has_completed)
            .action(act::ClearDone)
            .refine_style(footer_pill_chrome())
            .refine_layout(footer_pill_layout())
            .test_id(TEST_ID_CLEAR_DONE);

        let footer = ui::h_flex(|cx| {
            let mut children = vec![filters.into_element(cx)];
            if has_completed {
                children.push(clear_done_btn.into_element(cx));
            }
            children
        })
        .gap(Space::N3)
        .items_center()
        .justify_between()
        .w_full();

        let card = ui::container(|cx| {
            ui::single(
                cx,
                ui::v_flex(|cx| {
                    let mut sections = vec![
                        ui::container(|cx| ui::single(cx, header))
                            .px(Space::N6)
                            .pt(Space::N6)
                            .pb(Space::N4)
                            .w_full()
                            .into_element(cx),
                        ui::container(|cx| ui::single(cx, input_row))
                            .px(Space::N6)
                            .pb(Space::N4)
                            .w_full()
                            .into_element(cx),
                        ui::container(|cx| ui::single(cx, rows))
                            .px(Space::N6)
                            .pb(Space::N2)
                            .w_full()
                            .into_element(cx),
                    ];

                    if total_count > 0 {
                        sections.push(
                            ui::v_flex(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::Separator::new().into_element(cx),
                                    ui::container(|cx| ui::single(cx, footer))
                                        .px(Space::N6)
                                        .py(Space::N3p5)
                                        .bg(ColorRef::Color(footer_bg))
                                        .w_full()
                                        .into_element(cx),
                                ]
                            })
                            .gap(Space::N0)
                            .w_full()
                            .into_element(cx),
                        );
                    }

                    sections
                })
                .w_full(),
            )
        })
        .bg(ColorRef::Color(card))
        .border_1()
        .border_color(ColorRef::Color(border))
        .rounded(Radius::Lg)
        .shadow_xl()
        .overflow_hidden()
        .w_full()
        .max_w(Px(448.0))
        .max_h(Px(720.0))
        .test_id(TEST_ID_ROOT);

        ui::single(cx, todo_page(theme, card))
    }
}

fn todo_page(theme: ThemeSnapshot, content: impl UiChild) -> impl UiChild {
    ui::container(move |cx| {
        ui::single(
            cx,
            ui::v_flex(move |cx| ui::single(cx, content))
                .w_full()
                .h_full()
                .justify_center()
                .items_center(),
        )
    })
    .bg(ColorRef::Color(theme.color_token("background")))
    .p(Space::N4)
    .w_full()
    .h_full()
}

fn filter_chip(
    filter: TodoFilter,
    current: TodoFilter,
    action: impl Into<fret::ActionId>,
    test_id: &str,
) -> impl UiChild {
    let selected = filter == current;
    let action: fret::ActionId = action.into();

    ui::keyed(("todo-filter-chip", filter.label()), move |_cx| {
        shadcn::Button::new(filter.label())
            .variant(if selected {
                shadcn::ButtonVariant::Secondary
            } else {
                shadcn::ButtonVariant::Ghost
            })
            .size(shadcn::ButtonSize::Xs)
            .corner_radii_override(Corners::all(Px(9999.0)))
            .refine_style(footer_pill_chrome())
            .refine_layout(footer_pill_layout())
            .action(action)
            .test_id(test_id)
    })
}

fn todo_row(theme: ThemeSnapshot, row: &TodoRow) -> impl UiChild {
    let row_done = row.done;
    let row_id = row.id;
    let row_text = row.text.clone();
    let primary_foreground = theme.color_token("primary-foreground");
    let muted_foreground = theme.color_token("muted-foreground");
    let foreground = theme.color_token("foreground");
    let background = theme.color_token("background");
    let border = theme.color_token("border");
    let destructive = theme.color_token("destructive");
    let success = Color::from_srgb_hex_rgb(0x22_c5_5e);
    let done_surface = Color::from_srgb_hex_rgb(0xf8_fa_fc);
    let hover_border = alpha(theme.color_token("primary"), 0.20);
    let mut destructive_hover_bg = destructive;
    destructive_hover_bg.a = 0.10;
    let mut destructive_active_bg = destructive;
    destructive_active_bg.a = 0.16;

    ui::container(move |cx| {
        let mut hover = HoverRegionProps::default();
        hover.layout = decl_style::layout_style(&theme, LayoutRefinement::default().w_full());
        let hover_region = cx.hover_region(hover, move |cx, hovered| {
            let toggle_visual = if row_done {
                ui::v_flex(|cx| {
                    let icon_el = icon::icon_with(
                        cx,
                        IconId::new("lucide.check"),
                        Some(Px(14.0)),
                        Some(ColorRef::Color(primary_foreground)),
                    );
                    ui::single(cx, icon_el)
                })
                .w_px(Px(20.0))
                .h_px(Px(20.0))
                .justify_center()
                .items_center()
                .bg(ColorRef::Color(success))
                .rounded(Radius::Full)
                .into_element(cx)
            } else {
                icon::icon_with(
                    cx,
                    IconId::new("lucide.circle"),
                    Some(Px(20.0)),
                    Some(ColorRef::Color(muted_foreground)),
                )
            };

            let toggle = shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Ghost)
                .size(shadcn::ButtonSize::IconSm)
                .corner_radii_override(Corners::all(Px(9999.0)))
                .action(act::Toggle)
                .action_payload(row_id)
                .a11y_label(if row_done {
                    "标记为未完成"
                } else {
                    "标记为已完成"
                })
                .children([toggle_visual])
                .test_id(format!("{TEST_ID_DONE_PREFIX}{row_id}"));

            let text = if row_done {
                let rich = rich_strikethrough(&row_text, muted_foreground);
                let style = typography::TypographyPreset::control_ui(typography::UiTextSize::Sm)
                    .resolve(&theme);
                cx.styled_text_props(StyledTextProps {
                    layout: decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default().flex_1().min_w_0(),
                    ),
                    rich,
                    style: Some(style),
                    color: Some(muted_foreground),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Ellipsis,
                    align: TextAlign::Start,
                    ink_overflow: Default::default(),
                })
            } else {
                ui::text(row_text.clone())
                    .truncate()
                    .text_sm()
                    .font_medium()
                    .flex_1()
                    .min_w_0()
                    .text_color(ColorRef::Color(foreground))
                    .into_element(cx)
            };

            let leading = ui::h_flex(|cx| ui::children![cx; toggle, text])
                .gap(Space::N3)
                .items_center()
                .flex_1()
                .min_w_0();

            let delete_style = shadcn::raw::button::ButtonStyle::default()
                .background(
                    WidgetStateProperty::new(Some(ColorRef::Color(Color::TRANSPARENT)))
                        .when(
                            WidgetStates::HOVERED,
                            Some(ColorRef::Color(destructive_hover_bg)),
                        )
                        .when(
                            WidgetStates::ACTIVE,
                            Some(ColorRef::Color(destructive_active_bg)),
                        ),
                )
                .foreground(
                    WidgetStateProperty::new(Some(ColorRef::Color(muted_foreground)))
                        .when(WidgetStates::HOVERED, Some(ColorRef::Color(destructive)))
                        .when(WidgetStates::ACTIVE, Some(ColorRef::Color(destructive))),
                );

            let remove_button = shadcn::Button::new("")
                .variant(shadcn::ButtonVariant::Ghost)
                .size(shadcn::ButtonSize::IconSm)
                .corner_radii_override(Corners::all(Px(10.0)))
                .style(delete_style)
                .action(act::Remove)
                .action_payload(row_id)
                .a11y_label("删除任务")
                .children([icon::icon_with(
                    cx,
                    IconId::new("lucide.trash-2"),
                    Some(Px(16.0)),
                    Some(ColorRef::Color(muted_foreground)),
                )])
                .test_id(format!("{TEST_ID_REMOVE_PREFIX}{row_id}"));

            let remove = cx.interactivity_gate(true, hovered, move |cx| {
                vec![cx.opacity(if hovered { 1.0 } else { 0.0 }, move |cx| {
                    vec![remove_button.into_element(cx)]
                })]
            });

            let row = ui::h_flex(|cx| ui::children![cx; leading, remove])
                .gap(Space::N2)
                .items_center()
                .justify_between()
                .bg(ColorRef::Color(if row_done {
                    done_surface
                } else {
                    background
                }))
                .border_1()
                .border_color(ColorRef::Color(if row_done {
                    Color::TRANSPARENT
                } else if hovered {
                    hover_border
                } else {
                    border
                }))
                .rounded(Radius::Lg)
                .p(Space::N3p5)
                .w_full()
                .test_id(format!("{TEST_ID_ROW_PREFIX}{row_id}"));

            let row = if row_done {
                row.shadow_none()
            } else if hovered {
                row.shadow_md()
            } else {
                row.shadow_sm()
            };

            vec![row.into_element(cx)]
        });

        ui::single(cx, hover_region)
    })
    .w_full()
}

fn footer_pill_chrome() -> ChromeRefinement {
    ChromeRefinement::default().px(Space::N3)
}

fn footer_pill_layout() -> LayoutRefinement {
    LayoutRefinement::default().h_px(MetricRef::Px(Px(28.0)))
}

fn alpha(mut color: Color, a: f32) -> Color {
    color.a = a.clamp(0.0, 1.0);
    color
}

fn rich_strikethrough(text: &Arc<str>, strike_color: Color) -> AttributedText {
    let span = TextSpan {
        len: text.len(),
        shaping: Default::default(),
        paint: TextPaintStyle {
            strikethrough: Some(StrikethroughStyle {
                color: Some(strike_color),
                style: DecorationLineStyle::Solid,
            }),
            ..Default::default()
        },
    };
    AttributedText::new(Arc::clone(text), Arc::<[TextSpan]>::from([span]))
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("todo-demo")
        .window("todo-demo", (640.0, 720.0))
        .config_files(false)
        .setup(fret_icons_lucide::app::install)
        .setup(install_demo_theme)
        .view::<TodoDemoView>()?
        .run()
        .map_err(anyhow::Error::from)
}

fn install_demo_theme(app: &mut App) {
    shadcn::themes::apply_shadcn_new_york(
        app,
        shadcn::themes::ShadcnBaseColor::Slate,
        shadcn::themes::ShadcnColorScheme::Light,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_icons::IconRegistry;

    #[test]
    fn todo_demo_registers_vendor_icons_used_by_layout() {
        let mut app = App::new();
        fret_icons_lucide::app::install(&mut app);
        let icons = app
            .global::<IconRegistry>()
            .expect("expected icon registry in todo demo app");

        for id in [
            "lucide.list-todo",
            "lucide.plus",
            "lucide.circle",
            "lucide.check",
            "lucide.sparkles",
            "lucide.trash-2",
        ] {
            assert!(
                icons.resolve(&IconId::new(id)).is_ok(),
                "missing icon: {id}"
            );
        }
    }
}
