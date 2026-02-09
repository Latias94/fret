use super::super::*;

pub(super) fn preview_toggle_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
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
                LayoutRefinement::default().w_full().max_w(Px(560.0)),
            ),
            move |_cx| [body],
        )
    };

    let icon_item = |cx: &mut ElementContext<'_, App>, value: &'static str, label: &'static str| {
        shadcn::ToggleGroupItem::new(
            value,
            [shadcn::icon::icon(
                cx,
                fret_icons::IconId::new_static(match value {
                    "bold" => "lucide.bold",
                    "italic" => "lucide.italic",
                    _ => "lucide.underline",
                }),
            )],
        )
        .a11y_label(label)
    };

    let text_item = |cx: &mut ElementContext<'_, App>, value: &'static str, label: &'static str| {
        shadcn::ToggleGroupItem::new(value, [cx.text(label)]).a11y_label(format!("Toggle {label}"))
    };

    let demo_group = shadcn::ToggleGroup::multiple_uncontrolled(["bold", "italic"])
        .variant(shadcn::ToggleVariant::Outline)
        .items([
            icon_item(cx, "bold", "Toggle bold"),
            icon_item(cx, "italic", "Toggle italic"),
            icon_item(cx, "underline", "Toggle underline"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-demo");
    let demo = {
        let body = centered(cx, demo_group);
        section(cx, "Demo", body)
    };

    let outline_group = shadcn::ToggleGroup::multiple_uncontrolled(["left"])
        .variant(shadcn::ToggleVariant::Outline)
        .items([
            text_item(cx, "left", "Left"),
            text_item(cx, "center", "Center"),
            text_item(cx, "right", "Right"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-outline");
    let outline = {
        let body = centered(cx, outline_group);
        section(cx, "Outline", body)
    };

    let size_row = {
        let sm = shadcn::ToggleGroup::single_uncontrolled(Some("left"))
            .variant(shadcn::ToggleVariant::Outline)
            .size(shadcn::ToggleSize::Sm)
            .items([
                text_item(cx, "left", "Left"),
                text_item(cx, "center", "Center"),
                text_item(cx, "right", "Right"),
            ])
            .into_element(cx);
        let default = shadcn::ToggleGroup::single_uncontrolled(Some("left"))
            .variant(shadcn::ToggleVariant::Outline)
            .size(shadcn::ToggleSize::Default)
            .items([
                text_item(cx, "left", "Left"),
                text_item(cx, "center", "Center"),
                text_item(cx, "right", "Right"),
            ])
            .into_element(cx);
        let lg = shadcn::ToggleGroup::single_uncontrolled(Some("left"))
            .variant(shadcn::ToggleVariant::Outline)
            .size(shadcn::ToggleSize::Lg)
            .items([
                text_item(cx, "left", "Left"),
                text_item(cx, "center", "Center"),
                text_item(cx, "right", "Right"),
            ])
            .into_element(cx);

        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N2).items_start(),
            move |cx| {
                vec![
                    shadcn::typography::muted(cx, "Sm / Default / Lg"),
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        move |_cx| vec![sm, default, lg],
                    ),
                ]
            },
        )
        .test_id("ui-gallery-toggle-group-size")
    };
    let size = {
        let body = centered(cx, size_row);
        section(cx, "Size", body)
    };

    let spacing_group = shadcn::ToggleGroup::single_uncontrolled(Some("top"))
        .variant(shadcn::ToggleVariant::Outline)
        .size(shadcn::ToggleSize::Sm)
        .spacing(Space::N2)
        .items([
            text_item(cx, "top", "Top"),
            text_item(cx, "bottom", "Bottom"),
            text_item(cx, "left", "Left"),
            text_item(cx, "right", "Right"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-spacing");
    let spacing = {
        let body = centered(cx, spacing_group);
        section(cx, "Spacing", body)
    };

    let vertical_group = shadcn::ToggleGroup::multiple_uncontrolled(["bold", "italic"])
        .orientation(fret_ui_kit::primitives::toggle_group::ToggleGroupOrientation::Vertical)
        .spacing(Space::N1)
        .items([
            icon_item(cx, "bold", "Toggle bold"),
            icon_item(cx, "italic", "Toggle italic"),
            icon_item(cx, "underline", "Toggle underline"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-vertical");
    let vertical = {
        let body = centered(cx, vertical_group);
        section(cx, "Vertical", body)
    };

    let disabled_group = shadcn::ToggleGroup::multiple_uncontrolled(["bold"])
        .disabled(true)
        .variant(shadcn::ToggleVariant::Outline)
        .items([
            icon_item(cx, "bold", "Toggle bold"),
            icon_item(cx, "italic", "Toggle italic"),
            icon_item(cx, "underline", "Toggle underline"),
        ])
        .into_element(cx)
        .test_id("ui-gallery-toggle-group-disabled");
    let disabled = {
        let body = centered(cx, disabled_group);
        section(cx, "Disabled", body)
    };

    let rtl_group = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::ToggleGroup::single_uncontrolled(Some("list"))
                .variant(shadcn::ToggleVariant::Outline)
                .items([
                    text_item(cx, "list", "List"),
                    text_item(cx, "grid", "Grid"),
                    text_item(cx, "cards", "Cards"),
                ])
                .into_element(cx)
        },
    )
    .test_id("ui-gallery-toggle-group-rtl");
    let rtl = {
        let body = centered(cx, rtl_group);
        section(cx, "RTL", body)
    };

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Toggle Group docs order for quick visual lookup.",
    );
    let component_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| {
            vec![
                preview_hint,
                demo,
                outline,
                size,
                spacing,
                vertical,
                disabled,
                rtl,
            ]
        },
    );
    let component_panel = shell(cx, component_stack).test_id("ui-gallery-toggle-group-component");

    let code_block =
        |cx: &mut ElementContext<'_, App>, title: &'static str, snippet: &'static str| {
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                    .into_element(cx),
                shadcn::CardContent::new(vec![ui::text_block(cx, snippet).into_element(cx)])
                    .into_element(cx),
            ])
            .into_element(cx)
        };

    let code_stack = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                code_block(
                    cx,
                    "Demo",
                    "ToggleGroup::multiple_uncontrolled([\"bold\", \"italic\"])\n    .variant(ToggleVariant::Outline)\n    .item(ToggleGroupItem::new(\"bold\", [icon(\"lucide.bold\")]))",
                ),
                code_block(
                    cx,
                    "Size / Spacing",
                    "ToggleGroup::single_uncontrolled(Some(\"left\"))\n    .size(ToggleSize::Sm | Default | Lg)\n    .spacing(Space::N2)",
                ),
                code_block(
                    cx,
                    "Vertical / RTL",
                    "ToggleGroup::multiple_uncontrolled([\"bold\"])\n    .orientation(ToggleGroupOrientation::Vertical)\nwith_direction_provider(LayoutDirection::Rtl, ...)",
                ),
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
                    "Use Single mode for mutually-exclusive options (alignment, list/grid/cards).",
                ),
                shadcn::typography::muted(
                    cx,
                    "Use Multiple mode for formatting toggles where users may combine states.",
                ),
                shadcn::typography::muted(
                    cx,
                    "`spacing` is useful when each item needs stronger visual separation.",
                ),
                shadcn::typography::muted(
                    cx,
                    "For icon-only groups, keep explicit a11y labels for assistive technologies.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-toggle-group",
        component_panel,
        code_panel,
        notes_panel,
    )
}
