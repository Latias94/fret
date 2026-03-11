pub const SOURCE: &str = include_str!("borders.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let accordion = shadcn::Accordion::single_uncontrolled(Some("item-1"))
        .collapsible(true)
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(384.0))
                .min_w_0(),
        )
        .items([shadcn::AccordionItem::new(
            "item-1",
            shadcn::AccordionTrigger::new(vec![cx.text("Borders")]),
            shadcn::AccordionContent::new(vec![shadcn::raw::typography::p(
                cx,
                "Use an outer chrome wrapper when you want a bordered surface.",
            )]),
        )])
        .into_element(cx);

    let props = cx.with_theme(|theme| {
        decl_style::container_props(
            theme,
            ChromeRefinement::default().border_1().rounded(Radius::Md),
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(384.0))
                .min_w_0()
                .overflow_visible(),
        )
    });

    cx.container(props, move |_cx| [accordion])
        .test_id("ui-gallery-accordion-borders")
}
// endregion: example
