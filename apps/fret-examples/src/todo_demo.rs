use std::sync::Arc;

use fret::app::LocalState;
use fret::app::prelude::*;
use fret::env::{
    ViewportQueryHysteresis, primary_pointer_can_hover, viewport_tailwind, viewport_width_at_least,
};
use fret::icons::{IconId, icon};
use fret::style::{ChromeRefinement, ColorRef, Radius, Space, Theme, ThemeSnapshot};
use fret_core::scene::DashPatternV1;
use fret_core::{
    AttributedText, Color, Corners, DecorationLineStyle, Px, StrikethroughStyle, TextPaintStyle,
    TextSpan,
};
use fret_ui::Invalidation;
use fret_ui::element::AnyElement;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::{WidgetStateProperty, WidgetStates, typography};

mod act {
    fret::actions!([
        Add = "todo_demo.add.v1",
        ClearDone = "todo_demo.clear_done.v1"
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
const TODO_WINDOW_SIZE: (f64, f64) = (560.0, 660.0);
const TODO_WINDOW_MIN_SIZE: (f64, f64) = (420.0, 560.0);
const TODO_WINDOW_POSITION_LOGICAL: (i32, i32) = (120, 96);
const TODO_WINDOW_RESIZE_INCREMENTS: (f64, f64) = (20.0, 20.0);
const TODO_COMPACT_WIDTH: Px = Px(560.0);
const TODO_COMPACT_HEIGHT: Px = Px(640.0);
const TODO_ROOMY_HEIGHT: Px = Px(760.0);

#[derive(Debug, Clone, Copy, PartialEq)]
struct TodoResponsiveLayout {
    page_padding: Space,
    page_top_padding: Space,
    center_card_vertically: bool,
    section_padding_x: Space,
    stack_footer: bool,
    fill_card_height: bool,
    always_show_row_actions: bool,
    card_max_height: Px,
}

impl TodoResponsiveLayout {
    fn from_viewport(
        viewport_width: Px,
        viewport_height: Px,
        can_hover: bool,
        wide_breakpoint_active: bool,
    ) -> Self {
        let compact_width = viewport_width.0 < TODO_COMPACT_WIDTH.0;
        let compact_height = viewport_height.0 < TODO_COMPACT_HEIGHT.0;
        let roomy_height = viewport_height.0 >= TODO_ROOMY_HEIGHT.0;
        let center_card_vertically = !compact_width && !compact_height;

        let page_padding = if compact_width || compact_height {
            Space::N3
        } else {
            Space::N4
        };
        let page_top_padding = if compact_height {
            Space::N3
        } else if roomy_height {
            Space::N8
        } else {
            Space::N6
        };
        let section_padding_x = if compact_width { Space::N4 } else { Space::N6 };
        let card_shell_allowance = if compact_height {
            24.0
        } else if roomy_height {
            56.0
        } else {
            40.0
        };
        let card_max_height = Px((viewport_height.0 - card_shell_allowance).clamp(340.0, 720.0));

        Self {
            page_padding,
            page_top_padding,
            center_card_vertically,
            section_padding_x,
            stack_footer: compact_width,
            fill_card_height: compact_width || compact_height,
            always_show_row_actions: !can_hover || !wide_breakpoint_active,
            card_max_height,
        }
    }
}

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
            Self::All => "All",
            Self::Active => "Active",
            Self::Completed => "Completed",
        }
    }

    fn value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Active => "active",
            Self::Completed => "completed",
        }
    }

    fn from_value(value: Option<&str>) -> Self {
        match value {
            Some("active") => Self::Active,
            Some("completed") => Self::Completed,
            _ => Self::All,
        }
    }
}

struct TodoLocals {
    draft: LocalState<String>,
    filter: LocalState<Option<Arc<str>>>,
    next_id: LocalState<u64>,
    todos: LocalState<Vec<TodoRow>>,
}

impl TodoLocals {
    fn new(cx: &mut AppUi<'_, '_>) -> Self {
        Self {
            draft: cx.state().local::<String>(),
            filter: cx
                .state()
                .local_init(|| Some(Arc::<str>::from(TodoFilter::All.value()))),
            next_id: cx.state().local_init(|| 4u64),
            todos: cx.state().local_init(|| {
                vec![
                    TodoRow {
                        id: 1,
                        done: true,
                        text: Arc::from("Learn React Hooks"),
                    },
                    TodoRow {
                        id: 2,
                        done: true,
                        text: Arc::from("Master Tailwind CSS"),
                    },
                    TodoRow {
                        id: 3,
                        done: false,
                        text: Arc::from("Build a modern Todo app"),
                    },
                ]
            }),
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

struct TodoDemoView;

impl View for TodoDemoView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = cx.theme_snapshot();
        let locals = TodoLocals::new(cx);
        locals.bind_actions(cx);
        let viewport = cx.environment_viewport_bounds(Invalidation::Layout);
        let wide_breakpoint_active = viewport_width_at_least(
            cx,
            Invalidation::Layout,
            viewport_tailwind::SM,
            ViewportQueryHysteresis::default(),
        );
        let responsive = TodoResponsiveLayout::from_viewport(
            viewport.size.width,
            viewport.size.height,
            primary_pointer_can_hover(cx, Invalidation::Layout, true),
            wide_breakpoint_active,
        );

        let todos = locals.todos.layout_value(cx);
        let draft_value = locals.draft.layout_value(cx);
        let filter_value = TodoFilter::from_value(locals.filter.layout_value(cx).as_deref());

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
        let footer_bg = alpha(muted, 0.30);
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
            ui::text("Add a task to get started")
                .text_sm()
                .text_color(ColorRef::Color(muted_foreground))
                .into_element_in(cx)
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
                    ui::text("All tasks completed")
                        .text_sm()
                        .text_color(ColorRef::Color(muted_foreground))
                        .into_element(cx),
                ]
            })
            .gap(Space::N1)
            .items_center()
            .into_element_in(cx)
        } else {
            let task_label = if active_count == 1 { "task" } else { "tasks" };
            ui::text(format!("{active_count} {task_label} left"))
                .text_sm()
                .text_color(ColorRef::Color(muted_foreground))
                .into_element_in(cx)
        };

        let title_block = ui::v_flex(|cx| {
            ui::children![
                cx;
                ui::text("My tasks").text_base().font_semibold(),
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
                            ui::text("Progress")
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
                        .ui()
                        .rounded(Radius::Full)
                        .w_full()
                        .build()
                        .into_element(cx)
                        .test_id(TEST_ID_PROGRESS),
                ]
            })
            .gap(Space::N1p5)
            .w_full()
            .into_element_in(cx)
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
            .a11y_label("Add task")
            .corner_radii_override(Corners::all(Px(14.0)))
            .ui()
            .shadow_sm()
            .build()
            .children([icon::icon(cx, IconId::new("lucide.plus"))])
            .test_id(TEST_ID_ADD);

        let input = shadcn::Input::new(&locals.draft)
            .a11y_label("New task")
            .placeholder("Add a new task...")
            .submit_action(act::Add)
            .corner_radii_override(Corners::all(Px(14.0)))
            .test_id(TEST_ID_DRAFT)
            .ui()
            .shadow_sm()
            .flex_1()
            .min_w_0()
            .build();

        let input_row = ui::h_flex(|cx| ui::children![cx; input, add_btn])
            .gap(Space::N2)
            .items_center()
            .w_full();

        let rows_body = ui::v_flex(|cx| {
            if filtered_todos.is_empty() {
                let empty_label = match filter_value {
                    TodoFilter::All => "No tasks yet. Enjoy the break!",
                    TodoFilter::Active => "No active tasks",
                    TodoFilter::Completed => "No completed tasks",
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

            ui::for_each_keyed_with_cx(
                cx,
                &filtered_todos,
                |row| row.id,
                |cx, row| {
                    let theme = theme.clone();
                    todo_row(cx, theme, row, responsive.always_show_row_actions)
                },
            )
        })
        .gap(Space::N2)
        .w_full()
        .items_stretch();

        let rows = shadcn::ScrollArea::new([rows_body.into_element_in(cx)])
            .viewport_test_id(TEST_ID_ROWS)
            .ui()
            .w_full()
            .h_full()
            .flex_1()
            .min_h_0()
            .build()
            .into_element_in(cx);

        let rows_section = ui::container(|cx| ui::single(cx, rows))
            .px(responsive.section_padding_x)
            .pt(Space::N0)
            .pb(Space::N2)
            .w_full()
            .flex_1()
            .min_h_0();

        let filters = filter_group(cx, &locals.filter, theme.clone());

        let clear_done_btn = shadcn::Button::new("Clear completed")
            .variant(shadcn::ButtonVariant::Ghost)
            .size(shadcn::ButtonSize::Xs)
            .corner_radii_override(Corners::all(Px(9999.0)))
            .style(subtle_destructive_button_style(
                muted_foreground,
                destructive,
            ))
            .disabled(!has_completed)
            .action(act::ClearDone)
            .ui()
            .px(Space::N3)
            .h_px(Px(28.0))
            .build()
            .test_id(TEST_ID_CLEAR_DONE);

        let footer = if responsive.stack_footer {
            ui::v_flex(|cx| {
                let mut children = vec![filters];
                if has_completed {
                    children.push(
                        ui::h_flex(|cx| ui::single(cx, clear_done_btn))
                            .justify_end()
                            .w_full()
                            .into_element(cx),
                    );
                }
                children
            })
            .gap(Space::N2)
            .items_stretch()
            .w_full()
            .into_element_in(cx)
        } else {
            ui::h_flex(|cx| {
                let mut children = vec![filters];
                if has_completed {
                    children.push(clear_done_btn.into_element(cx));
                }
                children
            })
            .gap(Space::N3)
            .items_center()
            .justify_between()
            .w_full()
            .into_element_in(cx)
        };

        let card_min_height = if responsive.fill_card_height {
            responsive.card_max_height
        } else {
            Px(0.0)
        };

        let card = ui::v_flex(|cx| {
            let mut sections = vec![
                todo_card_section(header, responsive.section_padding_x, Space::N6, Space::N4)
                    .into_element(cx),
                todo_card_section(
                    input_row,
                    responsive.section_padding_x,
                    Space::N0,
                    Space::N4,
                )
                .into_element(cx),
                rows_section.into_element(cx),
            ];

            if total_count > 0 {
                sections.push(
                    todo_card_footer_section(footer, responsive.section_padding_x, footer_bg)
                        .into_element(cx),
                );
            }

            sections
        })
        .bg(ColorRef::Color(card))
        .border_1()
        .border_color(ColorRef::Color(border))
        .rounded(Radius::Lg)
        .shadow_lg()
        .overflow_hidden()
        .w_full()
        .max_w(Px(448.0))
        .min_h(card_min_height)
        .max_h(responsive.card_max_height)
        .items_stretch();
        let card = card.test_id(TEST_ID_ROOT);

        ui::single(cx, todo_page(theme, responsive, card))
    }
}

fn todo_page(
    theme: ThemeSnapshot,
    responsive: TodoResponsiveLayout,
    content: impl UiChild,
) -> impl UiChild {
    let page_top_padding = if responsive.center_card_vertically {
        responsive.page_padding
    } else {
        responsive.page_top_padding
    };
    ui::container(move |cx| {
        let content_column = ui::v_flex(move |cx| ui::single(cx, content))
            .w_full()
            .h_full()
            .items_center();
        let content_column = if responsive.center_card_vertically {
            content_column.justify_center()
        } else {
            content_column.justify_start()
        };
        ui::single(cx, content_column)
    })
    .bg(ColorRef::Color(theme.color_token("background")))
    .px(responsive.page_padding)
    .pt(page_top_padding)
    .pb(responsive.page_padding)
    .w_full()
    .h_full()
}

fn filter_group<'a, Cx>(
    cx: &mut Cx,
    filter: &LocalState<Option<Arc<str>>>,
    theme: ThemeSnapshot,
) -> AnyElement
where
    Cx: fret::app::ElementContextAccess<'a, App>,
{
    let style = shadcn::ToggleGroupStyle::default()
        .item_background(
            WidgetStateProperty::new(None)
                .when(
                    WidgetStates::HOVERED,
                    Some(ColorRef::Color(theme.color_token("accent"))),
                )
                .when(
                    WidgetStates::ACTIVE,
                    Some(ColorRef::Color(theme.color_token("accent"))),
                )
                .when(
                    WidgetStates::SELECTED,
                    Some(ColorRef::Color(theme.color_token("secondary"))),
                ),
        )
        .item_foreground(
            WidgetStateProperty::new(None)
                .when(
                    WidgetStates::HOVERED,
                    Some(ColorRef::Color(theme.color_token("foreground"))),
                )
                .when(
                    WidgetStates::ACTIVE,
                    Some(ColorRef::Color(theme.color_token("foreground"))),
                )
                .when(
                    WidgetStates::SELECTED,
                    Some(ColorRef::Color(theme.color_token("secondary-foreground"))),
                ),
        );

    shadcn::ToggleGroup::single(filter)
        .deselectable(false)
        .spacing(Space::N1)
        .style(style)
        .items([
            filter_group_item(cx, TodoFilter::All, TEST_ID_FILTER_ALL),
            filter_group_item(cx, TodoFilter::Active, TEST_ID_FILTER_ACTIVE),
            filter_group_item(cx, TodoFilter::Completed, TEST_ID_FILTER_COMPLETED),
        ])
        .into_element_in(cx)
}

fn filter_group_item<'a, Cx>(
    cx: &mut Cx,
    filter: TodoFilter,
    test_id: &str,
) -> shadcn::ToggleGroupItem
where
    Cx: fret::app::ElementContextAccess<'a, App>,
{
    shadcn::ToggleGroupItem::new(
        filter.value(),
        [ui::text(filter.label()).into_element_in(cx)],
    )
    .a11y_label(format!("Show {} tasks", filter.label().to_lowercase()))
    .test_id(test_id)
    .refine_style(ChromeRefinement::default().rounded(Radius::Full))
    .refine_layout(
        fret_ui_kit::LayoutRefinement::default()
            .h_px(Px(28.0))
            .min_h(Px(28.0)),
    )
}

fn todo_row<'a, Cx>(
    cx: &mut Cx,
    theme: ThemeSnapshot,
    row: &TodoRow,
    always_show_remove_action: bool,
) -> AnyElement
where
    Cx: fret::app::ElementContextAccess<'a, App>,
{
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

    let hover_region = ui::hover_region(move |cx, hovered| {
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
                "Mark as incomplete"
            } else {
                "Mark as complete"
            })
            .children([toggle_visual])
            .test_id(format!("{TEST_ID_DONE_PREFIX}{row_id}"));

        let text = if row_done {
            let rich = rich_strikethrough(&row_text, muted_foreground);
            let style = typography::TypographyPreset::control_ui(typography::UiTextSize::Sm)
                .resolve(&theme);
            ui::rich_text(rich)
                .text_style(style)
                .text_color(ColorRef::Color(muted_foreground))
                .truncate()
                .flex_1()
                .min_w_0()
                .into_element(cx)
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

        let remove_button = shadcn::Button::new("")
            .variant(shadcn::ButtonVariant::Ghost)
            .size(shadcn::ButtonSize::IconSm)
            .corner_radii_override(Corners::all(Px(10.0)))
            .style(subtle_destructive_button_style(
                muted_foreground,
                destructive,
            ))
            .action(act::Remove)
            .action_payload(row_id)
            .a11y_label("Delete task")
            .children([icon::icon_with(
                cx,
                IconId::new("lucide.trash-2"),
                Some(Px(16.0)),
                Some(ColorRef::Color(muted_foreground)),
            )])
            .test_id(format!("{TEST_ID_REMOVE_PREFIX}{row_id}"));

        let remove_visible = always_show_remove_action || hovered;
        let remove = cx.interactivity_gate(true, remove_visible, move |cx| {
            vec![
                cx.opacity(if remove_visible { 1.0 } else { 0.0 }, move |cx| {
                    vec![remove_button.into_element(cx)]
                }),
            ]
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
    })
    .w_full();

    ui::container(move |cx| ui::single(cx, hover_region))
        .w_full()
        .into_element_in(cx)
}

fn todo_card_section(
    content: impl UiChild,
    padding_x: Space,
    pt: Space,
    pb: Space,
) -> impl UiChild {
    ui::container(move |cx| ui::single(cx, content))
        .px(padding_x)
        .pt(pt)
        .pb(pb)
        .w_full()
}

fn todo_card_footer_section(
    content: impl UiChild,
    padding_x: Space,
    background: Color,
) -> impl UiChild {
    ui::container(move |cx| ui::single(cx, content))
        .px(padding_x)
        .py(Space::N3p5)
        .bg(ColorRef::Color(background))
        .w_full()
}

fn subtle_destructive_button_style(
    muted_foreground: Color,
    destructive: Color,
) -> shadcn::ButtonStyle {
    shadcn::ButtonStyle::default()
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
        )
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
        .window("todo-demo", TODO_WINDOW_SIZE)
        .window_min_size(TODO_WINDOW_MIN_SIZE)
        .window_position_logical(TODO_WINDOW_POSITION_LOGICAL)
        .window_resize_increments(TODO_WINDOW_RESIZE_INCREMENTS)
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
    use fret::advanced::view::{
        AppUiRenderRootState, ViewWindowState, render_root_with_app_ui, view_init_window, view_view,
    };
    use fret_core::{Point, PointerType, Rect, Size, WindowMetricsService};
    use fret_icons::IconRegistry;
    use fret_runtime::{FrameId, TickId};
    use fret_ui::UiTree;

    #[derive(Default)]
    struct FakeUiServices;

    impl fret_core::TextService for FakeUiServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
            _constraints: fret_core::TextConstraints,
        ) -> (fret_core::TextBlobId, fret_core::TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                fret_core::TextMetrics {
                    size: fret_core::Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for FakeUiServices {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for FakeUiServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for FakeUiServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _material: fret_core::MaterialId) -> bool {
            true
        }
    }

    fn render_todo_demo_snapshot_for_frame(
        app: &mut App,
        ui: &mut UiTree<App>,
        services: &mut FakeUiServices,
        window: WindowId,
        bounds: Rect,
        root_state: &mut AppUiRenderRootState,
        view: &mut TodoDemoView,
        frame_id: u64,
    ) -> fret_core::SemanticsSnapshot {
        app.set_frame_id(FrameId(frame_id));
        let root = render_root_with_app_ui(
            fret_ui::declarative::RenderRootContext::new(ui, app, services, window, bounds),
            "todo-demo-test",
            root_state,
            |cx| view.render(cx),
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        ui.semantics_snapshot()
            .expect("todo demo semantics snapshot should exist")
            .clone()
    }

    fn seed_window_metrics(app: &mut App, window: WindowId, bounds: Rect, scale_factor: f32) {
        app.with_global_mut_untracked(WindowMetricsService::default, |svc, _app| {
            svc.set_inner_size(window, bounds.size);
            svc.set_scale_factor(window, scale_factor);
            svc.set_focused(window, true);
        });
        app.with_global_mut_untracked(fret_ui::elements::ElementRuntime::new, |rt, _app| {
            rt.set_window_primary_pointer_type(window, PointerType::Unknown);
        });
    }

    fn render_todo_demo_runtime_snapshot_for_frame(
        app: &mut App,
        ui: &mut UiTree<App>,
        services: &mut FakeUiServices,
        window: WindowId,
        bounds: Rect,
        state: &mut ViewWindowState<TodoDemoView>,
        frame_id: u64,
        scale_factor: f32,
    ) -> fret_core::SemanticsSnapshot {
        app.set_tick_id(TickId(frame_id));
        app.set_frame_id(FrameId(frame_id));
        seed_window_metrics(app, window, bounds, scale_factor);

        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "todo-demo-runtime-test",
            |cx| view_view(cx, state),
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, scale_factor);
        ui.semantics_snapshot()
            .expect("todo demo runtime semantics snapshot should exist")
            .clone()
    }

    fn snapshot_test_ids(snapshot: &fret_core::SemanticsSnapshot) -> Vec<String> {
        let mut ids: Vec<String> = snapshot
            .nodes
            .iter()
            .filter_map(|node| node.test_id.as_ref().map(|id| id.to_string()))
            .collect();
        ids.sort();
        ids
    }

    fn node_by_test_id<'a>(
        snapshot: &'a fret_core::SemanticsSnapshot,
        test_id: &str,
    ) -> Option<&'a fret_core::SemanticsNode> {
        snapshot
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some(test_id))
    }

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

    #[test]
    fn todo_demo_responsive_layout_prefers_compact_footer_and_inline_actions_on_narrow_width() {
        let compact = TodoResponsiveLayout::from_viewport(Px(520.0), Px(700.0), true, false);

        assert_eq!(compact.page_padding, Space::N3);
        assert_eq!(compact.section_padding_x, Space::N4);
        assert!(!compact.center_card_vertically);
        assert!(compact.stack_footer);
        assert!(compact.fill_card_height);
        assert!(compact.always_show_row_actions);
    }

    #[test]
    fn todo_demo_responsive_layout_gives_roomy_shells_more_vertical_headroom() {
        let compact = TodoResponsiveLayout::from_viewport(Px(720.0), Px(560.0), true, true);
        let roomy = TodoResponsiveLayout::from_viewport(Px(720.0), Px(820.0), true, true);

        assert_eq!(compact.page_top_padding, Space::N3);
        assert_eq!(roomy.page_top_padding, Space::N8);
        assert!(roomy.card_max_height.0 > compact.card_max_height.0);
        assert!(compact.fill_card_height);
        assert!(!roomy.fill_card_height);
        assert!(!roomy.always_show_row_actions);
    }

    #[test]
    fn todo_demo_responsive_layout_centers_card_once_viewport_is_large_enough() {
        let compact = TodoResponsiveLayout::from_viewport(Px(520.0), Px(620.0), true, false);
        let regular = TodoResponsiveLayout::from_viewport(Px(560.0), Px(660.0), true, true);

        assert!(!compact.center_card_vertically);
        assert!(regular.center_card_vertically);
    }

    #[test]
    fn todo_demo_responsive_layout_keeps_inline_row_actions_for_non_hover_pointers() {
        let touch = TodoResponsiveLayout::from_viewport(Px(820.0), Px(760.0), false, true);

        assert!(touch.always_show_row_actions);
        assert!(!touch.stack_footer);
    }

    #[test]
    fn todo_demo_compact_start_snapshot_keeps_footer_filters_across_cache_reuse() {
        let mut app = App::new();
        install_demo_theme(&mut app);
        fret_icons_lucide::app::install(&mut app);

        let window = WindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(560.0)),
        );
        let mut ui = UiTree::<App>::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(true);
        let mut services = FakeUiServices;
        let mut root_state = AppUiRenderRootState::default();
        let mut view = TodoDemoView;

        let frame1 = render_todo_demo_snapshot_for_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut root_state,
            &mut view,
            1,
        );
        let frame2 = render_todo_demo_snapshot_for_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            bounds,
            &mut root_state,
            &mut view,
            2,
        );

        let mut failures: Vec<String> = Vec::new();
        for (label, snapshot) in [("frame1", &frame1), ("frame2", &frame2)] {
            let ids = snapshot_test_ids(snapshot);
            let rows = node_by_test_id(snapshot, TEST_ID_ROWS)
                .expect("rows viewport should exist in compact startup snapshot");
            if !ids.iter().any(|id| id == TEST_ID_FILTER_ALL) {
                failures.push(format!(
                    "{label} should keep the All filter chip in compact startup snapshot; ids={ids:?}"
                ));
            }
            if !ids.iter().any(|id| id == TEST_ID_FILTER_ACTIVE) {
                failures.push(format!(
                    "{label} should keep the Active filter chip in compact startup snapshot; ids={ids:?}"
                ));
            }
            if !ids.iter().any(|id| id == TEST_ID_FILTER_COMPLETED) {
                failures.push(format!(
                    "{label} should keep the Completed filter chip in compact startup snapshot; ids={ids:?}"
                ));
            }
            if rows.bounds.size.height.0 <= 0.0 {
                failures.push(format!(
                    "{label} rows viewport should keep positive height in compact startup snapshot; rows_bounds={:?}",
                    rows.bounds
                ));
            }
        }

        if !failures.is_empty() {
            panic!("{}", failures.join("\n"));
        }
    }

    #[test]
    fn todo_demo_resize_to_compact_keeps_footer_filters_across_cache_reuse() {
        let mut app = App::new();
        install_demo_theme(&mut app);
        fret_icons_lucide::app::install(&mut app);

        let window = WindowId::default();
        let roomy_bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(660.0)),
        );
        let compact_bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(560.0)),
        );
        let mut ui = UiTree::<App>::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(true);
        let mut services = FakeUiServices;
        let mut root_state = AppUiRenderRootState::default();
        let mut view = TodoDemoView;

        let _frame1 = render_todo_demo_snapshot_for_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            roomy_bounds,
            &mut root_state,
            &mut view,
            1,
        );
        let frame2 = render_todo_demo_snapshot_for_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            compact_bounds,
            &mut root_state,
            &mut view,
            2,
        );
        let frame3 = render_todo_demo_snapshot_for_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            compact_bounds,
            &mut root_state,
            &mut view,
            3,
        );

        let mut failures: Vec<String> = Vec::new();
        for (label, snapshot) in [("frame2", &frame2), ("frame3", &frame3)] {
            let ids = snapshot_test_ids(snapshot);
            let rows = node_by_test_id(snapshot, TEST_ID_ROWS)
                .expect("rows viewport should exist after resize to compact");
            for filter_id in [
                TEST_ID_FILTER_ALL,
                TEST_ID_FILTER_ACTIVE,
                TEST_ID_FILTER_COMPLETED,
            ] {
                if !ids.iter().any(|id| id == filter_id) {
                    failures.push(format!(
                        "{label} should keep {filter_id} after resize to compact; ids={ids:?}"
                    ));
                }
            }
            if rows.bounds.size.height.0 <= 0.0 {
                failures.push(format!(
                    "{label} rows viewport should keep positive height after resize to compact; rows_bounds={:?}",
                    rows.bounds
                ));
            }
        }

        if !failures.is_empty() {
            panic!("{}", failures.join("\n"));
        }
    }

    #[test]
    fn todo_demo_compact_start_keeps_footer_filters_after_many_reuse_frames() {
        let mut app = App::new();
        install_demo_theme(&mut app);
        fret_icons_lucide::app::install(&mut app);

        let window = WindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(560.0)),
        );
        let mut ui = UiTree::<App>::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(true);
        let mut services = FakeUiServices;
        let mut root_state = AppUiRenderRootState::default();
        let mut view = TodoDemoView;

        let mut last_snapshot = None;
        for frame_id in 1..=22 {
            last_snapshot = Some(render_todo_demo_snapshot_for_frame(
                &mut app,
                &mut ui,
                &mut services,
                window,
                bounds,
                &mut root_state,
                &mut view,
                frame_id,
            ));
        }

        let snapshot = last_snapshot.expect("expected final compact snapshot");
        let ids = snapshot_test_ids(&snapshot);
        let rows = node_by_test_id(&snapshot, TEST_ID_ROWS)
            .expect("rows viewport should exist at frame22");

        for filter_id in [
            TEST_ID_FILTER_ALL,
            TEST_ID_FILTER_ACTIVE,
            TEST_ID_FILTER_COMPLETED,
        ] {
            assert!(
                ids.iter().any(|id| id == filter_id),
                "frame22 should keep {filter_id} after many compact reuse frames; ids={ids:?}"
            );
        }
        assert!(
            rows.bounds.size.height.0 > 0.0,
            "frame22 rows viewport should keep positive height after many compact reuse frames; rows_bounds={:?}",
            rows.bounds
        );
    }

    #[test]
    fn todo_demo_view_runtime_cache_enable_transition_keeps_footer_filters_after_compact_resize() {
        let mut app = App::new();
        install_demo_theme(&mut app);
        fret_icons_lucide::app::install(&mut app);

        let window = WindowId::default();
        let roomy_bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(660.0)),
        );
        let compact_bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(420.0), Px(560.0)),
        );
        let mut ui = UiTree::<App>::new();
        ui.set_window(window);
        ui.set_debug_enabled(true);
        let mut services = FakeUiServices;
        let mut state = view_init_window::<TodoDemoView>(&mut app, window);

        let _frame1 = render_todo_demo_runtime_snapshot_for_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            roomy_bounds,
            &mut state,
            1,
            2.0,
        );

        // The desktop runner flips view-cache on from the engine-frame hook after the initial
        // render, so replay that order rather than assuming cache was enabled from frame 1.
        ui.set_view_cache_enabled(true);

        let frame2 = render_todo_demo_runtime_snapshot_for_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            compact_bounds,
            &mut state,
            2,
            2.0,
        );
        let frame3 = render_todo_demo_runtime_snapshot_for_frame(
            &mut app,
            &mut ui,
            &mut services,
            window,
            compact_bounds,
            &mut state,
            3,
            2.0,
        );

        let mut tracked_frames = vec![(2u64, frame2), (3u64, frame3)];
        for frame_id in 4..=22 {
            tracked_frames.push((
                frame_id,
                render_todo_demo_runtime_snapshot_for_frame(
                    &mut app,
                    &mut ui,
                    &mut services,
                    window,
                    compact_bounds,
                    &mut state,
                    frame_id,
                    2.0,
                ),
            ));
        }

        let mut failures: Vec<String> = Vec::new();
        for (frame_id, snapshot) in &tracked_frames {
            let label = format!("frame{frame_id}");
            let ids = snapshot_test_ids(snapshot);
            let rows = node_by_test_id(snapshot, TEST_ID_ROWS)
                .expect("rows viewport should exist in runtime compact snapshot");
            for filter_id in [
                TEST_ID_FILTER_ALL,
                TEST_ID_FILTER_ACTIVE,
                TEST_ID_FILTER_COMPLETED,
            ] {
                if !ids.iter().any(|id| id == filter_id) {
                    let cache_roots = ui.debug_cache_root_stats();
                    let removed = ui.debug_removed_subtrees();
                    failures.push(format!(
                        "{label} should keep {filter_id} through runtime cache-enable transition; ids={ids:?}; cache_roots={cache_roots:?}; removed={removed:?}"
                    ));
                }
            }
            if rows.bounds.size.height.0 <= 0.0 {
                failures.push(format!(
                    "{label} rows viewport should keep positive height through runtime cache-enable transition; rows_bounds={:?}",
                    rows.bounds
                ));
            }
        }

        if !failures.is_empty() {
            panic!("{}", failures.join("\n"));
        }
    }
}
