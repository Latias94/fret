use std::sync::Arc;
use std::time::{Duration, SystemTime};

use fret_kit::prelude::*;
use fret_query::ui::QueryElementContextExt as _;
use fret_query::{QueryKey, QueryPolicy, QueryState, QueryStatus, with_query_client};
use fret_selector::ui::SelectorElementContextExt as _;
use fret_ui_shadcn::state::{query_error_alert, query_status_badge};

const TEST_ID_INPUT: &str = "todo-input";
const TEST_ID_ADD: &str = "todo-add";
const TEST_ID_CLEAR_DONE: &str = "todo-clear-done";
const TEST_ID_FILTER_ALL: &str = "todo-filter-all";
const TEST_ID_FILTER_ACTIVE: &str = "todo-filter-active";
const TEST_ID_FILTER_COMPLETED: &str = "todo-filter-completed";
const TEST_ID_REFRESH_TIP: &str = "todo-refresh-tip";

fn todo_row_test_id(id: u64) -> Arc<str> {
    Arc::from(format!("todo-row-{id}"))
}

fn todo_remove_test_id(id: u64) -> Arc<str> {
    Arc::from(format!("todo-remove-{id}"))
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
    router: MessageRouter<Msg>,
    next_id: u64,
}

#[derive(Debug, Clone)]
enum Msg {
    Add,
    ClearDone,
    RefreshTip,
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

    fn as_label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Completed => "Completed",
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
    total: usize,
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

#[derive(Debug)]
struct TipData {
    text: Arc<str>,
}

fn tip_key() -> QueryKey<TipData> {
    QueryKey::new("todo.tip.v1", &0u8)
}

fn tip_policy() -> QueryPolicy {
    QueryPolicy {
        stale_time: Duration::from_secs(10),
        cache_time: Duration::from_secs(60),
        keep_previous_data_while_loading: true,
        ..Default::default()
    }
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::app_with_hooks("todo-demo", init_window, view, |d| d.on_command(on_command))?
        .with_main_window("todo_demo", (560.0, 520.0))
        .run()?;
    Ok(())
}

fn init_window(app: &mut App, window: AppWindowId) -> TodoState {
    let done_1 = app.models_mut().insert(false);
    let done_2 = app.models_mut().insert(true);
    let done_3 = app.models_mut().insert(false);
    let todos = app.models_mut().insert(vec![
        TodoItem {
            id: 1,
            done: done_1,
            text: Arc::from("Try the shadcn New York style"),
        },
        TodoItem {
            id: 2,
            done: done_2,
            text: Arc::from("Validate selector derived state"),
        },
        TodoItem {
            id: 3,
            done: done_3,
            text: Arc::from("Use query invalidation for async tips"),
        },
    ]);

    let prefix = format!("todo-demo.{window:?}.");
    TodoState {
        todos,
        draft: app.models_mut().insert(String::new()),
        filter: app.models_mut().insert(TodoFilter::All),
        router: MessageRouter::new(prefix),
        next_id: 4,
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut TodoState) -> ViewElements {
    st.router.clear();
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
                    let mut completed = 0usize;
                    for todo in todos {
                        let done = host.models().get_copied(&todo.done).unwrap_or(false);
                        if done {
                            completed += 1;
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
                    let total = todos.len();
                    TodoDerived {
                        rows: rows.into(),
                        total,
                        active: total.saturating_sub(completed),
                        completed,
                    }
                })
                .ok()
                .unwrap_or_else(|| TodoDerived {
                    rows: Arc::from([]),
                    total: 0,
                    active: 0,
                    completed: 0,
                })
        },
    );

    let tip_handle = cx.use_query(tip_key(), tip_policy(), move |_token| {
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::sleep(Duration::from_millis(120));

        Ok(TipData {
            text: Arc::from(format!("Tip fetched at {:?}", SystemTime::now())),
        })
    });
    let tip_state = cx
        .watch_model(tip_handle.model())
        .layout()
        .cloned_or_else(QueryState::<TipData>::default);

    let tip_status = query_status_badge(cx, &tip_state);
    let (tip_text, tip_color_key): (Arc<str>, &'static str) = match tip_state.status {
        QueryStatus::Idle | QueryStatus::Loading => {
            (Arc::from("Tip: loading..."), "muted-foreground")
        }
        QueryStatus::Error => {
            let err = tip_state
                .error
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| String::from("unknown error"));
            (Arc::from(format!("Tip error: {err}")), "destructive")
        }
        QueryStatus::Success => {
            let text = tip_state
                .data
                .as_ref()
                .map(|value| value.text.clone())
                .unwrap_or_else(|| Arc::from("Tip: no data"));
            (text, "muted-foreground")
        }
    };

    let tip_line = ui::h_flex(cx, |_cx| {
        [
            tip_status,
            ui::text(_cx, tip_text)
                .text_color(ColorRef::Color(theme.color_required(tip_color_key)))
                .into_element(_cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .w_full()
    .into_element(cx);

    let tip_error = query_error_alert(cx, &tip_state);
    let tip_block = ui::v_flex_build(cx, |_cx, out| {
        out.push(tip_line);
        if let Some(alert) = tip_error {
            out.push(alert);
        }
    })
    .gap(Space::N2)
    .w_full()
    .into_element(cx);

    let add_enabled = !draft_value.trim().is_empty();
    let add_cmd = st.router.cmd(Msg::Add);
    let clear_done_cmd = st.router.cmd(Msg::ClearDone);
    let refresh_tip_cmd = st.router.cmd(Msg::RefreshTip);
    let filter_all_cmd = st.router.cmd(Msg::SetFilter(TodoFilter::All));
    let filter_active_cmd = st.router.cmd(Msg::SetFilter(TodoFilter::Active));
    let filter_completed_cmd = st.router.cmd(Msg::SetFilter(TodoFilter::Completed));
    let add_button = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .disabled(!add_enabled)
        .on_click(add_cmd.clone())
        .children([icon::icon(cx, IconId::new("lucide.plus"))])
        .into_element(cx)
        .a11y_role(SemanticsRole::Button)
        .test_id(TEST_ID_ADD);

    let input = shadcn::Input::new(st.draft.clone())
        .placeholder("Add a task")
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
        "All",
        filter_all_cmd,
        filter_value == TodoFilter::All,
        TEST_ID_FILTER_ALL,
    );
    let filter_active = filter_chip(
        cx,
        "Active",
        filter_active_cmd,
        filter_value == TodoFilter::Active,
        TEST_ID_FILTER_ACTIVE,
    );
    let filter_completed = filter_chip(
        cx,
        "Completed",
        filter_completed_cmd,
        filter_value == TodoFilter::Completed,
        TEST_ID_FILTER_COMPLETED,
    );

    let filter_row = ui::h_flex(cx, |_cx| [filter_all, filter_active, filter_completed])
        .gap(Space::N2)
        .items_center()
        .w_full()
        .into_element(cx);

    let rows = ui::v_flex_build(cx, |cx, out| {
        for row in derived.rows.iter() {
            let remove_cmd = st.router.cmd(Msg::Remove(row.id));
            out.push(cx.keyed(row.id, |cx| todo_row(cx, &theme, row, remove_cmd.clone())));
        }

        if derived.rows.is_empty() {
            out.push(
                ui::text(
                    cx,
                    format!("No {} tasks", filter_value.as_label().to_lowercase()),
                )
                .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                .into_element(cx),
            );
        }
    })
    .gap(Space::N3)
    .w_full()
    .into_element(cx);

    let clear_done = shadcn::Button::new("Clear completed")
        .variant(shadcn::ButtonVariant::Secondary)
        .disabled(derived.completed == 0)
        .on_click(clear_done_cmd)
        .into_element(cx)
        .a11y_role(SemanticsRole::Button)
        .test_id(TEST_ID_CLEAR_DONE);

    let refresh_tip = shadcn::Button::new("Refresh tip")
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(refresh_tip_cmd)
        .into_element(cx)
        .a11y_role(SemanticsRole::Button)
        .test_id(TEST_ID_REFRESH_TIP);

    let summary = ui::text(
        cx,
        format!(
            "{} total | {} active | {} completed",
            derived.total, derived.active, derived.completed
        ),
    )
    .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
    .into_element(cx);

    let card = shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Todo").into_element(cx),
            shadcn::CardDescription::new(
                "Official best-practice baseline: model + selector + query.",
            )
            .into_element(cx),
            ui::h_flex(cx, |_cx| [clear_done, refresh_tip])
                .gap(Space::N2)
                .into_element(cx),
            summary,
            tip_block,
        ])
        .into_element(cx),
        shadcn::CardContent::new([ui::v_flex(cx, |_cx| [input_row, filter_row, rows])
            .gap(Space::N4)
            .w_full()
            .into_element(cx)])
        .into_element(cx),
    ])
    .ui()
    .bg(ColorRef::Color(theme.color_required("background")))
    .rounded(Radius::Lg)
    .border_1()
    .border_color(ColorRef::Color(theme.color_required("border")))
    .w_full()
    .max_w(Px(560.0))
    .into_element(cx);

    let page = ui::container(cx, |cx| {
        [ui::v_flex(cx, |_cx| [card])
            .w_full()
            .h_full()
            .justify_center()
            .items_center()
            .into_element(cx)]
    })
    .bg(ColorRef::Color(theme.color_required("muted")))
    .p(Space::N6)
    .w_full()
    .h_full()
    .into_element(cx);

    page.into()
}

fn filter_chip(
    cx: &mut ElementContext<'_, App>,
    label: &'static str,
    cmd: CommandId,
    selected: bool,
    test_id: &'static str,
) -> AnyElement {
    shadcn::Button::new(label)
        .variant(if selected {
            shadcn::ButtonVariant::Secondary
        } else {
            shadcn::ButtonVariant::Ghost
        })
        .size(shadcn::ButtonSize::Sm)
        .on_click(cmd)
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
    let checkbox = shadcn::Checkbox::new(row.done_model.clone()).into_element(cx);
    let remove_btn = shadcn::Button::new("")
        .size(shadcn::ButtonSize::Icon)
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(remove_cmd)
        .children([icon::icon(cx, IconId::new("lucide.trash"))])
        .into_element(cx)
        .a11y_role(SemanticsRole::Button)
        .test_id(todo_remove_test_id(row.id));

    let label = cx.text_props(TextProps {
        layout: Default::default(),
        text: row.text.clone(),
        style: None,
        color: Some(theme.color_required(if row.done {
            "muted-foreground"
        } else {
            "foreground"
        })),
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
    });

    let left = ui::h_flex(cx, |_cx| [checkbox, label])
        .gap(Space::N3)
        .items_center()
        .flex_1()
        .min_w_0()
        .into_element(cx);

    let row_body = ui::h_flex(cx, |_cx| [left, remove_btn])
        .w_full()
        .justify_between()
        .items_center()
        .into_element(cx);

    ui::container(cx, |_cx| [row_body])
        .border_1()
        .border_color(ColorRef::Color(theme.color_required("border")))
        .rounded(Radius::Md)
        .p(Space::N3)
        .w_full()
        .into_element(cx)
        .a11y_role(SemanticsRole::ListItem)
        .test_id(todo_row_test_id(row.id))
}

fn on_command(
    app: &mut App,
    _services: &mut dyn UiServices,
    window: AppWindowId,
    _ui: &mut UiTree<App>,
    state: &mut TodoState,
    cmd: &CommandId,
) {
    let Some(msg) = state.router.try_take(cmd) else {
        return;
    };

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
            app.request_redraw(window);
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
            app.request_redraw(window);
        }
        Msg::RefreshTip => {
            let _ = with_query_client(app, |client, app| {
                client.invalidate(app, tip_key());
            });
            app.request_redraw(window);
        }
        Msg::SetFilter(filter) => {
            let _ = app
                .models_mut()
                .update(&state.filter, |current| *current = filter);
            app.request_redraw(window);
        }
        Msg::Remove(id) => {
            let _ = app.models_mut().update(&state.todos, |todos| {
                todos.retain(|todo| todo.id != id);
            });
            app.request_redraw(window);
        }
    }
}
