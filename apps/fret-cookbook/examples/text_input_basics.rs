use fret::app::prelude::*;
use fret_app::{CommandMeta, CommandScope};
use fret_ui::{CommandAvailability, element::SemanticsDecoration};

mod act {
    fret::actions!([
        Submit = "cookbook.text_input_basics.submit",
        Clear = "cookbook.text_input_basics.clear"
    ]);
}

const TEST_ID_ROOT: &str = "cookbook.text_input_basics.root";
const TEST_ID_INPUT: &str = "cookbook.text_input_basics.input";
const TEST_ID_LEN: &str = "cookbook.text_input_basics.len";
const TEST_ID_SUBMITTED_COUNT: &str = "cookbook.text_input_basics.submitted_count";

fn install_commands(app: &mut App) {
    let submit: CommandId = act::Submit.into();
    let submit_meta = CommandMeta::new("Submit input")
        .with_description("Submits the current input value (clears on submit).")
        .with_category("Cookbook")
        .with_scope(CommandScope::Widget);
    app.commands_mut().register(submit, submit_meta);

    let clear: CommandId = act::Clear.into();
    let clear_meta = CommandMeta::new("Clear input")
        .with_description("Clears the current input value (Escape).")
        .with_category("Cookbook")
        .with_scope(CommandScope::Widget);
    app.commands_mut().register(clear, clear_meta);
}

struct TextInputBasicsView;

impl TextInputBasicsView {
    fn has_text(
        host: &mut dyn fret_ui::action::UiCommandAvailabilityActionHost,
        text: &LocalState<String>,
    ) -> bool {
        let text = text
            .read_in(host.models_mut(), Clone::clone)
            .ok()
            .unwrap_or_default();
        !text.trim().is_empty()
    }
}

impl View for TextInputBasicsView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let text_state = cx.state().local::<String>();
        let submitted_count_state = cx.state().local::<u32>();

        let text = cx
            .state()
            .watch(&text_state)
            .layout()
            .value_or_else(String::new);
        let submitted_count = cx
            .state()
            .watch(&submitted_count_state)
            .layout()
            .value_or(0);

        let text_len_chars = text.chars().count() as u32;
        let text_len = text_len_chars as f64;
        let submitted_count_u32 = submitted_count;
        let submitted_count = submitted_count_u32 as f64;

        let len_badge = shadcn::Badge::new(format!("Length: {text_len_chars} chars"))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::ProgressBar)
                    .test_id(TEST_ID_LEN)
                    .numeric_value(text_len)
                    .numeric_range(0.0, 1024.0),
            );

        let submitted_badge = shadcn::Badge::new(format!("Submitted: {submitted_count_u32}"))
            .variant(shadcn::BadgeVariant::Secondary)
            .a11y(
                SemanticsDecoration::default()
                    .role(SemanticsRole::ProgressBar)
                    .test_id(TEST_ID_SUBMITTED_COUNT)
                    .numeric_value(submitted_count)
                    .numeric_range(0.0, 1024.0),
            );

        let input = shadcn::Input::new(&text_state)
            .a11y_label("Message")
            .placeholder("Type something, then press Enter (Escape clears).")
            .submit_command(act::Submit.into())
            .cancel_command(act::Clear.into())
            .test_id(TEST_ID_INPUT);

        cx.actions().locals::<act::Submit>({
            let text_state = text_state.clone();
            let submitted_count_state = submitted_count_state.clone();
            move |tx| {
                let text = tx.value_or_else(&text_state, String::new);
                if text.trim().is_empty() {
                    return false;
                }

                let _ = tx.update(&submitted_count_state, |value| {
                    *value = value.saturating_add(1)
                });
                tx.set(&text_state, String::new())
            }
        });

        cx.actions().locals::<act::Clear>({
            let text_state = text_state.clone();
            move |tx| {
                let text = tx.value_or_else(&text_state, String::new);
                if text.trim().is_empty() {
                    return false;
                }

                tx.set(&text_state, String::new())
            }
        });

        cx.actions().availability::<act::Submit>({
            let text_state = text_state.clone();
            move |host, _acx| {
                if TextInputBasicsView::has_text(host, &text_state) {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
        });

        cx.actions().availability::<act::Clear>({
            let text_state = text_state.clone();
            move |host, _acx| {
                if TextInputBasicsView::has_text(host, &text_state) {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
        });

        let buttons = ui::h_flex(|cx| {
            ui::children![
                cx;
                shadcn::Button::new("Submit")
                    .variant(shadcn::ButtonVariant::Default)
                    .action(act::Submit),
                shadcn::Button::new("Clear")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::Clear),
            ]
        })
        .gap(Space::N2);

        let stats = ui::h_flex(|cx| ui::children![cx; len_badge, submitted_badge])
            .gap(Space::N2)
            .items_center();

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Text input basics"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new(
                            "A minimal Input example (Enter = submit, Escape = clear) with numeric semantics gates.",
                        ),
                    );
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(
                        cx,
                        ui::v_flex(|cx| ui::children![cx; input, buttons, stats]).gap(Space::N3),
                    );
                }),
            );
        })
        .ui()
        .w_full()
        .max_w(Px(560.0));

        fret_cookbook::scaffold::centered_page_background_ui(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-text-input-basics")
        .window("cookbook-text-input-basics", (640.0, 420.0))
        .config_files(false)
        .setup(install_commands)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<TextInputBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
