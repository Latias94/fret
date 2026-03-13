pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::accordion_single_uncontrolled(cx, Some("item-1"), |cx| {
            [shadcn::AccordionItem::new(
                "item-1",
                shadcn::AccordionTrigger::new(vec![cx.text("RTL")])
                    .test_id("ui-gallery-accordion-rtl-trigger"),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p("Ensure icons and spacing mirror correctly under RTL.")
                ]),
            )]
        })
            .collapsible(true)
            .refine_layout(
                LayoutRefinement::default()
                    .w_full()
                    .max_w(Px(640.0))
                    .min_w_0(),
            )
            .into_element(cx)
    })
    .test_id("ui-gallery-accordion-rtl")
}
// endregion: example
