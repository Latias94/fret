pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Accordion::single_uncontrolled(Some("item-1"))
            .collapsible(true)
            .refine_layout(
                LayoutRefinement::default()
                    .w_full()
                    .max_w(Px(640.0))
                    .min_w_0(),
            )
            .items([shadcn::AccordionItem::new(
                "item-1",
                shadcn::AccordionTrigger::new(vec![cx.text("RTL")])
                    .test_id("ui-gallery-accordion-rtl-trigger"),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p("Ensure icons and spacing mirror correctly under RTL.")
                ]),
            )])
            .into_element(cx)
    })
    .test_id("ui-gallery-accordion-rtl")
}
// endregion: example
