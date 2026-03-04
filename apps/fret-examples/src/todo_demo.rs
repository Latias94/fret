use std::sync::Arc;

use fret::prelude::*;
use fret_ui::element::SemanticsDecoration;

mod act {
    fret::actions!([
        Add = "todo_demo.add.v1",
        ClearDone = "todo_demo.clear_done.v1"
    ]);

    fret::payload_actions!([Remove(u64) = "todo_demo.remove.v2"]);
}

const TEST_ID_ROOT: &str = "todo_demo.root";
const TEST_ID_DRAFT: &str = "todo_demo.draft";
const TEST_ID_ADD: &str = "todo_demo.add";
const TEST_ID_CLEAR_DONE: &str = "todo_demo.clear_done";
const TEST_ID_PROGRESS: &str = "todo_demo.progress";
const TEST_ID_ROWS: &str = "todo_demo.rows";
const TEST_ID_ROW_PREFIX: &str = "todo_demo.row.";
const TEST_ID_REMOVE_PREFIX: &str = "todo_demo.remove.";

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

struct TodoDemoView {
    st: TodoState,
}

impl View for TodoDemoView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
        let done_1 = app.models_mut().insert(false);
        let done_2 = app.models_mut().insert(true);
        let done_3 = app.models_mut().insert(false);

        let todos = app.models_mut().insert(vec![
            TodoItem {
                id: 1,
                done: done_1,
                text: Arc::from("Action-first: one dispatch pipeline"),
            },
            TodoItem {
                id: 2,
                done: done_2,
                text: Arc::from("View runtime: notify → dirty → reuse"),
            },
            TodoItem {
                id: 3,
                done: done_3,
                text: Arc::from("Payload actions v2: parameterize remove"),
            },
        ]);

        let draft = app.models_mut().insert(String::new());
        let next_id = app.models_mut().insert(4u64);

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
            .into_element(cx)
            .test_id(TEST_ID_CLEAR_DONE);

        let header_actions = ui::h_flex(cx, |_cx| [progress, clear_done_btn])
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

        let add_btn = shadcn::Button::new("Add")
            .disabled(!add_enabled)
            .action(act::Add)
            .into_element(cx)
            .test_id(TEST_ID_ADD);

        let input = shadcn::Input::new(self.st.draft.clone())
            .a11y_label("New task")
            .placeholder("Add a task…")
            .submit_command(act::Add.into())
            .into_element(cx)
            .test_id(TEST_ID_DRAFT);

        let input_row = ui::h_flex(cx, |_cx| [input, add_btn])
            .gap(Space::N2)
            .items_center()
            .w_full()
            .into_element(cx);

        let rows = ui::v_flex_build(cx, |cx, out| {
            for t in &todos {
                let theme = theme_for_rows.clone();
                out.push(cx.keyed(t.id, |cx| todo_row(cx, theme, t)));
            }
        })
        .gap(Space::N2)
        .w_full()
        .into_element(cx)
        .test_id(TEST_ID_ROWS);

        let content = ui::v_flex(cx, |_cx| [input_row, rows])
            .gap(Space::N4)
            .w_full()
            .into_element(cx);

        let card = shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Todo demo (action-first)").into_element(cx),
                shadcn::CardDescription::new("View runtime + typed actions + payload remove.")
                    .into_element(cx),
                header_actions,
            ])
            .into_element(cx),
            shadcn::CardContent::new([content]).into_element(cx),
        ])
        .ui()
        .w_full()
        .max_w(Px(720.0))
        .into_element(cx);

        cx.on_action_notify::<act::Add>({
            let todos = self.st.todos.clone();
            let draft = self.st.draft.clone();
            let next_id = self.st.next_id.clone();
            move |host, _acx| {
                let text = host
                    .models_mut()
                    .read(&draft, |v| v.trim().to_string())
                    .ok()
                    .unwrap_or_default();
                if text.is_empty() {
                    return false;
                }

                let done = host.models_mut().insert(false);
                let id = host.models_mut().read(&next_id, |v| *v).ok().unwrap_or(0);
                let _ = host
                    .models_mut()
                    .update(&next_id, |v| *v = v.saturating_add(1).max(1));

                let _ = host.models_mut().update(&todos, |todos| {
                    todos.push(TodoItem {
                        id,
                        done,
                        text: Arc::from(text),
                    });
                });
                let _ = host.models_mut().update(&draft, String::clear);
                true
            }
        });

        cx.on_action_notify::<act::ClearDone>({
            let todos = self.st.todos.clone();
            move |host, _acx| {
                let snapshot = host
                    .models_mut()
                    .read(&todos, Clone::clone)
                    .ok()
                    .unwrap_or_default();

                let mut remove_ids = Vec::new();
                for t in &snapshot {
                    let done = host
                        .models_mut()
                        .read(&t.done, |v| *v)
                        .ok()
                        .unwrap_or(false);
                    if done {
                        remove_ids.push(t.id);
                    }
                }
                if remove_ids.is_empty() {
                    return false;
                }

                let _ = host.models_mut().update(&todos, |todos| {
                    todos.retain(|t| !remove_ids.contains(&t.id));
                });
                true
            }
        });

        cx.on_payload_action_notify::<act::Remove>({
            let todos = self.st.todos.clone();
            move |host, _acx, id| {
                let _ = host.models_mut().update(&todos, |todos| {
                    todos.retain(|t| t.id != id);
                });
                true
            }
        });

        ui::container(cx, |cx| {
            let centered = ui::v_flex(cx, |_cx| [card])
                .w_full()
                .h_full()
                .justify_center()
                .items_center()
                .into_element(cx);
            [centered]
        })
        .bg(ColorRef::Color(theme.color_token("muted")))
        .p(Space::N6)
        .w_full()
        .h_full()
        .into_element(cx)
        .test_id(TEST_ID_ROOT)
        .into()
    }
}

fn todo_row(cx: &mut ElementContext<'_, App>, theme: ThemeSnapshot, item: &TodoItem) -> AnyElement {
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

    let remove = shadcn::Button::new("Remove")
        .variant(shadcn::ButtonVariant::Secondary)
        .action(act::Remove)
        .action_payload(item.id)
        .test_id(format!("{TEST_ID_REMOVE_PREFIX}{}", item.id))
        .into_element(cx);

    ui::h_flex(cx, |_cx| [checkbox, text, remove])
        .gap(Space::N2)
        .items_center()
        .justify_between()
        .w_full()
        .into_element(cx)
        .test_id(format!("{TEST_ID_ROW_PREFIX}{}", item.id))
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("todo-demo")
        .window("todo-demo", (860.0, 640.0))
        .config_files(false)
        .run_view::<TodoDemoView>()
        .map_err(anyhow::Error::from)
}
