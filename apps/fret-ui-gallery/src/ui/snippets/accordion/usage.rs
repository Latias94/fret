pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::AccordionRoot::single_uncontrolled(Some("item-1"))
        .collapsible(true)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_w(Px(384.0)),
        )
        .children([shadcn::AccordionItemPart::new("item-1")
            .test_id("ui-gallery-accordion-usage-item")
            .trigger(
                shadcn::AccordionTriggerPart::new(vec![cx.text("Is it accessible?")])
                    .test_id("ui-gallery-accordion-usage-trigger"),
            )
            .content(
                shadcn::AccordionContentPart::new(ui::children![
                    cx;
                    shadcn::raw::typography::p("Yes. It adheres to the WAI-ARIA design pattern.")
                ])
                .test_id("ui-gallery-accordion-usage-panel"),
            )])
        .into_element(cx)
        .test_id("ui-gallery-accordion-usage")
}
// endregion: example
