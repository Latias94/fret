pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::accordion::composable as acc;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let item = acc::AccordionItem::new(Arc::from("item-1"))
        .trigger(acc::AccordionTrigger::new(vec![
            cx.text("Is it accessible?"),
        ]))
        .content(acc::AccordionContent::new(vec![
            shadcn::raw::typography::p("Yes. It adheres to the WAI-ARIA design pattern.")
                .into_element(cx),
        ]));

    acc::AccordionRoot::single_uncontrolled(Some("item-1"))
        .collapsible(true)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .max_w(Px(384.0)),
        )
        .items([item])
        .into_element(cx)
        .test_id("ui-gallery-accordion-usage")
}
// endregion: example
