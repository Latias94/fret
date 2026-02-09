use super::*;

#[test]
fn web_vs_fret_layout_field_input_geometry() {
    let web = read_web_golden("field-input");
    let theme = web_theme(&web);

    let web_username_label =
        web_find_by_tag_and_text(&theme.root, "label", "Username").expect("web username label");
    let web_password_label =
        web_find_by_tag_and_text(&theme.root, "label", "Password").expect("web password label");
    let web_username_input = find_first(&theme.root, &|n| n.tag == "input").expect("web input");
    let web_inputs: Vec<&WebNode> = {
        let mut out = Vec::new();
        fn walk<'a>(n: &'a WebNode, out: &mut Vec<&'a WebNode>) {
            if n.tag == "input" {
                out.push(n);
            }
            for c in &n.children {
                walk(c, out);
            }
        }
        walk(&theme.root, &mut out);
        out.sort_by(|a, b| {
            a.rect
                .y
                .partial_cmp(&b.rect.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    };
    let web_password_input = web_inputs.get(1).copied().unwrap_or(web_username_input);
    let web_username_desc = web_find_by_tag_and_text(
        &theme.root,
        "p",
        "Choose a unique username for your account.",
    )
    .expect("web username desc");
    let web_password_desc = web_find_by_tag_and_text(&theme.root, "p", "Must be at least 8")
        .expect("web password desc");

    let web_root = web_find_smallest_container(
        &theme.root,
        &[web_username_label, web_password_desc, web_password_input],
    )
    .expect("web root container");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let username: Model<String> = cx.app.models_mut().insert(String::new());
        let password: Model<String> = cx.app.models_mut().insert(String::new());

        let username_label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:username:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Username").into_element(cx)],
        );
        let username_input = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::TextField,
                label: Some(Arc::from("Golden:field-input:username:input")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Input::new(username)
                        .a11y_label("Username")
                        .placeholder("Max Leiter")
                        .into_element(cx),
                ]
            },
        );
        let username_desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:username:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new(
                        "Choose a unique username for your account.",
                    )
                    .into_element(cx),
                ]
            },
        );

        let username_field =
            fret_ui_shadcn::Field::new(vec![username_label, username_input, username_desc])
                .into_element(cx);

        let password_label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:password:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Password").into_element(cx)],
        );
        let password_input = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::TextField,
                label: Some(Arc::from("Golden:field-input:password:input")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Input::new(password)
                        .a11y_label("Password")
                        .placeholder("????????")
                        .into_element(cx),
                ]
            },
        );
        let password_desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:password:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new("Must be at least 8 characters long.")
                        .into_element(cx),
                ]
            },
        );

        let password_field =
            fret_ui_shadcn::Field::new(vec![password_label, password_desc, password_input])
                .into_element(cx);

        let group =
            fret_ui_shadcn::FieldGroup::new(vec![username_field, password_field]).into_element(cx);
        let set = fret_ui_shadcn::FieldSet::new(vec![group]).into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default().w_px(Px(web_root.rect.w)),
                ),
                ..Default::default()
            },
            move |_cx| vec![set],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-input:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(&snap, SemanticsRole::Panel, Some("Golden:field-input:root"))
        .expect("fret root");

    let username_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-input:username:label"),
    )
    .expect("fret username label");
    let username_input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:field-input:username:input"),
    )
    .expect("fret username input");
    let username_desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-input:username:desc"),
    )
    .expect("fret username desc");

    let password_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-input:password:label"),
    )
    .expect("fret password label");
    let password_input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:field-input:password:input"),
    )
    .expect("fret password input");
    let password_desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-input:password:desc"),
    )
    .expect("fret password desc");

    assert_close_px(
        "field-input root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );

    assert_close_px(
        "field-input username label y",
        username_label.bounds.origin.y,
        web_username_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-input username input y",
        username_input.bounds.origin.y,
        web_username_input.rect.y,
        1.0,
    );
    assert_close_px(
        "field-input username desc y",
        username_desc.bounds.origin.y,
        web_username_desc.rect.y,
        1.0,
    );

    let username_label_to_input_gap = username_input.bounds.origin.y.0
        - (username_label.bounds.origin.y.0 + username_label.bounds.size.height.0);
    assert!(
        (username_label_to_input_gap - 12.0).abs() <= 1.0,
        "field-input username label->input gap: expected ~12 got={username_label_to_input_gap}"
    );

    let username_input_to_desc_gap = username_desc.bounds.origin.y.0
        - (username_input.bounds.origin.y.0 + username_input.bounds.size.height.0);
    assert!(
        (username_input_to_desc_gap - 12.0).abs() <= 1.0,
        "field-input username input->desc gap: expected ~12 got={username_input_to_desc_gap}"
    );

    assert_close_px(
        "field-input password label y",
        password_label.bounds.origin.y,
        web_password_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-input password desc y",
        password_desc.bounds.origin.y,
        web_password_desc.rect.y,
        1.0,
    );
    assert_close_px(
        "field-input password input y",
        password_input.bounds.origin.y,
        web_password_input.rect.y,
        1.0,
    );

    let password_label_to_desc_gap = password_desc.bounds.origin.y.0
        - (password_label.bounds.origin.y.0 + password_label.bounds.size.height.0);
    assert!(
        (password_label_to_desc_gap - 8.0).abs() <= 1.0,
        "field-input password label->desc gap: expected ~8 got={password_label_to_desc_gap}"
    );

    let password_desc_to_input_gap = password_input.bounds.origin.y.0
        - (password_desc.bounds.origin.y.0 + password_desc.bounds.size.height.0);
    assert!(
        (password_desc_to_input_gap - 12.0).abs() <= 1.0,
        "field-input password desc->input gap: expected ~12 got={password_desc_to_input_gap}"
    );

    let field_to_field_gap = password_label.bounds.origin.y.0
        - (username_desc.bounds.origin.y.0 + username_desc.bounds.size.height.0);
    assert!(
        (field_to_field_gap - 28.0).abs() <= 1.0,
        "field-input field->field gap: expected ~28 got={field_to_field_gap}"
    );
}

#[test]
fn web_vs_fret_layout_field_checkbox_geometry() {
    let web = read_web_golden("field-checkbox");
    let theme = web_theme(&web);

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root");
    let web_outer_group =
        web_find_by_class_tokens(&theme.root, &["flex", "w-full", "flex-col", "gap-7"])
            .expect("web outer group");
    let web_row_1 = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "group")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == "horizontal")
    })
    .expect("web field row");
    let web_sync_field = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "group")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == "horizontal")
            && contains_text(n, "Sync Desktop & Documents folders")
    })
    .expect("web sync field");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let checked_1: Model<bool> = cx.app.models_mut().insert(true);
        let checked_2: Model<bool> = cx.app.models_mut().insert(false);
        let checked_3: Model<bool> = cx.app.models_mut().insert(false);
        let checked_4: Model<bool> = cx.app.models_mut().insert(false);
        let checked_5: Model<bool> = cx.app.models_mut().insert(true);

        let legend = fret_ui_shadcn::FieldLegend::new("Show these items on the desktop")
            .variant(fret_ui_shadcn::FieldLegendVariant::Label)
            .into_element(cx);
        let description = fret_ui_shadcn::FieldDescription::new(
            "Select the items you want to show on the desktop.",
        )
        .into_element(cx);

        let row_1 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-checkbox:row-1")),
                ..Default::default()
            },
            move |cx| {
                let checkbox = fret_ui_shadcn::Checkbox::new(checked_1)
                    .a11y_label("Hard disks")
                    .into_element(cx);
                let label = fret_ui_shadcn::FieldLabel::new("Hard disks").into_element(cx);
                vec![
                    fret_ui_shadcn::Field::new(vec![checkbox, label])
                        .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                ]
            },
        );

        let row_2 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-checkbox:row-2")),
                ..Default::default()
            },
            move |cx| {
                let checkbox = fret_ui_shadcn::Checkbox::new(checked_2)
                    .a11y_label("External disks")
                    .into_element(cx);
                let label = fret_ui_shadcn::FieldLabel::new("External disks").into_element(cx);
                vec![
                    fret_ui_shadcn::Field::new(vec![checkbox, label])
                        .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                ]
            },
        );

        let row_3 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-checkbox:row-3")),
                ..Default::default()
            },
            move |cx| {
                let checkbox = fret_ui_shadcn::Checkbox::new(checked_3)
                    .a11y_label("CDs, DVDs, and iPods")
                    .into_element(cx);
                let label =
                    fret_ui_shadcn::FieldLabel::new("CDs, DVDs, and iPods").into_element(cx);
                vec![
                    fret_ui_shadcn::Field::new(vec![checkbox, label])
                        .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                ]
            },
        );

        let row_4 = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-checkbox:row-4")),
                ..Default::default()
            },
            move |cx| {
                let checkbox = fret_ui_shadcn::Checkbox::new(checked_4)
                    .a11y_label("Connected servers")
                    .into_element(cx);
                let label = fret_ui_shadcn::FieldLabel::new("Connected servers").into_element(cx);
                vec![
                    fret_ui_shadcn::Field::new(vec![checkbox, label])
                        .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                ]
            },
        );

        let checkbox_group = fret_ui_shadcn::FieldGroup::new(vec![row_1, row_2, row_3, row_4])
            .gap(Space::N3)
            .into_element(cx);

        let fieldset = fret_ui_shadcn::FieldSet::new(vec![legend, description, checkbox_group])
            .into_element(cx);

        let sync_field = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-checkbox:sync-field")),
                ..Default::default()
            },
            move |cx| {
                let checkbox = fret_ui_shadcn::Checkbox::new(checked_5)
                    .a11y_label("Sync")
                    .into_element(cx);
                let content = fret_ui_shadcn::FieldContent::new(vec![
                    fret_ui_shadcn::FieldLabel::new("Sync Desktop & Documents folders").into_element(cx),
                    fret_ui_shadcn::FieldDescription::new(
                        "Your Desktop & Documents folders are being synced with iCloud Drive. You can access them from other devices.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx);

                vec![
                    fret_ui_shadcn::Field::new(vec![checkbox, content])
                        .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                        .into_element(cx),
                ]
            },
        );

        let group = fret_ui_shadcn::FieldGroup::new(vec![
            fieldset,
            fret_ui_shadcn::FieldSeparator::new().into_element(cx),
            sync_field,
        ])
        .into_element(cx);

        let group = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-checkbox:group")),
                ..Default::default()
            },
            move |_cx| vec![group],
        );

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default().w_px(Px(web_root.rect.w)),
                ),
                ..Default::default()
            },
            move |_cx| vec![group],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-checkbox:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-checkbox:root"),
    )
    .expect("fret root");
    let group = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-checkbox:group"),
    )
    .expect("fret group");

    let row_1 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-checkbox:row-1"),
    )
    .expect("fret row 1");
    let row_2 = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-checkbox:row-2"),
    )
    .expect("fret row 2");
    let sync_field = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-checkbox:sync-field"),
    )
    .expect("fret sync field");

    assert_close_px(
        "field-checkbox root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );
    assert_close_px(
        "field-checkbox group width",
        group.bounds.size.width,
        web_outer_group.rect.w,
        1.0,
    );

    let row_gap = row_2.bounds.origin.y.0 - (row_1.bounds.origin.y.0 + row_1.bounds.size.height.0);
    assert!(
        (row_gap - 12.0).abs() <= 1.0,
        "field-checkbox inner group gap: expected ~12 got={row_gap}"
    );

    assert_close_px(
        "field-checkbox row height",
        row_1.bounds.size.height,
        web_row_1.rect.h,
        1.0,
    );
    assert_close_px(
        "field-checkbox sync-field y",
        sync_field.bounds.origin.y,
        web_sync_field.rect.y,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_field_switch_geometry() {
    let web = read_web_golden("field-switch");
    let theme = web_theme(&web);

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root");
    let web_switch = find_first(&theme.root, &|n| {
        n.tag == "button"
            && n.attrs.get("role").is_some_and(|v| v == "switch")
            && n.attrs.get("data-state").is_some_and(|v| v == "unchecked")
    })
    .expect("web switch");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let checked: Model<bool> = cx.app.models_mut().insert(false);

        let content = fret_ui_shadcn::FieldContent::new(vec![
            fret_ui_shadcn::FieldLabel::new("Multi-factor authentication").into_element(cx),
            fret_ui_shadcn::FieldDescription::new(
                "Enable multi-factor authentication. If you do not have a two-factor device, you can use a one-time code sent to your email.",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let switch = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-switch:switch")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Switch::new(checked)
                        .a11y_label("2fa")
                        .refine_layout(LayoutRefinement::default().flex_shrink_0())
                        .into_element(cx),
                ]
            },
        );

        let field = fret_ui_shadcn::Field::new(vec![content, switch])
            .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
            .into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default().w_px(Px(web_root.rect.w)),
                ),
                ..Default::default()
            },
            move |_cx| vec![field],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-switch:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-switch:root"),
    )
    .expect("fret root");
    let switch = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-switch:switch"),
    )
    .expect("fret switch");

    assert_close_px(
        "field-switch root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );
    assert_close_px(
        "field-switch switch x",
        switch.bounds.origin.x,
        web_switch.rect.x,
        1.0,
    );
    assert_close_px(
        "field-switch switch y",
        switch.bounds.origin.y,
        web_switch.rect.y,
        1.0,
    );
    assert_close_px(
        "field-switch switch w",
        switch.bounds.size.width,
        web_switch.rect.w,
        1.0,
    );
    assert_close_px(
        "field-switch switch h",
        switch.bounds.size.height,
        web_switch.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_field_select_geometry() {
    let web = read_web_golden("field-select");
    let theme = web_theme(&web);

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root");
    let web_label =
        web_find_by_tag_and_text(&theme.root, "label", "Department").expect("web label");
    let web_trigger = find_first(&theme.root, &|n| {
        n.tag == "button" && n.attrs.get("role").is_some_and(|v| v == "combobox")
    })
    .expect("web trigger");
    let web_desc =
        web_find_by_tag_and_text(&theme.root, "p", "Select your department or area of work.")
            .expect("web desc");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
        let open: Model<bool> = cx.app.models_mut().insert(false);

        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-select:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Department").into_element(cx)],
        );

        let trigger = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-select:trigger")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Select::new(value, open)
                        .placeholder("Choose department")
                        .items([
                            fret_ui_shadcn::SelectItem::new("engineering", "Engineering"),
                            fret_ui_shadcn::SelectItem::new("design", "Design"),
                            fret_ui_shadcn::SelectItem::new("marketing", "Marketing"),
                            fret_ui_shadcn::SelectItem::new("sales", "Sales"),
                            fret_ui_shadcn::SelectItem::new("support", "Customer Support"),
                            fret_ui_shadcn::SelectItem::new("hr", "Human Resources"),
                            fret_ui_shadcn::SelectItem::new("finance", "Finance"),
                            fret_ui_shadcn::SelectItem::new("operations", "Operations"),
                        ])
                        .refine_layout(LayoutRefinement::default().w_full())
                        .into_element(cx),
                ]
            },
        );

        let desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-select:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new(
                        "Select your department or area of work.",
                    )
                    .into_element(cx),
                ]
            },
        );

        let field = fret_ui_shadcn::Field::new(vec![label, trigger, desc]).into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default().w_px(Px(web_root.rect.w)),
                ),
                ..Default::default()
            },
            move |_cx| vec![field],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-select:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-select:root"),
    )
    .expect("fret root");
    let label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-select:label"),
    )
    .expect("fret label");
    let trigger = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-select:trigger"),
    )
    .expect("fret trigger");
    let desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-select:desc"),
    )
    .expect("fret desc");

    assert_close_px(
        "field-select root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );

    assert_close_px(
        "field-select label y",
        label.bounds.origin.y,
        web_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-select trigger y",
        trigger.bounds.origin.y,
        web_trigger.rect.y,
        1.0,
    );
    assert_close_px(
        "field-select desc y",
        desc.bounds.origin.y,
        web_desc.rect.y,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_field_radio_geometry() {
    let web = read_web_golden("field-radio");
    let theme = web_theme(&web);

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root");
    let web_label =
        web_find_by_tag_and_text(&theme.root, "label", "Subscription Plan").expect("web label");
    let web_desc =
        web_find_by_tag_and_text(&theme.root, "p", "Yearly and lifetime").expect("web desc");
    let web_group = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "radiogroup")
    })
    .expect("web radio group");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-radio:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Subscription Plan").into_element(cx)],
        );

        let desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-radio:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new(
                        "Yearly and lifetime plans offer significant savings.",
                    )
                    .into_element(cx),
                ]
            },
        );

        let radio_group = {
            let items = vec![
                fret_ui_shadcn::RadioGroupItem::new("monthly", "Monthly ($9.99/month)").children(
                    vec![fret_ui_shadcn::FieldLabel::new("Monthly ($9.99/month)").into_element(cx)],
                ),
                fret_ui_shadcn::RadioGroupItem::new("yearly", "Yearly ($99.99/year)").children(
                    vec![fret_ui_shadcn::FieldLabel::new("Yearly ($99.99/year)").into_element(cx)],
                ),
                fret_ui_shadcn::RadioGroupItem::new("lifetime", "Lifetime ($299.99)").children(
                    vec![fret_ui_shadcn::FieldLabel::new("Lifetime ($299.99)").into_element(cx)],
                ),
            ];

            items
                .into_iter()
                .fold(
                    fret_ui_shadcn::RadioGroup::uncontrolled(Some("monthly")),
                    |group, item| group.item(item),
                )
                .a11y_label("Subscription Plan")
                .into_element(cx)
        };

        let set = fret_ui_shadcn::FieldSet::new(vec![label, desc, radio_group]).into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default().w_px(Px(web_root.rect.w)),
                ),
                ..Default::default()
            },
            move |_cx| vec![set],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-radio:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(&snap, SemanticsRole::Panel, Some("Golden:field-radio:root"))
        .expect("fret root");
    let label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-radio:label"),
    )
    .expect("fret label");
    let desc = find_semantics(&snap, SemanticsRole::Panel, Some("Golden:field-radio:desc"))
        .expect("fret desc");
    let group = find_semantics(&snap, SemanticsRole::RadioGroup, Some("Subscription Plan"))
        .or_else(|| find_semantics(&snap, SemanticsRole::RadioGroup, None))
        .expect("fret radio group");

    assert_close_px(
        "field-radio root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );

    assert_close_px(
        "field-radio label y",
        label.bounds.origin.y,
        web_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-radio desc y",
        desc.bounds.origin.y,
        web_desc.rect.y,
        1.0,
    );
    assert_close_px(
        "field-radio group y",
        group.bounds.origin.y,
        web_group.rect.y,
        1.0,
    );
    assert_close_px(
        "field-radio group h",
        group.bounds.size.height,
        web_group.rect.h,
        2.0,
    );
}

#[test]
fn web_vs_fret_layout_field_textarea_geometry() {
    let web = read_web_golden("field-textarea");
    let theme = web_theme(&web);

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root");
    let web_label = web_find_by_tag_and_text(&theme.root, "label", "Feedback").expect("web label");
    let web_textarea = find_first(&theme.root, &|n| n.tag == "textarea").expect("web textarea");
    let web_desc =
        web_find_by_tag_and_text(&theme.root, "p", "Share your thoughts").expect("web desc");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let model: Model<String> = cx.app.models_mut().insert(String::new());

        let label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-textarea:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Feedback").into_element(cx)],
        );

        let textarea = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-textarea:textarea")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Textarea::new(model)
                        .a11y_label("Feedback")
                        .into_element(cx),
                ]
            },
        );

        let desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-textarea:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new("Share your thoughts about our service.")
                        .into_element(cx),
                ]
            },
        );

        let field = fret_ui_shadcn::Field::new(vec![label, textarea, desc]).into_element(cx);
        let group = fret_ui_shadcn::FieldGroup::new(vec![field]).into_element(cx);
        let set = fret_ui_shadcn::FieldSet::new(vec![group]).into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default().w_px(Px(web_root.rect.w)),
                ),
                ..Default::default()
            },
            move |_cx| vec![set],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-textarea:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-textarea:root"),
    )
    .expect("fret root");
    let label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-textarea:label"),
    )
    .expect("fret label");
    let textarea = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-textarea:textarea"),
    )
    .expect("fret textarea wrapper");
    let desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-textarea:desc"),
    )
    .expect("fret desc");

    assert_close_px(
        "field-textarea root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );

    assert_close_px(
        "field-textarea label y",
        label.bounds.origin.y,
        web_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-textarea textarea y",
        textarea.bounds.origin.y,
        web_textarea.rect.y,
        1.0,
    );
    assert_close_px(
        "field-textarea textarea h",
        textarea.bounds.size.height,
        web_textarea.rect.h,
        1.0,
    );
    assert_close_px(
        "field-textarea desc y",
        desc.bounds.origin.y,
        web_desc.rect.y,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_field_group_geometry() {
    let web = read_web_golden("field-group");
    let theme = web_theme(&web);

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root");
    let web_responses_label =
        web_find_by_tag_and_text(&theme.root, "label", "Responses").expect("web responses label");
    let web_responses_desc =
        web_find_by_tag_and_text(&theme.root, "p", "Get notified when ChatGPT")
            .expect("web responses desc");
    let web_tasks_label =
        web_find_by_tag_and_text(&theme.root, "label", "Tasks").expect("web tasks label");

    let web_responses_row = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "group")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == "horizontal")
            && contains_text(n, "Push notifications")
            && contains_id(n, "push")
    })
    .expect("web responses row");
    let web_push_tasks_row = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "group")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == "horizontal")
            && contains_text(n, "Push notifications")
            && contains_id(n, "push-tasks")
    })
    .expect("web push tasks row");
    let web_email_tasks_row = find_first(&theme.root, &|n| {
        n.attrs.get("role").is_some_and(|v| v == "group")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == "horizontal")
            && contains_text(n, "Email notifications")
            && contains_id(n, "email-tasks")
    })
    .expect("web email tasks row");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let responses_push: Model<bool> = cx.app.models_mut().insert(true);
        let tasks_push: Model<bool> = cx.app.models_mut().insert(false);
        let tasks_email: Model<bool> = cx.app.models_mut().insert(false);

        let responses_label = fret_ui_shadcn::FieldLabel::new("Responses")
            .into_element(cx)
            .attach_semantics(fret_ui::element::SemanticsDecoration {
                role: Some(SemanticsRole::Panel),
                label: Some(Arc::from("Golden:field-group:responses:label")),
                ..Default::default()
            });
        let responses_desc = fret_ui_shadcn::FieldDescription::new(
            "Get notified when ChatGPT responds to requests that take time, like research or image generation.",
        )
        .into_element(cx)
        .attach_semantics(fret_ui::element::SemanticsDecoration {
            role: Some(SemanticsRole::Panel),
            label: Some(Arc::from("Golden:field-group:responses:desc")),
            ..Default::default()
        });
        let responses_row = {
            let checkbox = fret_ui_shadcn::Checkbox::new(responses_push)
                .disabled(true)
                .a11y_label("push")
                .into_element(cx);
            let label = fret_ui_shadcn::FieldLabel::new("Push notifications").into_element(cx);
            fret_ui_shadcn::Field::new(vec![checkbox, label])
                .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                .into_element(cx)
        }
        .attach_semantics(fret_ui::element::SemanticsDecoration {
            role: Some(SemanticsRole::Panel),
            label: Some(Arc::from("Golden:field-group:responses:row")),
            ..Default::default()
        });
        let responses_checkbox_group = fret_ui_shadcn::FieldGroup::new(vec![responses_row])
            .checkbox_group()
            .into_element(cx);
        let responses_fieldset = fret_ui_shadcn::FieldSet::new(vec![
            responses_label,
            responses_desc,
            responses_checkbox_group,
        ])
        .into_element(cx);

        let tasks_label = fret_ui_shadcn::FieldLabel::new("Tasks")
            .into_element(cx)
            .attach_semantics(fret_ui::element::SemanticsDecoration {
                role: Some(SemanticsRole::Panel),
                label: Some(Arc::from("Golden:field-group:tasks:label")),
                ..Default::default()
            });
        let tasks_desc = fret_ui_shadcn::FieldDescription::new(
            "Get notified when tasks you've created have updates. Manage tasks",
        )
        .into_element(cx)
        .attach_semantics(fret_ui::element::SemanticsDecoration {
            role: Some(SemanticsRole::Panel),
            label: Some(Arc::from("Golden:field-group:tasks:desc")),
            ..Default::default()
        });
        let tasks_row_push = {
            let checkbox = fret_ui_shadcn::Checkbox::new(tasks_push)
                .a11y_label("push-tasks")
                .into_element(cx);
            let label = fret_ui_shadcn::FieldLabel::new("Push notifications").into_element(cx);
            fret_ui_shadcn::Field::new(vec![checkbox, label])
                .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                .into_element(cx)
        }
        .attach_semantics(fret_ui::element::SemanticsDecoration {
            role: Some(SemanticsRole::Panel),
            label: Some(Arc::from("Golden:field-group:tasks:push-row")),
            ..Default::default()
        });
        let tasks_row_email = {
            let checkbox = fret_ui_shadcn::Checkbox::new(tasks_email)
                .a11y_label("email-tasks")
                .into_element(cx);
            let label = fret_ui_shadcn::FieldLabel::new("Email notifications").into_element(cx);
            fret_ui_shadcn::Field::new(vec![checkbox, label])
                .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                .into_element(cx)
        }
        .attach_semantics(fret_ui::element::SemanticsDecoration {
            role: Some(SemanticsRole::Panel),
            label: Some(Arc::from("Golden:field-group:tasks:email-row")),
            ..Default::default()
        });
        let tasks_checkbox_group =
            fret_ui_shadcn::FieldGroup::new(vec![tasks_row_push, tasks_row_email])
                .checkbox_group()
                .into_element(cx);
        let tasks_fieldset =
            fret_ui_shadcn::FieldSet::new(vec![tasks_label, tasks_desc, tasks_checkbox_group])
                .into_element(cx);

        let separator = fret_ui_shadcn::FieldSeparator::new().into_element(cx);

        let group =
            fret_ui_shadcn::FieldGroup::new(vec![responses_fieldset, separator, tasks_fieldset])
                .into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default().w_px(Px(web_root.rect.w)),
                ),
                ..Default::default()
            },
            move |_cx| vec![group],
        );

        vec![
            root.attach_semantics(fret_ui::element::SemanticsDecoration {
                role: Some(SemanticsRole::Panel),
                label: Some(Arc::from("Golden:field-group:root")),
                ..Default::default()
            }),
        ]
    });

    let root = find_semantics(&snap, SemanticsRole::Panel, Some("Golden:field-group:root"))
        .expect("fret root");
    let responses_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-group:responses:label"),
    )
    .expect("fret responses label");
    let responses_desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-group:responses:desc"),
    )
    .expect("fret responses desc");
    let responses_row = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-group:responses:row"),
    )
    .expect("fret responses row");
    let tasks_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-group:tasks:label"),
    )
    .expect("fret tasks label");

    let tasks_row_push = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-group:tasks:push-row"),
    )
    .expect("fret tasks push row");
    let tasks_row_email = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-group:tasks:email-row"),
    )
    .expect("fret tasks email row");

    assert_close_px(
        "field-group root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );
    assert_close_px(
        "field-group responses label y",
        responses_label.bounds.origin.y,
        web_responses_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-group responses desc y",
        responses_desc.bounds.origin.y,
        web_responses_desc.rect.y,
        1.0,
    );
    assert_close_px(
        "field-group responses desc h",
        responses_desc.bounds.size.height,
        web_responses_desc.rect.h,
        1.0,
    );
    assert_close_px(
        "field-group responses row y",
        responses_row.bounds.origin.y,
        web_responses_row.rect.y,
        1.0,
    );
    assert_close_px(
        "field-group responses row h",
        responses_row.bounds.size.height,
        web_responses_row.rect.h,
        1.0,
    );
    assert_close_px(
        "field-group tasks label y",
        tasks_label.bounds.origin.y,
        web_tasks_label.rect.y,
        1.0,
    );

    let fret_first_fieldset_to_tasks_label = tasks_label.bounds.origin.y.0
        - (responses_row.bounds.origin.y.0 + responses_row.bounds.size.height.0);
    let web_first_fieldset_to_tasks_label =
        web_tasks_label.rect.y - (web_responses_row.rect.y + web_responses_row.rect.h);
    assert!(
        (fret_first_fieldset_to_tasks_label - web_first_fieldset_to_tasks_label).abs() <= 1.0,
        "field-group responses row -> tasks label: expected≈{web_first_fieldset_to_tasks_label} got={fret_first_fieldset_to_tasks_label}"
    );

    let tasks_gap = tasks_row_email.bounds.origin.y.0
        - (tasks_row_push.bounds.origin.y.0 + tasks_row_push.bounds.size.height.0);
    assert!(
        (tasks_gap - 12.0).abs() <= 1.0,
        "field-group checkbox-group gap: expected ~12 got={tasks_gap}"
    );

    assert_close_px(
        "field-group tasks push row h",
        tasks_row_push.bounds.size.height,
        web_push_tasks_row.rect.h,
        1.0,
    );
    assert_close_px(
        "field-group tasks email row h",
        tasks_row_email.bounds.size.height,
        web_email_tasks_row.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_field_fieldset_geometry() {
    let web = read_web_golden("field-fieldset");
    let theme = web_theme(&web);

    let web_root = web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md", "space-y-6"])
        .expect("web root");
    let web_legend =
        web_find_by_tag_and_text(&theme.root, "legend", "Address Information").expect("web legend");
    let web_desc = web_find_by_tag_and_text(&theme.root, "p", "We need your address")
        .expect("web description");

    let web_street_label =
        web_find_by_tag_and_text(&theme.root, "label", "Street Address").expect("web street label");
    let web_city_label =
        web_find_by_tag_and_text(&theme.root, "label", "City").expect("web city label");
    let web_zip_label =
        web_find_by_tag_and_text(&theme.root, "label", "Postal Code").expect("web zip label");

    let web_street_input = find_first(&theme.root, &|n| {
        n.tag == "input" && n.id.as_deref().is_some_and(|id| id == "street")
    })
    .expect("web street input");
    let web_city_input = find_first(&theme.root, &|n| {
        n.tag == "input" && n.id.as_deref().is_some_and(|id| id == "city")
    })
    .expect("web city input");
    let web_zip_input = find_first(&theme.root, &|n| {
        n.tag == "input" && n.id.as_deref().is_some_and(|id| id == "zip")
    })
    .expect("web zip input");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let street: Model<String> = cx.app.models_mut().insert(String::new());
        let city: Model<String> = cx.app.models_mut().insert(String::new());
        let zip: Model<String> = cx.app.models_mut().insert(String::new());

        let legend = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-fieldset:legend")),
                ..Default::default()
            },
            move |cx| {
                vec![fret_ui_shadcn::FieldLegend::new("Address Information").into_element(cx)]
            },
        );
        let desc = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-fieldset:desc")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::FieldDescription::new(
                        "We need your address to deliver your order.",
                    )
                    .into_element(cx),
                ]
            },
        );

        let street_label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-fieldset:street:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Street Address").into_element(cx)],
        );
        let street_input = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::TextField,
                label: Some(Arc::from("Golden:field-fieldset:street:input")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Input::new(street)
                        .a11y_label("Street Address")
                        .placeholder("123 Main St")
                        .into_element(cx),
                ]
            },
        );
        let street_field =
            fret_ui_shadcn::Field::new(vec![street_label, street_input]).into_element(cx);

        let city_label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-fieldset:city:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("City").into_element(cx)],
        );
        let city_input = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::TextField,
                label: Some(Arc::from("Golden:field-fieldset:city:input")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Input::new(city)
                        .a11y_label("City")
                        .placeholder("New York")
                        .into_element(cx),
                ]
            },
        );
        let city_field = fret_ui_shadcn::Field::new(vec![city_label, city_input]).into_element(cx);

        let zip_label = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-fieldset:zip:label")),
                ..Default::default()
            },
            move |cx| vec![fret_ui_shadcn::FieldLabel::new("Postal Code").into_element(cx)],
        );
        let zip_input = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::TextField,
                label: Some(Arc::from("Golden:field-fieldset:zip:input")),
                ..Default::default()
            },
            move |cx| {
                vec![
                    fret_ui_shadcn::Input::new(zip)
                        .a11y_label("Postal Code")
                        .placeholder("90502")
                        .into_element(cx),
                ]
            },
        );
        let zip_field = fret_ui_shadcn::Field::new(vec![zip_label, zip_input]).into_element(cx);

        let grid = cx.grid(
            GridProps {
                cols: 2,
                gap: Px(16.0),
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default().w_full(),
                ),
                ..Default::default()
            },
            move |_cx| vec![city_field, zip_field],
        );

        let group = fret_ui_shadcn::FieldGroup::new(vec![street_field, grid]).into_element(cx);
        let set = fret_ui_shadcn::FieldSet::new(vec![legend, desc, group]).into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default().w_px(Px(web_root.rect.w)),
                ),
                ..Default::default()
            },
            move |_cx| vec![set],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-fieldset:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-fieldset:root"),
    )
    .expect("fret root");
    let legend = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-fieldset:legend"),
    )
    .expect("fret legend");
    let desc = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-fieldset:desc"),
    )
    .expect("fret desc");

    let street_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-fieldset:street:label"),
    )
    .expect("fret street label");
    let street_input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:field-fieldset:street:input"),
    )
    .expect("fret street input");

    let city_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-fieldset:city:label"),
    )
    .expect("fret city label");
    let city_input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:field-fieldset:city:input"),
    )
    .expect("fret city input");

    let zip_label = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-fieldset:zip:label"),
    )
    .expect("fret zip label");
    let zip_input = find_semantics(
        &snap,
        SemanticsRole::TextField,
        Some("Golden:field-fieldset:zip:input"),
    )
    .expect("fret zip input");

    assert_close_px(
        "field-fieldset root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );

    assert_close_px(
        "field-fieldset legend y",
        legend.bounds.origin.y,
        web_legend.rect.y,
        1.0,
    );
    assert_close_px(
        "field-fieldset desc y",
        desc.bounds.origin.y,
        web_desc.rect.y,
        1.0,
    );

    assert_close_px(
        "field-fieldset street label y",
        street_label.bounds.origin.y,
        web_street_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-fieldset street input y",
        street_input.bounds.origin.y,
        web_street_input.rect.y,
        1.0,
    );
    assert_close_px(
        "field-fieldset city label y",
        city_label.bounds.origin.y,
        web_city_label.rect.y,
        1.0,
    );
    assert_close_px(
        "field-fieldset zip label y",
        zip_label.bounds.origin.y,
        web_zip_label.rect.y,
        1.0,
    );

    let fret_city_x = city_input.bounds.origin.x.0 - root.bounds.origin.x.0;
    let web_city_x = web_city_input.rect.x - web_root.rect.x;
    assert!(
        (fret_city_x - web_city_x).abs() <= 1.0,
        "field-fieldset city input x: expected≈{web_city_x} got={fret_city_x}"
    );

    let fret_zip_x = zip_input.bounds.origin.x.0 - root.bounds.origin.x.0;
    let web_zip_x = web_zip_input.rect.x - web_root.rect.x;
    assert!(
        (fret_zip_x - web_zip_x).abs() <= 1.0,
        "field-fieldset zip input x: expected≈{web_zip_x} got={fret_zip_x}"
    );
}

#[test]
fn web_vs_fret_layout_field_choice_card_geometry() {
    let web = read_web_golden("field-choice-card");
    let theme = web_theme(&web);

    let web_root =
        web_find_by_class_tokens(&theme.root, &["w-full", "max-w-md"]).expect("web root");

    let web_radio_group = find_first(&theme.root, &|n| {
        n.tag == "div"
            && n.attrs.get("role").is_some_and(|v| v == "radiogroup")
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("grid gap-3"))
    })
    .expect("web radio group");

    let web_card_kubernetes = find_first(&theme.root, &|n| {
        n.tag == "label"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("has-[>[data-slot=field]]:w-full"))
            && contains_text(n, "Kubernetes")
    })
    .expect("web kubernetes card");

    let web_card_vm = find_first(&theme.root, &|n| {
        n.tag == "label"
            && n.class_name
                .as_deref()
                .is_some_and(|c| c.contains("has-[>[data-slot=field]]:w-full"))
            && contains_text(n, "Virtual Machine")
    })
    .expect("web vm card");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let mut services = StyleAwareServices::default();
    let snap = run_fret_root_with_services(bounds, &mut services, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let selected: Model<Option<Arc<str>>> =
            cx.app.models_mut().insert(Some(Arc::from("kubernetes")));

        let root =
            radio_group_prim::RadioGroupRoot::new(selected).a11y_label("Compute Environment");
        let values: Arc<[Arc<str>]> = Arc::from([Arc::from("kubernetes"), Arc::from("vm")]);
        let disabled: Arc<[bool]> = Arc::from([false, false]);

        let mut list_props = RovingFlexProps::default();
        list_props.flex.layout = fret_ui_kit::declarative::style::layout_style(
            &theme,
            fret_ui_kit::LayoutRefinement::default().w_full(),
        );
        list_props.flex.gap = MetricRef::space(Space::N3).resolve(&theme);

        let pressable_layout = fret_ui_kit::declarative::style::layout_style(
            &theme,
            fret_ui_kit::LayoutRefinement::default().w_full(),
        );

        let chrome = ChromeRefinement::default()
            .rounded_md()
            .border_1()
            .border_color(fret_ui_kit::ColorRef::Color(border))
            .p_4();

        let make_card = |cx: &mut fret_ui::ElementContext<'_, App>,
                         title: &'static str,
                         desc: &'static str,
                         checked: bool| {
            let content = fret_ui_shadcn::FieldContent::new(vec![
                fret_ui_shadcn::FieldTitle::new(title).into_element(cx),
                fret_ui_shadcn::FieldDescription::new(desc).into_element(cx),
            ])
            .into_element(cx);

            let radio_stub_layout = fret_ui_kit::declarative::style::layout_style(
                &Theme::global(&*cx.app),
                fret_ui_kit::LayoutRefinement::default()
                    .w_px(Px(16.0))
                    .h_px(Px(16.0))
                    .flex_shrink_0(),
            );
            let radio_stub = cx.container(
                ContainerProps {
                    layout: radio_stub_layout,
                    ..Default::default()
                },
                |_cx| Vec::new(),
            );

            let field = fret_ui_shadcn::Field::new(vec![content, radio_stub])
                .orientation(fret_ui_shadcn::FieldOrientation::Horizontal)
                .into_element(cx);

            let mut props = fret_ui_kit::declarative::style::container_props(
                &theme,
                chrome.clone(),
                fret_ui_kit::LayoutRefinement::default().w_full(),
            );
            if checked {
                // Matches upstream `has-data-[state=checked]:bg-primary/5` (visual-only).
                if let Some(primary) = theme.color_by_key("primary/5") {
                    props.background = Some(primary);
                }
            }

            cx.container(props, move |_cx| vec![field])
        };

        let list = root
            .clone()
            .list(values.clone(), disabled.clone())
            .into_element(cx, list_props, move |cx| {
                let kubernetes = root
                    .item("kubernetes")
                    .label("Kubernetes")
                    .index(0)
                    .set_size(Some(2))
                    .tab_stop(true)
                    .into_element(
                        cx,
                        &root,
                        PressableProps {
                            layout: pressable_layout,
                            ..Default::default()
                        },
                        move |cx, _st, checked| {
                            vec![make_card(
                                cx,
                                "Kubernetes",
                                "Run GPU workloads on a K8s configured cluster.",
                                checked,
                            )]
                        },
                    );

                let vm = root
                    .item("vm")
                    .label("Virtual Machine")
                    .index(1)
                    .set_size(Some(2))
                    .into_element(
                        cx,
                        &root,
                        PressableProps {
                            layout: pressable_layout,
                            ..Default::default()
                        },
                        move |cx, _st, checked| {
                            vec![make_card(
                                cx,
                                "Virtual Machine",
                                "Access a VM configured cluster to run GPU workloads.",
                                checked,
                            )]
                        },
                    );

                vec![kubernetes, vm]
            });

        let set = fret_ui_shadcn::FieldSet::new(vec![
            fret_ui_shadcn::FieldLabel::new("Compute Environment").into_element(cx),
            fret_ui_shadcn::FieldDescription::new(
                "Select the compute environment for your cluster.",
            )
            .into_element(cx),
            list,
        ])
        .into_element(cx);

        let root = cx.container(
            ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(
                    &Theme::global(&*cx.app),
                    fret_ui_kit::LayoutRefinement::default().w_px(Px(web_root.rect.w)),
                ),
                ..Default::default()
            },
            move |_cx| vec![set],
        );

        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-choice-card:root")),
                ..Default::default()
            },
            move |_cx| vec![root],
        )]
    });

    let root = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-choice-card:root"),
    )
    .expect("fret root");
    let radio_group = find_semantics(
        &snap,
        SemanticsRole::RadioGroup,
        Some("Compute Environment"),
    )
    .or_else(|| find_semantics(&snap, SemanticsRole::RadioGroup, None))
    .expect("fret radio group");
    let kubernetes =
        find_semantics(&snap, SemanticsRole::RadioButton, Some("Kubernetes")).expect("fret k8s");
    let vm = find_semantics(&snap, SemanticsRole::RadioButton, Some("Virtual Machine"))
        .expect("fret vm");

    assert_close_px(
        "field-choice-card root width",
        root.bounds.size.width,
        web_root.rect.w,
        1.0,
    );
    assert_close_px(
        "field-choice-card kubernetes y",
        kubernetes.bounds.origin.y,
        web_card_kubernetes.rect.y,
        2.0,
    );
    assert_close_px(
        "field-choice-card kubernetes w",
        kubernetes.bounds.size.width,
        web_card_kubernetes.rect.w,
        2.0,
    );
    assert_close_px(
        "field-choice-card vm y",
        vm.bounds.origin.y,
        web_card_vm.rect.y,
        2.0,
    );
    assert_close_px(
        "field-choice-card vm w",
        vm.bounds.size.width,
        web_card_vm.rect.w,
        2.0,
    );
    assert_close_px(
        "field-choice-card radiogroup y",
        radio_group.bounds.origin.y,
        web_radio_group.rect.y,
        1.0,
    );
    let fret_card_delta_y = vm.bounds.origin.y.0 - kubernetes.bounds.origin.y.0;
    let web_card_delta_y = web_card_vm.rect.y - web_card_kubernetes.rect.y;
    assert!(
        (fret_card_delta_y - web_card_delta_y).abs() <= 2.0,
        "field-choice-card card delta y: expected≈{web_card_delta_y} got={fret_card_delta_y}"
    );

    assert_close_px(
        "field-choice-card radiogroup-to-root gap",
        Px(radio_group.bounds.origin.y.0 - root.bounds.origin.y.0),
        web_radio_group.rect.y - web_root.rect.y,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_field_slider_track_geometry_matches_web() {
    let web = read_web_golden("field-slider");
    let theme = web_theme(&web);

    let web_slider = find_first(&theme.root, &|n| {
        n.tag == "span"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Price Range")
    })
    .expect("web slider");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let (ui, snap, _root) = run_fret_root_with_ui(bounds, |cx| {
        let model: Model<Vec<f32>> = cx.app.models_mut().insert(vec![200.0, 800.0]);
        let slider = fret_ui_shadcn::Slider::new(model)
            .range(0.0, 1000.0)
            .step(10.0)
            .a11y_label("Price Range")
            .into_element(cx);

        let field = fret_ui_shadcn::Field::new(vec![
            fret_ui_shadcn::FieldTitle::new("Price Range").into_element(cx),
            fret_ui_shadcn::FieldDescription::new("Set your budget range ($200 - 800).")
                .into_element(cx),
            slider,
        ])
        .into_element(cx);

        vec![cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_slider.rect.w)),
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![field],
        )]
    });

    let thumb = find_semantics(&snap, SemanticsRole::Slider, Some("Price Range"))
        .or_else(|| find_semantics(&snap, SemanticsRole::Slider, None))
        .expect("fret slider thumb semantics");
    let slider = thumb
        .parent
        .and_then(|parent| snap.nodes.iter().find(|n| n.id == parent))
        .unwrap_or(thumb);

    assert_close_px(
        "field-slider track w",
        slider.bounds.size.width,
        web_slider.rect.w,
        1.0,
    );
    assert_close_px(
        "field-slider track h",
        slider.bounds.size.height,
        web_slider.rect.h,
        1.0,
    );

    let _ = ui.debug_node_bounds(slider.id).expect("fret slider bounds");
}

#[test]
fn web_vs_fret_layout_field_slider_thumb_insets_match_web() {
    let web = read_web_golden("field-slider");
    let theme = web_theme(&web);

    let web_slider = find_first(&theme.root, &|n| {
        n.tag == "span"
            && n.attrs
                .get("aria-label")
                .is_some_and(|v| v == "Price Range")
    })
    .expect("web slider");

    let mut web_thumbs = find_all(web_slider, &|n| {
        n.tag == "span" && n.attrs.get("role").is_some_and(|r| r == "slider")
    });
    assert_eq!(
        web_thumbs.len(),
        2,
        "expected 2 web slider thumbs; got={}",
        web_thumbs.len()
    );
    web_thumbs.sort_by(|a, b| a.rect.x.total_cmp(&b.rect.x));
    let web_thumb_dx: Vec<f32> = web_thumbs
        .iter()
        .map(|thumb| thumb.rect.x - web_slider.rect.x)
        .collect();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let model: Model<Vec<f32>> = cx.app.models_mut().insert(vec![200.0, 800.0]);
        let slider = fret_ui_shadcn::Slider::new(model)
            .range(0.0, 1000.0)
            .step(10.0)
            .a11y_label("Price Range")
            .into_element(cx);

        let field = fret_ui_shadcn::Field::new(vec![
            fret_ui_shadcn::FieldTitle::new("Price Range").into_element(cx),
            fret_ui_shadcn::FieldDescription::new("Set your budget range ($200 - 800).")
                .into_element(cx),
            slider,
        ])
        .into_element(cx);

        vec![cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_slider.rect.w)),
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![field],
        )]
    });

    let mut fret_thumbs: Vec<&fret_core::SemanticsNode> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::Slider && n.label.as_deref() == Some("Price Range"))
        .collect();
    assert_eq!(
        fret_thumbs.len(),
        2,
        "expected 2 fret slider thumbs; got={}",
        fret_thumbs.len()
    );
    fret_thumbs.sort_by(|a, b| a.bounds.origin.x.0.total_cmp(&b.bounds.origin.x.0));

    let slider = fret_thumbs[0]
        .parent
        .and_then(|parent| snap.nodes.iter().find(|n| n.id == parent))
        .unwrap_or(fret_thumbs[0]);

    let fret_thumb_dx: Vec<f32> = fret_thumbs
        .iter()
        .map(|thumb| thumb.bounds.origin.x.0 - slider.bounds.origin.x.0)
        .collect();

    assert_close_px(
        "field-slider thumb[0] inset x",
        Px(fret_thumb_dx[0]),
        web_thumb_dx[0],
        1.0,
    );
    assert_close_px(
        "field-slider thumb[1] inset x",
        Px(fret_thumb_dx[1]),
        web_thumb_dx[1],
        1.0,
    );

    // Sanity: thumbs should remain within the slider bounds (Radix `getThumbInBoundsOffset`).
    for (i, thumb) in fret_thumbs.iter().enumerate() {
        assert!(
            thumb.bounds.origin.x.0 >= slider.bounds.origin.x.0 - 0.5,
            "thumb[{i}] x should not underflow slider bounds: thumb.x={} slider.x={}",
            thumb.bounds.origin.x.0,
            slider.bounds.origin.x.0
        );
        assert!(
            thumb.bounds.origin.x.0 + thumb.bounds.size.width.0
                <= slider.bounds.origin.x.0 + slider.bounds.size.width.0 + 0.5,
            "thumb[{i}] should not overflow slider bounds: thumb.right={} slider.right={}",
            thumb.bounds.origin.x.0 + thumb.bounds.size.width.0,
            slider.bounds.origin.x.0 + slider.bounds.size.width.0
        );
    }
}

#[test]
fn web_vs_fret_layout_field_demo_separator_height_matches_web() {
    let web = read_web_golden("field-demo");
    let theme = web_theme(&web);
    let web_sep = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_all_tokens(n, &["relative", "-my-2", "h-5", "text-sm"])
    })
    .expect("web field-separator");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let sep = fret_ui_shadcn::FieldSeparator::new()
            .refine_layout(
                LayoutRefinement::default()
                    .mt_neg(Space::N0)
                    .mb_neg(Space::N0),
            )
            .into_element(cx);
        let sep = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-demo:separator")),
                ..Default::default()
            },
            move |_cx| vec![sep],
        );

        vec![cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_sep.rect.w)),
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![sep],
        )]
    });

    let sep = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-demo:separator"),
    )
    .expect("fret field-separator");

    assert_close_px(
        "field-demo separator h",
        sep.bounds.size.height,
        web_sep.rect.h,
        1.0,
    );
}

#[test]
fn web_vs_fret_layout_field_responsive_orientation_places_input_beside_content() {
    let web = read_web_golden("field-responsive");
    let theme = web_theme(&web);

    let web_max_w = find_first(&theme.root, &|n| {
        n.tag == "div" && class_has_token(n, "max-w-4xl")
    })
    .expect("web max-w-4xl container");

    let web_content = find_first(&theme.root, &|n| {
        n.tag == "div"
            && class_has_token(n, "group/field-content")
            && contains_text(n, "Provide your full name")
    })
    .expect("web field-content");

    let web_input = find_first(&theme.root, &|n| n.tag == "input" && contains_id(n, "name"))
        .expect("web input");

    let web_dx = web_input.rect.x - web_content.rect.x;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    let snap = run_fret_root(bounds, |cx| {
        let theme = Theme::global(&*cx.app).clone();
        let content_layout =
            decl_style::layout_style(&theme, LayoutRefinement::default().flex_1().min_w_0());

        let content = fret_ui_shadcn::FieldContent::new(vec![
            fret_ui_shadcn::FieldLabel::new("Name").into_element(cx),
            fret_ui_shadcn::FieldDescription::new("Provide your full name for identification")
                .into_element(cx),
        ])
        .into_element(cx);

        let content = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: content_layout,
                role: SemanticsRole::Panel,
                label: Some(Arc::from("Golden:field-responsive:content")),
                ..Default::default()
            },
            move |_cx| vec![content],
        );

        let model: Model<String> = cx.app.models_mut().insert(String::new());
        let input = fret_ui_shadcn::Input::new(model)
            .a11y_label("NameInput")
            .placeholder("Evil Rabbit")
            .into_element(cx);

        let field = fret_ui_shadcn::Field::new(vec![content, input])
            .orientation(fret_ui_shadcn::FieldOrientation::Responsive)
            .into_element(cx);

        vec![cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(web_max_w.rect.w)),
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |_cx| vec![field],
        )]
    });

    let fret_content = find_semantics(
        &snap,
        SemanticsRole::Panel,
        Some("Golden:field-responsive:content"),
    )
    .expect("fret field-content");
    let fret_input = find_semantics(&snap, SemanticsRole::TextField, Some("NameInput"))
        .or_else(|| find_semantics(&snap, SemanticsRole::TextField, None))
        .expect("fret input");

    let fret_dx = fret_input.bounds.origin.x.0 - fret_content.bounds.origin.x.0;

    assert!(
        fret_dx >= 1.0,
        "expected responsive field to place input beside content; dx={fret_dx} (content={:?} input={:?})",
        fret_content.bounds,
        fret_input.bounds
    );
    assert_close_px("field-responsive input dx", Px(fret_dx), web_dx, 12.0);
}
