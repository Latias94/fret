pub const SOURCE: &str = include_str!("multiple.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::accordion_multiple_uncontrolled(cx, ["notifications"], |cx| {
        [
            shadcn::AccordionItem::new(
                "notifications",
                shadcn::AccordionTrigger::new(vec![cx.text("Notifications")])
                    .test_id("ui-gallery-accordion-multiple-trigger-notifications"),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p("Configure email, push, and in-app notifications.")
                ])
                .test_id("ui-gallery-accordion-multiple-content-notifications"),
            ),
            shadcn::AccordionItem::new(
                "security",
                shadcn::AccordionTrigger::new(vec![cx.text("Security")])
                    .test_id("ui-gallery-accordion-multiple-trigger-security"),
                shadcn::AccordionContent::new(ui::children![
                    cx;
                    shadcn::raw::typography::p("Manage passwords, 2FA, and active sessions.")
                ]),
            ),
        ]
    })
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(512.0))
            .min_w_0(),
    )
    .into_element(cx)
    .test_id("ui-gallery-accordion-multiple")
}
// endregion: example
