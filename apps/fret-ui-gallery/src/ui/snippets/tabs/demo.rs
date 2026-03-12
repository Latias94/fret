pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::UiCx;
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

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

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let name = cx.local_model_keyed("name", || "Pedro Duarte".to_string());
    let username = cx.local_model_keyed("username", || "@peduarte".to_string());
    let current_password = cx.local_model_keyed("current_password", String::new);
    let new_password = cx.local_model_keyed("new_password", String::new);

    let account_card = {
        let header = shadcn::CardHeader::new(ui::children![
            cx;
            shadcn::CardTitle::new("Account"),
            shadcn::CardDescription::new(
                "Make changes to your account here. Click save when you're done.",
            )
        ]);

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
        let content = shadcn::CardContent::new(ui::children![cx; content]);
        let footer =
            shadcn::CardFooter::new(ui::children![cx; shadcn::Button::new("Save changes")]);
        shadcn::Card::new(ui::children![cx; header, content, footer]).into_element(cx)
    };

    let password_card = {
        let header = shadcn::CardHeader::new(ui::children![
            cx;
            shadcn::CardTitle::new("Password"),
            shadcn::CardDescription::new(
                "Change your password here. After saving, you'll be logged out.",
            )
        ]);

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
        let content = shadcn::CardContent::new(ui::children![cx; content]);
        let footer =
            shadcn::CardFooter::new(ui::children![cx; shadcn::Button::new("Save password")]);
        shadcn::Card::new(ui::children![cx; header, content, footer]).into_element(cx)
    };

    shadcn::Tabs::uncontrolled(Some(Arc::<str>::from("account")))
        .list_full_width(true)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .items([
            shadcn::TabsItem::new("account", "Account", [account_card])
                .trigger_test_id("ui-gallery-tabs-demo-trigger-account"),
            shadcn::TabsItem::new("password", "Password", [password_card])
                .trigger_test_id("ui-gallery-tabs-demo-trigger-password"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-tabs-demo")
}

// endregion: example
