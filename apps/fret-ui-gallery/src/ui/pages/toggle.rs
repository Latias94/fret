use super::super::*;

pub(super) fn preview_toggle(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
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
                LayoutRefinement::default().w_full().max_w(Px(480.0)),
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let demo = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Toggle::uncontrolled(false)
                        .variant(shadcn::ToggleVariant::Outline)
                        .size(shadcn::ToggleSize::Sm)
                        .a11y_label("Toggle bookmark")
                        .children([
                            shadcn::icon::icon(
                                cx,
                                fret_icons::IconId::new_static("lucide.bookmark"),
                            ),
                            cx.text("Bookmark"),
                        ])
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-toggle-demo");

        let body = centered(cx, row);
        section(cx, "Demo", body)
    };

    let outline = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Toggle::uncontrolled(false)
                        .variant(shadcn::ToggleVariant::Outline)
                        .a11y_label("Toggle italic")
                        .children([
                            shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.italic")),
                            cx.text("Italic"),
                        ])
                        .into_element(cx),
                    shadcn::Toggle::uncontrolled(false)
                        .variant(shadcn::ToggleVariant::Outline)
                        .a11y_label("Toggle bold")
                        .children([
                            shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.bold")),
                            cx.text("Bold"),
                        ])
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-toggle-outline");

        let body = centered(cx, row);
        section(cx, "Outline", body)
    };

    let with_text = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Toggle::uncontrolled(false)
                        .a11y_label("Toggle italic with text")
                        .children([
                            shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.italic")),
                            cx.text("Italic"),
                        ])
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-toggle-with-text");

        let body = centered(cx, row);
        section(cx, "With Text", body)
    };

    let size = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Toggle::uncontrolled(false)
                        .variant(shadcn::ToggleVariant::Outline)
                        .size(shadcn::ToggleSize::Sm)
                        .a11y_label("Toggle small")
                        .label("Small")
                        .into_element(cx),
                    shadcn::Toggle::uncontrolled(false)
                        .variant(shadcn::ToggleVariant::Outline)
                        .size(shadcn::ToggleSize::Default)
                        .a11y_label("Toggle default")
                        .label("Default")
                        .into_element(cx),
                    shadcn::Toggle::uncontrolled(false)
                        .variant(shadcn::ToggleVariant::Outline)
                        .size(shadcn::ToggleSize::Lg)
                        .a11y_label("Toggle large")
                        .label("Large")
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-toggle-size");

        let body = centered(cx, row);
        section(cx, "Size", body)
    };

    let disabled = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Toggle::uncontrolled(false)
                        .disabled(true)
                        .a11y_label("Toggle disabled")
                        .label("Disabled")
                        .into_element(cx),
                    shadcn::Toggle::uncontrolled(false)
                        .disabled(true)
                        .variant(shadcn::ToggleVariant::Outline)
                        .a11y_label("Toggle disabled outline")
                        .label("Disabled")
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-toggle-disabled");

        let body = centered(cx, row);
        section(cx, "Disabled", body)
    };

    let rtl = {
        let rtl_toggle = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Toggle::uncontrolled(false)
                    .variant(shadcn::ToggleVariant::Outline)
                    .size(shadcn::ToggleSize::Sm)
                    .a11y_label("Toggle bookmark rtl")
                    .children([
                        shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.bookmark")),
                        cx.text("Bookmark"),
                    ])
                    .into_element(cx)
            },
        )
        .test_id("ui-gallery-toggle-rtl");

        let body = centered(cx, rtl_toggle);
        section(cx, "RTL", body)
    };

    let preview_hint = shadcn::typography::muted(
        cx,
        "Preview follows shadcn Toggle docs order for quick visual lookup.",
    );
    let component_panel = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N6)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |_cx| vec![preview_hint, demo, outline, with_text, size, disabled, rtl],
    );

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
                    "Toggle::uncontrolled(false)\n    .variant(ToggleVariant::Outline)\n    .size(ToggleSize::Sm)\n    .children([icon(\"lucide.bookmark\"), text(\"Bookmark\")])",
                ),
                code_block(
                    cx,
                    "Outline + With Text",
                    "Toggle::uncontrolled(false)\n    .variant(ToggleVariant::Outline)\n    .children([icon(\"lucide.italic\"), text(\"Italic\")])",
                ),
                code_block(
                    cx,
                    "Size + Disabled",
                    "Toggle::uncontrolled(false).size(ToggleSize::Sm | Default | Lg)\nToggle::uncontrolled(false).disabled(true)",
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
                    "Use Outline when toggle sits in dense toolbars and needs stronger boundaries.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Prefer icon + short text labels so state remains understandable in compact layouts.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Keep a11y_label explicit for icon-heavy toggles to improve accessibility tree quality.",
                ),
                shadcn::typography::muted(
                    cx,
                    "For quick keyboard validation, tab through toggles and verify pressed visual parity.",
                ),
            ]
        },
    );
    let notes_panel = shell(cx, notes_stack);

    super::render_component_page_tabs(
        cx,
        "ui-gallery-toggle",
        component_panel,
        code_panel,
        notes_panel,
    )
}
