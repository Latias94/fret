// region: example
use fret_app::App;
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct TabsModels {
    name: Option<Model<String>>,
    username: Option<Model<String>>,
    current_password: Option<Model<String>>,
    new_password: Option<Model<String>>,
}

fn ensure_models(
    cx: &mut ElementContext<'_, App>,
) -> (Model<String>, Model<String>, Model<String>, Model<String>) {
    let state = cx.with_state(TabsModels::default, |st| st.clone());

    let name = match state.name {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert("Pedro Duarte".to_string());
            cx.with_state(TabsModels::default, |st| st.name = Some(model.clone()));
            model
        }
    };
    let username = match state.username {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert("@peduarte".to_string());
            cx.with_state(TabsModels::default, |st| st.username = Some(model.clone()));
            model
        }
    };
    let current_password = match state.current_password {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(TabsModels::default, |st| {
                st.current_password = Some(model.clone())
            });
            model
        }
    };
    let new_password = match state.new_password {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(TabsModels::default, |st| st.new_password = Some(model.clone()));
            model
        }
    };

    (name, username, current_password, new_password)
}

fn field(
    cx: &mut ElementContext<'_, App>,
    label: &'static str,
    model: Model<String>,
    a11y: &'static str,
) -> AnyElement {
    let input = shadcn::Input::new(model)
        .a11y_label(a11y)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| vec![shadcn::Label::new(label).into_element(cx), input],
    )
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    let (name, username, current_password, new_password) = ensure_models(cx);

    let account_card = {
        let header = shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Account").into_element(cx),
            shadcn::CardDescription::new(
                "Make changes to your account here. Click save when you're done.",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |cx| vec![
                field(cx, "Name", name.clone(), "Name"),
                field(cx, "Username", username.clone(), "Username"),
            ],
        );
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

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |cx| vec![
                field(
                    cx,
                    "Current password",
                    current_password.clone(),
                    "Current password",
                ),
                field(cx, "New password", new_password.clone(), "New password"),
            ],
        );
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
            shadcn::TabsItem::new("account", "Account", [account_card]),
            shadcn::TabsItem::new("password", "Password", [password_card]),
        ])
        .into_element(cx)
        .test_id("ui-gallery-tabs-demo")
}

// endregion: example

