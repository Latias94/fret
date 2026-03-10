use std::sync::Arc;

use fret::app::prelude::*;
use fret_ui::element::SemanticsDecoration;

mod act {
    fret::actions!([
        Add = "cookbook.simple_todo_v2_target.add.v1",
        ClearDone = "cookbook.simple_todo_v2_target.clear_done.v1"
    ]);

    fret::payload_actions!([
        Toggle(u64) = "cookbook.simple_todo_v2_target.toggle.v1",
        Remove(u64) = "cookbook.simple_todo_v2_target.remove.v1"
    ]);
}

const TEST_ID_ROOT: &str = "cookbook.simple_todo_v2_target.root";
const TEST_ID_DRAFT: &str = "cookbook.simple_todo_v2_target.draft";
const TEST_ID_ADD: &str = "cookbook.simple_todo_v2_target.add";
const TEST_ID_CLEAR_DONE: &str = "cookbook.simple_todo_v2_target.clear_done";
const TEST_ID_PROGRESS: &str = "cookbook.simple_todo_v2_target.progress";
const TEST_ID_ROWS: &str = "cookbook.simple_todo_v2_target.rows";
const TEST_ID_NOTE: &str = "cookbook.simple_todo_v2_target.note";
const TEST_ID_ROW_PREFIX: &str = "cookbook.simple_todo_v2_target.row.";
const TEST_ID_TOGGLE_PREFIX: &str = "cookbook.simple_todo_v2_target.toggle.";
const TEST_ID_REMOVE_PREFIX: &str = "cookbook.simple_todo_v2_target.remove.";

#[derive(Debug, Clone)]
struct TodoRow {
    id: u64,
    done: bool,
    text: Arc<str>,
}

struct SimpleTodoV2TargetView;

impl View for SimpleTodoV2TargetView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_, KernelApp>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();
        let theme_for_rows = theme.clone();

        let draft_state = cx.state().local::<String>();
        let next_id_state = cx.state().local_init(|| 3u64);
        let todos_state = cx.state().local_init(|| {
            vec![
                TodoRow {
                    id: 1,
                    done: false,
                    text: Arc::from("Keep the whole keyed list in LocalState<Vec<_>>"),
                },
                TodoRow {
                    id: 2,
                    done: true,
                    text: Arc::from("Use payload actions for per-row toggle/remove"),
                },
            ]
        });

        let todos = cx.state().watch(&todos_state).layout().value_or_default();
        let draft_value = cx.state().watch(&draft_state).layout().value_or_default();

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
            .placeholder("Add a local-state task?")
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
        .test_id(TEST_ID_ROWS);

        let note = ui::text(
            "Comparison target: the list lives in LocalState<Vec<TodoRow>> and row toggles use grouped actions/state helpers plus payload actions.",
        )
        .text_xs()
        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
        .test_id(TEST_ID_NOTE);

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Simple todo v2 target"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new(
                            "Comparison target: local draft + LocalState<Vec<_>> + checkbox snapshot/actions (no selector/query).",
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
        .max_w(Px(640.0));

        cx.actions().locals::<act::Add>({
            let draft_state = draft_state.clone();
            let next_id_state = next_id_state.clone();
            let todos_state = todos_state.clone();
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
            let todos_state = todos_state.clone();
            move |tx| {
                tx.update_if(&todos_state, |rows| {
                    let before = rows.len();
                    rows.retain(|row| !row.done);
                    rows.len() != before
                })
            }
        });

        cx.actions()
            .payload_local_update_if::<act::Toggle, Vec<TodoRow>>(&todos_state, |rows, id| {
                if let Some(row) = rows.iter_mut().find(|row| row.id == id) {
                    row.done = !row.done;
                    true
                } else {
                    false
                }
            });

        cx.actions()
            .payload_local_update_if::<act::Remove, Vec<TodoRow>>(&todos_state, |rows, id| {
                let before = rows.len();
                rows.retain(|row| row.id != id);
                rows.len() != before
            });

        fret_cookbook::scaffold::centered_page_muted_ui(cx, TEST_ID_ROOT, card).into()
    }
}

fn todo_row(theme: ThemeSnapshot, row: &TodoRow) -> impl UiChildIntoElement<KernelApp> {
    let checkbox = shadcn::Checkbox::from_checked(row.done)
        .action(act::Toggle)
        .action_payload(row.id)
        .a11y_label(row.text.clone())
        .test_id(format!("{TEST_ID_TOGGLE_PREFIX}{}", row.id));

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
        .variant(shadcn::ButtonVariant::Ghost)
        .size(shadcn::ButtonSize::Sm)
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

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-simple-todo-v2-target")
        .window("cookbook-simple-todo-v2-target", (720.0, 600.0))
        .config_files(false)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<SimpleTodoV2TargetView>()
        .map_err(anyhow::Error::from)
}
