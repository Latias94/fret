pub const SOURCE: &str = include_str!("controls.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    checkbox: Option<Model<bool>>,
    switch: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (checkbox, switch) = cx.with_state(Models::default, |st| {
        (st.checkbox.clone(), st.switch.clone())
    });
    let (checkbox, switch) = match (checkbox, switch) {
        (Some(checkbox), Some(switch)) => (checkbox, switch),
        _ => {
            let checkbox = cx.app.models_mut().insert(false);
            let switch = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.checkbox = Some(checkbox.clone());
                st.switch = Some(switch.clone());
            });
            (checkbox, switch)
        }
    };

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
