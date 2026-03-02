pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    name: Option<Model<String>>,
    number: Option<Model<String>>,
}

fn ensure_models<H: UiHost>(cx: &mut ElementContext<'_, H>) -> (Model<String>, Model<String>) {
    let state = cx.with_state(Models::default, |st| st.clone());
    match (state.name, state.number) {
        (Some(name), Some(number)) => (name, number),
        _ => {
            let models = cx.app.models_mut();
            let name = models.insert(String::new());
            let number = models.insert(String::new());
            cx.with_state(Models::default, |st| {
                st.name = Some(name.clone());
                st.number = Some(number.clone());
            });
            (name, number)
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (rtl_name, rtl_number) = ensure_models(cx);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::FieldSet::new([
                shadcn::FieldLegend::new("طريقة الدفع").into_element(cx),
                shadcn::FieldDescription::new("جميع المعاملات آمنة ومشفرة").into_element(cx),
                shadcn::FieldGroup::new([
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("الاسم على البطاقة").into_element(cx),
                        shadcn::Input::new(rtl_name)
                            .a11y_label("الاسم على البطاقة")
                            .placeholder("Evil Rabbit")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("رقم البطاقة").into_element(cx),
                        shadcn::Input::new(rtl_number)
                            .a11y_label("رقم البطاقة")
                            .placeholder("1234 5678 9012 3456")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .refine_layout(max_w_md)
            .into_element(cx)
        },
    )
    .test_id("ui-gallery-field-rtl")
}
// endregion: example
