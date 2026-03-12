pub const SOURCE: &str = include_str!("checked_state.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let checked_controlled = cx.local_model_keyed("checked_controlled", || true);
    let checked_optional = cx.local_model_keyed("checked_optional", || None::<bool>);

    ui::v_flex(|cx| {
        vec![
            ui::h_flex(|cx| {
                vec![
                    shadcn::Checkbox::new(checked_controlled)
                        .control_id("ui-gallery-checkbox-controlled")
                        .a11y_label("Controlled checkbox")
                        .test_id("ui-gallery-checkbox-controlled")
                        .into_element(cx),
                    shadcn::FieldLabel::new("Controlled checked state")
                        .for_control("ui-gallery-checkbox-controlled")
                        .test_id("ui-gallery-checkbox-controlled-label")
                        .into_element(cx),
                ]
            })
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3)
            .items_center()
            .into_element(cx),
            ui::h_flex(|cx| {
                vec![
                    shadcn::Checkbox::new_optional(checked_optional)
                        .control_id("ui-gallery-checkbox-optional")
                        .a11y_label("Optional checkbox")
                        .test_id("ui-gallery-checkbox-optional")
                        .into_element(cx),
                    shadcn::FieldLabel::new("Optional / indeterminate state")
                        .for_control("ui-gallery-checkbox-optional")
                        .test_id("ui-gallery-checkbox-optional-label")
                        .into_element(cx),
                ]
            })
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3)
            .items_center()
            .into_element(cx),
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-checked-state")
}
// endregion: example
