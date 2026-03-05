use fret::prelude::*;
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

struct TextInputBasicsView {
    text: Model<String>,
    submitted_count: Model<u32>,
}

impl TextInputBasicsView {
    fn has_text(
        host: &mut dyn fret_ui::action::UiCommandAvailabilityActionHost,
        text: &Model<String>,
    ) -> bool {
        let text = host
            .models_mut()
            .read(text, Clone::clone)
            .ok()
            .unwrap_or_default();
        !text.trim().is_empty()
    }

    fn has_text_for_action(
        host: &mut dyn fret_ui::action::UiFocusActionHost,
        text: &Model<String>,
    ) -> bool {
        let text = host
            .models_mut()
            .read(text, Clone::clone)
            .ok()
            .unwrap_or_default();
        !text.trim().is_empty()
    }
}

impl View for TextInputBasicsView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
        Self {
            text: app.models_mut().insert(String::new()),
            submitted_count: app.models_mut().insert(0),
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let text = cx
            .watch_model(&self.text)
            .layout()
            .cloned_or_else(String::new);
        let submitted_count = cx.watch_model(&self.submitted_count).layout().copied_or(0);

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

        let input = shadcn::Input::new(self.text.clone())
            .a11y_label("Message")
            .placeholder("Type something, then press Enter (Escape clears).")
            .submit_command(act::Submit.into())
            .cancel_command(act::Clear.into())
            .test_id(TEST_ID_INPUT)
            .into_element(cx);

        cx.on_action::<act::Submit>({
            let text = self.text.clone();
            let submitted_count = self.submitted_count.clone();
            move |host, acx| {
                if !TextInputBasicsView::has_text_for_action(host, &text) {
                    return false;
                }

                let _ = host
                    .models_mut()
                    .update(&submitted_count, |v| *v = v.saturating_add(1));
                let _ = host.models_mut().update(&text, String::clear);
                host.request_redraw(acx.window);
                host.notify(acx);
                true
            }
        });

        cx.on_action::<act::Clear>({
            let text = self.text.clone();
            move |host, acx| {
                if !TextInputBasicsView::has_text_for_action(host, &text) {
                    return false;
                }

                let _ = host.models_mut().update(&text, String::clear);
                host.request_redraw(acx.window);
                host.notify(acx);
                true
            }
        });

        cx.on_action_availability::<act::Submit>({
            let text = self.text.clone();
            move |host, _acx| {
                if TextInputBasicsView::has_text(host, &text) {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
        });

        cx.on_action_availability::<act::Clear>({
            let text = self.text.clone();
            move |host, _acx| {
                if TextInputBasicsView::has_text(host, &text) {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
        });

        let buttons = ui::h_flex(|cx| {
            [
                shadcn::Button::new("Submit")
                    .variant(shadcn::ButtonVariant::Default)
                    .action(act::Submit)
                    .into_element(cx),
                shadcn::Button::new("Clear")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::Clear)
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .into_element(cx);

        let stats = ui::h_flex(|_cx| [len_badge, submitted_badge])
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
                ui::v_flex(|_cx| [input, buttons, stats])
                    .gap(Space::N3)
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .ui()
        .w_full()
        .max_w(Px(560.0))
        .into_element(cx);

        fret_cookbook::scaffold::centered_page_background(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-text-input-basics")
        .window("cookbook-text-input-basics", (640.0, 420.0))
        .config_files(false)
        .install_app(install_commands)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<TextInputBasicsView>()
        .map_err(anyhow::Error::from)
}
