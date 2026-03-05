pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    email: Option<Model<String>>,
    password: Option<Model<String>>,
}

fn ensure_models(cx: &mut ElementContext<'_, App>) -> (Model<String>, Model<String>) {
    let state = cx.with_state(Models::default, |st| st.clone());

    let email = match state.email {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.email = Some(model.clone()));
            model
        }
    };

    let password = match state.password {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.password = Some(model.clone()));
            model
        }
    };

    (email, password)
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let (email, password) = ensure_models(cx);

    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Login to your account")
                .into_element(cx)
                .test_id("ui-gallery-card-demo-title"),
            shadcn::CardDescription::new("Enter your email below to login to your account")
                .into_element(cx)
                .test_id("ui-gallery-card-demo-description"),
            shadcn::CardAction::new([shadcn::Button::new("Sign Up")
                .variant(shadcn::ButtonVariant::Link)
                .size(shadcn::ButtonSize::Sm)
                .into_element(cx)
                .test_id("ui-gallery-card-demo-sign-up")])
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![
            ui::v_flex(|cx| {
                let email = ui::v_stack(|cx| {
                    vec![
                        shadcn::Label::new("Email").into_element(cx),
                        shadcn::Input::new(email.clone())
                            .a11y_label("Email")
                            .placeholder("m@example.com")
                            .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .into_element(cx);

                let password = ui::v_stack(|cx| {
                    vec![
                        ui::h_flex(|cx| {
                            vec![
                                shadcn::Label::new("Password").into_element(cx),
                                shadcn::Button::new("Forgot your password?")
                                    .variant(shadcn::ButtonVariant::Link)
                                    .size(shadcn::ButtonSize::Sm)
                                    .into_element(cx),
                            ]
                        })
                        .layout(LayoutRefinement::default().w_full())
                        .justify_between()
                        .items_center()
                        .into_element(cx),
                        shadcn::Input::new(password.clone())
                            .a11y_label("Password")
                            .placeholder("••••••••")
                            .into_element(cx),
                    ]
                })
                .gap(Space::N2)
                .into_element(cx);

                vec![email, password]
            })
            .gap(Space::N6)
            .layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardFooter::new(vec![
            shadcn::Button::new("Login")
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
            shadcn::Button::new("Login with Google")
                .variant(shadcn::ButtonVariant::Outline)
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
        ])
        .direction(shadcn::CardFooterDirection::Column)
        .gap(Space::N2)
        .into_element(cx),
    ])
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-card-demo")
}
// endregion: example
