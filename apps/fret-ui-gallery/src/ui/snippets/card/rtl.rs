pub const SOURCE: &str = include_str!("rtl.rs");

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
                shadcn::Label::new("البريد الإلكتروني").into_element(cx),
                shadcn::Input::new(email)
                    .a11y_label("البريد الإلكتروني")
                    .placeholder("m@example.com")
                    .test_id("ui-gallery-card-rtl-email-input")
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
                        shadcn::Label::new("كلمة المرور").into_element(cx),
                        supporting_link(
                            cx,
                            "نسيت كلمة المرور؟",
                            LayoutRefinement::default().mr_auto(),
                            "ui-gallery-card-rtl-forgot-password",
                        ),
                    ]
                })
                .items_center()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx),
                shadcn::Input::new(password)
                    .a11y_label("كلمة المرور")
                    .test_id("ui-gallery-card-rtl-password-input")
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

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("تسجيل الدخول إلى حسابك")
                            .test_id("ui-gallery-card-rtl-title"),
                        shadcn::card_description(
                            "أدخل بريدك الإلكتروني أدناه لتسجيل الدخول إلى حسابك",
                        )
                        .test_id("ui-gallery-card-rtl-description"),
                        shadcn::card_action(|cx| {
                            ui::children![
                                cx;
                                shadcn::Button::new("إنشاء حساب")
                                    .variant(shadcn::ButtonVariant::Link)
                                    .ui()
                                    .test_id("ui-gallery-card-rtl-sign-up"),
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
                        shadcn::Button::new("تسجيل الدخول")
                            .refine_layout(LayoutRefinement::default().w_full())
                            .test_id("ui-gallery-card-rtl-login"),
                        shadcn::Button::new("تسجيل الدخول باستخدام Google")
                            .variant(shadcn::ButtonVariant::Outline)
                            .refine_layout(LayoutRefinement::default().w_full())
                            .test_id("ui-gallery-card-rtl-login-with-google"),
                    ]
                })
                .direction(shadcn::CardFooterDirection::Column)
                .gap(Space::N2),
            ]
        })
        .refine_layout(max_w_sm)
        .into_element(cx)
    })
    .test_id("ui-gallery-card-rtl")
}
// endregion: example
