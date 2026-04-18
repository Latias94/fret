pub const SOURCE: &str = include_str!("align.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui::h_flex(|cx| {
        let start_content = shadcn::PopoverContent::build(cx, |cx| [cx.text("Aligned to start")])
            .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
            .test_id("ui-gallery-popover-align-start-content");
        let center_content = shadcn::PopoverContent::build(cx, |cx| [cx.text("Aligned to center")])
            .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
            .test_id("ui-gallery-popover-align-center-content");
        let end_content = shadcn::PopoverContent::build(cx, |cx| [cx.text("Aligned to end")])
            .refine_layout(LayoutRefinement::default().w_px(Px(160.0)))
            .test_id("ui-gallery-popover-align-end-content");

        vec![
            shadcn::Popover::new(
                cx,
                shadcn::PopoverTrigger::build(
                    shadcn::Button::new("Start")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .test_id("ui-gallery-popover-align-start-trigger"),
                ),
                start_content,
            )
            .align(shadcn::PopoverAlign::Start)
            .into_element(cx),
            shadcn::Popover::new(
                cx,
                shadcn::PopoverTrigger::build(
                    shadcn::Button::new("Center")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .test_id("ui-gallery-popover-align-center-trigger"),
                ),
                center_content,
            )
            .align(shadcn::PopoverAlign::Center)
            .into_element(cx),
            shadcn::Popover::new(
                cx,
                shadcn::PopoverTrigger::build(
                    shadcn::Button::new("End")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .test_id("ui-gallery-popover-align-end-trigger"),
                ),
                end_content,
            )
            .align(shadcn::PopoverAlign::End)
            .into_element(cx),
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
