use super::super::*;

pub(super) fn preview_field(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct FieldPageModels {
        username: Option<Model<String>>,
        password: Option<Model<String>>,
        feedback: Option<Model<String>>,
        street: Option<Model<String>>,
        city: Option<Model<String>>,
        zip: Option<Model<String>>,
        responsive_message: Option<Model<String>>,
        select_value: Option<Model<Option<Arc<str>>>>,
        select_open: Option<Model<bool>>,
        slider_values: Option<Model<Vec<f32>>>,
        checkbox_a: Option<Model<bool>>,
        checkbox_b: Option<Model<bool>>,
        switch_enabled: Option<Model<bool>>,
        rtl_name: Option<Model<String>>,
        rtl_number: Option<Model<String>>,
    }

    let (
        username,
        password,
        feedback,
        street,
        city,
        zip,
        responsive_message,
        select_value,
        select_open,
        slider_values,
        checkbox_a,
        checkbox_b,
        switch_enabled,
        rtl_name,
        rtl_number,
    ) = cx.with_state(FieldPageModels::default, |st| {
        (
            st.username.clone(),
            st.password.clone(),
            st.feedback.clone(),
            st.street.clone(),
            st.city.clone(),
            st.zip.clone(),
            st.responsive_message.clone(),
            st.select_value.clone(),
            st.select_open.clone(),
            st.slider_values.clone(),
            st.checkbox_a.clone(),
            st.checkbox_b.clone(),
            st.switch_enabled.clone(),
            st.rtl_name.clone(),
            st.rtl_number.clone(),
        )
    });

    let (
        username,
        password,
        feedback,
        street,
        city,
        zip,
        responsive_message,
        select_value,
        select_open,
        slider_values,
        checkbox_a,
        checkbox_b,
        switch_enabled,
        rtl_name,
        rtl_number,
    ) = match (
        username,
        password,
        feedback,
        street,
        city,
        zip,
        responsive_message,
        select_value,
        select_open,
        slider_values,
        checkbox_a,
        checkbox_b,
        switch_enabled,
        rtl_name,
        rtl_number,
    ) {
        (
            Some(username),
            Some(password),
            Some(feedback),
            Some(street),
            Some(city),
            Some(zip),
            Some(responsive_message),
            Some(select_value),
            Some(select_open),
            Some(slider_values),
            Some(checkbox_a),
            Some(checkbox_b),
            Some(switch_enabled),
            Some(rtl_name),
            Some(rtl_number),
        ) => (
            username,
            password,
            feedback,
            street,
            city,
            zip,
            responsive_message,
            select_value,
            select_open,
            slider_values,
            checkbox_a,
            checkbox_b,
            switch_enabled,
            rtl_name,
            rtl_number,
        ),
        _ => {
            let username = cx.app.models_mut().insert(String::new());
            let password = cx.app.models_mut().insert(String::new());
            let feedback = cx.app.models_mut().insert(String::new());
            let street = cx.app.models_mut().insert(String::new());
            let city = cx.app.models_mut().insert(String::new());
            let zip = cx.app.models_mut().insert(String::new());
            let responsive_message = cx.app.models_mut().insert(String::new());
            let select_value = cx
                .app
                .models_mut()
                .insert(Some(Arc::<str>::from("engineering")));
            let select_open = cx.app.models_mut().insert(false);
            let slider_values = cx.app.models_mut().insert(vec![200.0, 800.0]);
            let checkbox_a = cx.app.models_mut().insert(true);
            let checkbox_b = cx.app.models_mut().insert(false);
            let switch_enabled = cx.app.models_mut().insert(false);
            let rtl_name = cx.app.models_mut().insert(String::new());
            let rtl_number = cx.app.models_mut().insert(String::new());

            cx.with_state(FieldPageModels::default, |st| {
                st.username = Some(username.clone());
                st.password = Some(password.clone());
                st.feedback = Some(feedback.clone());
                st.street = Some(street.clone());
                st.city = Some(city.clone());
                st.zip = Some(zip.clone());
                st.responsive_message = Some(responsive_message.clone());
                st.select_value = Some(select_value.clone());
                st.select_open = Some(select_open.clone());
                st.slider_values = Some(slider_values.clone());
                st.checkbox_a = Some(checkbox_a.clone());
                st.checkbox_b = Some(checkbox_b.clone());
                st.switch_enabled = Some(switch_enabled.clone());
                st.rtl_name = Some(rtl_name.clone());
                st.rtl_number = Some(rtl_number.clone());
            });

            (
                username,
                password,
                feedback,
                street,
                city,
                zip,
                responsive_message,
                select_value,
                select_open,
                slider_values,
                checkbox_a,
                checkbox_b,
                switch_enabled,
                rtl_name,
                rtl_number,
            )
        }
    };

    let theme = Theme::global(&*cx.app).clone();
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
        cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                LayoutRefinement::default().w_full().max_w(Px(900.0)),
            ),
            move |_cx| [body],
        )
    };
    let section_card =
        |cx: &mut ElementContext<'_, App>, title: &'static str, content: AnyElement| {
            let card = shell(cx, content);
            let centered_card = centered(cx, card);
            section(cx, title, centered_card)
        };

    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    let input = {
        let content = shadcn::FieldSet::new([shadcn::FieldGroup::new([
            shadcn::Field::new([
                shadcn::FieldLabel::new("Username").into_element(cx),
                shadcn::Input::new(username.clone())
                    .placeholder("Max Leiter")
                    .a11y_label("Username")
                    .into_element(cx),
                shadcn::FieldDescription::new("Choose a unique username.").into_element(cx),
            ])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("Password").into_element(cx),
                shadcn::FieldDescription::new("Must be at least 8 characters long.")
                    .into_element(cx),
                shadcn::Input::new(password)
                    .placeholder("••••••••")
                    .a11y_label("Password")
                    .into_element(cx),
            ])
            .into_element(cx),
        ])
        .into_element(cx)])
        .refine_layout(max_w_md.clone())
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-field-input"));
        section_card(cx, "Input", content)
    };

    let textarea = {
        let content = shadcn::FieldSet::new([shadcn::FieldGroup::new([shadcn::Field::new([
            shadcn::FieldLabel::new("Feedback").into_element(cx),
            shadcn::Textarea::new(feedback)
                .a11y_label("Feedback")
                .refine_layout(LayoutRefinement::default().h_px(Px(96.0)))
                .into_element(cx),
            shadcn::FieldDescription::new("Share your thoughts about our service.")
                .into_element(cx),
        ])
        .into_element(cx)])
        .into_element(cx)])
        .refine_layout(max_w_md.clone())
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-field-textarea"));
        section_card(cx, "Textarea", content)
    };
    let select = {
        let content = shadcn::Field::new([
            shadcn::FieldLabel::new("Department").into_element(cx),
            shadcn::Select::new(select_value, select_open)
                .placeholder("Choose department")
                .items([
                    shadcn::SelectItem::new("engineering", "Engineering"),
                    shadcn::SelectItem::new("design", "Design"),
                    shadcn::SelectItem::new("marketing", "Marketing"),
                    shadcn::SelectItem::new("operations", "Operations"),
                ])
                .into_element(cx),
            shadcn::FieldDescription::new("Select your department or area of work.")
                .into_element(cx),
        ])
        .refine_layout(max_w_md.clone())
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-field-select"));
        section_card(cx, "Select", content)
    };

    let slider = {
        let content = shadcn::Field::new([
            shadcn::FieldTitle::new("Price Range").into_element(cx),
            shadcn::FieldDescription::new("Set your budget range ($200-$800).").into_element(cx),
            shadcn::Slider::new(slider_values)
                .range(0.0, 1000.0)
                .step(10.0)
                .a11y_label("Price Range")
                .into_element(cx),
        ])
        .refine_layout(max_w_md.clone())
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-field-slider"));
        section_card(cx, "Slider", content)
    };

    let fieldset = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N4),
            |cx| {
                vec![
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("City").into_element(cx),
                        shadcn::Input::new(city)
                            .placeholder("New York")
                            .a11y_label("City")
                            .into_element(cx),
                    ])
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("Postal Code").into_element(cx),
                        shadcn::Input::new(zip)
                            .placeholder("90502")
                            .a11y_label("Postal Code")
                            .into_element(cx),
                    ])
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                ]
            },
        );

        let content = shadcn::FieldSet::new([
            shadcn::FieldLegend::new("Address Information").into_element(cx),
            shadcn::FieldDescription::new("We need your address to deliver your order.")
                .into_element(cx),
            shadcn::FieldGroup::new([
                shadcn::Field::new([
                    shadcn::FieldLabel::new("Street Address").into_element(cx),
                    shadcn::Input::new(street)
                        .placeholder("123 Main St")
                        .a11y_label("Street Address")
                        .into_element(cx),
                ])
                .into_element(cx),
                row,
            ])
            .into_element(cx),
        ])
        .refine_layout(max_w_md.clone())
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-field-fieldset"));
        section_card(cx, "Fieldset", content)
    };

    let checkbox = {
        let content = shadcn::FieldGroup::new([shadcn::FieldSet::new([
            shadcn::FieldLegend::new("Show these items on the desktop")
                .variant(shadcn::FieldLegendVariant::Label)
                .into_element(cx),
            shadcn::FieldDescription::new("Select the items you want to show.").into_element(cx),
            shadcn::FieldGroup::new([
                shadcn::Field::new([
                    shadcn::Checkbox::new(checkbox_a.clone())
                        .a11y_label("Hard disks")
                        .into_element(cx),
                    shadcn::FieldLabel::new("Hard disks").into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::Checkbox::new(checkbox_b.clone())
                        .a11y_label("External disks")
                        .into_element(cx),
                    shadcn::FieldLabel::new("External disks").into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx),
            ])
            .checkbox_group()
            .into_element(cx),
        ])
        .into_element(cx)])
        .refine_layout(max_w_md.clone())
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-field-checkbox"));
        section_card(cx, "Checkbox", content)
    };

    let radio = {
        let content = shadcn::FieldSet::new([
            shadcn::FieldLabel::new("Subscription Plan").into_element(cx),
            shadcn::FieldDescription::new("Yearly and lifetime plans offer significant savings.")
                .into_element(cx),
            shadcn::RadioGroup::uncontrolled(Some("monthly"))
                .a11y_label("Subscription Plan")
                .item(shadcn::RadioGroupItem::new(
                    "monthly",
                    "Monthly ($9.99/month)",
                ))
                .item(shadcn::RadioGroupItem::new(
                    "yearly",
                    "Yearly ($99.99/year)",
                ))
                .item(shadcn::RadioGroupItem::new(
                    "lifetime",
                    "Lifetime ($299.99)",
                ))
                .into_element(cx),
        ])
        .refine_layout(max_w_md.clone())
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-field-radio"));
        section_card(cx, "Radio", content)
    };

    let switch = {
        let content = shadcn::Field::new([
            shadcn::FieldContent::new([
                shadcn::FieldLabel::new("Multi-factor authentication").into_element(cx),
                shadcn::FieldDescription::new(
                    "Enable MFA. If no dedicated device is available, use one-time email codes.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Switch::new(switch_enabled)
                .a11y_label("Multi-factor authentication")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(max_w_md.clone())
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-field-switch"));
        section_card(cx, "Switch", content)
    };

    let choice_card = {
        let content = shadcn::FieldSet::new([
            shadcn::FieldLabel::new("Compute Environment").into_element(cx),
            shadcn::FieldDescription::new("Select the compute environment for your cluster.")
                .into_element(cx),
            shadcn::RadioGroup::uncontrolled(Some("kubernetes"))
                .a11y_label("Compute environment")
                .item(
                    shadcn::RadioGroupItem::new("kubernetes", "Kubernetes")
                        .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
                        .child(
                            shadcn::FieldContent::new([
                                shadcn::FieldTitle::new("Kubernetes").into_element(cx),
                                shadcn::FieldDescription::new(
                                    "Run GPU workloads on a K8s configured cluster.",
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx),
                        ),
                )
                .item(
                    shadcn::RadioGroupItem::new("vm", "Virtual Machine")
                        .variant(shadcn::RadioGroupItemVariant::ChoiceCard)
                        .child(
                            shadcn::FieldContent::new([
                                shadcn::FieldTitle::new("Virtual Machine").into_element(cx),
                                shadcn::FieldDescription::new(
                                    "Access a VM configured cluster to run GPU workloads.",
                                )
                                .into_element(cx),
                            ])
                            .into_element(cx),
                        ),
                )
                .into_element(cx),
        ])
        .refine_layout(max_w_md.clone())
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-field-choice-card"));
        section_card(cx, "Choice Card", content)
    };

    let field_group = {
        let content = shadcn::FieldGroup::new([
            shadcn::FieldSet::new([
                shadcn::FieldLabel::new("Responses").into_element(cx),
                shadcn::FieldDescription::new("Get notified for long-running responses.")
                    .into_element(cx),
                shadcn::FieldGroup::new([shadcn::Field::new([
                    shadcn::Checkbox::new(checkbox_a.clone())
                        .disabled(true)
                        .a11y_label("Push notifications")
                        .into_element(cx),
                    shadcn::FieldLabel::new("Push notifications").into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx)])
                .checkbox_group()
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::FieldSeparator::new().into_element(cx),
            shadcn::FieldSet::new([
                shadcn::FieldLabel::new("Tasks").into_element(cx),
                shadcn::FieldDescription::new("Get notified when task status changes.")
                    .into_element(cx),
                shadcn::FieldGroup::new([shadcn::Field::new([
                    shadcn::Checkbox::new(checkbox_b.clone())
                        .a11y_label("Email notifications")
                        .into_element(cx),
                    shadcn::FieldLabel::new("Email notifications").into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx)])
                .checkbox_group()
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .refine_layout(max_w_md.clone())
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-field-group"));
        section_card(cx, "Field Group", content)
    };
    let rtl = {
        let content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::FieldSet::new([
                    shadcn::FieldLegend::new("طريقة الدفع").into_element(cx),
                    shadcn::FieldDescription::new("جميع المعاملات آمنة ومشفرة").into_element(cx),
                    shadcn::FieldGroup::new([
                        shadcn::Field::new([
                            shadcn::FieldLabel::new("الاسم على البطاقة").into_element(cx),
                            shadcn::Input::new(rtl_name)
                                .a11y_label("الاسم على البطاقة")
                                .placeholder("Evil Rabbit")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::Field::new([
                            shadcn::FieldLabel::new("رقم البطاقة").into_element(cx),
                            shadcn::Input::new(rtl_number)
                                .a11y_label("رقم البطاقة")
                                .placeholder("1234 5678 9012 3456")
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .refine_layout(max_w_md.clone())
                .into_element(cx)
            },
        )
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-field-rtl"));
        section_card(cx, "RTL", content)
    };

    let responsive = {
        let content = shadcn::FieldSet::new([
            shadcn::FieldLegend::new("Profile").into_element(cx),
            shadcn::FieldDescription::new("Fill in your profile information.").into_element(cx),
            shadcn::FieldSeparator::new().into_element(cx),
            shadcn::FieldGroup::new([
                shadcn::Field::new([
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Name").into_element(cx),
                        shadcn::FieldDescription::new("Provide your full name for identification.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::Input::new(username.clone())
                        .placeholder("Evil Rabbit")
                        .a11y_label("Name")
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Responsive)
                .into_element(cx),
                shadcn::FieldSeparator::new().into_element(cx),
                shadcn::Field::new([
                    shadcn::FieldContent::new([
                        shadcn::FieldLabel::new("Message").into_element(cx),
                        shadcn::FieldDescription::new("Keep it short, preferably under 100 chars.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::Textarea::new(responsive_message)
                        .a11y_label("Message")
                        .refine_layout(LayoutRefinement::default().h_px(Px(96.0)).min_w(Px(280.0)))
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Responsive)
                .into_element(cx),
                shadcn::FieldSeparator::new().into_element(cx),
                shadcn::Field::new([
                    shadcn::Button::new("Submit").into_element(cx),
                    shadcn::Button::new("Cancel")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Responsive)
                .into_element(cx),
            ])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(760.0)))
        .into_element(cx)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-field-responsive"));
        section_card(cx, "Responsive Layout", content)
    };

    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "Preview follows shadcn Field docs order: Input, Textarea, Select, Slider, Fieldset, Checkbox, Radio, Switch, Choice Card, Field Group, RTL, Responsive Layout.",
                ),
                input,
                textarea,
                select,
                slider,
                fieldset,
                checkbox,
                radio,
                switch,
                choice_card,
                field_group,
                rtl,
                responsive,
            ]
        },
    );
    let component_panel = shell(cx, component_stack)
        .attach_semantics(SemanticsDecoration::default().test_id("ui-gallery-field-component"));

    let code_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Input + Description").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(cx, "Field::new([FieldLabel, Input, FieldDescription]).into_element(cx);")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Choice Card").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            "RadioGroupItem::new(...).variant(RadioGroupItemVariant::ChoiceCard).child(FieldContent::new([...]).into_element(cx));",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Responsive Orientation").into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![
                        ui::text_block(
                            cx,
                            "Field::new([...]).orientation(FieldOrientation::Responsive);",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
            ]
        },
    );
    let code_panel = shell(cx, code_stack);

    let notes_stack = stack::vstack(
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
                    "Field page now follows upstream docs section order for deterministic parity checks.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Each section keeps a stable test_id so diag scripts can target specific examples.",
                ),
                shadcn::typography::muted(
                    cx,
                    "RTL and Responsive samples are included to exercise orientation and direction contracts.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-field",
        component_panel,
        code_panel,
        notes_panel,
    )
}
