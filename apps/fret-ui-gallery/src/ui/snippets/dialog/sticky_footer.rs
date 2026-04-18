pub const SOURCE: &str = include_str!("sticky_footer.rs");

// region: example
use fret::{AppComponentCx, UiChild};
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
                let text = ui::raw_text(format!(
                    "{prefix} {}: This dialog row is intentionally verbose to validate scroll behavior and footer visibility.",
                    index + 1
                ))
                .layout(LayoutRefinement::default().w_full().min_w_0());

                if index == 0 {
                    text.test_id("ui-gallery-dialog-sticky-footer-row-01")
                        .into_element(cx)
                } else {
                    text.into_element(cx)
                }
            })
            .collect::<Vec<_>>()
    })
    .gap(Space::N2)
    .items_stretch()
    .layout(LayoutRefinement::default().w_full().min_w_0())
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    shadcn::Dialog::new(open.clone())
        .children([
            shadcn::DialogPart::trigger(shadcn::DialogTrigger::build(
                shadcn::Button::new("Sticky Footer")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dialog-sticky-footer-trigger"),
            )),
            shadcn::DialogPart::content_with(move |cx| {
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

                shadcn::DialogContent::new([])
                    .with_children(cx, |cx| {
                        vec![
                            shadcn::DialogHeader::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::DialogTitle::new("Sticky Footer")
                                        .into_element(cx)
                                        .test_id("ui-gallery-dialog-sticky-footer-title"),
                                shadcn::DialogDescription::new(
                                    "This dialog has a sticky footer that stays visible while the content scrolls.",
                                )
                                .into_element(cx)
                                .test_id("ui-gallery-dialog-sticky-footer-description"),
                            ]
                            })
                            .test_id("ui-gallery-dialog-sticky-footer-header"),
                            scroll_body,
                            shadcn::DialogFooter::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::DialogClose::from_scope().build(
                                        cx,
                                        shadcn::Button::new("Close")
                                            .variant(shadcn::ButtonVariant::Outline)
                                            .test_id("ui-gallery-dialog-sticky-footer-close"),
                                    ),
                                ]
                            }),
                        ]
                    })
                    .test_id("ui-gallery-dialog-sticky-footer-content")
            }),
        ])
        .into_element(cx)
}
// endregion: example
