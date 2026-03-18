pub const SOURCE: &str = include_str!("sticky_footer.rs");

// region: example
use fret::{UiChild, UiCx};
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

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    let open_for_trigger = open.clone();
    shadcn::Dialog::new(open.clone()).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("Sticky Footer")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dialog-sticky-footer-trigger")
                .toggle_model(open_for_trigger.clone())
                .into_element(cx)
        },
        move |cx| {
            let scroll_body = shadcn::ScrollArea::new(ui::children![
                cx;
                lorem_block(cx, "Sticky", 14)
            ])
            .refine_layout(
                LayoutRefinement::default()
                    .w_full()
                    .h_px(Px(220.0))
                    .min_w_0()
                    .min_h_0(),
            )
            .viewport_test_id("ui-gallery-dialog-sticky-footer-viewport")
            .into_element(cx);

            shadcn::DialogContent::new(ui::children![
                cx;
                shadcn::DialogHeader::new(ui::children![
                    cx;
                    shadcn::DialogTitle::new("Sticky Footer"),
                    shadcn::DialogDescription::new(
                        "The footer remains visible while the content area scrolls.",
                    )
                ]),
                scroll_body,
                shadcn::DialogFooter::new(ui::children![
                    cx;
                    shadcn::DialogClose::from_scope().build(
                        cx,
                        shadcn::Button::new("Close").variant(shadcn::ButtonVariant::Outline),
                    ),
                    shadcn::Button::new("Save changes"),
                ]),
            ])
            .into_element(cx)
            .test_id("ui-gallery-dialog-sticky-footer-content")
        },
    )
}
// endregion: example
