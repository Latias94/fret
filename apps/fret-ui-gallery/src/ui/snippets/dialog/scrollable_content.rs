pub const SOURCE: &str = include_str!("scrollable_content.rs");

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
                let text = ui::raw_text(format!(
                    "{prefix} {}: This dialog row is intentionally verbose to validate scroll behavior and footer visibility.",
                    index + 1
                ))
                .layout(LayoutRefinement::default().w_full().min_w_0());

                if index == 0 {
                    text.test_id("ui-gallery-dialog-scrollable-row-01")
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

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);

    shadcn::Dialog::new(open.clone())
        .children([
            shadcn::DialogPart::trigger(shadcn::DialogTrigger::build(
                shadcn::Button::new("Scrollable Content")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dialog-scrollable-trigger"),
            )),
            shadcn::DialogPart::content_with(move |cx| {
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

                shadcn::DialogContent::new([])
                    .with_children(cx, |cx| {
                        vec![
                            shadcn::DialogHeader::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::DialogTitle::new("Scrollable Content").into_element(cx),
                                    shadcn::DialogDescription::new(
                                        "Long content can scroll while the header stays in view.",
                                    )
                                    .into_element(cx),
                                ]
                            }),
                            scroll_body,
                        ]
                    })
                    .test_id("ui-gallery-dialog-scrollable-content")
            }),
        ])
        .into_element(cx)
}
// endregion: example
