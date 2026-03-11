pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    rtl: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let rtl = cx.with_state(Models::default, |st| st.rtl.clone());
    let rtl = rtl.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(true);
        cx.with_state(Models::default, |st| st.rtl = Some(model.clone()));
        model
    });

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Field::new([
            shadcn::Checkbox::new(rtl)
                .control_id("ui-gallery-checkbox-rtl")
                .a11y_label("RTL notifications")
                .test_id("ui-gallery-checkbox-rtl")
                .into_element(cx),
            shadcn::FieldLabel::new("Enable notifications (RTL)")
                .for_control("ui-gallery-checkbox-rtl")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(420.0)))
        .into_element(cx)
    })
    .test_id("ui-gallery-checkbox-rtl-field")
}
// endregion: example
