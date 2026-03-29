pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_kit::ui::UiElementSinkExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let rtl_name = cx.local_model_keyed("rtl_name", String::new);
    let rtl_number = cx.local_model_keyed("rtl_number", String::new);
    let rtl_cvv = cx.local_model_keyed("rtl_cvv", String::new);
    let rtl_comments = cx.local_model_keyed("rtl_comments", String::new);
    let rtl_same_as_shipping = cx.local_model_keyed("rtl_same_as_shipping", || true);
    let rtl_expiry_month = cx.local_model_keyed("rtl_expiry_month", || None::<Arc<str>>);
    let rtl_expiry_month_open = cx.local_model_keyed("rtl_expiry_month_open", || false);
    let rtl_expiry_year = cx.local_model_keyed("rtl_expiry_year", || None::<Arc<str>>);
    let rtl_expiry_year_open = cx.local_model_keyed("rtl_expiry_year_open", || false);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));
    let name_id = "ui-gallery-field-rtl-card-name";
    let number_id = "ui-gallery-field-rtl-card-number";
    let month_id = "ui-gallery-field-rtl-expiry-month";
    let year_id = "ui-gallery-field-rtl-expiry-year";
    let cvv_id = "ui-gallery-field-rtl-cvv";
    let same_as_shipping_id = "ui-gallery-field-rtl-same-as-shipping";
    let comments_id = "ui-gallery-field-rtl-comments";

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let expiry_row = ui::h_flex(|cx| {
            vec![
                shadcn::Field::build(|cx, out| {
                    out.push_ui(cx, shadcn::FieldLabel::new("الشهر").for_control(month_id));
                    out.push_ui(
                        cx,
                        shadcn::Select::new(rtl_expiry_month, rtl_expiry_month_open)
                            .control_id(month_id)
                            .a11y_label("الشهر")
                            .value(shadcn::SelectValue::new().placeholder("ش.ش"))
                            .items([
                                shadcn::SelectItem::new("01", "٠١"),
                                shadcn::SelectItem::new("02", "٠٢"),
                                shadcn::SelectItem::new("03", "٠٣"),
                                shadcn::SelectItem::new("04", "٠٤"),
                                shadcn::SelectItem::new("05", "٠٥"),
                                shadcn::SelectItem::new("06", "٠٦"),
                                shadcn::SelectItem::new("07", "٠٧"),
                                shadcn::SelectItem::new("08", "٠٨"),
                                shadcn::SelectItem::new("09", "٠٩"),
                                shadcn::SelectItem::new("10", "١٠"),
                                shadcn::SelectItem::new("11", "١١"),
                                shadcn::SelectItem::new("12", "١٢"),
                            ]),
                    );
                })
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
                shadcn::Field::build(|cx, out| {
                    out.push_ui(cx, shadcn::FieldLabel::new("السنة").for_control(year_id));
                    out.push_ui(
                        cx,
                        shadcn::Select::new(rtl_expiry_year, rtl_expiry_year_open)
                            .control_id(year_id)
                            .a11y_label("السنة")
                            .value(shadcn::SelectValue::new().placeholder("YYYY"))
                            .items([
                                shadcn::SelectItem::new("2026", "٢٠٢٦"),
                                shadcn::SelectItem::new("2027", "٢٠٢٧"),
                                shadcn::SelectItem::new("2028", "٢٠٢٨"),
                                shadcn::SelectItem::new("2029", "٢٠٢٩"),
                            ]),
                    );
                })
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::FieldLabel::new("CVV")
                        .for_control(cvv_id)
                        .into_element(cx),
                    shadcn::Input::new(rtl_cvv)
                        .control_id(cvv_id)
                        .placeholder("123")
                        .a11y_label("CVV")
                        .into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
            ]
        })
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

        shadcn::FieldGroup::new([
            shadcn::FieldSet::new([
                shadcn::FieldLegend::new("طريقة الدفع").into_element(cx),
                shadcn::FieldDescription::new("جميع المعاملات آمنة ومشفرة").into_element(cx),
                shadcn::FieldGroup::new([
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("الاسم على البطاقة")
                            .for_control(name_id)
                            .into_element(cx),
                        shadcn::Input::new(rtl_name)
                            .control_id(name_id)
                            .a11y_label("الاسم على البطاقة")
                            .placeholder("Evil Rabbit")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::Field::new([
                        shadcn::FieldLabel::new("رقم البطاقة")
                            .for_control(number_id)
                            .into_element(cx),
                        shadcn::Input::new(rtl_number)
                            .control_id(number_id)
                            .a11y_label("رقم البطاقة")
                            .placeholder("1234 5678 9012 3456")
                            .into_element(cx),
                        shadcn::FieldDescription::new("أدخل رقم بطاقتك المكون من 16 رقمًا")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    expiry_row,
                ])
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::FieldSeparator::new().into_element(cx),
            shadcn::FieldSet::new([
                shadcn::FieldLegend::new("عنوان الفوترة").into_element(cx),
                shadcn::FieldDescription::new("عنوان الفوترة المرتبط بطريقة الدفع الخاصة بك")
                    .into_element(cx),
                shadcn::FieldGroup::new([shadcn::Field::new([
                    shadcn::Checkbox::new(rtl_same_as_shipping)
                        .control_id(same_as_shipping_id)
                        .a11y_label("نفس عنوان الشحن")
                        .into_element(cx),
                    shadcn::FieldLabel::new("نفس عنوان الشحن")
                        .for_control(same_as_shipping_id)
                        .into_element(cx),
                ])
                .orientation(shadcn::FieldOrientation::Horizontal)
                .into_element(cx)])
                .checkbox_group()
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::FieldSet::new([shadcn::FieldGroup::new([shadcn::Field::new([
                shadcn::FieldLabel::new("تعليقات")
                    .for_control(comments_id)
                    .into_element(cx),
                shadcn::Textarea::new(rtl_comments)
                    .control_id(comments_id)
                    .placeholder("أضف أي تعليقات إضافية")
                    .a11y_label("تعليقات")
                    .refine_layout(LayoutRefinement::default().h_px(Px(96.0)))
                    .into_element(cx),
            ])
            .into_element(cx)])
            .into_element(cx)])
            .into_element(cx),
            shadcn::Field::new([
                shadcn::Button::new("إرسال").into_element(cx),
                shadcn::Button::new("إلغاء")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx),
        ])
        .refine_layout(max_w_md)
        .into_element(cx)
    })
    .test_id("ui-gallery-field-rtl")
}
// endregion: example
