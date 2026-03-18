pub const SOURCE: &str = include_str!("sides.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::{IntoUiElement, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn side_button<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    title: &'static str,
    direction: shadcn::DrawerDirection,
    open: Model<bool>,
    test_id_prefix: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let open_for_trigger = open.clone();
    let open_for_close = open.clone();
    shadcn::Drawer::new(open)
        .direction(direction)
        .children([
            shadcn::DrawerPart::trigger(shadcn::DrawerTrigger::build(
                shadcn::Button::new(title)
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(open_for_trigger.clone())
                    .test_id(format!("{test_id_prefix}-trigger")),
            )),
            shadcn::DrawerPart::content_with(move |cx| {
                shadcn::DrawerContent::new(ui::children![
                    cx;
                    shadcn::DrawerHeader::new(ui::children![
                        cx;
                        shadcn::DrawerTitle::new(format!("{title} Drawer")),
                        shadcn::DrawerDescription::new(
                            "Use the `direction` prop to control drawer placement.",
                        )
                    ]),
                    shadcn::DrawerFooter::new(ui::children![
                        cx;
                        shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(open_for_close.clone())
                    ]),
                ])
                .into_element(cx)
                .test_id(format!("{test_id_prefix}-content"))
            }),
        ])
        .into_element(cx)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let top_open = cx.local_model_keyed("top_open", || false);
    let right_open = cx.local_model_keyed("right_open", || false);
    let bottom_open = cx.local_model_keyed("bottom_open", || false);
    let left_open = cx.local_model_keyed("left_open", || false);

    ui::h_flex(|cx| {
        vec![
            side_button(
                cx,
                "Top",
                shadcn::DrawerDirection::Top,
                top_open.clone(),
                "ui-gallery-drawer-side-top",
            )
            .into_element(cx),
            side_button(
                cx,
                "Right",
                shadcn::DrawerDirection::Right,
                right_open.clone(),
                "ui-gallery-drawer-side-right",
            )
            .into_element(cx),
            side_button(
                cx,
                "Bottom",
                shadcn::DrawerDirection::Bottom,
                bottom_open.clone(),
                "ui-gallery-drawer-side-bottom",
            )
            .into_element(cx),
            side_button(
                cx,
                "Left",
                shadcn::DrawerDirection::Left,
                left_open.clone(),
                "ui-gallery-drawer-side-left",
            )
            .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .wrap()
    .w_full()
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-drawer-sides")
}
// endregion: example
