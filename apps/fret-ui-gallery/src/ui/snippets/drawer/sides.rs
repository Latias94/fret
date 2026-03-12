pub const SOURCE: &str = include_str!("sides.rs");

// region: example
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn side_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    title: &'static str,
    direction: shadcn::DrawerDirection,
    open: Model<bool>,
    test_id_prefix: &'static str,
) -> AnyElement {
    let open_for_trigger = open.clone();
    let open_for_close = open.clone();
    shadcn::Drawer::new(open).direction(direction).into_element(
        cx,
        move |cx| {
            shadcn::Button::new(title)
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(open_for_trigger.clone())
                .test_id(format!("{test_id_prefix}-trigger"))
                .into_element(cx)
        },
        move |cx| {
            shadcn::DrawerContent::new([
                shadcn::DrawerHeader::new([
                    shadcn::DrawerTitle::new(format!("{title} Drawer")).into_element(cx),
                    shadcn::DrawerDescription::new(
                        "Use the `direction` prop to control drawer placement.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::DrawerFooter::new([shadcn::Button::new("Close")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(open_for_close.clone())
                    .into_element(cx)])
                .into_element(cx),
            ])
            .into_element(cx)
            .test_id(format!("{test_id_prefix}-content"))
        },
    )
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
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
            ),
            side_button(
                cx,
                "Right",
                shadcn::DrawerDirection::Right,
                right_open.clone(),
                "ui-gallery-drawer-side-right",
            ),
            side_button(
                cx,
                "Bottom",
                shadcn::DrawerDirection::Bottom,
                bottom_open.clone(),
                "ui-gallery-drawer-side-bottom",
            ),
            side_button(
                cx,
                "Left",
                shadcn::DrawerDirection::Left,
                left_open.clone(),
                "ui-gallery-drawer-side-left",
            ),
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
