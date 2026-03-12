pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::UiCx;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn email_field(email: Model<String>) -> impl IntoUiElement<fret_app::App> + use<> {
    ui::v_stack(|cx| {
        vec![
            shadcn::Label::new("Email").into_element(cx),
            shadcn::Input::new(email)
                .a11y_label("Email")
                .placeholder("m@example.com")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
}

fn password_field(password: Model<String>) -> impl IntoUiElement<fret_app::App> + use<> {
    ui::v_stack(|cx| {
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
            shadcn::Input::new(password)
                .a11y_label("Password")
                .placeholder("••••••••")
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
}

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let email = cx.local_model_keyed("email", String::new);
    let password = cx.local_model_keyed("password", String::new);

    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    shadcn::Card::build(|cx, out| {
        out.extend([
            shadcn::card_header(cx, |cx| {
                vec![
                    shadcn::card_title(cx, "Login to your account")
                        .test_id("ui-gallery-card-demo-title"),
                    shadcn::card_description(cx, "Enter your email below to login to your account")
                        .test_id("ui-gallery-card-demo-description"),
                    shadcn::card_action(cx, |cx| {
                        vec![
                            shadcn::Button::new("Sign Up")
                                .variant(shadcn::ButtonVariant::Link)
                                .size(shadcn::ButtonSize::Sm)
                                .into_element(cx)
                                .test_id("ui-gallery-card-demo-sign-up"),
                        ]
                    }),
                ]
            }),
            shadcn::card_content(cx, |cx| {
                vec![
                    ui::v_flex(|cx| {
                        vec![
                            email_field(email.clone()).into_element(cx),
                            password_field(password.clone()).into_element(cx),
                        ]
                    })
                    .gap(Space::N6)
                    .layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                ]
            }),
            shadcn::CardFooter::build(|cx, out| {
                out.extend([
                    shadcn::Button::new("Login")
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx)
                        .test_id("ui-gallery-card-demo-login"),
                    shadcn::Button::new("Login with Google")
                        .variant(shadcn::ButtonVariant::Outline)
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx)
                        .test_id("ui-gallery-card-demo-login-google"),
                ]);
            })
            .direction(shadcn::CardFooterDirection::Column)
            .gap(Space::N2)
            .into_element(cx),
        ]);
    })
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-card-demo")
}
// endregion: example
