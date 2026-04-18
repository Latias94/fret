pub const SOURCE: &str = include_str!("disabled.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::accordion_single_uncontrolled(cx, Some("item-1"), |cx| {
        [shadcn::AccordionItem::new(
            "item-1",
            shadcn::AccordionTrigger::new(vec![cx.text("Disabled")]),
            shadcn::AccordionContent::new(ui::children![
                cx;
                shadcn::raw::typography::p("This item is disabled and should not be interactive.")
            ]),
        )
        .disabled(true)]
    })
    .collapsible(true)
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(384.0))
            .min_w_0(),
    )
    .into_element(cx)
    .test_id("ui-gallery-accordion-disabled")
}
// endregion: example
