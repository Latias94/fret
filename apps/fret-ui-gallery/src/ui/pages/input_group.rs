use super::super::*;

pub(super) fn preview_input_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct InputGroupPageModels {
        demo_value: Option<Model<String>>,
        inline_start: Option<Model<String>>,
        inline_end: Option<Model<String>>,
        block_start: Option<Model<String>>,
        block_end: Option<Model<String>>,
        icon_value: Option<Model<String>>,
        text_value: Option<Model<String>>,
        button_value: Option<Model<String>>,
        kbd_value: Option<Model<String>>,
        dropdown_value: Option<Model<String>>,
        spinner_value: Option<Model<String>>,
        textarea_value: Option<Model<String>>,
        custom_value: Option<Model<String>>,
        rtl_value: Option<Model<String>>,
    }

    let (
        demo_value,
        inline_start,
        inline_end,
        block_start,
        block_end,
        icon_value,
        text_value,
        button_value,
        kbd_value,
        dropdown_value,
        spinner_value,
        textarea_value,
        custom_value,
        rtl_value,
    ) = cx.with_state(InputGroupPageModels::default, |st| {
        (
            st.demo_value.clone(),
            st.inline_start.clone(),
            st.inline_end.clone(),
            st.block_start.clone(),
            st.block_end.clone(),
            st.icon_value.clone(),
            st.text_value.clone(),
            st.button_value.clone(),
            st.kbd_value.clone(),
            st.dropdown_value.clone(),
            st.spinner_value.clone(),
            st.textarea_value.clone(),
            st.custom_value.clone(),
            st.rtl_value.clone(),
        )
    });

    let (
        demo_value,
        inline_start,
        inline_end,
        block_start,
        block_end,
        icon_value,
        text_value,
        button_value,
        kbd_value,
        dropdown_value,
        spinner_value,
        textarea_value,
        custom_value,
        rtl_value,
    ) = match (
        demo_value,
        inline_start,
        inline_end,
        block_start,
        block_end,
        icon_value,
        text_value,
        button_value,
        kbd_value,
        dropdown_value,
        spinner_value,
        textarea_value,
        custom_value,
        rtl_value,
    ) {
        (
            Some(demo_value),
            Some(inline_start),
            Some(inline_end),
            Some(block_start),
            Some(block_end),
            Some(icon_value),
            Some(text_value),
            Some(button_value),
            Some(kbd_value),
            Some(dropdown_value),
            Some(spinner_value),
            Some(textarea_value),
            Some(custom_value),
            Some(rtl_value),
        ) => (
            demo_value,
            inline_start,
            inline_end,
            block_start,
            block_end,
            icon_value,
            text_value,
            button_value,
            kbd_value,
            dropdown_value,
            spinner_value,
            textarea_value,
            custom_value,
            rtl_value,
        ),
        _ => {
            let demo_value = cx.app.models_mut().insert(String::new());
            let inline_start = cx.app.models_mut().insert(String::new());
            let inline_end = cx.app.models_mut().insert(String::new());
            let block_start = cx.app.models_mut().insert(String::new());
            let block_end = cx.app.models_mut().insert(String::new());
            let icon_value = cx.app.models_mut().insert(String::new());
            let text_value = cx.app.models_mut().insert(String::new());
            let button_value = cx.app.models_mut().insert(String::new());
            let kbd_value = cx.app.models_mut().insert(String::new());
            let dropdown_value = cx.app.models_mut().insert(String::new());
            let spinner_value = cx.app.models_mut().insert(String::new());
            let textarea_value = cx.app.models_mut().insert(String::new());
            let custom_value = cx.app.models_mut().insert(String::new());
            let rtl_value = cx.app.models_mut().insert(String::new());

            cx.with_state(InputGroupPageModels::default, |st| {
                st.demo_value = Some(demo_value.clone());
                st.inline_start = Some(inline_start.clone());
                st.inline_end = Some(inline_end.clone());
                st.block_start = Some(block_start.clone());
                st.block_end = Some(block_end.clone());
                st.icon_value = Some(icon_value.clone());
                st.text_value = Some(text_value.clone());
                st.button_value = Some(button_value.clone());
                st.kbd_value = Some(kbd_value.clone());
                st.dropdown_value = Some(dropdown_value.clone());
                st.spinner_value = Some(spinner_value.clone());
                st.textarea_value = Some(textarea_value.clone());
                st.custom_value = Some(custom_value.clone());
                st.rtl_value = Some(rtl_value.clone());
            });

            (
                demo_value,
                inline_start,
                inline_end,
                block_start,
                block_end,
                icon_value,
                text_value,
                button_value,
                kbd_value,
                dropdown_value,
                spinner_value,
                textarea_value,
                custom_value,
                rtl_value,
            )
        }
    };

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(860.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let body = centered(cx, card);
            section(cx, title, body)
        };

    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let demo = {
        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(max_w_xs.clone()),
            |cx| {
                vec![
                    shadcn::InputGroup::new(demo_value.clone())
                        .a11y_label("Search")
                        .leading([shadcn::InputGroupText::new("icon").into_element(cx)])
                        .trailing([shadcn::InputGroupButton::new("Go")
                            .variant(shadcn::ButtonVariant::Ghost)
                            .into_element(cx)])
                        .trailing_has_button(true)
                        .test_id("ui-gallery-input-group-demo")
                        .into_element(cx),
                    shadcn::InputGroup::new(demo_value.clone())
                        .textarea()
                        .a11y_label("Message")
                        .block_end([
                            shadcn::InputGroupText::new("Ctrl+Enter to send").into_element(cx),
                            shadcn::InputGroupButton::new("Send")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::InputGroupButtonSize::Sm)
                                .into_element(cx),
                        ])
                        .block_end_border_top(true)
                        .textarea_min_height(Px(90.0))
                        .test_id("ui-gallery-input-group-demo-textarea")
                        .into_element(cx),
                ]
            },
        );
        section_card(cx, "Demo", content)
    };

    let align_inline_start = {
        let content = shadcn::InputGroup::new(inline_start)
            .a11y_label("Inline start addon")
            .leading([shadcn::InputGroupText::new("@").into_element(cx)])
            .refine_layout(max_w_xs.clone())
            .test_id("ui-gallery-input-group-align-inline-start")
            .into_element(cx);
        section_card(cx, "Align / inline-start", content)
    };

    let align_inline_end = {
        let content = shadcn::InputGroup::new(inline_end)
            .a11y_label("Inline end addon")
            .trailing([shadcn::InputGroupText::new(".com").into_element(cx)])
            .refine_layout(max_w_xs.clone())
            .test_id("ui-gallery-input-group-align-inline-end")
            .into_element(cx);
        section_card(cx, "Align / inline-end", content)
    };

    let align_block_start = {
        let content = shadcn::InputGroup::new(block_start)
            .a11y_label("Block start addon")
            .block_start([shadcn::InputGroupText::new("Write a concise title").into_element(cx)])
            .block_start_border_bottom(true)
            .refine_layout(max_w_xs.clone())
            .test_id("ui-gallery-input-group-align-block-start")
            .into_element(cx);
        section_card(cx, "Align / block-start", content)
    };

    let align_block_end = {
        let content = shadcn::InputGroup::new(block_end)
            .textarea()
            .a11y_label("Block end addon")
            .block_end([
                shadcn::InputGroupText::new("0/200").into_element(cx),
                shadcn::InputGroupButton::new("Publish")
                    .size(shadcn::InputGroupButtonSize::Sm)
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
            ])
            .block_end_border_top(true)
            .textarea_min_height(Px(84.0))
            .refine_layout(max_w_xs.clone())
            .test_id("ui-gallery-input-group-align-block-end")
            .into_element(cx);
        section_card(cx, "Align / block-end", content)
    };

    let icon = {
        let content = shadcn::InputGroup::new(icon_value)
            .a11y_label("Icon example")
            .leading([shadcn::InputGroupText::new("search").into_element(cx)])
            .refine_layout(max_w_xs.clone())
            .test_id("ui-gallery-input-group-icon")
            .into_element(cx);
        section_card(cx, "Icon", content)
    };

    let text = {
        let content = shadcn::InputGroup::new(text_value)
            .a11y_label("Text example")
            .control_test_id("ui-gallery-input-group-text-control")
            .leading([shadcn::InputGroupText::new("https://")
                .into_element(cx)
                .test_id("ui-gallery-input-group-text-leading")])
            .trailing([shadcn::InputGroupText::new(".com")
                .into_element(cx)
                .test_id("ui-gallery-input-group-text-trailing")])
            .refine_layout(max_w_xs.clone())
            .test_id("ui-gallery-input-group-text")
            .into_element(cx);
        section_card(cx, "Text", content)
    };

    let button = {
        let content = shadcn::InputGroup::new(button_value)
            .a11y_label("Button example")
            .trailing([shadcn::InputGroupButton::new("Search")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx)])
            .trailing_has_button(true)
            .refine_layout(max_w_xs.clone())
            .test_id("ui-gallery-input-group-button")
            .into_element(cx);
        section_card(cx, "Button", content)
    };

    let kbd = {
        let content = shadcn::InputGroup::new(kbd_value)
            .a11y_label("Kbd example")
            .leading([shadcn::InputGroupText::new("Ctrl").into_element(cx)])
            .trailing([shadcn::InputGroupText::new("K").into_element(cx)])
            .leading_has_kbd(true)
            .trailing_has_kbd(true)
            .refine_layout(max_w_xs.clone())
            .test_id("ui-gallery-input-group-kbd")
            .into_element(cx);
        section_card(cx, "Kbd", content)
    };

    let dropdown = {
        let content = shadcn::InputGroup::new(dropdown_value)
            .a11y_label("Dropdown example")
            .leading([
                shadcn::InputGroupButton::new("All")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .into_element(cx),
                shadcn::InputGroupText::new("v").into_element(cx),
            ])
            .leading_has_button(true)
            .refine_layout(max_w_xs.clone())
            .test_id("ui-gallery-input-group-dropdown")
            .into_element(cx);
        section_card(cx, "Dropdown", content)
    };

    let spinner = {
        let content = shadcn::InputGroup::new(spinner_value)
            .a11y_label("Spinner example")
            .leading([shadcn::Spinner::new().into_element(cx)])
            .refine_layout(max_w_xs.clone())
            .test_id("ui-gallery-input-group-spinner")
            .into_element(cx);
        section_card(cx, "Spinner", content)
    };

    let textarea = {
        let content = shadcn::InputGroup::new(textarea_value)
            .textarea()
            .a11y_label("Textarea example")
            .block_end([
                shadcn::InputGroupText::new("Shift+Enter for newline").into_element(cx),
                shadcn::InputGroupButton::new("Send")
                    .size(shadcn::InputGroupButtonSize::Sm)
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
            ])
            .block_end_border_top(true)
            .textarea_min_height(Px(100.0))
            .refine_layout(max_w_xs.clone())
            .test_id("ui-gallery-input-group-textarea")
            .into_element(cx);
        section_card(cx, "Textarea", content)
    };

    let custom_input = {
        let content = shadcn::InputGroup::new(custom_value)
            .textarea()
            .a11y_label("Custom input example")
            .block_start([shadcn::InputGroupText::new("Custom control (approx)").into_element(cx)])
            .block_start_border_bottom(true)
            .block_end([shadcn::InputGroupButton::new("Resize")
                .variant(shadcn::ButtonVariant::Ghost)
                .size(shadcn::InputGroupButtonSize::Sm)
                .into_element(cx)])
            .block_end_border_top(true)
            .textarea_min_height(Px(88.0))
            .refine_layout(max_w_xs.clone())
            .test_id("ui-gallery-input-group-custom")
            .into_element(cx);
        section_card(cx, "Custom Input", content)
    };

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::InputGroup::new(rtl_value)
                    .a11y_label("RTL input group")
                    .leading([shadcn::InputGroupText::new("lock").into_element(cx)])
                    .trailing([shadcn::InputGroupText::new("sk-...").into_element(cx)])
                    .refine_layout(max_w_xs.clone())
                    .into_element(cx)
            },
        )
        .test_id("ui-gallery-input-group-rtl");

        section_card(cx, "RTL", rtl_content)
    };

    let component_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Preview follows shadcn Input Group docs order: Demo, Align (inline-start/inline-end/block-start/block-end), Icon, Text, Button, Kbd, Dropdown, Spinner, Textarea, Custom Input, RTL.",
                ),
                demo,
                align_inline_start,
                align_inline_end,
                align_block_start,
                align_block_end,
                icon,
                text,
                button,
                kbd,
                dropdown,
                spinner,
                textarea,
                custom_input,
                rtl,
            ]
        },
    );
    let component_panel =
        shell(cx, component_panel_body).test_id("ui-gallery-input-group-component");

    let code_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Inline Addons").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"InputGroup::new(model).leading([InputGroupText::new("@")]).trailing([InputGroupText::new(".com")]);"#,
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Buttons and Kbd").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            r#"InputGroup::new(model).trailing([InputGroupButton::new("Search")]).trailing_has_button(true).trailing_has_kbd(true);"#,
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Textarea Layout").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            "InputGroup::new(model).textarea().block_end([...]).block_end_border_top(true).textarea_min_height(Px(100.0));",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
            ]
        },
    );
    let code_panel = shell(cx, code_panel_body);

    let notes_panel_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::h4(cx, "Notes"),
                shadcn::typography::muted(
                    cx,
                    "InputGroup API is slot based (`leading/trailing/block_start/block_end`) rather than explicit addon-align enums.",
                ),
                shadcn::typography::muted(
                    cx,
                    "`Custom Input` docs scenario is represented as composition approximation in current API.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Each section has stable test_id for future diag scripts.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_panel_body);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-input-group",
        component_panel,
        code_panel,
        notes_panel,
    )
}
