pub const SOURCE: &str = include_str!("scrollable_content.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn lorem_block<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    prefix: &'static str,
    lines: usize,
) -> impl IntoUiElement<H> + use<H> {
    let _ = cx;
    ui::v_flex(move |cx| {
        (0..lines)
            .map(|index| {
                cx.text(format!(
                    "{prefix} {}: This dialog row is intentionally verbose to validate scroll behavior and footer visibility.",
                    index + 1
                ))
            })
            .collect::<Vec<_>>()
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full())
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.local_model_keyed("open", || false);

    let open_for_trigger = open.clone();
    shadcn::Dialog::new(open.clone()).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("Scrollable Content")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dialog-scrollable-trigger")
                .toggle_model(open_for_trigger.clone())
                .into_element(cx)
        },
        move |cx| {
            let scroll_body = shadcn::ScrollArea::new(ui::children![
                cx;
                lorem_block(cx, "Scrollable", 14)
            ])
            .refine_layout(
                LayoutRefinement::default()
                    .w_full()
                    .h_px(Px(240.0))
                    .min_w_0()
                    .min_h_0(),
            )
            .viewport_test_id("ui-gallery-dialog-scrollable-viewport")
            .into_element(cx);

            shadcn::DialogContent::new(ui::children![
                cx;
                shadcn::DialogHeader::new(ui::children![
                    cx;
                    shadcn::DialogTitle::new("Scrollable Content"),
                    shadcn::DialogDescription::new(
                        "Long content can scroll while the header stays in view.",
                    )
                ]),
                scroll_body,
            ])
            .into_element(cx)
            .test_id("ui-gallery-dialog-scrollable-content")
        },
    )
}
// endregion: example
