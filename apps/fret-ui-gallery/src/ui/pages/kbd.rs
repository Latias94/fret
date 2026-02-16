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
        let content = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Kbd::new("Ctrl").into_element(cx),
                    shadcn::Kbd::new("K").into_element(cx),
                    shadcn::Kbd::new("Enter").into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-kbd-demo");
        content
    };

    let group = {
        let content = shadcn::KbdGroup::new([
            shadcn::Kbd::new("Cmd").into_element(cx),
            shadcn::Kbd::new("Shift").into_element(cx),
            shadcn::Kbd::new("P").into_element(cx),
        ])
        .into_element(cx)
        .test_id("ui-gallery-kbd-group");
        content
    };

    let button = {
        let content = shadcn::Button::new("Command Palette")
            .variant(shadcn::ButtonVariant::Outline)
            .children([shadcn::KbdGroup::new([
                shadcn::Kbd::new("Cmd").into_element(cx),
                shadcn::Kbd::new("K").into_element(cx),
            ])
            .into_element(cx)])
            .on_click(CMD_APP_OPEN)
            .into_element(cx)
            .test_id("ui-gallery-kbd-button");
        content
    };

    let tooltip = {
        let content = shadcn::TooltipProvider::new()
            .delay_duration_frames(10)
            .skip_delay_duration_frames(5)
            .with(cx, |cx| {
                vec![
                    shadcn::Tooltip::new(
                        shadcn::Button::new("Save")
                            .variant(shadcn::ButtonVariant::Outline)
                            .into_element(cx),
                        shadcn::TooltipContent::new(vec![stack::hstack(
                            cx,
                            stack::HStackProps::default().gap(Space::N2).items_center(),
                            |cx| {
                                vec![
                                    cx.text("Save file"),
                                    shadcn::Kbd::new("Cmd").into_element(cx),
                                    shadcn::Kbd::new("S").into_element(cx),
                                ]
                            },
                        )])
                        .into_element(cx),
                    )
                    .arrow(true)
                    .open_delay_frames(10)
                    .close_delay_frames(10)
                    .into_element(cx)
                    .test_id("ui-gallery-kbd-tooltip"),
                ]
            })
            .into_iter()
            .next()
            .expect("kbd tooltip provider should return one root");

        content
    };

    let input_group = {
        let content = shadcn::InputGroup::new(input_group_value)
            .a11y_label("Search")
            .trailing([shadcn::KbdGroup::new([
                shadcn::Kbd::new("Ctrl").into_element(cx),
                shadcn::Kbd::new("K").into_element(cx),
            ])
            .into_element(cx)])
            .trailing_has_kbd(true)
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(360.0)))
            .into_element(cx)
            .test_id("ui-gallery-kbd-input-group");

        content
    };

    let rtl = {
        let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::KbdGroup::new([
                    shadcn::Kbd::new("Ctrl").into_element(cx),
                    shadcn::Kbd::new("Shift").into_element(cx),
                    shadcn::Kbd::new("B").into_element(cx),
                ])
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-kbd-rtl");

        rtl_content
    };

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
                    "Kbd uses tokenized muted surfaces and is intended for shortcut display rather than free text chips.",
                ),
                shadcn::typography::muted(
                    cx,
                    "`Tooltip` and `Input Group` examples are composition patterns from shadcn docs.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Each section has stable test_id for future diag scripts.",
                ),
            ]
        },
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Kbd docs order: Demo, Group, Button, Tooltip, Input Group, RTL."),
        vec![
            DocSection::new("Demo", demo)
                .description("Basic kbd tokens for a shortcut chord.")
                .code("rust", r#"shadcn::Kbd::new("Ctrl").into_element(cx);"#),
            DocSection::new("Group", group)
                .description("Use `KbdGroup` to keep spacing consistent across tokens.")
                .code(
                    "rust",
                    r#"shadcn::KbdGroup::new([
    shadcn::Kbd::new("Cmd").into_element(cx),
    shadcn::Kbd::new("K").into_element(cx),
])
.into_element(cx);"#,
                ),
            DocSection::new("Button", button)
                .description("kbd tokens can be composed into button labels for discoverability."),
            DocSection::new("Tooltip", tooltip)
                .description("Tooltips often include shortcut hints for expert users."),
            DocSection::new("Input Group", input_group)
                .description("Trailing kbd hints can be rendered inside an input group.")
                .code(
                    "rust",
                    r#"shadcn::InputGroup::new(query)
    .trailing([shadcn::KbdGroup::new([/* ... */]).into_element(cx)])
    .trailing_has_kbd(true);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("kbd token order should respect right-to-left direction context."),
            DocSection::new("Notes", notes)
                .description("Implementation notes and regression guidelines."),
        ],
    )
    .test_id("ui-gallery-kbd-component");

    vec![body]
}
