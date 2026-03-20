use std::sync::Arc;

use fret::app::LocalState;
use fret::app::prelude::*;
use fret::semantics::SemanticsRole;
use fret::style::{ColorRef, Space, Theme, ThemeSnapshot};
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
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();
        let theme_for_rows = theme.clone();

        let draft_state = cx.state().local::<String>();
        let next_id_state = cx.state().local_init(|| 4u64);
        let todos_state = cx.state().local_init(|| {
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

        bind_todo_actions(cx, &draft_state, &next_id_state, &todos_state);

        let todos = todos_state.layout_value(cx);
        let draft_value = draft_state.layout_value(cx);

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
            .submit_action(act::Add)
            .test_id(TEST_ID_DRAFT);

        let input_row = ui::h_flex(|cx| ui::children![cx; input, add_btn])
            .gap(Space::N2)
            .items_center()
            .w_full();

        let rows = ui::v_flex(|cx| {
            if todos.is_empty() {
                return ui::children![
                    cx;
                    ui::text("No tasks yet. Add one above.")
                        .text_sm()
                        .text_color(ColorRef::Color(
                            theme_for_rows.color_token("muted-foreground"),
                        ))
                ];
            }

            ui::for_each_keyed(
                cx,
                &todos,
                |row| row.id,
                |row| {
                    let theme = theme_for_rows.clone();
                    todo_row(theme, row)
                },
            )
        })
        .gap(Space::N2)
        .w_full()
        .test_id(TEST_ID_ROWS);

        let note = ui::text(
            "App-grade comparison target: this demo now keeps the keyed list in LocalState<Vec<_>>, renders rows through ui::for_each_keyed(...), and uses payload actions for row toggle/remove.",
        )
        .text_xs()
        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
        .test_id(TEST_ID_NOTE);

        let card = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Todo demo (action-first)"),
                        shadcn::card_description(
                            "App-grade local list + typed actions + payload row actions.",
                        ),
                        header_actions,
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::single(
                        cx,
                        ui::v_flex(|cx| ui::children![cx; input_row, rows, note])
                            .gap(Space::N4)
                            .w_full(),
                    )
                }),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(720.0));

        ui::single(cx, todo_page(theme, card))
    }
}

fn bind_todo_actions(
    cx: &mut AppUi<'_, '_>,
    draft_state: &LocalState<String>,
    next_id_state: &LocalState<u64>,
    todos_state: &LocalState<Vec<TodoRow>>,
) {
    cx.actions()
        .locals_with((draft_state, next_id_state, todos_state))
        .on::<act::Add>(|tx, (draft_state, next_id_state, todos_state)| {
            let text = tx.value(&draft_state).trim().to_string();
            if text.is_empty() {
                return false;
            }

            let id = tx.value(&next_id_state);
            let _ = tx.update(&next_id_state, |value| *value = value.saturating_add(1));

            if !tx.update(&todos_state, |rows| {
                rows.push(TodoRow {
                    id,
                    done: false,
                    text: Arc::from(text),
                });
            }) {
                return false;
            }

            tx.set(&draft_state, String::new())
        });

    cx.actions()
        .locals_with(todos_state)
        .on::<act::ClearDone>(|tx, todos_state| {
            tx.update_if(&todos_state, |rows| {
                let before = rows.len();
                rows.retain(|row| !row.done);
                rows.len() != before
            })
        });

    cx.actions()
        .local(todos_state)
        .payload_update_if::<act::Toggle>(|rows, id| {
            if let Some(row) = rows.iter_mut().find(|row| row.id == id) {
                row.done = !row.done;
                true
            } else {
                false
            }
        });

    cx.actions()
        .local(todos_state)
        .payload_update_if::<act::Remove>(|rows, id| {
            let before = rows.len();
            rows.retain(|row| row.id != id);
            rows.len() != before
        });
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
    .bg(ColorRef::Color(theme.color_token("muted")))
    .p(Space::N6)
    .w_full()
    .h_full()
    .test_id(TEST_ID_ROOT)
}

fn todo_row(theme: ThemeSnapshot, row: &TodoRow) -> impl UiChild {
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
        .view::<TodoDemoView>()?
        .run()
        .map_err(anyhow::Error::from)
}
