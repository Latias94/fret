pub const SOURCE: &str = include_str!("sides.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::{IntoUiElement, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn side_body<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    title: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let _ = cx;
    ui::v_stack(move |cx| {
        (0..6)
            .map(|index| {
                cx.text(format!(
                    "{title} drawer example {}. Use the `direction` prop to control drawer placement.",
                    index + 1
                ))
            })
            .collect::<Vec<_>>()
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
}

fn side_button<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    title: &'static str,
    direction: shadcn::DrawerDirection,
    open: Model<bool>,
    test_id_prefix: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let open_for_trigger = open.clone();
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
                let body = ui::v_stack(|cx| vec![side_body(cx, title).into_element(cx)])
                    .px_4()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx);

                shadcn::DrawerContent::new([])
                    .children(|cx| {
                        ui::children![
                            cx;
                            shadcn::DrawerHeader::new([]).children(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::DrawerTitle::new(format!("{title} Drawer")),
                                    shadcn::DrawerDescription::new(
                                        "Use the `direction` prop to control drawer placement.",
                                    )
                                ]
                            }),
                            body,
                            shadcn::DrawerFooter::new([]).children(|cx| {
                                ui::children![
                                    cx;
                                    shadcn::Button::new("Submit"),
                                    shadcn::DrawerClose::from_scope().child(
                                        shadcn::Button::new("Cancel")
                                            .variant(shadcn::ButtonVariant::Outline),
                                    )
                                ]
                            })
                        ]
                    })
                    .test_id(format!("{test_id_prefix}-content"))
                    .into_element(cx)
            }),
        ])
        .into_element(cx)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
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
