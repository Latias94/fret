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

struct FormBasicsView;

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

    fn validate_in(
        models: &mut fret_runtime::ModelStore,
        name: &LocalState<String>,
        email: &LocalState<String>,
    ) -> Option<String> {
        let name = name.read_in(models, Clone::clone).ok().unwrap_or_default();
        let email = email.read_in(models, Clone::clone).ok().unwrap_or_default();
        Self::validate(&name, &email)
    }
}

impl View for FormBasicsView {
    fn init(_app: &mut App, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let name_state = cx.use_local::<String>();
        let email_state = cx.use_local::<String>();
        let error_state = cx.use_local::<Option<String>>();

        let name = name_state.watch(cx).layout().value_or_else(String::new);
        let email = email_state.watch(cx).layout().value_or_else(String::new);
        let error = error_state.watch(cx).layout().value_or_default();

        let can_submit = FormBasicsView::validate(&name, &email).is_none();

        cx.on_action_notify_models::<act::Submit>({
            let name_state = name_state.clone();
            let email_state = email_state.clone();
            let error_state = error_state.clone();
            move |models| {
                let err = FormBasicsView::validate_in(models, &name_state, &email_state);
                error_state.set_in(models, err)
            }
        });

        cx.on_action_notify_models::<act::Reset>({
            let name_state = name_state.clone();
            let email_state = email_state.clone();
            let error_state = error_state.clone();
            move |models| {
                let ok = name_state.set_in(models, String::new());
                let ok = email_state.set_in(models, String::new()) && ok;
                error_state.set_in(models, None) && ok
            }
        });

        cx.on_action_availability::<act::Submit>({
            let name_state = name_state.clone();
            let email_state = email_state.clone();
            move |host, _acx| {
                if FormBasicsView::validate_in(host.models_mut(), &name_state, &email_state)
                    .is_none()
                {
                    CommandAvailability::Available
                } else {
                    CommandAvailability::Blocked
                }
            }
        });
        cx.on_action_availability::<act::Reset>(|_host, _acx| CommandAvailability::Available);

        let name_input = ui::v_flex(|cx| {
            ui::children![cx;
                shadcn::Label::new("Name"),
                shadcn::Input::new(&name_state)
                    .a11y_label("Name")
                    .placeholder("Jane Doe")
                    .test_id(TEST_ID_NAME),
            ]
        })
        .gap(Space::N1);

        let email_input = ui::v_flex(|cx| {
            ui::children![cx;
                shadcn::Label::new("Email"),
                shadcn::Input::new(&email_state)
                    .a11y_label("Email")
                    .placeholder("jane@example.com")
                    .submit_command(act::Submit.into())
                    .test_id(TEST_ID_EMAIL),
            ]
        })
        .gap(Space::N1);

        let (error_title, error_description) = match error {
            Some(msg) => ("Validation error", msg),
            None => ("OK", "Ready to submit.".to_string()),
        };

        let error_row = shadcn::Alert::build(|cx, out| {
            out.push_ui(cx, shadcn::AlertTitle::new(error_title));
            out.push_ui(cx, shadcn::AlertDescription::new(error_description));
        })
        .ui()
        .test_id(TEST_ID_ERROR);

        let valid = shadcn::Badge::new(if can_submit { "Valid" } else { "Invalid" })
            .variant(if can_submit {
                shadcn::BadgeVariant::Default
            } else {
                shadcn::BadgeVariant::Destructive
            })
            .ui()
            .semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::ProgressBar)
                    .test_id(TEST_ID_VALID)
                    .numeric_value(if can_submit { 1.0 } else { 0.0 })
                    .numeric_range(0.0, 1.0),
            );

        let buttons = ui::h_flex(|cx| {
            ui::children![cx;
                shadcn::Button::new("Submit")
                    .variant(shadcn::ButtonVariant::Default)
                    .action(act::Submit)
                    .disabled(!can_submit)
                    .test_id(TEST_ID_SUBMIT),
                shadcn::Button::new("Reset")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::Reset)
                    .test_id(TEST_ID_RESET),
            ]
        })
        .gap(Space::N2)
        .items_center();

        let body =
            ui::v_flex(|cx| ui::children![cx; name_input, email_input, error_row, valid, buttons])
                .gap(Space::N3);

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Form basics"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new(
                            "A minimal form with validation (no extra form registry dependency).",
                        ),
                    );
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(cx, body);
                }),
            );
        })
        .ui()
        .w_full()
        .max_w(Px(640.0));

        fret_cookbook::scaffold::centered_page_muted_ui(cx, TEST_ID_ROOT, card).into()
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
