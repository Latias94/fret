pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn field(
    label: &'static str,
    model: Model<String>,
    a11y: &'static str,
    password: bool,
) -> impl IntoUiElement<fret_app::App> + use<> {
    let mut input = shadcn::Input::new(model)
        .a11y_label(a11y)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0());
    if password {
        input = input.password();
    }

    ui::v_flex(move |cx| ui::children![cx; shadcn::Label::new(label), input])
        .gap(Space::N2)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let name = cx.local_model_keyed("name", || "Pedro Duarte".to_string());
    let username = cx.local_model_keyed("username", || "@peduarte".to_string());
    let current_password = cx.local_model_keyed("current_password", String::new);
    let new_password = cx.local_model_keyed("new_password", String::new);

    let account_card = {
        let content = ui::v_flex(move |cx| {
            ui::children![
                cx;
                field("Name", name.clone(), "Name", false),
                field("Username", username.clone(), "Username", false)
            ]
        })
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0());
        shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("Account"),
                        shadcn::card_description(
                            "Make changes to your account here. Click save when you're done.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| ui::children![cx; content]),
                shadcn::card_footer(|cx| {
                    ui::children![cx; shadcn::Button::new("Save changes")]
                }),
            ]
        })
        .into_element(cx)
    };

    let password_card = {
        let content = ui::v_flex(move |cx| {
            ui::children![
                cx;
                field(
                    "Current password",
                    current_password.clone(),
                    "Current password",
                    true,
                ),
                field("New password", new_password.clone(), "New password", true)
            ]
        })
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0());
        shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("Password"),
                        shadcn::card_description(
                            "Change your password here. After saving, you'll be logged out.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| ui::children![cx; content]),
                shadcn::card_footer(|cx| {
                    ui::children![cx; shadcn::Button::new("Save password")]
                }),
            ]
        })
        .into_element(cx)
    };

    shadcn::tabs_uncontrolled(cx, Some("account"), |_cx| {
        [
            shadcn::TabsItem::new("account", "Account", [account_card])
                .trigger_test_id("ui-gallery-tabs-demo-trigger-account"),
            shadcn::TabsItem::new("password", "Password", [password_card])
                .trigger_test_id("ui-gallery-tabs-demo-trigger-password"),
        ]
    })
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(384.0))
            .min_w_0(),
    )
    .into_element(cx)
    .test_id("ui-gallery-tabs-demo")
}

// endregion: example
