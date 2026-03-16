use std::sync::Arc;

use fret::app::LocalState;
use fret::app::prelude::*;
use fret::semantics::SemanticsRole;
use fret::style::{ColorRef, Space, Theme, ThemeSnapshot};
use fret_ui::element::SemanticsDecoration;

mod act {
    fret::actions!([
        Add = "cookbook.simple_todo.add.v1",
        ClearDone = "cookbook.simple_todo.clear_done.v1"
    ]);

    fret::payload_actions!([Toggle(u64) = "cookbook.simple_todo.toggle.v1"]);
}

const TEST_ID_ROOT: &str = "cookbook.simple_todo.root";
const TEST_ID_DRAFT: &str = "cookbook.simple_todo.draft";
const TEST_ID_ADD: &str = "cookbook.simple_todo.add";
const TEST_ID_CLEAR_DONE: &str = "cookbook.simple_todo.clear_done";
const TEST_ID_PROGRESS: &str = "cookbook.simple_todo.progress";
const TEST_ID_ROWS: &str = "cookbook.simple_todo.rows";
const TEST_ID_ROW_PREFIX: &str = "cookbook.simple_todo.row.";

#[derive(Debug, Clone)]
struct TodoRow {
    id: u64,
    done: bool,
    text: Arc<str>,
}

struct SimpleTodoView;

impl View for SimpleTodoView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();
        let theme_for_rows = theme.clone();

        let draft_state = cx.state().local::<String>();
        let next_id_state = cx.state().local_init(|| 3u64);
        let todos_state = cx.state().local_init(|| {
            vec![
                TodoRow {
                    id: 1,
                    done: false,
                    text: Arc::from("Use keyed rows for dynamic lists"),
                },
                TodoRow {
                    id: 2,
                    done: true,
                    text: Arc::from("Try the shadcn theme tokens"),
                },
            ]
        });

        bind_todo_actions(cx, &draft_state, &next_id_state, &todos_state);

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
            .submit_action(act::Add)
            .test_id(TEST_ID_DRAFT);

        let input_row = ui::h_flex(|cx| ui::children![cx; input, add_btn])
            .gap(Space::N2)
            .items_center()
            .w_full();

        let rows = ui::v_flex(|cx| {
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
        .test_id(TEST_ID_ROWS);

        let card = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Simple todo"),
                        shadcn::card_description(
                            "LocalState<Vec<_>> + typed actions + keyed lists (no selector/query).",
                        ),
                        header_actions,
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::single(
                        cx,
                        ui::v_flex(|cx| ui::children![cx; input_row, rows])
                            .gap(Space::N4)
                            .w_full(),
                    )
                }),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(560.0));

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card)
    }
}

fn bind_todo_actions(
    cx: &mut AppUi<'_, '_>,
    draft_state: &LocalState<String>,
    next_id_state: &LocalState<u64>,
    todos_state: &LocalState<Vec<TodoRow>>,
) {
    cx.actions().locals::<act::Add>({
        let draft_state = LocalState::clone(draft_state);
        let next_id_state = LocalState::clone(next_id_state);
        let todos_state = LocalState::clone(todos_state);
        move |tx| {
            let text = tx
                .value_or_else(&draft_state, String::new)
                .trim()
                .to_string();
            if text.is_empty() {
                return false;
            }

            let id = tx.value_or(&next_id_state, 0);
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
        }
    });

    cx.actions().locals::<act::ClearDone>({
        let todos_state = LocalState::clone(todos_state);
        move |tx| {
            tx.update_if(&todos_state, |rows| {
                let before = rows.len();
                rows.retain(|row| !row.done);
                rows.len() != before
            })
        }
    });

    cx.actions()
        .payload_local_update_if::<act::Toggle, Vec<TodoRow>>(todos_state, |rows, id| {
            if let Some(row) = rows.iter_mut().find(|row| row.id == id) {
                row.done = !row.done;
                true
            } else {
                false
            }
        });
}

fn todo_row(theme: ThemeSnapshot, row: &TodoRow) -> impl UiChild {
    let checkbox = shadcn::Checkbox::from_checked(row.done)
        .action(act::Toggle)
        .action_payload(row.id)
        .a11y_label(row.text.clone());

    let text = ui::text(row.text.clone())
        .truncate()
        .text_sm()
        .text_color(ColorRef::Color(if row.done {
            theme.color_token("muted-foreground")
        } else {
            theme.color_token("foreground")
        }));

    ui::h_flex(|cx| ui::children![cx; checkbox, text])
        .gap(Space::N2)
        .items_center()
        .w_full()
        .test_id(format!("{TEST_ID_ROW_PREFIX}{}", row.id))
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-simple-todo")
        .window("cookbook-simple-todo", (640.0, 560.0))
        .config_files(false)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<SimpleTodoView>()?
        .run()
        .map_err(anyhow::Error::from)
}
