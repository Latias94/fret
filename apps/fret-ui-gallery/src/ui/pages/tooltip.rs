use super::super::*;
use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_tooltip(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
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
        .open_delay_frames(10)
        .close_delay_frames(10)
        .into_element(cx)
    };

    let body = shadcn::TooltipProvider::new()
        .delay_duration_frames(10)
        .skip_delay_duration_frames(5)
        .with(cx, |cx| {
            let demo_tooltip = shadcn::Tooltip::new(
                shadcn::Button::new("Hover")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
                shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
                    cx,
                    "Add to library",
                )])
                .into_element(cx),
            )
            .arrow(true)
            .side(shadcn::TooltipSide::Top)
            .open_delay_frames(10)
            .close_delay_frames(10)
            .into_element(cx)
            .test_id("ui-gallery-tooltip-demo");

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

            let keyboard_tooltip = shadcn::Tooltip::new(
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::IconSm)
                    .children([shadcn::icon::icon(
                        cx,
                        fret_icons::IconId::new_static("lucide.save"),
                    )])
                    .into_element(cx),
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
            .open_delay_frames(10)
            .close_delay_frames(10)
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
            .open_delay_frames(10)
            .close_delay_frames(10)
            .into_element(cx)
            .test_id("ui-gallery-tooltip-disabled");

            let rtl_row = fret_ui_kit::primitives::direction::with_direction_provider(
                cx,
                fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
                |cx| {
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
                                make_tooltip(
                                    cx,
                                    "أعلى",
                                    shadcn::TooltipSide::Top,
                                    "إضافة إلى المكتبة",
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
                },
            )
            .test_id("ui-gallery-tooltip-rtl");

            let notes = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N2)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                |cx| {
                    vec![
                        shadcn::typography::muted(
                            cx,
                            "Wrap related tooltips in one TooltipProvider to get consistent delay-group behavior.",
                        ),
                        shadcn::typography::muted(
                            cx,
                            "Use concise content in tooltip panels; longer explanations should move to Popover or Dialog.",
                        ),
                        shadcn::typography::muted(
                            cx,
                            "For disabled actions, use a non-disabled wrapper as trigger so hover/focus feedback still works.",
                        ),
                        shadcn::typography::muted(
                            cx,
                            "Keep tooltip content keyboard-accessible: focus the trigger and verify `aria-describedby`.",
                        ),
                    ]
                },
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
.open_delay_frames(10)
.close_delay_frames(10)
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
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::IconSm)
        .children([shadcn::icon::icon(
            cx,
            fret_icons::IconId::new_static("lucide.save"),
        )])
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
