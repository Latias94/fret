pub const SOURCE: &str = include_str!("grid.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let first = cx.local_model_keyed("first", String::new);
    let last = cx.local_model_keyed("last", String::new);
    let max_w_sm = LayoutRefinement::default().w_full().max_w(Px(420.0));
    let first_id = "ui-gallery-input-grid-first-name";
    let last_id = "ui-gallery-input-grid-last-name";

    ui::h_row(|cx| {
        vec![
            shadcn::Field::new([
                shadcn::FieldLabel::new("First Name")
                    .for_control(first_id)
                    .into_element(cx),
                shadcn::Input::new(first)
                    .control_id(first_id)
                    .placeholder("Jordan")
                    .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("Last Name")
                    .for_control(last_id)
                    .into_element(cx),
                shadcn::Input::new(last)
                    .control_id(last_id)
                    .placeholder("Lee")
                    .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        ]
    })
    .layout(max_w_sm)
    .gap(Space::N4)
    .items_start()
    .into_element(cx)
    .test_id("ui-gallery-input-grid")
}
// endregion: example
