// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N6)
            .items_center()
            .layout(LayoutRefinement::default().w_full())
            .justify_center(),
        |cx| {
            vec![
                shadcn::Popover::new_controllable(cx, None, false)
                    .align(shadcn::PopoverAlign::Start)
                    .into_element(
                        cx,
                        |cx| {
                            let trigger = shadcn::Button::new("Start")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .into_element(cx)
                                .test_id("ui-gallery-popover-align-start-trigger");
                            shadcn::PopoverTrigger::new(trigger).into_element(cx)
                        },
                        |cx| {
                            shadcn::PopoverContent::new([cx.text("Aligned to start")])
                                .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                .into_element(cx)
                                .test_id("ui-gallery-popover-align-start-content")
                        },
                    ),
                shadcn::Popover::new_controllable(cx, None, false)
                    .align(shadcn::PopoverAlign::Center)
                    .into_element(
                        cx,
                        |cx| {
                            let trigger = shadcn::Button::new("Center")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .into_element(cx)
                                .test_id("ui-gallery-popover-align-center-trigger");
                            shadcn::PopoverTrigger::new(trigger).into_element(cx)
                        },
                        |cx| {
                            shadcn::PopoverContent::new([cx.text("Aligned to center")])
                                .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                .into_element(cx)
                                .test_id("ui-gallery-popover-align-center-content")
                        },
                    ),
                shadcn::Popover::new_controllable(cx, None, false)
                    .align(shadcn::PopoverAlign::End)
                    .into_element(
                        cx,
                        |cx| {
                            let trigger = shadcn::Button::new("End")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .into_element(cx)
                                .test_id("ui-gallery-popover-align-end-trigger");
                            shadcn::PopoverTrigger::new(trigger).into_element(cx)
                        },
                        |cx| {
                            shadcn::PopoverContent::new([cx.text("Aligned to end")])
                                .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
                                .into_element(cx)
                                .test_id("ui-gallery-popover-align-end-content")
                        },
                    ),
            ]
        },
    )
    .test_id("ui-gallery-popover-align")
}
// endregion: example
