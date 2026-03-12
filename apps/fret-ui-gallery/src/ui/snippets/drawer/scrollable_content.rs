pub const SOURCE: &str = include_str!("scrollable_content.rs");

// region: example
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn paragraph_block<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    prefix: &'static str,
    rows: usize,
) -> AnyElement {
    ui::v_flex(|cx| {
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
    .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.local_model(|| false);
    let trigger_open = open.clone();
    let close_open = open.clone();

    shadcn::Drawer::new(open)
        .direction(shadcn::DrawerDirection::Right)
        .into_element(
            cx,
            move |cx| {
                shadcn::Button::new("Scrollable Content")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(trigger_open.clone())
                    .test_id("ui-gallery-drawer-scrollable-trigger")
                    .into_element(cx)
            },
            move |cx| {
                let scroller = shadcn::ScrollArea::new([paragraph_block(cx, "Scrollable", 14)])
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

                shadcn::DrawerContent::new([
                    shadcn::DrawerHeader::new([
                        shadcn::DrawerTitle::new("Scrollable Content").into_element(cx),
                        shadcn::DrawerDescription::new(
                            "Keep actions visible while the content scrolls.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    padded,
                    shadcn::DrawerFooter::new([
                        shadcn::Button::new("Submit").into_element(cx),
                        shadcn::Button::new("Cancel")
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(close_open.clone())
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx)
                .test_id("ui-gallery-drawer-scrollable-content")
            },
        )
}
// endregion: example
