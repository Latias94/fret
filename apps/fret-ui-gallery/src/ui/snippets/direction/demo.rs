pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let email = cx.local_model_keyed("direction_demo_email", String::new);
    let password = cx.local_model_keyed("direction_demo_password", String::new);
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    shadcn::DirectionProvider::new(shadcn::LayoutDirection::Rtl).into_element(cx, move |cx| {
        ui::v_flex(|cx| {
            vec![
                shadcn::card(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_header(|cx| {
                            ui::children![
                                cx;
                                shadcn::card_title("تسجيل الدخول إلى حسابك"),
                                shadcn::card_description(
                                    "أدخل بريدك الإلكتروني أدناه لتسجيل الدخول إلى حسابك",
                                ),
                                shadcn::card_action(|cx| {
                                    ui::children![
                                        cx;
                                        shadcn::Button::new("إنشاء حساب")
                                            .variant(shadcn::ButtonVariant::Link),
                                    ]
                                }),
                            ]
                        }),
                        shadcn::card_content(|cx| {
                            ui::children![
                                cx;
                                ui::v_flex(|cx| {
                                    let email = ui::v_stack(|cx| {
                                        vec![
                                            shadcn::Label::new("البريد الإلكتروني")
                                                .into_element(cx),
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
                                                    shadcn::Label::new("كلمة المرور")
                                                        .into_element(cx),
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
                                .layout(LayoutRefinement::default().w_full()),
                            ]
                        }),
                        shadcn::card_footer(|cx| {
                            ui::children![
                                cx;
                                shadcn::Button::new("تسجيل الدخول")
                                    .refine_layout(LayoutRefinement::default().w_full())
                                    .ui()
                                    .test_id("ui-gallery-direction-demo-login"),
                                shadcn::Button::new("تسجيل الدخول باستخدام Google")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .refine_layout(LayoutRefinement::default().w_full())
                                    .ui()
                                    .test_id("ui-gallery-direction-demo-login-google"),
                            ]
                        })
                        .direction(shadcn::CardFooterDirection::Column)
                        .gap(Space::N2),
                    ]
                })
                .refine_layout(max_w_sm.clone())
                .into_element(cx)
                .test_id("ui-gallery-direction-demo-card"),
            ]
        })
        .justify_center()
        .layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id("ui-gallery-direction-demo")
    })
}
// endregion: example
