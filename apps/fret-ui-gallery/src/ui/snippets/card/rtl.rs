pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_app::App;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

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

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Card::build(|cx, out| {
            out.extend([
                shadcn::card_header(cx, |cx| {
                    vec![
                        shadcn::card_title(cx, "تسجيل الدخول إلى حسابك"),
                        shadcn::card_description(
                            cx,
                            "أدخل بريدك الإلكتروني أدناه لتسجيل الدخول إلى حسابك",
                        ),
                        shadcn::card_action(cx, |cx| {
                            vec![
                                shadcn::Button::new("إنشاء حساب")
                                    .variant(shadcn::ButtonVariant::Link)
                                    .into_element(cx),
                            ]
                        }),
                    ]
                }),
                shadcn::card_content(cx, |cx| {
                    vec![
                        ui::v_flex(|cx| {
                            let email = ui::v_stack(|cx| {
                                vec![
                                    shadcn::Label::new("البريد الإلكتروني").into_element(cx),
                                    shadcn::Input::new(email.clone())
                                        .a11y_label("البريد الإلكتروني")
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
                                            shadcn::Label::new("كلمة المرور").into_element(cx),
                                            shadcn::Button::new("نسيت كلمة المرور؟")
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
                                        .a11y_label("كلمة المرور")
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
                    ]
                }),
                shadcn::CardFooter::build(|cx, out| {
                    out.extend([
                        shadcn::Button::new("تسجيل الدخول")
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx)
                            .test_id("ui-gallery-card-rtl-login"),
                        shadcn::Button::new("تسجيل الدخول باستخدام Google")
                            .variant(shadcn::ButtonVariant::Outline)
                            .refine_layout(LayoutRefinement::default().w_full())
                            .into_element(cx)
                            .test_id("ui-gallery-card-rtl-login-with-google"),
                    ]);
                })
                .direction(shadcn::CardFooterDirection::Column)
                .gap(Space::N2)
                .into_element(cx),
            ]);
        })
        .refine_layout(max_w_sm)
        .into_element(cx)
    })
    .test_id("ui-gallery-card-rtl")
}
// endregion: example
