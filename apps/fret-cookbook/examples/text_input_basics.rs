use std::sync::Arc;

use fret::prelude::*;
use fret_app::{CommandMeta, CommandScope};
use fret_ui::{
    CommandAvailability,
    action::{OnCommand, OnCommandAvailability},
    element::SemanticsDecoration,
};

const CMD_SUBMIT: &str = "cookbook.text_input_basics.submit";
const CMD_CLEAR: &str = "cookbook.text_input_basics.clear";

const TEST_ID_ROOT: &str = "cookbook.text_input_basics.root";
const TEST_ID_INPUT: &str = "cookbook.text_input_basics.input";
const TEST_ID_LEN: &str = "cookbook.text_input_basics.len";
const TEST_ID_SUBMITTED_COUNT: &str = "cookbook.text_input_basics.submitted_count";

fn install_commands(app: &mut App) {
    let submit = CommandId::from(CMD_SUBMIT);
    let submit_meta = CommandMeta::new("Submit input")
        .with_description("Submits the current input value (clears on submit).")
        .with_category("Cookbook")
        .with_scope(CommandScope::Widget);
    app.commands_mut().register(submit, submit_meta);

    let clear = CommandId::from(CMD_CLEAR);
    let clear_meta = CommandMeta::new("Clear input")
        .with_description("Clears the current input value (Escape).")
        .with_category("Cookbook")
        .with_scope(CommandScope::Widget);
    app.commands_mut().register(clear, clear_meta);
}

fn command_handlers(
    text: Model<String>,
    submitted_count: Model<u32>,
) -> (OnCommand, OnCommandAvailability) {
    let text_for_command = text.clone();
    let text_for_availability = text;

    let submitted_count_for_command = submitted_count;

    let on_command: OnCommand = Arc::new(move |host, acx, command| match command.as_str() {
        CMD_SUBMIT => {
            let text = host
                .models_mut()
                .read(&text_for_command, Clone::clone)
                .ok()
                .unwrap_or_default();
            if text.trim().is_empty() {
                return false;
            }

            let _ = host
                .models_mut()
                .update(&submitted_count_for_command, |v| *v = v.saturating_add(1));
            let _ = host.models_mut().update(&text_for_command, String::clear);

            host.request_redraw(acx.window);
            true
        }
        CMD_CLEAR => {
            let _ = host.models_mut().update(&text_for_command, String::clear);
            host.request_redraw(acx.window);
            true
        }
        _ => false,
    });

    let on_command_availability: OnCommandAvailability = Arc::new(move |host, _acx, command| {
        let text = host
            .models_mut()
            .read(&text_for_availability, Clone::clone)
            .ok()
            .unwrap_or_default();
        let has_text = !text.trim().is_empty();

        match command.as_str() {
            CMD_SUBMIT | CMD_CLEAR => {
                if has_text {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
            _ => CommandAvailability::NotHandled,
        }
    });

    (on_command, on_command_availability)
}

struct TextInputBasicsState {
    text: Model<String>,
    submitted_count: Model<u32>,
}

struct TextInputBasicsProgram;

impl MvuProgram for TextInputBasicsProgram {
    type State = TextInputBasicsState;
    type Message = ();

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        Self::State {
            text: app.models_mut().insert(String::new()),
            submitted_count: app.models_mut().insert(0),
        }
    }

    fn update(_app: &mut App, _state: &mut Self::State, _message: Self::Message) {}

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        _msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        // Attach command handlers to the window's declarative root so Enter/Escape work even when
        // focus is inside the text input node.
        let base_root = cx.root_id();

        let theme = Theme::global(&*cx.app).snapshot();
        let submit_cmd = CommandId::from(CMD_SUBMIT);
        let clear_cmd = CommandId::from(CMD_CLEAR);

        let text = cx
            .watch_model(&state.text)
            .layout()
            .cloned_or_else(String::new);
        let submitted_count = cx.watch_model(&state.submitted_count).layout().copied_or(0);

        let text_len_chars = text.chars().count() as u32;
        let text_len = text_len_chars as f64;
        let submitted_count_u32 = submitted_count;
        let submitted_count = submitted_count_u32 as f64;

        let len_badge = shadcn::Badge::new(format!("Length: {text_len_chars} chars"))
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::ProgressBar)
                    .test_id(TEST_ID_LEN)
                    .numeric_value(text_len)
                    .numeric_range(0.0, 1024.0),
            );

        let submitted_badge = shadcn::Badge::new(format!("Submitted: {submitted_count_u32}"))
            .variant(shadcn::BadgeVariant::Secondary)
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::ProgressBar)
                    .test_id(TEST_ID_SUBMITTED_COUNT)
                    .numeric_value(submitted_count)
                    .numeric_range(0.0, 1024.0),
            );

        let input = shadcn::Input::new(state.text.clone())
            .a11y_label("Message")
            .placeholder("Type something, then press Enter (Escape clears).")
            .submit_command(submit_cmd.clone())
            .cancel_command(clear_cmd.clone())
            .test_id(TEST_ID_INPUT)
            .into_element(cx);

        let buttons = ui::h_flex(cx, |cx| {
            [
                shadcn::Button::new("Submit")
                    .variant(shadcn::ButtonVariant::Default)
                    .on_click(submit_cmd.clone())
                    .into_element(cx),
                shadcn::Button::new("Clear")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(clear_cmd.clone())
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .into_element(cx);

        let stats = ui::h_flex(cx, |_cx| [len_badge, submitted_badge])
            .gap(Space::N2)
            .items_center()
            .into_element(cx);

        let card = shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Text input basics").into_element(cx),
                shadcn::CardDescription::new(
                    "A minimal Input example (Enter = submit, Escape = clear) with numeric semantics gates.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([
                ui::v_flex(cx, |_cx| [input, buttons, stats])
                    .gap(Space::N3)
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .ui()
        .w_full()
        .max_w(Px(560.0))
        .into_element(cx);

        let root = ui::container(cx, |cx| {
            [ui::v_flex(cx, |_cx| [card])
                .gap(Space::N6)
                .items_center()
                .justify_center()
                .size_full()
                .into_element(cx)]
        })
        .bg(ColorRef::Color(theme.color_token("background")))
        .p(Space::N6)
        .into_element(cx)
        .test_id(TEST_ID_ROOT);

        let (on_command, on_command_availability) =
            command_handlers(state.text.clone(), state.submitted_count.clone());
        cx.command_on_command_for(base_root, on_command);
        cx.command_on_command_availability_for(base_root, on_command_availability);

        root.into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-text-input-basics")
        .window("cookbook-text-input-basics", (640.0, 420.0))
        .config_files(false)
        .install_app(install_commands)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_mvu::<TextInputBasicsProgram>()
        .map_err(anyhow::Error::from)
}
