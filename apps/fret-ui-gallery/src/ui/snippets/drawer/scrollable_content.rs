pub const SOURCE: &str = include_str!("scrollable_content.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn paragraph_block<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    prefix: &'static str,
    rows: usize,
) -> impl IntoUiElement<H> + use<H> {
    let _ = cx;
    ui::v_flex(move |cx| {
        (0..rows)
            .map(|index| {
                cx.text(format!(
                    "{prefix} {}: Drawer scroll content for parity checks.",
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
    let open = cx.local_model(|| false);
    let trigger_open = open.clone();

    shadcn::Drawer::new(open)
        .direction(shadcn::DrawerDirection::Right)
        .children([
            shadcn::DrawerPart::trigger(shadcn::DrawerTrigger::build(
                shadcn::Button::new("Scrollable Content")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(trigger_open.clone())
                    .test_id("ui-gallery-drawer-scrollable-trigger"),
            )),
            shadcn::DrawerPart::content_with(move |cx| {
                let scroller = shadcn::ScrollArea::new(ui::children![
                    cx;
                    paragraph_block(cx, "Scrollable", 14)
                ])
                .refine_layout(
                    LayoutRefinement::default()
                        .w_full()
                        .h_px(Px(220.0))
                        .min_w_0()
                        .min_h_0(),
                )
                .viewport_test_id("ui-gallery-drawer-scrollable-viewport")
                .into_element(cx);

                let padded = {
                    let theme = Theme::global(&*cx.app);
                    let props = decl_style::container_props(
                        theme,
                        ChromeRefinement::default().px(Space::N4),
                        LayoutRefinement::default().w_full(),
                    );
                    cx.container(props, move |_cx| [scroller])
                };

                shadcn::DrawerContent::new([])
                    .with_children(cx, |cx| {
                        vec![
                            shadcn::DrawerHeader::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::DrawerTitle::new("Scrollable Content").into_element(cx),
                                    shadcn::DrawerDescription::new(
                                        "Keep actions visible while the content scrolls.",
                                    )
                                    .into_element(cx),
                                ]
                            }),
                            padded,
                            shadcn::DrawerFooter::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::Button::new("Submit").into_element(cx),
                                    shadcn::DrawerClose::from_scope().build(
                                        cx,
                                        shadcn::Button::new("Cancel")
                                            .variant(shadcn::ButtonVariant::Outline),
                                    ),
                                ]
                            }),
                        ]
                    })
                    .test_id("ui-gallery-drawer-scrollable-content")
            }),
        ])
        .into_element(cx)
}
// endregion: example
