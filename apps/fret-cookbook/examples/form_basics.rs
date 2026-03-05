use fret::prelude::*;
use fret_ui::CommandAvailability;
use fret_ui::element::SemanticsDecoration;

mod act {
    fret::actions!([
        Submit = "cookbook.form_basics.submit.v1",
        Reset = "cookbook.form_basics.reset.v1"
    ]);
}

const TEST_ID_ROOT: &str = "cookbook.form_basics.root";
const TEST_ID_NAME: &str = "cookbook.form_basics.name";
const TEST_ID_EMAIL: &str = "cookbook.form_basics.email";
const TEST_ID_ERROR: &str = "cookbook.form_basics.error";
const TEST_ID_VALID: &str = "cookbook.form_basics.valid";
const TEST_ID_SUBMIT: &str = "cookbook.form_basics.submit";
const TEST_ID_RESET: &str = "cookbook.form_basics.reset";

struct FormBasicsView {
    name: Model<String>,
    email: Model<String>,
    error: Model<Option<String>>,
}

impl FormBasicsView {
    fn validate(name: &str, email: &str) -> Option<String> {
        if name.trim().is_empty() {
            return Some("Name is required.".to_string());
        }
        if email.trim().is_empty() {
            return Some("Email is required.".to_string());
        }
        if !email.contains('@') {
            return Some("Email must contain '@'.".to_string());
        }
        None
    }
}

impl View for FormBasicsView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
        Self {
            name: app.models_mut().insert(String::new()),
            email: app.models_mut().insert(String::new()),
            error: app.models_mut().insert(None::<String>),
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let name = cx
            .watch_model(&self.name)
            .layout()
            .cloned_or_else(String::new);
        let email = cx
            .watch_model(&self.email)
            .layout()
            .cloned_or_else(String::new);
        let error = cx.watch_model(&self.error).layout().cloned_or_default();

        let can_submit = FormBasicsView::validate(&name, &email).is_none();

        cx.on_action_notify_models::<act::Submit>({
            let name_model = self.name.clone();
            let email_model = self.email.clone();
            let error_model = self.error.clone();
            move |models| {
                let name = models
                    .read(&name_model, Clone::clone)
                    .ok()
                    .unwrap_or_default();
                let email = models
                    .read(&email_model, Clone::clone)
                    .ok()
                    .unwrap_or_default();
                let err = FormBasicsView::validate(&name, &email);
                models.update(&error_model, |v| *v = err).is_ok()
            }
        });

        cx.on_action_notify_models::<act::Reset>({
            let name_model = self.name.clone();
            let email_model = self.email.clone();
            let error_model = self.error.clone();
            move |models| {
                let ok = models.update(&name_model, String::clear).is_ok();
                let ok = models.update(&email_model, String::clear).is_ok() && ok;
                let ok = models.update(&error_model, |v| *v = None).is_ok() && ok;
                ok
            }
        });

        cx.on_action_availability::<act::Submit>(move |_host, _acx| {
            if can_submit {
                CommandAvailability::Available
            } else {
                CommandAvailability::Blocked
            }
        });
        cx.on_action_availability::<act::Reset>(|_host, _acx| CommandAvailability::Available);

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("Form basics").into_element(cx),
            shadcn::CardDescription::new(
                "A minimal form with validation (no extra form registry dependency).",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let name_input = ui::v_flex(|cx| {
            [
                shadcn::Label::new("Name").into_element(cx),
                shadcn::Input::new(self.name.clone())
                    .a11y_label("Name")
                    .placeholder("Jane Doe")
                    .test_id(TEST_ID_NAME)
                    .into_element(cx),
            ]
        })
        .gap(Space::N1)
        .into_element(cx);

        let email_input = ui::v_flex(|cx| {
            [
                shadcn::Label::new("Email").into_element(cx),
                shadcn::Input::new(self.email.clone())
                    .a11y_label("Email")
                    .placeholder("jane@example.com")
                    .submit_command(act::Submit.into())
                    .test_id(TEST_ID_EMAIL)
                    .into_element(cx),
            ]
        })
        .gap(Space::N1)
        .into_element(cx);

        let error_row = match error {
            Some(msg) => shadcn::Alert::new([
                shadcn::AlertTitle::new("Validation error").into_element(cx),
                shadcn::AlertDescription::new(msg).into_element(cx),
            ])
            .ui()
            .test_id(TEST_ID_ERROR)
            .into_element(cx),
            None => shadcn::Alert::new([
                shadcn::AlertTitle::new("OK").into_element(cx),
                shadcn::AlertDescription::new("Ready to submit.").into_element(cx),
            ])
            .ui()
            .test_id(TEST_ID_ERROR)
            .into_element(cx),
        };

        let valid = shadcn::Badge::new(if can_submit { "Valid" } else { "Invalid" })
            .variant(if can_submit {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Destructive
            })
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::ProgressBar)
                    .test_id(TEST_ID_VALID)
                    .numeric_value(if can_submit { 1.0 } else { 0.0 })
                    .numeric_range(0.0, 1.0),
            );

        let buttons = ui::h_flex(|cx| {
            [
                shadcn::Button::new("Submit")
                    .variant(shadcn::ButtonVariant::Default)
                    .action(act::Submit)
                    .disabled(!can_submit)
                    .test_id(TEST_ID_SUBMIT)
                    .into_element(cx),
                shadcn::Button::new("Reset")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::Reset)
                    .test_id(TEST_ID_RESET)
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let body = ui::v_flex(|_cx| [name_input, email_input, error_row, valid, buttons])
            .gap(Space::N3)
            .into_element(cx);

        let card = shadcn::Card::new([header, shadcn::CardContent::new([body]).into_element(cx)])
            .ui()
            .w_full()
            .max_w(Px(640.0))
            .into_element(cx);

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-form-basics")
        .window("cookbook-form-basics", (720.0, 520.0))
        .config_files(false)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<FormBasicsView>()
        .map_err(anyhow::Error::from)
}
