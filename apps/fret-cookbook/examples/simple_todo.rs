use std::sync::Arc;

use fret::prelude::*;
use fret_ui::element::SemanticsDecoration;

mod act {
    fret::actions!([
        Add = "cookbook.simple_todo.add.v1",
        ClearDone = "cookbook.simple_todo.clear_done.v1"
    ]);
}

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
    next_id: Model<u64>,
}

struct SimpleTodoView {
    st: TodoState,
}

impl View for SimpleTodoView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
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

        let next_id = app.models_mut().insert(3u64);

        Self {
            st: TodoState {
                todos,
                draft,
                next_id,
            },
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();
        let theme_for_rows = theme.clone();

        let todos = cx.watch_model(&self.st.todos).layout().cloned_or_default();
        let draft_value = cx.watch_model(&self.st.draft).layout().cloned_or_default();

        let mut done_count = 0usize;
        for t in &todos {
            if cx.watch_model(&t.done).paint().copied_or_default() {
                done_count += 1;
            }
        }
        let total_count = todos.len();

        let add_enabled = !draft_value.trim().is_empty();

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
            .action(act::ClearDone)
            .test_id(TEST_ID_CLEAR_DONE);

        let header_actions = ui::h_flex(|cx| ui::children![cx; progress, clear_done_btn])
            .gap(Space::N2)
            .items_center();

        let add_btn = shadcn::Button::new("Add")
            .disabled(!add_enabled)
            .action(act::Add)
            .test_id(TEST_ID_ADD);

        let input = shadcn::Input::new(self.st.draft.clone())
            .a11y_label("New task")
            .placeholder("Add a task?")
            .submit_command(act::Add.into())
            .test_id(TEST_ID_DRAFT);

        let input_row = ui::h_flex(|cx| ui::children![cx; input, add_btn])
            .gap(Space::N2)
            .items_center()
            .w_full();

        let rows = ui::v_flex_build(|cx, out| {
            for t in &todos {
                let theme = theme_for_rows.clone();
                out.push(cx.keyed(t.id, |cx| todo_row(cx, theme, t)));
            }
        })
        .gap(Space::N2)
        .test_id(TEST_ID_ROWS);

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Simple todo"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new(
                            "Model + typed actions + keyed lists (no selector/query).",
                        ),
                    );
                    out.push_ui(cx, header_actions);
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(
                        cx,
                        ui::v_flex(|cx| ui::children![cx; input_row, rows])
                            .gap(Space::N4)
                            .w_full(),
                    );
                }),
            );
        })
        .ui()
        .w_full()
        .max_w(Px(560.0))
        .into_element(cx);

        cx.on_action_notify_models::<act::Add>({
            let todos = self.st.todos.clone();
            let draft = self.st.draft.clone();
            let next_id = self.st.next_id.clone();
            move |models| {
                let text = models
                    .read(&draft, |v| v.trim().to_string())
                    .ok()
                    .unwrap_or_default();
                if text.is_empty() {
                    return false;
                }

                let done = models.insert(false);
                let id = models.read(&next_id, |v| *v).ok().unwrap_or(0);
                let _ = models.update(&next_id, |v| *v = v.saturating_add(1));

                let _ = models.update(&todos, |todos| {
                    todos.push(TodoItem {
                        id,
                        done,
                        text: Arc::from(text),
                    });
                });
                let _ = models.update(&draft, String::clear);
                true
            }
        });

        cx.on_action_notify_models::<act::ClearDone>({
            let todos = self.st.todos.clone();
            move |models| {
                let snapshot = models.read(&todos, Clone::clone).ok().unwrap_or_default();

                let mut remove_ids = Vec::new();
                for t in &snapshot {
                    let done = models.read(&t.done, |v| *v).ok().unwrap_or(false);
                    if done {
                        remove_ids.push(t.id);
                    }
                }
                if remove_ids.is_empty() {
                    return false;
                }

                let _ = models.update(&todos, |todos| {
                    todos.retain(|t| !remove_ids.contains(&t.id));
                });
                true
            }
        });

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn todo_row(cx: &mut ElementContext<'_, App>, theme: ThemeSnapshot, item: &TodoItem) -> AnyElement {
    let done = cx.watch_model(&item.done).paint().copied_or_default();

    let checkbox = shadcn::Checkbox::new(item.done.clone());

    let text = ui::text(item.text.clone())
        .truncate()
        .text_sm()
        .text_color(ColorRef::Color(if done {
            theme.color_token("muted-foreground")
        } else {
            theme.color_token("foreground")
        }));

    ui::h_flex(|cx| ui::children![cx; checkbox, text])
        .gap(Space::N2)
        .items_center()
        .w_full()
        .test_id(format!("{TEST_ID_ROW_PREFIX}{}", item.id))
        .into_element(cx)
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-simple-todo")
        .window("cookbook-simple-todo", (640.0, 560.0))
        .config_files(false)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<SimpleTodoView>()
        .map_err(anyhow::Error::from)
}
