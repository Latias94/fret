pub const SOURCE: &str = include_str!("snap_points.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
}

fn open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = open_model(cx);
    let trigger_open = open.clone();
    let close_open = open.clone();

    shadcn::Drawer::new(open)
        .snap_points(vec![
            shadcn::DrawerSnapPoint::Fraction(0.25),
            shadcn::DrawerSnapPoint::Fraction(0.5),
            shadcn::DrawerSnapPoint::Fraction(1.0),
        ])
        .into_element(
            cx,
            move |cx| {
                shadcn::Button::new("Snap Points")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(trigger_open.clone())
                    .test_id("ui-gallery-drawer-snap-points-trigger")
                    .into_element(cx)
            },
            move |cx| {
                shadcn::DrawerContent::new([
                    shadcn::DrawerHeader::new([
                        shadcn::DrawerTitle::new("Snap Points").into_element(cx),
                        shadcn::DrawerDescription::new(
                            "Releasing a drag settles to the nearest snap point (Vaul-style).",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::DrawerFooter::new([shadcn::Button::new("Close")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(close_open.clone())
                        .into_element(cx)])
                    .into_element(cx),
                ])
                .drag_handle_test_id("ui-gallery-drawer-snap-points-handle")
                .into_element(cx)
                .test_id("ui-gallery-drawer-snap-points-content")
            },
        )
}
// endregion: example
