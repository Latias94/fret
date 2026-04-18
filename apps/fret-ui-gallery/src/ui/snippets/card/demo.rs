pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::SemanticsRole;
use fret_ui::Theme;
use fret_ui::element::{AnyElement, CrossAlign, GridProps, PressableKeyActivation, PressableProps};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{IntoUiElement, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn supporting_link(
    cx: &mut AppComponentCx<'_>,
    label: &'static str,
    layout: LayoutRefinement,
    test_id: &'static str,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).snapshot();
    let container = decl_style::container_props(&theme, ChromeRefinement::default(), layout);
    let mut props = PressableProps::default();
    props.key_activation = PressableKeyActivation::EnterOnly;
    props.a11y.role = Some(SemanticsRole::Link);
    props.a11y.label = Some(label.into());

    let pressable = cx
        .pressable(props, move |cx, _state| {
            cx.pressable_on_activate(Arc::new(|host, action_cx, _reason| {
                host.request_redraw(action_cx.window);
            }));

            vec![ui::text(label).text_sm().nowrap().into_element(cx)]
        })
        .test_id(format!("{test_id}.chrome"));

    cx.container(container, move |_cx| vec![pressable])
        .test_id(test_id)
}

fn email_field(
    cx: &mut AppComponentCx<'_>,
    email: Model<String>,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let theme = Theme::global(&*cx.app).snapshot();
    let gap = MetricRef::space(Space::N2).resolve(&theme);
    let layout = decl_style::layout_style(&theme, LayoutRefinement::default().w_full().min_w_0());

    cx.grid(
        GridProps {
            layout,
            cols: 1,
            gap: gap.into(),
            align: CrossAlign::Start,
            ..Default::default()
        },
        |cx| {
            vec![
                shadcn::Label::new("Email").into_element(cx),
                shadcn::Input::new(email)
                    .a11y_label("Email")
                    .placeholder("m@example.com")
                    .test_id("ui-gallery-card-demo-email-input")
                    .into_element(cx),
            ]
        },
    )
}

fn password_field(
    cx: &mut AppComponentCx<'_>,
    password: Model<String>,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let theme = Theme::global(&*cx.app).snapshot();
    let gap = MetricRef::space(Space::N2).resolve(&theme);
    let layout = decl_style::layout_style(&theme, LayoutRefinement::default().w_full().min_w_0());

    cx.grid(
        GridProps {
            layout,
            cols: 1,
            gap: gap.into(),
            align: CrossAlign::Start,
            ..Default::default()
        },
        |cx| {
            vec![
                ui::h_row(|cx| {
                    vec![
                        shadcn::Label::new("Password").into_element(cx),
                        supporting_link(
                            cx,
                            "Forgot your password?",
                            LayoutRefinement::default().ml_auto(),
                            "ui-gallery-card-demo-forgot-password",
                        ),
                    ]
                })
                .items_center()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx),
                shadcn::Input::new(password)
                    .a11y_label("Password")
                    .test_id("ui-gallery-card-demo-password-input")
                    .into_element(cx),
            ]
        },
    )
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let email = cx.local_model_keyed("email", String::new);
    let password = cx.local_model_keyed("password", String::new);

    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Login to your account")
                        .test_id("ui-gallery-card-demo-title"),
                    shadcn::card_description("Enter your email below to login to your account")
                        .test_id("ui-gallery-card-demo-description"),
                    shadcn::card_action(|cx| {
                        ui::children![
                            cx;
                            shadcn::Button::new("Sign Up")
                                .variant(shadcn::ButtonVariant::Link)
                                .ui()
                                .test_id("ui-gallery-card-demo-sign-up"),
                        ]
                    }),
                ]
            }),
            shadcn::card_content(|cx| {
                ui::children![
                    cx;
                    ui::v_flex(|cx| {
                        vec![
                            email_field(cx, email.clone()).into_element(cx),
                            password_field(cx, password.clone()).into_element(cx),
                        ]
                    })
                    .gap(Space::N6)
                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                ]
            }),
            shadcn::card_footer(|cx| {
                ui::children![
                    cx;
                    shadcn::Button::new("Login")
                        .refine_layout(LayoutRefinement::default().w_full())
                        .test_id("ui-gallery-card-demo-login"),
                    shadcn::Button::new("Login with Google")
                        .variant(shadcn::ButtonVariant::Outline)
                        .refine_layout(LayoutRefinement::default().w_full())
                        .test_id("ui-gallery-card-demo-login-google"),
                ]
            })
            .direction(shadcn::CardFooterDirection::Column)
            .gap(Space::N2),
        ]
    })
    .refine_layout(max_w_sm)
    .into_element(cx)
    .test_id("ui-gallery-card-demo")
}
// endregion: example
