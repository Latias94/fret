use super::super::super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(in crate::ui) fn preview_tabs(
    cx: &mut ElementContext<'_, App>,
    _value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    #[derive(Default, Clone)]
    struct TabsModels {
        name: Option<Model<String>>,
        username: Option<Model<String>>,
        current_password: Option<Model<String>>,
        new_password: Option<Model<String>>,
    }

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
            cx.with_state(TabsModels::default, |st| {
                st.new_password = Some(model.clone())
            });
            model
        }
    };

    let primary = cx.with_theme(|theme| theme.color_token("primary"));
    let line_style = shadcn::tabs::TabsStyle::default()
        .trigger_background(fret_ui_kit::WidgetStateProperty::new(Some(
            ColorRef::Color(CoreColor::TRANSPARENT),
        )))
        .trigger_border_color(
            fret_ui_kit::WidgetStateProperty::new(Some(ColorRef::Color(CoreColor::TRANSPARENT)))
                .when(
                    fret_ui_kit::WidgetStates::SELECTED,
                    Some(ColorRef::Color(primary)),
                ),
        );

    let demo = {
        let field = |cx: &mut ElementContext<'_, App>,
                     label: &'static str,
                     model: Model<String>,
                     a11y: &'static str| {
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
        };

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
                move |cx| {
                    vec![
                        field(cx, "Name", name.clone(), "Name"),
                        field(cx, "Username", username.clone(), "Username"),
                    ]
                },
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
                move |cx| {
                    vec![
                        field(
                            cx,
                            "Current password",
                            current_password.clone(),
                            "Current password",
                        ),
                        field(cx, "New password", new_password.clone(), "New password"),
                    ]
                },
            );
            let content = shadcn::CardContent::new(vec![content]).into_element(cx);
            let footer = shadcn::CardFooter::new(vec![
                shadcn::Button::new("Save password").into_element(cx),
            ])
            .into_element(cx);
            shadcn::Card::new(vec![header, content, footer]).into_element(cx)
        };

        shadcn::Tabs::uncontrolled(Some("account"))
            .list_full_width(true)
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .items([
                shadcn::TabsItem::new("account", "Account", [account_card]),
                shadcn::TabsItem::new("password", "Password", [password_card]),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-demo")
    };

    let list = shadcn::Tabs::uncontrolled(Some("home"))
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .items([
            shadcn::TabsItem::new("home", "Home", Vec::<AnyElement>::new()),
            shadcn::TabsItem::new("settings", "Settings", Vec::<AnyElement>::new()),
        ])
        .into_element(cx)
        .test_id("ui-gallery-tabs-list");

    let disabled = shadcn::Tabs::uncontrolled(Some("home"))
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .items([
            shadcn::TabsItem::new("home", "Home", Vec::<AnyElement>::new()),
            shadcn::TabsItem::new("settings", "Settings", Vec::<AnyElement>::new()).disabled(true),
        ])
        .into_element(cx)
        .test_id("ui-gallery-tabs-disabled");

    let icons = shadcn::Tabs::uncontrolled(Some("preview"))
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .items([
            shadcn::TabsItem::new("preview", "Preview", Vec::<AnyElement>::new())
                .trigger_leading_icon(IconId::new_static("lucide.app-window")),
            shadcn::TabsItem::new("code", "Code", Vec::<AnyElement>::new())
                .trigger_leading_icon(IconId::new_static("lucide.code")),
        ])
        .into_element(cx)
        .test_id("ui-gallery-tabs-icons");

    let line = shadcn::Tabs::uncontrolled(Some("preview"))
        .style(line_style.clone())
        .refine_style(ChromeRefinement::default().bg(ColorRef::Color(CoreColor::TRANSPARENT)))
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .items([
            shadcn::TabsItem::new("preview", "Preview", Vec::<AnyElement>::new())
                .trigger_leading_icon(IconId::new_static("lucide.app-window")),
            shadcn::TabsItem::new("code", "Code", Vec::<AnyElement>::new())
                .trigger_leading_icon(IconId::new_static("lucide.code")),
        ])
        .into_element(cx)
        .test_id("ui-gallery-tabs-line");

    let vertical = shadcn::Tabs::uncontrolled(Some("preview"))
        .orientation(shadcn::tabs::TabsOrientation::Vertical)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .items([
            shadcn::TabsItem::new("preview", "Preview", Vec::<AnyElement>::new())
                .trigger_leading_icon(IconId::new_static("lucide.app-window")),
            shadcn::TabsItem::new("code", "Code", Vec::<AnyElement>::new())
                .trigger_leading_icon(IconId::new_static("lucide.code")),
        ])
        .into_element(cx)
        .test_id("ui-gallery-tabs-vertical");

    let vertical_line = shadcn::Tabs::uncontrolled(Some("preview"))
        .orientation(shadcn::tabs::TabsOrientation::Vertical)
        .style(line_style.clone())
        .refine_style(ChromeRefinement::default().bg(ColorRef::Color(CoreColor::TRANSPARENT)))
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .items([
            shadcn::TabsItem::new("preview", "Preview", Vec::<AnyElement>::new())
                .trigger_leading_icon(IconId::new_static("lucide.app-window")),
            shadcn::TabsItem::new("code", "Code", Vec::<AnyElement>::new())
                .trigger_leading_icon(IconId::new_static("lucide.code")),
        ])
        .into_element(cx)
        .test_id("ui-gallery-tabs-vertical-line");

    let extras = {
        let muted = shadcn::typography::muted(
            cx,
            "Extras are Fret-specific regression gates (not part of upstream shadcn TabsDemo).",
        );

        let flex_1_triggers = shadcn::Tabs::uncontrolled(Some("overview"))
            .list_full_width(true)
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
            .items([
                shadcn::TabsItem::new("overview", "Overview", Vec::<AnyElement>::new())
                    .trigger_test_id("ui-gallery-tabs-flex1-trigger-overview"),
                shadcn::TabsItem::new("analytics", "Analytics", Vec::<AnyElement>::new())
                    .trigger_test_id("ui-gallery-tabs-flex1-trigger-analytics"),
                shadcn::TabsItem::new("reports", "Reports", Vec::<AnyElement>::new())
                    .trigger_test_id("ui-gallery-tabs-flex1-trigger-reports"),
            ])
            .into_element(cx)
            .test_id("ui-gallery-tabs-flex1");

        let rtl = doc_layout::rtl(cx, |cx| {
            shadcn::Tabs::uncontrolled(Some("preview"))
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
                .items([
                    shadcn::TabsItem::new("preview", "Preview", Vec::<AnyElement>::new()),
                    shadcn::TabsItem::new("code", "Code", Vec::<AnyElement>::new()),
                ])
                .into_element(cx)
        })
        .test_id("ui-gallery-tabs-rtl");

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |_cx| vec![muted, flex_1_triggers, rtl],
        )
        .test_id("ui-gallery-tabs-extras")
    };

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows `tabs-demo.tsx` (new-york-v4) order: Demo, list-only, disabled, icons, line, vertical, vertical line.",
            "Fret shadcn `Input` does not implement a password-masked input yet; password fields here are plain text (parity gap).",
            "API reference: `ecosystem/fret-ui-shadcn/src/tabs.rs`.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("A set of layered sections of content that are displayed one at a time."),
        vec![
            DocSection::new("Demo", demo)
                .description("Account/password card example with inputs and footer actions.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::Tabs::uncontrolled(Some("account"))
    .list_full_width(true)
    .items([
        shadcn::TabsItem::new("account", "Account", [account_card]),
        shadcn::TabsItem::new("password", "Password", [password_card]),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("List", list)
                .description("Tabs list without any mounted content.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::Tabs::uncontrolled(Some("home")).items([
    shadcn::TabsItem::new("home", "Home", Vec::<AnyElement>::new()),
    shadcn::TabsItem::new("settings", "Settings", Vec::<AnyElement>::new()),
]);"#,
                ),
            DocSection::new("Disabled", disabled)
                .description("Disable individual triggers.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::TabsItem::new("settings", "Settings", Vec::<AnyElement>::new())
    .disabled(true)"#,
                ),
            DocSection::new("Icons", icons)
                .description("Compose icons into triggers.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::Tabs::uncontrolled(Some("preview")).items([
    shadcn::TabsItem::new("preview", "Preview", Vec::<AnyElement>::new())
        .trigger_leading_icon(IconId::new_static("lucide.app-window")),
    shadcn::TabsItem::new("code", "Code", Vec::<AnyElement>::new())
        .trigger_leading_icon(IconId::new_static("lucide.code")),
]);"#,
                ),
            DocSection::new("Line", line)
                .description("Line-style list with transparent background.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::Tabs::uncontrolled(Some("preview"))
    .style(line_style)
    .refine_style(ChromeRefinement::default().bg(ColorRef::Color(CoreColor::TRANSPARENT)))"#,
                ),
            DocSection::new("Vertical", vertical)
                .description("Vertical orientation (Radix parity).")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::Tabs::uncontrolled(Some("preview"))
    .orientation(shadcn::tabs::TabsOrientation::Vertical)"#,
                ),
            DocSection::new("Vertical (Line)", vertical_line)
                .description("Vertical + line style.")
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"shadcn::Tabs::uncontrolled(Some("preview"))
    .orientation(shadcn::tabs::TabsOrientation::Vertical)
    .style(line_style)"#,
                ),
            DocSection::new("Extras", extras)
                .description("Fret-specific regression gates (flex-1 triggers + RTL).")
                .max_w(Px(980.0))
                .code(
                    "rust",
                    r#"shadcn::Tabs::uncontrolled(Some("overview"))
    .list_full_width(true)
    .items([/* ... */])
    .into_element(cx);"#,
                ),
            DocSection::new("Notes", notes).description("Parity notes and references."),
        ],
    );

    vec![body.test_id("ui-gallery-tabs")]
}
