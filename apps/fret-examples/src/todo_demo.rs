use std::sync::Arc;

use fret::{FretApp, advanced::prelude::*, shadcn};
use fret_ui::element::SemanticsDecoration;

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
const TEST_ID_DONE_PREFIX: &str = "todo_demo.done.";
const TEST_ID_ROW_PREFIX: &str = "todo_demo.row.";
const TEST_ID_REMOVE_PREFIX: &str = "todo_demo.remove.";
const TEST_ID_NOTE: &str = "todo_demo.note";

#[derive(Clone)]
struct TodoRow {
    id: u64,
    done: bool,
    text: Arc<str>,
}

struct TodoDemoView;

impl View for TodoDemoView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, KernelApp>) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();
        let theme_for_rows = theme.clone();

        let draft_state = cx.use_local::<String>();
        let next_id_state = cx.use_local_with(|| 4u64);
        let todos_state = cx.use_local_with(|| {
            vec![
                TodoRow {
                    id: 1,
                    done: false,
                    text: Arc::from("Action-first: one dispatch pipeline"),
                },
                TodoRow {
                    id: 2,
                    done: true,
                    text: Arc::from("View runtime: notify -> dirty -> reuse"),
                },
                TodoRow {
                    id: 3,
                    done: false,
                    text: Arc::from("Payload actions v2: parameterize toggle/remove"),
                },
            ]
        });

        let todos = todos_state.layout(cx).value_or_default();
        let draft_value = draft_state.layout(cx).value_or_default();

        let done_count = todos.iter().filter(|row| row.done).count();
        let total_count = todos.len();
        let add_enabled = !draft_value.trim().is_empty();

        let progress = shadcn::Badge::new(format!("{done_count}/{total_count} done"))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
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

        let input = shadcn::Input::new(&draft_state)
            .a11y_label("New task")
            .placeholder("Add a task?")
            .submit_command(act::Add.into())
            .test_id(TEST_ID_DRAFT);

        let input_row = ui::h_flex(|cx| ui::children![cx; input, add_btn])
            .gap(Space::N2)
            .items_center()
            .w_full();

        let rows = ui::v_flex_build(|cx, out| {
            if todos.is_empty() {
                out.push_ui(
                    cx,
                    ui::text("No tasks yet. Add one above.")
                        .text_sm()
                        .text_color(ColorRef::Color(
                            theme_for_rows.color_token("muted-foreground"),
                        )),
                );
                return;
            }

            for row in &todos {
                let theme = theme_for_rows.clone();
                out.push_ui(cx, ui::keyed(row.id, |_cx| todo_row(theme, row)));
            }
        })
        .gap(Space::N2)
        .w_full()
        .test_id(TEST_ID_ROWS);

        let note = ui::text(
            "App-grade comparison target: this demo now keeps the keyed list in LocalState<Vec<_>> and uses payload actions for row toggle/remove. The next migration target should be the scaffold simple-todo template, not more tracked-write helpers.",
        )
        .text_xs()
        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
        .test_id(TEST_ID_NOTE);

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Todo demo (action-first)"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new(
                            "App-grade local list + typed actions + payload row actions.",
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
                        ui::v_flex(|cx| ui::children![cx; input_row, rows, note])
                            .gap(Space::N4)
                            .w_full(),
                    );
                }),
            );
        })
        .ui()
        .w_full()
        .max_w(Px(720.0));

        cx.on_action_notify_models::<act::Add>({
            let draft_state = draft_state.clone();
            let next_id_state = next_id_state.clone();
            let todos_state = todos_state.clone();
            move |models| {
                let text = draft_state
                    .value_in_or_else(models, String::new)
                    .trim()
                    .to_string();
                if text.is_empty() {
                    return false;
                }

                let id = next_id_state
                    .read_in(models, |value| *value)
                    .ok()
                    .unwrap_or(1);
                let _ = next_id_state.update_in(models, |value| *value = value.saturating_add(1));

                if !todos_state.update_in(models, |rows| {
                    rows.push(TodoRow {
                        id,
                        done: false,
                        text: Arc::from(text),
                    });
                }) {
                    return false;
                }

                draft_state.set_in(models, String::new())
            }
        });

        cx.on_action_notify_models::<act::ClearDone>({
            let todos_state = todos_state.clone();
            move |models| {
                todos_state.update_in_if(models, |rows| {
                    let before = rows.len();
                    rows.retain(|row| !row.done);
                    rows.len() != before
                })
            }
        });

        cx.on_payload_action_notify_local_update_if::<act::Toggle, Vec<TodoRow>>(
            &todos_state,
            |rows, id| {
                if let Some(row) = rows.iter_mut().find(|row| row.id == id) {
                    row.done = !row.done;
                    true
                } else {
                    false
                }
            },
        );

        cx.on_payload_action_notify_local_update_if::<act::Remove, Vec<TodoRow>>(
            &todos_state,
            |rows, id| {
                let before = rows.len();
                rows.retain(|row| row.id != id);
                rows.len() != before
            },
        );

        ui::container(|cx| {
            ui::children![
                cx;
                ui::v_flex(|cx| ui::children![cx; card])
                    .w_full()
                    .h_full()
                    .justify_center()
                    .items_center()
            ]
        })
        .bg(ColorRef::Color(theme.color_token("muted")))
        .p(Space::N6)
        .w_full()
        .h_full()
        .test_id(TEST_ID_ROOT)
        .into_element(cx)
        .into()
    }
}

fn todo_row(theme: ThemeSnapshot, row: &TodoRow) -> impl UiChildIntoElement<KernelApp> {
    let checkbox = shadcn::Checkbox::from_checked(row.done)
        .action(act::Toggle)
        .action_payload(row.id)
        .a11y_label(row.text.clone())
        .test_id(format!("{TEST_ID_DONE_PREFIX}{}", row.id));

    let text = ui::text(row.text.clone())
        .truncate()
        .text_sm()
        .flex_1()
        .min_w_0()
        .text_color(ColorRef::Color(if row.done {
            theme.color_token("muted-foreground")
        } else {
            theme.color_token("foreground")
        }));

    let leading = ui::h_flex(|cx| ui::children![cx; checkbox, text])
        .gap(Space::N2)
        .items_start()
        .flex_1()
        .min_w_0();

    let remove = shadcn::Button::new("Remove")
        .variant(shadcn::ButtonVariant::Secondary)
        .action(act::Remove)
        .action_payload(row.id)
        .test_id(format!("{TEST_ID_REMOVE_PREFIX}{}", row.id));

    ui::h_flex(|cx| ui::children![cx; leading, remove])
        .gap(Space::N2)
        .items_start()
        .justify_between()
        .w_full()
        .test_id(format!("{TEST_ID_ROW_PREFIX}{}", row.id))
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("todo-demo")
        .window("todo-demo", (860.0, 640.0))
        .config_files(false)
        .run_view::<TodoDemoView>()
        .map_err(anyhow::Error::from)
}
