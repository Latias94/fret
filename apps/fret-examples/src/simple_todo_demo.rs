use std::sync::Arc;

use fret::app::LocalState;
use fret::app::prelude::*;
use fret::semantics::SemanticsRole;
use fret::style::{ColorRef, Space, Theme, ThemeSnapshot};
use fret_bootstrap::ui_app_driver;
use fret_runtime::PlatformCapabilities;
use fret_ui::element::SemanticsDecoration;

mod act {
    fret::actions!([
        Add = "simple_todo_demo.add.v1",
        ClearDone = "simple_todo_demo.clear_done.v1"
    ]);

    fret::payload_actions!([
        Toggle(u64) = "simple_todo_demo.toggle.v1",
        Remove(u64) = "simple_todo_demo.remove.v1"
    ]);
}

const TEST_ID_ROOT: &str = "simple-todo.root";
const TEST_ID_INPUT: &str = "simple-todo.input";
const TEST_ID_ADD: &str = "simple-todo.add";
const TEST_ID_CLEAR_DONE: &str = "simple-todo.clear-done";
const TEST_ID_PROGRESS: &str = "simple-todo.progress";
const TEST_ID_ROWS: &str = "simple-todo.rows";
const TEST_ID_ROW_PREFIX: &str = "simple-todo.row.";
const TEST_ID_DONE_PREFIX: &str = "simple-todo.done.";
const TEST_ID_REMOVE_PREFIX: &str = "simple-todo.remove.";

#[derive(Clone)]
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
                    text: Arc::from("Keep the default lane on typed payload actions"),
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
            .placeholder("Add a task...")
            .submit_action(act::Add)
            .test_id(TEST_ID_INPUT);

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
        .gap(Space::N3)
        .w_full()
        .test_id(TEST_ID_ROWS);

        let card = shadcn::card(|cx| {
            ui::children![cx;
                shadcn::card_header(|cx| {
                    ui::children![cx;
                        shadcn::card_title("Simple Todo"),
                        shadcn::card_description(
                            "View runtime + typed actions + keyed lists (no selector/query).",
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

        ui::single(cx, todo_page(theme, card))
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

    cx.actions()
        .payload_local_update_if::<act::Remove, Vec<TodoRow>>(todos_state, |rows, id| {
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
        .text_color(ColorRef::Color(if row.done {
            theme.color_token("muted-foreground")
        } else {
            theme.color_token("foreground")
        }));

    let remove_btn = shadcn::Button::new("Remove")
        .variant(shadcn::ButtonVariant::Ghost)
        .action(act::Remove)
        .action_payload(row.id)
        .test_id(format!("{TEST_ID_REMOVE_PREFIX}{}", row.id));

    ui::h_flex(|cx| ui::children![cx; checkbox, text, remove_btn])
        .gap(Space::N2)
        .items_center()
        .w_full()
        .test_id(format!("{TEST_ID_ROW_PREFIX}{}", row.id))
}

fn install_demo_theme(app: &mut App) {
    shadcn::themes::apply_shadcn_new_york(
        app,
        shadcn::themes::ShadcnBaseColor::Slate,
        shadcn::themes::ShadcnColorScheme::Light,
    );
}

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    install_demo_theme(&mut app);
    app
}

pub fn build_runner_config() -> fret_launch::WinitRunnerConfig {
    fret_launch::WinitRunnerConfig {
        main_window_title: "fret-demo simple-todo".to_string(),
        main_window_size: fret_launch::WindowLogicalSize::new(560.0, 520.0),
        ..Default::default()
    }
}

pub fn build_fn_driver() -> impl fret_launch::WinitAppDriver {
    ui_app_driver::UiAppDriver::new(
        "simple-todo-demo",
        fret::advanced::view::view_init_window::<SimpleTodoView>,
        fret::advanced::view::view_view::<SimpleTodoView>,
    )
    .on_preferences(
        ui_app_driver::default_on_preferences::<
            fret::advanced::view::ViewWindowState<SimpleTodoView>,
        >,
    )
    .into_fn_driver()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let app = build_app();
    let config = build_runner_config();
    let driver = build_fn_driver();
    crate::run_native_with_compat_driver(config, app, driver)
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> anyhow::Result<()> {
    Ok(())
}
