pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    text_input: Option<Model<String>>,
    switch: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (text_input, switch) = cx.with_state(Models::default, |st| {
        (st.text_input.clone(), st.switch.clone())
    });
    let (text_input, switch) = match (text_input, switch) {
        (Some(text_input), Some(switch)) => (text_input, switch),
        _ => {
            let text_input = cx.app.models_mut().insert(String::new());
            let switch = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.text_input = Some(text_input.clone());
                st.switch = Some(switch.clone());
            });
            (text_input, switch)
        }
    };

    let max_w_md = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(520.0));

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::FieldSet::new([
            shadcn::FieldLegend::new("الملف الشخصي").into_element(cx),
            shadcn::FieldDescription::new("تحقق من محاذاة الحقول والنصوص تحت RTL.")
                .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("البريد الإلكتروني").into_element(cx),
                shadcn::Input::new(text_input.clone())
                    .a11y_label("البريد الإلكتروني")
                    .placeholder("name@example.com")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("تفعيل الإشعارات")
                    .for_control("ui-gallery-form-switch-rtl")
                    .into_element(cx),
                shadcn::Switch::new(switch.clone())
                    .control_id("ui-gallery-form-switch-rtl")
                    .a11y_label("تفعيل الإشعارات")
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx),
        ])
        .refine_layout(max_w_md)
        .into_element(cx)
    })
    .test_id("ui-gallery-form-rtl")
}
// endregion: example
