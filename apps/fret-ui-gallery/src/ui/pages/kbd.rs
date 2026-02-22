use super::super::*;
use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_kbd(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct KbdPageModels {
        input_group_value: Option<Model<String>>,
    }

    let input_group_value =
        cx.with_state(KbdPageModels::default, |st| st.input_group_value.clone());
    let input_group_value = match input_group_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(KbdPageModels::default, |st| {
                st.input_group_value = Some(model.clone())
            });
            model
        }
    };

    let demo = {
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N4).items_center(),
            |cx| {
                vec![
                    shadcn::KbdGroup::new([
                        shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
                            cx,
                            fret_icons::IconId::new_static("lucide.command"),
                        )])
                        .into_element(cx),
                        shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
                            cx,
                            fret_icons::IconId::new_static("lucide.arrow-big-up"),
                        )])
                        .into_element(cx),
                        shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
                            cx,
                            fret_icons::IconId::new_static("lucide.option"),
                        )])
                        .into_element(cx),
                        shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
                            cx,
                            fret_icons::IconId::new_static("lucide.chevron-up"),
                        )])
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::KbdGroup::new([
                        shadcn::Kbd::new("Ctrl").into_element(cx),
                        ui::text(cx, "+").into_element(cx),
                        shadcn::Kbd::new("B").into_element(cx),
                    ])
                    .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-kbd-demo")
    };

    let group = {
        let theme = Theme::global(&*cx.app).clone();
        let muted_fg = theme.color_token("muted-foreground");

        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N1).items_center(),
            |cx| {
                vec![
                    ui::text(cx, "Use")
                        .text_sm()
                        .text_color(ColorRef::Color(muted_fg))
                        .into_element(cx),
                    shadcn::KbdGroup::new([
                        shadcn::Kbd::new("Ctrl + B").into_element(cx),
                        shadcn::Kbd::new("Ctrl + K").into_element(cx),
                    ])
                    .into_element(cx),
                    ui::text(cx, "to open the command palette")
                        .text_sm()
                        .text_color(ColorRef::Color(muted_fg))
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-kbd-group")
    };

    let button = {
        let accept = shadcn::Button::new("Accept")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .refine_style(ChromeRefinement::default().pr(Space::N2))
            .children([shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
                cx,
                fret_icons::IconId::new_static("lucide.corner-down-left"),
            )])
            .into_element(cx)])
            .into_element(cx)
            .test_id("ui-gallery-kbd-button-accept");

        let cancel = shadcn::Button::new("Cancel")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .refine_style(ChromeRefinement::default().pr(Space::N2))
            .children([shadcn::Kbd::new("Esc").into_element(cx)])
            .into_element(cx)
            .test_id("ui-gallery-kbd-button-cancel");

        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            move |_cx| vec![accept, cancel],
        )
        .test_id("ui-gallery-kbd-button")
    };

    let tooltip = {
        shadcn::TooltipProvider::new()
            .delay_duration_frames(10)
            .skip_delay_duration_frames(5)
            .with(cx, |cx| {
                let save = shadcn::Tooltip::new(
                    shadcn::Button::new("Save")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .into_element(cx),
                    shadcn::TooltipContent::new(vec![stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                ui::text(cx, "Save Changes").into_element(cx),
                                shadcn::Kbd::new("S").into_element(cx),
                            ]
                        },
                    )])
                    .into_element(cx),
                )
                .arrow(true)
                .open_delay_frames(10)
                .close_delay_frames(10)
                .into_element(cx);

                let print = shadcn::Tooltip::new(
                    shadcn::Button::new("Print")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .into_element(cx),
                    shadcn::TooltipContent::new(vec![stack::hstack(
                        cx,
                        stack::HStackProps::default().gap(Space::N2).items_center(),
                        |cx| {
                            vec![
                                ui::text(cx, "Print Document").into_element(cx),
                                shadcn::KbdGroup::new([
                                    shadcn::Kbd::new("Ctrl").into_element(cx),
                                    shadcn::Kbd::new("P").into_element(cx),
                                ])
                                .into_element(cx),
                            ]
                        },
                    )])
                    .into_element(cx),
                )
                .arrow(true)
                .open_delay_frames(10)
                .close_delay_frames(10)
                .into_element(cx);

                vec![
                    shadcn::ButtonGroup::new([save.into(), print.into()])
                        .into_element(cx)
                        .test_id("ui-gallery-kbd-tooltip"),
                ]
            })
            .into_iter()
            .next()
            .expect("kbd tooltip provider should return one root")
    };

    let input_group = {
        let theme = Theme::global(&*cx.app).clone();
        let muted_fg = theme.color_token("muted-foreground");

        let search_icon = shadcn::icon::icon_with(
            cx,
            fret_icons::IconId::new_static("lucide.search"),
            Some(Px(16.0)),
            Some(ColorRef::Color(muted_fg)),
        );

        shadcn::InputGroup::new(input_group_value)
            .a11y_label("Search")
            .leading([search_icon])
            .trailing([
                shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
                    cx,
                    fret_icons::IconId::new_static("lucide.command"),
                )])
                .into_element(cx),
                shadcn::Kbd::new("K").into_element(cx),
            ])
            .trailing_has_kbd(true)
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(360.0)))
            .into_element(cx)
            .test_id("ui-gallery-kbd-input-group")
    };

    let rtl = doc_layout::rtl(cx, |cx| {
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N4).items_center(),
            |cx| {
                vec![
                    shadcn::KbdGroup::new([
                        shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
                            cx,
                            fret_icons::IconId::new_static("lucide.command"),
                        )])
                        .into_element(cx),
                        shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
                            cx,
                            fret_icons::IconId::new_static("lucide.arrow-big-up"),
                        )])
                        .into_element(cx),
                        shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
                            cx,
                            fret_icons::IconId::new_static("lucide.option"),
                        )])
                        .into_element(cx),
                        shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(
                            cx,
                            fret_icons::IconId::new_static("lucide.chevron-up"),
                        )])
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::KbdGroup::new([
                        shadcn::Kbd::new("Ctrl").into_element(cx),
                        ui::text(cx, "+").into_element(cx),
                        shadcn::Kbd::new("B").into_element(cx),
                    ])
                    .into_element(cx),
                ]
            },
        )
    })
    .test_id("ui-gallery-kbd-rtl");

    let notes = doc_layout::notes(
        cx,
        [
            "Kbd is a fixed-height control; text placement uses bounds-as-line-box to keep the glyph visually centered.",
            "Tooltip and Input Group examples follow the upstream shadcn docs structure (v4 / New York).",
            "Each section has stable test_id for diag scripts and future gates.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Kbd docs order: Demo, Group, Button, Tooltip, Input Group, RTL."),
        vec![
            DocSection::new("Demo", demo)
                .description("Two shortcut display patterns (icons and chord).")
                .code(
                    "rust",
                    r#"stack::vstack(cx, stack::VStackProps::default().gap(Space::N4).items_center(), |cx| {
    vec![
        shadcn::KbdGroup::new([
            shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(cx, fret_icons::IconId::new_static("lucide.command"))]).into_element(cx),
            shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(cx, fret_icons::IconId::new_static("lucide.arrow-big-up"))]).into_element(cx),
            shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(cx, fret_icons::IconId::new_static("lucide.option"))]).into_element(cx),
            shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(cx, fret_icons::IconId::new_static("lucide.chevron-up"))]).into_element(cx),
        ])
        .into_element(cx),
        shadcn::KbdGroup::new([
            shadcn::Kbd::new("Ctrl").into_element(cx),
            ui::text(cx, "+").into_element(cx),
            shadcn::Kbd::new("B").into_element(cx),
        ])
        .into_element(cx),
    ]
});"#,
                ),
            DocSection::new("Group", group)
                .description("Use `KbdGroup` to keep spacing consistent across tokens.")
                .code(
                    "rust",
                    r#"let muted_fg = Theme::global(&*cx.app).color_token("muted-foreground");

stack::hstack(cx, stack::HStackProps::default().gap(Space::N1).items_center(), |cx| {
    vec![
        ui::text(cx, "Use").text_sm().text_color(ColorRef::Color(muted_fg)).into_element(cx),
        shadcn::KbdGroup::new([
            shadcn::Kbd::new("Ctrl + B").into_element(cx),
            shadcn::Kbd::new("Ctrl + K").into_element(cx),
        ])
        .into_element(cx),
        ui::text(cx, "to open the command palette").text_sm().text_color(ColorRef::Color(muted_fg)).into_element(cx),
    ]
});"#,
                ),
            DocSection::new("Button", button)
                .description("kbd tokens can be composed into button labels for discoverability.")
                .code(
                    "rust",
                    r#"shadcn::Button::new("Accept")
    .variant(shadcn::ButtonVariant::Outline)
    .size(shadcn::ButtonSize::Sm)
    .refine_style(ChromeRefinement::default().pr(Space::N2))
    .children([shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(cx, fret_icons::IconId::new_static("lucide.corner-down-left"))]).into_element(cx)])
    .into_element(cx);"#,
                ),
            DocSection::new("Tooltip", tooltip)
                .description("Tooltips often include shortcut hints for expert users.")
                .code(
                    "rust",
                    r#"let save = shadcn::Tooltip::new(
    shadcn::Button::new("Save")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into_element(cx),
    shadcn::TooltipContent::new(vec![stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| vec![ui::text(cx, "Save Changes").into_element(cx), shadcn::Kbd::new("S").into_element(cx)],
    )])
    .into_element(cx),
)
.arrow(true)
.into_element(cx);"#,
                ),
            DocSection::new("Input Group", input_group)
                .description("Trailing kbd hints can be rendered inside an input group.")
                .code(
                    "rust",
                    r#"let muted_fg = Theme::global(&*cx.app).color_token("muted-foreground");
let search_icon = shadcn::icon::icon_with(
    cx,
    fret_icons::IconId::new_static("lucide.search"),
    Some(Px(16.0)),
    Some(ColorRef::Color(muted_fg)),
);

shadcn::InputGroup::new(query)
    .a11y_label("Search")
    .leading([search_icon])
    .trailing([
        shadcn::Kbd::from_children([shadcn::kbd::kbd_icon(cx, fret_icons::IconId::new_static("lucide.command"))]).into_element(cx),
        shadcn::Kbd::new("K").into_element(cx),
    ])
    .trailing_has_kbd(true);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("kbd token order should respect right-to-left direction context.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| shadcn::KbdGroup::new([
        shadcn::Kbd::new("Ctrl").into_element(cx),
        shadcn::Kbd::new("Shift").into_element(cx),
        shadcn::Kbd::new("B").into_element(cx),
    ])
    .into_element(cx),
);"#,
                ),
            DocSection::new("Notes", notes)
                .description("Implementation notes and regression guidelines."),
        ],
    )
    .test_id("ui-gallery-kbd-component");

    vec![body]
}
