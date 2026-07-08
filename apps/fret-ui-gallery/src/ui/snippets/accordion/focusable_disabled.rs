pub const SOURCE: &str = include_str!("focusable_disabled.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::accordion_single_uncontrolled(cx, Some("item-1"), |cx| {
        [shadcn::AccordionItem::new(
            "item-1",
            shadcn::AccordionTrigger::new(vec![decl_text::text_button_label(
                cx,
                "Open non-collapsible",
            )])
                .test_id("ui-gallery-accordion-focusable-disabled-trigger"),
            shadcn::AccordionContent::new(ui::children![
                cx;
                shadcn::typography::p(
                    "The open trigger is aria-disabled for assistive tech, but it should still keep its focus route while suppressing activation."
                )
            ])
            .test_id("ui-gallery-accordion-focusable-disabled-panel"),
        )]
    })
    .collapsible(false)
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(384.0))
            .min_w_0(),
    )
    .into_element(cx)
    .test_id("ui-gallery-accordion-focusable-disabled")
}
// endregion: example
