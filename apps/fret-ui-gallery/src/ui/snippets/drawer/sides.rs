pub const SOURCE: &str = include_str!("sides.rs");

// region: example
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    top_open: Option<Model<bool>>,
    right_open: Option<Model<bool>>,
    bottom_open: Option<Model<bool>>,
    left_open: Option<Model<bool>>,
}

fn models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Models {
    cx.with_state(Models::default, |st| st.clone())
}

fn ensure_open<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Option<Model<bool>>,
    set: impl FnOnce(&mut Models, Model<bool>),
) -> Model<bool> {
    match model {
        Some(model) => model,
        None => {
            let inserted = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| set(st, inserted.clone()));
            inserted
        }
    }
}

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
    let st = models(cx);
    let top_open = ensure_open(cx, st.top_open, |st, model| st.top_open = Some(model));
    let right_open = ensure_open(cx, st.right_open, |st, model| st.right_open = Some(model));
    let bottom_open = ensure_open(cx, st.bottom_open, |st, model| st.bottom_open = Some(model));
    let left_open = ensure_open(cx, st.left_open, |st, model| st.left_open = Some(model));

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
