pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::UiCx;
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn field(
    cx: &mut UiCx<'_>,
    label: &'static str,
    model: Model<String>,
    a11y: &'static str,
    password: bool,
) -> AnyElement {
    let mut input = shadcn::Input::new(model)
        .a11y_label(a11y)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0());
    if password {
        input = input.password();
    }
    let input = input.into_element(cx);
    ui::v_flex(move |cx| vec![shadcn::Label::new(label).into_element(cx), input])
        .gap(Space::N2)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
}

pub fn render(cx: &mut UiCx<'_>) -> AnyElement {
    let name = cx.local_model_keyed("name", || "Pedro Duarte".to_string());
    let username = cx.local_model_keyed("username", || "@peduarte".to_string());
    let current_password = cx.local_model_keyed("current_password", String::new);
    let new_password = cx.local_model_keyed("new_password", String::new);

    let account_card = {
        let header = shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Account").into_element(cx),
            shadcn::CardDescription::new(
                "Make changes to your account here. Click save when you're done.",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let content = ui::v_flex(move |cx| {
            vec![
                field(cx, "Name", name.clone(), "Name", false),
                field(cx, "Username", username.clone(), "Username", false),
            ]
        })
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);
        let content = shadcn::CardContent::new(vec![content]).into_element(cx);
        let footer =
            shadcn::CardFooter::new(vec![shadcn::Button::new("Save changes").into_element(cx)])
                .into_element(cx);
        shadcn::Card::new(vec![header, content, footer]).into_element(cx)
    };

    let password_card = {
        let header = shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Password").into_element(cx),
            shadcn::CardDescription::new(
                "Change your password here. After saving, you'll be logged out.",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let content = ui::v_flex(move |cx| {
            vec![
                field(
                    cx,
                    "Current password",
                    current_password.clone(),
                    "Current password",
                    true,
                ),
                field(
                    cx,
                    "New password",
                    new_password.clone(),
                    "New password",
                    true,
                ),
            ]
        })
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);
        let content = shadcn::CardContent::new(vec![content]).into_element(cx);
        let footer =
            shadcn::CardFooter::new(vec![shadcn::Button::new("Save password").into_element(cx)])
                .into_element(cx);
        shadcn::Card::new(vec![header, content, footer]).into_element(cx)
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
