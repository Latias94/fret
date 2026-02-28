use super::super::*;
use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_tooltip(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use std::time::Duration;

    let make_tooltip = |cx: &mut ElementContext<'_, App>,
                        label: &'static str,
                        side: shadcn::TooltipSide,
                        content: &'static str| {
        shadcn::Tooltip::new(
            shadcn::Button::new(label)
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
            shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(cx, content)])
                .into_element(cx),
        )
        .arrow(true)
        .side(side)
        .into_element(cx)
    };

    let make_tooltip_with_test_ids =
        |cx: &mut ElementContext<'_, App>,
         label: &'static str,
         trigger_test_id: &'static str,
         side: shadcn::TooltipSide,
         content: &'static str,
         panel_test_id: &'static str,
         text_test_id: &'static str| {
            shadcn::Tooltip::new(
                shadcn::Button::new(label)
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id(trigger_test_id)
                    .into_element(cx),
                shadcn::TooltipContent::new(vec![
                    shadcn::TooltipContent::text(cx, content).test_id(text_test_id),
                ])
                .into_element(cx),
            )
            .arrow(true)
            .side(side)
            .panel_test_id(panel_test_id)
            .into_element(cx)
        };

    let body = shadcn::TooltipProvider::new()
        .delay(Duration::ZERO)
        .timeout_duration(Duration::from_millis(400))
        .with(cx, |cx| {
            let demo_tooltip = shadcn::Tooltip::new(
                shadcn::Button::new("Hover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-tooltip-demo-trigger")
                    .into_element(cx),
                shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
                    cx,
                    "Add to library",
                )])
                .into_element(cx),
            )
            .arrow(true)
            .arrow_test_id("ui-gallery-tooltip-demo-arrow")
            .side(shadcn::TooltipSide::Top)
            .panel_test_id("ui-gallery-tooltip-demo-panel")
            .into_element(cx)
            .test_id("ui-gallery-tooltip-demo");

            let focus_start = shadcn::Button::new("Focus Start")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-tooltip-focus-start")
                .into_element(cx);
            let focus_tooltip = shadcn::Tooltip::new(
                shadcn::Button::new("Focus Trigger")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-tooltip-focus-trigger")
                    .into_element(cx),
                shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
                    cx,
                    "Opens on keyboard focus",
                )
                .test_id("ui-gallery-tooltip-focus-text")])
                .into_element(cx),
            )
            .arrow(true)
            .arrow_test_id("ui-gallery-tooltip-focus-arrow")
            .side(shadcn::TooltipSide::Top)
            .panel_test_id("ui-gallery-tooltip-focus-panel")
            .into_element(cx)
            .test_id("ui-gallery-tooltip-focus");

            let focus_row = stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N2).items_center(),
                |_cx| vec![focus_start, focus_tooltip],
            )
            .test_id("ui-gallery-tooltip-focus-row");

            let side_row = stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N2).items_center(),
                |cx| {
                    vec![
                        make_tooltip(cx, "Left", shadcn::TooltipSide::Left, "Add to library"),
                        make_tooltip(cx, "Top", shadcn::TooltipSide::Top, "Add to library"),
                        make_tooltip(cx, "Bottom", shadcn::TooltipSide::Bottom, "Add to library"),
                        make_tooltip(cx, "Right", shadcn::TooltipSide::Right, "Add to library"),
                    ]
                },
            )
            .test_id("ui-gallery-tooltip-sides");

            let keyboard_icon = doc_layout::icon(cx, "lucide.save")
                .test_id("ui-gallery-tooltip-keyboard-icon");
            let keyboard_trigger = shadcn::Button::new("")
                .a11y_label("Save")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::IconSm)
                .children([keyboard_icon])
                .test_id("ui-gallery-tooltip-keyboard-trigger")
                .into_element(cx);
            let keyboard_tooltip = shadcn::Tooltip::new(
                keyboard_trigger,
                shadcn::TooltipContent::new(vec![stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            cx.text("Save Changes"),
                            shadcn::Kbd::new("S").into_element(cx),
                        ]
                    },
                )])
                .into_element(cx),
            )
            .side(shadcn::TooltipSide::Top)
            .into_element(cx)
            .test_id("ui-gallery-tooltip-keyboard");

            let disabled_trigger =
                stack::hstack(cx, stack::HStackProps::default().items_center(), |cx| {
                    vec![
                        shadcn::Button::new("Disabled")
                            .variant(shadcn::ButtonVariant::Outline)
                            .disabled(true)
                            .into_element(cx),
                    ]
                });
            let disabled_tooltip = shadcn::Tooltip::new(
                disabled_trigger,
                shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
                    cx,
                    "This feature is currently unavailable",
                )])
                .into_element(cx),
            )
            .side(shadcn::TooltipSide::Top)
            .into_element(cx)
            .test_id("ui-gallery-tooltip-disabled");

            let rtl_row = doc_layout::rtl(cx, |cx| {
                    stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                make_tooltip(
                                    cx,
                                    "يسار",
                                    shadcn::TooltipSide::Left,
                                    "إضافة إلى المكتبة",
                                ),
                                make_tooltip_with_test_ids(
                                    cx,
                                    "أعلى",
                                    "ui-gallery-tooltip-rtl-top-trigger",
                                    shadcn::TooltipSide::Top,
                                    "إضافة إلى المكتبة",
                                    "ui-gallery-tooltip-rtl-top-panel",
                                    "ui-gallery-tooltip-rtl-top-text",
                                ),
                                make_tooltip(
                                    cx,
                                    "أسفل",
                                    shadcn::TooltipSide::Bottom,
                                    "إضافة إلى المكتبة",
                                ),
                                make_tooltip(
                                    cx,
                                    "يمين",
                                    shadcn::TooltipSide::Right,
                                    "إضافة إلى المكتبة",
                                ),
                            ]
                        },
                    )
                })
                .test_id("ui-gallery-tooltip-rtl");

            let notes = doc_layout::notes(
                cx,
                [
                    "Wrap related tooltips in one TooltipProvider to get consistent delay-group behavior.",
                    "Use concise content in tooltip panels; longer explanations should move to Popover or Dialog.",
                    "For disabled actions, use a non-disabled wrapper as trigger so hover/focus feedback still works.",
                    "Keep tooltip content keyboard-accessible: focus the trigger and verify `aria-describedby`.",
                ],
            );

            let page = doc_layout::render_doc_page(
                cx,
                Some("Preview follows shadcn Tooltip docs order for quick visual lookup."),
                vec![
                    DocSection::new("Demo", demo_tooltip)
                .description("Basic tooltip with an arrow and a short content label.")
                .code(
                    "rust",
                    r#"let tooltip = shadcn::Tooltip::new(
    shadcn::Button::new("Hover")
        .variant(shadcn::ButtonVariant::Outline)
        .into_element(cx),
    shadcn::TooltipContent::new([shadcn::TooltipContent::text(cx, "Add to library")])
        .into_element(cx),
)
.arrow(true)
.side(shadcn::TooltipSide::Top)
.into_element(cx);"#,
                ),
                    DocSection::new("Keyboard Focus", focus_row)
                        .description("Tooltips should open when the trigger receives keyboard focus.")
                        .code(
                            "rust",
                            r#"shadcn::Tooltip::new(
    shadcn::Button::new("Focus Trigger")
        .variant(shadcn::ButtonVariant::Outline)
        .into_element(cx),
    shadcn::TooltipContent::new([shadcn::TooltipContent::text(
        cx,
        "Opens on keyboard focus",
    )])
    .into_element(cx),
)
.side(shadcn::TooltipSide::Top)
.arrow(true)
.into_element(cx);"#,
                        ),
                    DocSection::new("Side", side_row)
                        .description("Tooltips can be placed on the four sides of the trigger.")
                        .code(
                            "rust",
                            r#"for side in [
    shadcn::TooltipSide::Left,
    shadcn::TooltipSide::Top,
    shadcn::TooltipSide::Bottom,
    shadcn::TooltipSide::Right,
] {
    shadcn::Tooltip::new(trigger, content)
        .side(side)
        .into_element(cx);
}"#,
                        ),
                    DocSection::new("With Keyboard Shortcut", keyboard_tooltip).description(
                        "Compose richer content (e.g. key hints) inside the tooltip panel.",
                    )
                    .code(
                        "rust",
                        r#"shadcn::Tooltip::new(
    shadcn::Button::new("")
        .a11y_label("Save")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::IconSm)
        .icon(fret_icons::IconId::new_static("lucide.save"))
        .into_element(cx),
    shadcn::TooltipContent::new(vec![stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| vec![cx.text("Save Changes"), shadcn::Kbd::new("S").into_element(cx)],
    )])
    .into_element(cx),
)
.side(shadcn::TooltipSide::Top)
.arrow(true)
.into_element(cx);"#,
                    ),
                    DocSection::new("Disabled Button", disabled_tooltip).description(
                        "Use a non-disabled wrapper as the trigger so hover/focus can still open the tooltip.",
                    )
                    .code(
                        "rust",
                        r#"let trigger = stack::hstack(
    cx,
    stack::HStackProps::default().items_center(),
    |cx| {
        vec![shadcn::Button::new("Disabled")
            .variant(shadcn::ButtonVariant::Outline)
            .disabled(true)
            .into_element(cx)]
    },
);

shadcn::Tooltip::new(
    trigger,
    shadcn::TooltipContent::new([shadcn::TooltipContent::text(cx, "Unavailable")])
        .into_element(cx),
)
.side(shadcn::TooltipSide::Top)
.into_element(cx);"#,
                    ),
                    DocSection::new("RTL", rtl_row)
                        .description("Tooltip placement and alignment should work under RTL.")
                        .code(
                            "rust",
                            r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| shadcn::Tooltip::new(trigger, content)
        .side(shadcn::TooltipSide::Left)
        .into_element(cx),
);"#,
                        ),
                    DocSection::new("Notes", notes)
                        .description("Implementation notes and regression guidelines."),
                ],
            )
            .test_id("ui-gallery-tooltip-component");

            vec![page]
        })
        .into_iter()
        .next()
        .expect("tooltip provider returns one root element");

    vec![body]
}
