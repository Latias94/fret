use std::sync::Arc;

use fret::prelude::*;
use fret_ui::element::SemanticsDecoration;

const TEST_ID_ROOT: &str = "cookbook.simple_todo.root";
const TEST_ID_DRAFT: &str = "cookbook.simple_todo.draft";
const TEST_ID_ADD: &str = "cookbook.simple_todo.add";
const TEST_ID_CLEAR_DONE: &str = "cookbook.simple_todo.clear_done";
const TEST_ID_PROGRESS: &str = "cookbook.simple_todo.progress";
const TEST_ID_ROWS: &str = "cookbook.simple_todo.rows";
const TEST_ID_ROW_PREFIX: &str = "cookbook.simple_todo.row.";

#[derive(Clone)]
struct TodoItem {
    id: u64,
    done: Model<bool>,
    text: Arc<str>,
}

struct TodoState {
    todos: Model<Vec<TodoItem>>,
    draft: Model<String>,
    next_id: u64,
}

#[derive(Debug, Clone)]
enum Msg {
    Add,
    ClearDone,
    Remove(u64),
}

struct SimpleTodoProgram;

impl MvuProgram for SimpleTodoProgram {
    type State = TodoState;
    type Message = Msg;

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        let done_1 = app.models_mut().insert(false);
        let done_2 = app.models_mut().insert(true);
        let todos = app.models_mut().insert(vec![
            TodoItem {
                id: 1,
                done: done_1,
                text: Arc::from("Use keyed rows for dynamic lists"),
            },
            TodoItem {
                id: 2,
                done: done_2,
                text: Arc::from("Try the shadcn theme tokens"),
            },
        ]);
        let draft = app.models_mut().insert(String::new());

        TodoState {
            todos,
            draft,
            next_id: 3,
        }
    }

    fn update(app: &mut App, st: &mut Self::State, msg: Self::Message) {
        match msg {
            Msg::Add => {
                let text = app
                    .models()
                    .read(&st.draft, |v| v.trim().to_string())
                    .ok()
                    .unwrap_or_default();
                if text.is_empty() {
                    return;
                }

                let done = app.models_mut().insert(false);
                let id = st.next_id;
                st.next_id = st.next_id.saturating_add(1);

                let _ = app.models_mut().update(&st.todos, |todos| {
                    todos.push(TodoItem {
                        id,
                        done,
                        text: Arc::from(text),
                    });
                });
                let _ = app.models_mut().update(&st.draft, |v| v.clear());
            }
            Msg::ClearDone => {
                let todos = app
                    .models()
                    .read(&st.todos, Clone::clone)
                    .ok()
                    .unwrap_or_default();

                let mut remove_ids = Vec::new();
                for t in &todos {
                    let done = app.models().read(&t.done, |v| *v).ok().unwrap_or(false);
                    if done {
                        remove_ids.push(t.id);
                    }
                }
                if remove_ids.is_empty() {
                    return;
                }

                let _ = app.models_mut().update(&st.todos, |todos| {
                    todos.retain(|t| !remove_ids.contains(&t.id));
                });
            }
            Msg::Remove(id) => {
                let _ = app.models_mut().update(&st.todos, |todos| {
                    todos.retain(|t| t.id != id);
                });
            }
        }
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        st: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();
        let theme_for_rows = theme.clone();

        let todos = cx.watch_model(&st.todos).layout().cloned_or_default();
        let draft_value = cx.watch_model(&st.draft).paint().cloned_or_default();

        let mut done_count = 0usize;
        for t in &todos {
            if cx.watch_model(&t.done).paint().copied_or_default() {
                done_count += 1;
            }
        }
        let total_count = todos.len();

        let add_enabled = !draft_value.trim().is_empty();
        let add_cmd = msg.cmd(Msg::Add);
        let clear_done_cmd = msg.cmd(Msg::ClearDone);

        let progress = shadcn::Badge::new(format!("{done_count}/{total_count} done"))
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::ProgressBar)
                    .test_id(TEST_ID_PROGRESS)
                    .numeric_value(done_count as f64)
                    .numeric_range(0.0, (total_count.max(1)) as f64),
            );

        let clear_done_btn = shadcn::Button::new("Clear done")
            .variant(shadcn::ButtonVariant::Secondary)
            .disabled(done_count == 0)
            .on_click(clear_done_cmd)
            .into_element(cx)
            .test_id(TEST_ID_CLEAR_DONE);

        let header_actions = ui::h_flex(cx, |_cx| [progress, clear_done_btn])
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

        let add_btn = shadcn::Button::new("Add")
            .disabled(!add_enabled)
            .on_click(add_cmd.clone())
            .into_element(cx)
            .test_id(TEST_ID_ADD);

        let input = shadcn::Input::new(st.draft.clone())
            .a11y_label("New task")
            .placeholder("Add a task…")
            .submit_command(add_cmd.clone())
            .into_element(cx)
            .test_id(TEST_ID_DRAFT);

        let input_row = ui::h_flex(cx, |_cx| [input, add_btn])
            .gap(Space::N2)
            .items_center()
            .w_full()
            .into_element(cx);

        let rows = ui::v_flex_build(cx, |cx, out| {
            for t in &todos {
                let remove_cmd = msg.cmd(Msg::Remove(t.id));
                let theme = theme_for_rows.clone();
                out.push(cx.keyed(t.id, |cx| todo_row(cx, theme, t, remove_cmd)));
            }
        })
        .gap(Space::N2)
        .into_element(cx)
        .test_id(TEST_ID_ROWS);

        let content = ui::v_flex(cx, |_cx| [input_row, rows])
            .gap(Space::N4)
            .w_full()
            .into_element(cx);

        let card = shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Simple todo").into_element(cx),
                shadcn::CardDescription::new(
                    "Model + MVU messages + keyed lists (no selector/query).",
                )
                .into_element(cx),
                header_actions,
            ])
            .into_element(cx),
            shadcn::CardContent::new([content]).into_element(cx),
        ])
        .ui()
        .w_full()
        .max_w(Px(560.0))
        .into_element(cx);

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn todo_row(
    cx: &mut ElementContext<'_, App>,
    theme: ThemeSnapshot,
    item: &TodoItem,
    remove_cmd: CommandId,
) -> AnyElement {
    let done = cx.watch_model(&item.done).paint().copied_or_default();

    let checkbox = shadcn::Checkbox::new(item.done.clone()).into_element(cx);

    let text = ui::text(cx, item.text.clone())
        .truncate()
        .text_sm()
        .text_color(ColorRef::Color(if done {
            theme.color_token("muted-foreground")
        } else {
            theme.color_token("foreground")
        }))
        .into_element(cx);

    let remove_btn = shadcn::Button::new("Remove")
        .variant(shadcn::ButtonVariant::Ghost)
        .on_click(remove_cmd)
        .into_element(cx);

    ui::h_flex(cx, |_cx| [checkbox, text, remove_btn])
        .gap(Space::N2)
        .items_center()
        .w_full()
        .into_element(cx)
        .test_id(format!("{TEST_ID_ROW_PREFIX}{}", item.id))
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-simple-todo")
        .window("cookbook-simple-todo", (640.0, 560.0))
        .config_files(false)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_mvu::<SimpleTodoProgram>()
        .map_err(anyhow::Error::from)
}
