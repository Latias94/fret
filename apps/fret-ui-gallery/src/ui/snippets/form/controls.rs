pub const SOURCE: &str = include_str!("controls.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let checkbox = cx.local_model_keyed("checkbox", || false);
    let switch = cx.local_model_keyed("switch", || false);

    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(520.0));

    ui::v_stack(|cx| {
        vec![
            ui::h_row(|cx| {
                vec![
                    shadcn::Checkbox::new(checkbox)
                        .a11y_label("Accept terms")
                        .into_element(cx),
                    shadcn::Label::new("Accept terms").into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
            ui::h_row(|cx| {
                vec![
                    shadcn::Switch::new(switch)
                        .a11y_label("Enable feature")
                        .into_element(cx),
                    shadcn::Label::new("Enable feature").into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
        ]
    })
    .gap(Space::N3)
    .layout(max_w_md)
    .items_start()
    .into_element(cx)
    .test_id("ui-gallery-form-controls")
}
// endregion: example
