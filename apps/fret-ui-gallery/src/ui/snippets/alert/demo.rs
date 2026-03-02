// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn build_alert<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    test_id: &'static str,
    variant: shadcn::AlertVariant,
    icon_name: &'static str,
    title: &'static str,
    description: &'static str,
) -> AnyElement {
    shadcn::Alert::new([
        shadcn::icon::icon(cx, fret_icons::IconId::new_static(icon_name)),
        shadcn::AlertTitle::new(title).into_element(cx),
        shadcn::AlertDescription::new(description).into_element(cx),
    ])
    .variant(variant)
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
    .into_element(cx)
    .test_id(test_id)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                build_alert(
                    cx,
                    "ui-gallery-alert-demo-success",
                    shadcn::AlertVariant::Default,
                    "lucide.circle-check",
                    "Payment successful",
                    "Your payment of $29.99 has been processed and a receipt has been emailed.",
                ),
                build_alert(
                    cx,
                    "ui-gallery-alert-demo-info",
                    shadcn::AlertVariant::Default,
                    "lucide.info",
                    "New feature available",
                    "Dark mode support is now available in account settings.",
                ),
            ]
        },
    )
    .test_id("ui-gallery-alert-demo")
}
// endregion: example
