//! Legacy MVU todo demo (compat).
//!
//! Prefer `todo_demo.rs` (View runtime + typed actions) for new code.

use std::sync::Arc;

use fret::dev::{DevStateExport, DevStateHook, DevStateHooks};
use fret::legacy::prelude::*;
use fret_core::scene::DashPatternV1;
use fret_core::{Color, DecorationLineStyle, StrikethroughStyle, TextPaintStyle, TextSpan};
use fret_icons_lucide::generated_ids::lucide;
use fret_selector::ui::SelectorElementContextExt as _;
use fret_ui_kit::{WidgetStateProperty, WidgetStates};
use serde_json::{Value, json};

const TEST_ID_INPUT: &str = "todo-input";
const TEST_ID_ADD: &str = "todo-add";
const TEST_ID_CLEAR_DONE: &str = "todo-clear-done";
const TEST_ID_FILTER_ALL: &str = "todo-filter-all";
const TEST_ID_FILTER_ACTIVE: &str = "todo-filter-active";
const TEST_ID_FILTER_COMPLETED: &str = "todo-filter-completed";
const TEST_ID_HEADER_ICON: &str = "todo-header-icon";
const TEST_ID_HEADER_DATE: &str = "todo-header-date";

const DEV_STATE_TODO_STATE_KEY: &str = "todo.demo.state.v1";

#[derive(Debug, Default)]
struct TodoDevStateIncoming {
    snapshot: Option<TodoDevStateSnapshot>,
}

#[derive(Debug, Clone)]
struct TodoDevStateSnapshot {
    draft: String,
    filter: TodoFilter,
    todos: Vec<TodoDevStateTodo>,
}

#[derive(Debug, Clone)]
struct TodoDevStateTodo {
    id: u64,
    done: bool,
    text: Arc<str>,
}

#[derive(Debug, Clone)]
struct TodoDevStateModels {
    todos: Model<Vec<TodoItem>>,
    draft: Model<String>,
    filter: Model<TodoFilter>,
}

fn todo_row_test_id(id: u64) -> Arc<str> {
    Arc::from(format!("todo-item-{id}-indicator"))
}

fn todo_done_test_id(id: u64) -> Arc<str> {
    Arc::from(format!("todo-item-{id}-done"))
}

fn todo_label_test_id(id: u64) -> Arc<str> {
    Arc::from(format!("todo-item-{id}-label"))
}

fn todo_remove_test_id(id: u64) -> Arc<str> {
    Arc::from(format!("todo-item-{id}-remove"))
}

fn with_alpha(mut color: Color, alpha: f32) -> Color {
    color.a = alpha;
    color
}

fn mix_colors(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

fn en_weekday(weekday: time::Weekday) -> &'static str {
    match weekday {
        time::Weekday::Monday => "Monday",
        time::Weekday::Tuesday => "Tuesday",
        time::Weekday::Wednesday => "Wednesday",
        time::Weekday::Thursday => "Thursday",
        time::Weekday::Friday => "Friday",
        time::Weekday::Saturday => "Saturday",
        time::Weekday::Sunday => "Sunday",
    }
}

fn en_month(month: time::Month) -> &'static str {
    match month {
        time::Month::January => "January",
        time::Month::February => "February",
        time::Month::March => "March",
        time::Month::April => "April",
        time::Month::May => "May",
        time::Month::June => "June",
        time::Month::July => "July",
        time::Month::August => "August",
        time::Month::September => "September",
        time::Month::October => "October",
        time::Month::November => "November",
        time::Month::December => "December",
    }
}

fn today_label() -> Arc<str> {
    let now = time::OffsetDateTime::now_utc();
    let date = now.date();
    Arc::from(format!(
        "{}, {} {}, {}",
        en_weekday(date.weekday()),
        en_month(date.month()),
        date.day(),
        date.year()
    ))
}

#[derive(Clone)]
struct TodoItem {
    id: u64,
    done: Model<bool>,
    text: Arc<str>,
}

struct TodoState {
    todos: Model<Vec<TodoItem>>,
    draft: Model<String>,
    filter: Model<TodoFilter>,
    next_id: u64,
}

#[derive(Debug, Clone)]
enum Msg {
    Add,
    ClearDone,
    SetFilter(TodoFilter),
    Remove(u64),
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
}

#[derive(Clone, PartialEq)]
struct TodoDerivedDeps {
    todos_rev: u64,
    done_revs: Vec<u64>,
    filter: TodoFilter,
}

#[derive(Clone)]
struct TodoDerived {
    rows: Arc<[TodoRowSnapshot]>,
    active: usize,
    completed: usize,
}

#[derive(Clone)]
struct TodoRowSnapshot {
    id: u64,
    done: bool,
    done_model: Model<bool>,
    text: Arc<str>,
}

pub fn run() -> anyhow::Result<()> {
    fret::mvu::app::<TodoProgram>("todo-demo")?
        .init_app(|app| {
            app.with_global_mut_untracked(DevStateHooks::default, |hooks, _app| {
                hooks.register(
                    DevStateHook::new(DEV_STATE_TODO_STATE_KEY, |app| {
                        let Some(models) = app.global::<TodoDevStateModels>() else {
                            return DevStateExport::Noop;
                        };
                        DevStateExport::Set(export_todo_dev_state(app, models))
                    })
                    .with_import(|app, value| {
                        let snapshot = parse_todo_dev_state(value)?;
                        app.with_global_mut_untracked(TodoDevStateIncoming::default, |st, _app| {
                            st.snapshot = Some(snapshot);
                        });
                        Ok(())
                    }),
                );
            });
        })
        .with_main_window("todo_demo", (560.0, 600.0))
        .run()?;
    Ok(())
}

struct TodoProgram;

impl MvuProgram for TodoProgram {
    type State = TodoState;
    type Message = Msg;

    fn init(app: &mut App, window: AppWindowId) -> Self::State {
        init_window(app, window)
    }

    fn update(app: &mut App, state: &mut Self::State, message: Self::Message) {
        update(app, state, message);
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        view(cx, state, msg)
    }
}

fn init_window(app: &mut App, _window: AppWindowId) -> TodoState {
    let incoming =
        app.with_global_mut_untracked(TodoDevStateIncoming::default, |st, _app| st.snapshot.take());

    let restored = incoming.map(|snapshot| {
        let mut max_id = 0u64;
        let todos_vec: Vec<TodoItem> = snapshot
            .todos
            .into_iter()
            .map(|todo| {
                max_id = max_id.max(todo.id);
                TodoItem {
                    id: todo.id,
                    done: app.models_mut().insert(todo.done),
                    text: todo.text,
                }
            })
            .collect();

        let todos = app.models_mut().insert(todos_vec);
        let draft = app.models_mut().insert(snapshot.draft);
        let filter = app.models_mut().insert(snapshot.filter);
        let next_id = max_id.saturating_add(1).max(1);

        TodoState {
            todos,
            draft,
            filter,
            next_id,
        }
    });

    let state = if let Some(restored) = restored {
        restored
    } else {
        let done_1 = app.models_mut().insert(true);
        let done_2 = app.models_mut().insert(false);
        let todos = app.models_mut().insert(vec![
            TodoItem {
                id: 1,
                done: done_1,
                text: Arc::from("Learn React and Tailwind"),
            },
            TodoItem {
                id: 2,
                done: done_2,
                text: Arc::from("Build a polished Todo app"),
            },
        ]);

        TodoState {
            todos,
            draft: app.models_mut().insert(String::new()),
            filter: app.models_mut().insert(TodoFilter::All),
            next_id: 3,
        }
    };

    app.set_global(TodoDevStateModels {
        todos: state.todos.clone(),
        draft: state.draft.clone(),
        filter: state.filter.clone(),
    });

    state
}

fn parse_todo_dev_state(incoming: Value) -> Result<TodoDevStateSnapshot, String> {
    let Some(obj) = incoming.as_object() else {
        return Err("expected object".to_string());
    };
    let version = obj
        .get("version")
        .and_then(Value::as_u64)
        .ok_or_else(|| "missing version".to_string())?;
    if version != 1 {
        return Err("unsupported version".to_string());
    }

    let draft = obj
        .get("draft")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let filter = obj
        .get("filter")
        .and_then(Value::as_str)
        .and_then(parse_filter)
        .unwrap_or(TodoFilter::All);

    let mut items: Vec<TodoDevStateTodo> = Vec::new();
    if let Some(todos_arr) = obj.get("todos").and_then(Value::as_array) {
        for todo in todos_arr {
            let Some(todo_obj) = todo.as_object() else {
                continue;
            };
            let Some(id) = todo_obj.get("id").and_then(Value::as_u64) else {
                continue;
            };
            let done = todo_obj
                .get("done")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let text = todo_obj
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default();
            items.push(TodoDevStateTodo {
                id,
                done,
                text: Arc::from(text),
            });
        }
    }

    Ok(TodoDevStateSnapshot {
        draft,
        filter,
        todos: items,
    })
}

fn parse_filter(raw: &str) -> Option<TodoFilter> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "all" => Some(TodoFilter::All),
        "active" => Some(TodoFilter::Active),
        "completed" => Some(TodoFilter::Completed),
        _ => None,
    }
}

fn export_todo_dev_state(app: &App, models: &TodoDevStateModels) -> Value {
    let todos = app
        .models()
        .read(&models.todos, Clone::clone)
        .ok()
        .unwrap_or_default();

    let todos = todos
        .into_iter()
        .map(|todo| {
            let done = app.models().get_copied(&todo.done).unwrap_or(false);
            json!({
                "id": todo.id,
                "done": done,
                "text": todo.text.as_ref(),
            })
        })
        .collect::<Vec<_>>();

    let draft = app
        .models()
        .read(&models.draft, Clone::clone)
        .ok()
        .unwrap_or_default();

    let filter = app
        .models()
        .get_copied(&models.filter)
        .unwrap_or(TodoFilter::All);

    let filter = match filter {
        TodoFilter::All => "all",
        TodoFilter::Active => "active",
        TodoFilter::Completed => "completed",
    };

    json!({
        "version": 1,
        "draft": draft,
        "filter": filter,
        "todos": todos,
    })
}

fn view(
    cx: &mut ElementContext<'_, App>,
    st: &mut TodoState,
    msg: &mut MessageRouter<Msg>,
) -> Elements {
    let theme = Theme::global(&*cx.app).snapshot();

    let draft_value = cx.watch_model(&st.draft).layout().cloned_or_default();

    let filter_value = cx
        .watch_model(&st.filter)
        .layout()
        .copied_or(TodoFilter::All);

    let derived = cx.use_selector(
        |cx| {
            let filter = cx
                .watch_model(&st.filter)
                .layout()
                .copied_or(TodoFilter::All);
            let (todos_rev, done_revs) = cx
                .watch_model(&st.todos)
                .layout()
                .read(|host, todos| {
                    let todos_rev = st.todos.revision(host).unwrap_or(0);
                    let done_revs = todos
                        .iter()
                        .map(|todo| (todo.done.id(), todo.done.revision(host).unwrap_or(0)))
                        .collect::<Vec<_>>();
                    (todos_rev, done_revs)
                })
                .ok()
                .unwrap_or((0, Vec::new()));

            for (id, _rev) in &done_revs {
                cx.observe_model_id(*id, Invalidation::Layout);
            }

            TodoDerivedDeps {
                todos_rev,
                done_revs: done_revs.into_iter().map(|(_, rev)| rev).collect(),
                filter,
            }
        },
        |cx| {
            cx.watch_model(&st.todos)
                .layout()
                .read(|host, todos| {
                    let filter = host
                        .models()
                        .get_copied(&st.filter)
                        .unwrap_or(TodoFilter::All);
                    let mut rows = Vec::new();
                    let mut active = 0usize;
                    let mut completed = 0usize;
                    for todo in todos {
                        let done = host.models().get_copied(&todo.done).unwrap_or(false);
                        if done {
                            completed += 1;
                        } else {
                            active += 1;
                        }
                        if filter.matches(done) {
                            rows.push(TodoRowSnapshot {
                                id: todo.id,
                                done,
                                done_model: todo.done.clone(),
                                text: todo.text.clone(),
                            });
                        }
                    }
                    TodoDerived {
                        rows: rows.into(),
                        active,
                        completed,
                    }
                })
                .ok()
                .unwrap_or_else(|| TodoDerived {
                    rows: Arc::from([]),
                    active: 0,
                    completed: 0,
                })
        },
    );

    let add_enabled = !draft_value.trim().is_empty();
    let add_cmd = msg.cmd(Msg::Add);
    let clear_done_cmd = msg.cmd(Msg::ClearDone);
    let filter_all_cmd = msg.cmd(Msg::SetFilter(TodoFilter::All));
    let filter_active_cmd = msg.cmd(Msg::SetFilter(TodoFilter::Active));
    let filter_completed_cmd = msg.cmd(Msg::SetFilter(TodoFilter::Completed));
    let add_button = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .disabled(!add_enabled)
        .on_click(add_cmd.clone())
        .children([icon::icon(cx, lucide::PLUS)])
        .into_element(cx)
        .a11y_role(SemanticsRole::Button)
        .test_id(TEST_ID_ADD);

    let input = shadcn::Input::new(st.draft.clone())
        .placeholder("Add a new task…")
        .submit_command(add_cmd.clone())
        .into_element(cx)
        .a11y_role(SemanticsRole::TextField)
        .test_id(TEST_ID_INPUT);

    let input_row = ui::h_flex(cx, |_cx| [input, add_button])
        .gap(Space::N2)
        .items_center()
        .w_full()
        .into_element(cx);

    let filter_all = filter_chip(
        cx,
        &theme,
        "All",
        filter_all_cmd,
        filter_value == TodoFilter::All,
        TEST_ID_FILTER_ALL,
    );
    let filter_active = filter_chip(
        cx,
        &theme,
        "Active",
        filter_active_cmd,
        filter_value == TodoFilter::Active,
        TEST_ID_FILTER_ACTIVE,
    );
    let filter_completed = filter_chip(
        cx,
        &theme,
        "Completed",
        filter_completed_cmd,
        filter_value == TodoFilter::Completed,
        TEST_ID_FILTER_COMPLETED,
    );

    // Match shadcn tabs: list track `bg-muted`, trigger `data-[state=active]:bg-background`.
    let filter_track_bg = theme.color_token("muted");
    let filter_row = ui::container(cx, |cx| {
        [
            ui::h_flex(cx, |_cx| [filter_all, filter_active, filter_completed])
                .gap(Space::N1)
                .items_center()
                .w_full()
                .into_element(cx),
        ]
    })
    .bg(ColorRef::Color(filter_track_bg))
    .rounded(Radius::Lg)
    .p(Space::N1)
    .w_full()
    .into_element(cx);

    let rows = ui::v_flex_build(cx, |cx, out| {
        if derived.rows.is_empty() {
            let icon = icon::icon_with(
                cx,
                lucide::LIST_TODO,
                Some(Px(40.0)),
                Some(ColorRef::Color(theme.color_token("muted-foreground"))),
            );
            let label = ui::text(cx, "No tasks found")
                .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                .into_element(cx);
            let body = ui::v_flex(cx, |_cx| [icon, label])
                .gap(Space::N2)
                .items_center()
                .justify_center()
                .w_full()
                .h_full()
                .into_element(cx);

            out.push(
                ui::container(cx, |_cx| [body])
                    .border_1()
                    .border_color(ColorRef::Color(theme.color_token("border")))
                    .style(ChromeRefinement::default().border_dash(DashPatternV1::new(
                        Px(4.0),
                        Px(4.0),
                        Px(0.0),
                    )))
                    .rounded(Radius::Lg)
                    .bg(ColorRef::Color(theme.color_token("card")))
                    .w_full()
                    .h_full()
                    .into_element(cx),
            );
        } else {
            for row in derived.rows.iter() {
                let remove_cmd = msg.cmd(Msg::Remove(row.id));
                out.push(cx.keyed(row.id, |cx| todo_row(cx, &theme, row, remove_cmd.clone())));
            }
        }
    })
    .gap(Space::N2)
    .w_full()
    .into_element(cx);

    let rows_scroll = shadcn::ScrollArea::new([rows])
        .ui()
        .w_full()
        .min_h(Px(200.0))
        .max_h(Px(260.0))
        .into_element(cx);

    let header = shadcn::CardHeader::new([ui::h_flex(cx, |_cx| {
        let app_icon = {
            let icon = icon::icon_with(
                _cx,
                lucide::LIST_TODO,
                Some(Px(24.0)),
                Some(ColorRef::Color(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                })),
            );

            ui::container(_cx, |cx| {
                [ui::h_flex(cx, |_cx| [icon])
                    .w_full()
                    .h_full()
                    .items_center()
                    .justify_center()
                    .into_element(cx)]
            })
            .w_px(Px(40.0))
            .h_px(Px(40.0))
            .bg(ColorRef::Color(theme.color_token("primary")))
            .rounded(Radius::Lg)
            .into_element(_cx)
            .test_id(TEST_ID_HEADER_ICON)
        };

        let today = today_label();
        let (desc_px, desc_line_height) = {
            let px = theme
                .metric_by_key("component.card.description_px")
                .or_else(|| theme.metric_by_key("font.size"))
                .unwrap_or_else(|| theme.metric_token("font.size"));
            let line_height = theme
                .metric_by_key("component.card.description_line_height")
                .or_else(|| theme.metric_by_key("font.line_height"))
                .unwrap_or_else(|| theme.metric_token("font.line_height"));
            (px, line_height)
        };
        let date = ui::h_flex(_cx, |cx| {
            ui::children![
                cx;
                icon::icon_with(
                    cx,
                    lucide::CALENDAR_DAYS,
                    Some(Px(12.0)),
                    Some(ColorRef::Color(theme.color_token("muted-foreground"))),
                ),
                ui::text(cx, today)
                    .text_size_px(desc_px)
                    .line_height_px(desc_line_height)
                    .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
                    .font_normal()
                    .nowrap()
                    .overflow(TextOverflow::Ellipsis)
                    .flex_1()
                    .min_w_0()
                    .text_color(ColorRef::Color(theme.color_token("muted-foreground"))),
            ]
        })
        .gap(Space::N1)
        .items_center()
        .flex_1()
        .min_w_0()
        .into_element(_cx)
        .test_id(TEST_ID_HEADER_DATE);

        let title = ui::v_flex(_cx, |_cx| {
            [shadcn::CardTitle::new("My Tasks").into_element(_cx), date]
        })
        .gap(Space::N1)
        .flex_1()
        .w_full()
        .min_w_0()
        .into_element(_cx);

        let left = ui::h_flex(_cx, |_cx| [app_icon, title])
            .gap(Space::N3)
            .items_center()
            .flex_1()
            .min_w_0()
            .into_element(_cx);

        let badge = shadcn::Badge::new(format!("{} remaining", derived.active))
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(_cx);

        let badge = ui::container(_cx, |_cx| [badge])
            .ml_auto()
            .into_element(_cx);

        [left, badge]
    })
    .w_full()
    .items_center()
    .into_element(cx)])
    .refine_style(ChromeRefinement::default().pb(Space::N4))
    .into_element(cx);

    let footer_actions = (derived.completed > 0).then(|| {
        let clear_icon = icon::icon_with(cx, lucide::X, Some(Px(12.0)), None);
        let clear_label = ui::label(cx, "Clear completed").nowrap().into_element(cx);
        let clear_done = shadcn::Button::new("Clear completed")
            .variant(shadcn::ButtonVariant::Ghost)
            .size(shadcn::ButtonSize::Sm)
            .on_click(clear_done_cmd)
            .refine_layout(LayoutRefinement::default().flex_shrink_0())
            .children([clear_icon, clear_label])
            .into_element(cx)
            .a11y_role(SemanticsRole::Button)
            .test_id(TEST_ID_CLEAR_DONE);

        let footer_row = ui::h_flex(cx, |_cx| [clear_done])
            .w_full()
            .justify_end()
            .into_element(cx);

        ui::v_flex(cx, |_cx| {
            [shadcn::Separator::new().into_element(_cx), footer_row]
        })
        .gap(Space::N2)
        .w_full()
        .into_element(cx)
    });

    let content = shadcn::CardContent::new([ui::v_flex_build(cx, |_cx, out| {
        out.push(input_row);
        out.push(filter_row);
        out.push(rows_scroll);
        if let Some(footer) = footer_actions {
            out.push(footer);
        }
    })
    .gap(Space::N6)
    .w_full()
    .into_element(cx)])
    .into_element(cx);

    let card_border = with_alpha(theme.color_token("border"), 0.6);
    let card = shadcn::Card::new([header, content])
        .refine_style(
            ChromeRefinement::default()
                .shadow_xl()
                .border_color(ColorRef::Color(card_border)),
        )
        .ui()
        .w_full()
        .max_w(Px(480.0))
        .into_element(cx);

    // shadcn demo pages often use `bg-slate-50`; approximate by mixing `background` with `muted`.
    // (We can't express Tailwind's `slate-50` directly as a theme token today.)
    let page_bg = mix_colors(
        theme.color_token("background"),
        theme.color_token("muted"),
        0.5,
    );
    let page = ui::container(cx, |cx| {
        [ui::v_flex(cx, |_cx| [card])
            .w_full()
            .h_full()
            .justify_center()
            .items_center()
            .into_element(cx)]
    })
    .bg(ColorRef::Color(page_bg))
    .p(Space::N6)
    .w_full()
    .h_full()
    .into_element(cx);

    page.into()
}

fn filter_chip(
    cx: &mut ElementContext<'_, App>,
    theme: &ThemeSnapshot,
    label: &'static str,
    cmd: CommandId,
    selected: bool,
    test_id: &'static str,
) -> AnyElement {
    let selected_style = ChromeRefinement::default()
        .bg(ColorRef::Color(theme.color_token("background")))
        .shadow_sm();

    let fg = if selected {
        WidgetStateProperty::new(Some(ColorRef::Color(theme.color_token("foreground"))))
    } else {
        WidgetStateProperty::new(Some(ColorRef::Color(theme.color_token("muted-foreground"))))
            .when(
                WidgetStates::HOVERED,
                Some(ColorRef::Color(theme.color_token("foreground"))),
            )
            .when(
                WidgetStates::ACTIVE,
                Some(ColorRef::Color(theme.color_token("foreground"))),
            )
    };

    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Ghost)
        .size(shadcn::ButtonSize::Sm)
        .on_click(cmd)
        .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
        .style(shadcn::button::ButtonStyle::default().foreground(fg))
        .refine_style(if selected {
            selected_style
        } else {
            ChromeRefinement::default()
        })
        .into_element(cx)
        .a11y_role(SemanticsRole::Button)
        .test_id(test_id)
}

fn todo_row(
    cx: &mut ElementContext<'_, App>,
    theme: &ThemeSnapshot,
    row: &TodoRowSnapshot,
    remove_cmd: CommandId,
) -> AnyElement {
    let muted_bg = theme.color_token("muted");
    let theme = theme.clone();
    let done_model = row.done_model.clone();
    let done = row.done;
    let text = row.text.clone();
    let id = row.id;
    let remove_cmd = remove_cmd.clone();

    cx.hover_region(HoverRegionProps::default(), move |cx, hovered| {
        let checkbox = shadcn::Checkbox::new(done_model.clone())
            .test_id(todo_done_test_id(id))
            .into_element(cx);

        let fg = theme.color_token(if done {
            "muted-foreground"
        } else {
            "foreground"
        });

        let mut span = TextSpan::new(text.len());
        if done {
            span.paint = TextPaintStyle::default().with_strikethrough(StrikethroughStyle {
                color: Some(theme.color_token("muted-foreground")),
                style: DecorationLineStyle::Solid,
            });
        }
        let rich = fret_core::AttributedText::new(text.clone(), Arc::from([span]));
        let mut text_layout = fret_ui::element::LayoutStyle::default();
        text_layout.size.width = fret_ui::element::Length::Fill;
        let label_text = cx.styled_text_props(fret_ui::element::StyledTextProps {
            layout: text_layout,
            rich,
            style: None,
            color: Some(fg),
            align: fret_core::TextAlign::Start,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            ink_overflow: Default::default(),
        });
        let label = ui::container(cx, |_cx| [label_text])
            .flex_1()
            .min_w_0()
            .into_element(cx)
            .test_id(todo_label_test_id(id));

        let theme_for_remove_btn = theme.clone();
        let remove_btn = cx.hover_region(HoverRegionProps::default(), move |cx, btn_hovered| {
            let remove_icon_color = if !hovered {
                Color::TRANSPARENT
            } else if btn_hovered {
                theme_for_remove_btn.color_token("destructive")
            } else {
                theme_for_remove_btn.color_token("muted-foreground")
            };

            let remove_icon = icon::icon_with(
                cx,
                lucide::TRASH_2,
                Some(Px(16.0)),
                Some(ColorRef::Color(remove_icon_color)),
            );

            let destructive_bg = with_alpha(theme_for_remove_btn.color_token("destructive"), 0.12);
            let btn_bg = if btn_hovered {
                ColorRef::Color(destructive_bg)
            } else {
                ColorRef::Color(Color::TRANSPARENT)
            };

            let btn = shadcn::Button::new("")
                .size(shadcn::ButtonSize::IconSm)
                .variant(shadcn::ButtonVariant::Ghost)
                .refine_layout(LayoutRefinement::default().ml_auto())
                .refine_style(ChromeRefinement::default().bg(btn_bg))
                .on_click(remove_cmd.clone())
                .children([remove_icon])
                .into_element(cx)
                .a11y_role(SemanticsRole::Button)
                .a11y_label("Remove task")
                .test_id(todo_remove_test_id(id));

            vec![btn]
        });

        let left = ui::h_flex(cx, |_cx| [checkbox, label])
            .gap(Space::N3)
            .items_center()
            .flex_1()
            .min_w_0()
            .into_element(cx);

        let row_body = ui::h_flex(cx, |_cx| [left, remove_btn])
            .w_full()
            .items_center()
            .into_element(cx);

        let (bg, border) = if hovered {
            (muted_bg, theme.color_token("border"))
        } else {
            (theme.color_token("card"), Color::TRANSPARENT)
        };

        let root = ui::container(cx, |_cx| [row_body])
            .bg(ColorRef::Color(bg))
            .border_1()
            .border_color(ColorRef::Color(border))
            .rounded(Radius::Md)
            .shadow_xs()
            .p(Space::N3)
            .w_full()
            .into_element(cx)
            .a11y_role(SemanticsRole::ListItem)
            .test_id(todo_row_test_id(id));

        vec![root]
    })
}

fn update(app: &mut App, state: &mut TodoState, msg: Msg) {
    match msg {
        Msg::Add => {
            let draft = app
                .models()
                .read(&state.draft, Clone::clone)
                .ok()
                .unwrap_or_default();
            let text = draft.trim();
            if text.is_empty() {
                return;
            }

            let item = TodoItem {
                id: state.next_id,
                done: app.models_mut().insert(false),
                text: Arc::from(text),
            };
            state.next_id = state.next_id.saturating_add(1);

            let _ = app.models_mut().update(&state.todos, |todos| {
                todos.insert(0, item);
            });
            let _ = app.models_mut().update(&state.draft, String::clear);
        }
        Msg::ClearDone => {
            let snapshot = app
                .models()
                .read(&state.todos, Clone::clone)
                .ok()
                .unwrap_or_default();
            let keep = snapshot
                .into_iter()
                .filter(|todo| app.models().get_copied(&todo.done).is_none_or(|done| !done))
                .collect::<Vec<_>>();
            let _ = app.models_mut().update(&state.todos, |todos| *todos = keep);
        }
        Msg::SetFilter(filter) => {
            let _ = app
                .models_mut()
                .update(&state.filter, |current| *current = filter);
        }
        Msg::Remove(id) => {
            let _ = app.models_mut().update(&state.todos, |todos| {
                todos.retain(|todo| todo.id != id);
            });
        }
    }
}
