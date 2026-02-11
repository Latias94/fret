use super::super::super::super::*;

pub(in crate::ui) fn preview_chrome_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    last_action: Model<Arc<str>>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
) -> Vec<AnyElement> {
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: exercise hover/focus/pressed chrome under view-cache + shell."),
                cx.text(
                    "This page intentionally mixes many focusable widgets and overlay triggers.",
                ),
            ]
        },
    );

    let body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4),
        |cx| {
            let mut out = Vec::new();

            out.extend(preview_overlay(
                cx,
                popover_open,
                dialog_open,
                alert_dialog_open,
                sheet_open,
                portal_geometry_popover_open,
                dropdown_open,
                context_menu_open,
                context_menu_edge_open,
                last_action,
            ));

            let controls = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N3),
                |cx| {
                    let mut out: Vec<AnyElement> = Vec::new();

                    let row = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                shadcn::Button::new("One")
                                    .test_id("ui-gallery-chrome-btn-1")
                                    .into_element(cx),
                                shadcn::Button::new("Two")
                                    .variant(shadcn::ButtonVariant::Secondary)
                                    .test_id("ui-gallery-chrome-btn-2")
                                    .into_element(cx),
                                shadcn::Button::new("Three")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .test_id("ui-gallery-chrome-btn-3")
                                    .into_element(cx),
                                shadcn::Button::new("Disabled")
                                    .disabled(true)
                                    .test_id("ui-gallery-chrome-btn-disabled")
                                    .into_element(cx),
                            ]
                        },
                    );
                    out.push(row);

                    let fields = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_start(),
                        |cx| {
                            vec![
                                stack::vstack(
                                    cx,
                                    stack::VStackProps::default().gap(Space::N1),
                                    |cx| {
                                        let input = shadcn::Input::new(text_input.clone())
                                            .a11y_label("Chrome torture input")
                                            .placeholder("Type")
                                            .into_element(cx);
                                        let input = input.attach_semantics(
                                            SemanticsDecoration::default()
                                                .role(fret_core::SemanticsRole::TextField)
                                                .test_id("ui-gallery-chrome-text-input"),
                                        );
                                        vec![cx.text("Text input"), input]
                                    },
                                ),
                                stack::vstack(
                                    cx,
                                    stack::VStackProps::default().gap(Space::N1),
                                    |cx| {
                                        let textarea = shadcn::Textarea::new(text_area.clone())
                                            .a11y_label("Chrome torture textarea")
                                            .into_element(cx);
                                        let textarea = textarea.attach_semantics(
                                            SemanticsDecoration::default()
                                                .role(fret_core::SemanticsRole::TextField)
                                                .test_id("ui-gallery-chrome-text-area"),
                                        );
                                        vec![cx.text("Text area"), textarea]
                                    },
                                ),
                            ]
                        },
                    );
                    out.push(fields);

                    let toggles = stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N3).items_center(),
                        |cx| {
                            vec![
                                shadcn::Checkbox::new(checkbox.clone())
                                    .a11y_label("Chrome torture checkbox")
                                    .test_id("ui-gallery-chrome-checkbox")
                                    .into_element(cx),
                                shadcn::Switch::new(switch.clone())
                                    .a11y_label("Chrome torture switch")
                                    .test_id("ui-gallery-chrome-switch")
                                    .into_element(cx),
                            ]
                        },
                    );
                    out.push(toggles);

                    out
                },
            );
            out.push(controls);

            out
        },
    );

    let content = body.attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-chrome-torture-root"),
    );

    vec![header, content]
}
