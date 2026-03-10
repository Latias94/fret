pub const SOURCE: &str = include_str!("align.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ui::h_flex(|cx| {
        let start_content = shadcn::PopoverContent::new([cx.text("Aligned to start")])
            .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
            .test_id("ui-gallery-popover-align-start-content");
        let center_content = shadcn::PopoverContent::new([cx.text("Aligned to center")])
            .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
            .test_id("ui-gallery-popover-align-center-content");
        let end_content = shadcn::PopoverContent::new([cx.text("Aligned to end")])
            .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
            .test_id("ui-gallery-popover-align-end-content");

        vec![
            shadcn::Popover::new_controllable(cx, None, false)
                .align(shadcn::PopoverAlign::Start)
                .build(
                    cx,
                    shadcn::PopoverTrigger::build(
                        shadcn::Button::new("Start")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .test_id("ui-gallery-popover-align-start-trigger"),
                    ),
                    start_content,
                ),
            shadcn::Popover::new_controllable(cx, None, false)
                .align(shadcn::PopoverAlign::Center)
                .build(
                    cx,
                    shadcn::PopoverTrigger::build(
                        shadcn::Button::new("Center")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .test_id("ui-gallery-popover-align-center-trigger"),
                    ),
                    center_content,
                ),
            shadcn::Popover::new_controllable(cx, None, false)
                .align(shadcn::PopoverAlign::End)
                .build(
                    cx,
                    shadcn::PopoverTrigger::build(
                        shadcn::Button::new("End")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .test_id("ui-gallery-popover-align-end-trigger"),
                    ),
                    end_content,
                ),
        ]
    })
    .gap(Space::N6)
    .items_center()
    .layout(LayoutRefinement::default().w_full())
    .justify_center()
    .into_element(cx)
    .test_id("ui-gallery-popover-align")
}
// endregion: example
