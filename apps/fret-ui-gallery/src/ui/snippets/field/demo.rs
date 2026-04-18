pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let card_name = cx.local_model_keyed("card_name", String::new);
    let card_number = cx.local_model_keyed("card_number", String::new);
    let cvv = cx.local_model_keyed("cvv", String::new);
    let comments = cx.local_model_keyed("comments", String::new);
    let same_as_shipping = cx.local_model_keyed("same_as_shipping", || true);
    let expiry_month = cx.local_model_keyed("expiry_month", || None::<Arc<str>>);
    let expiry_month_open = cx.local_model_keyed("expiry_month_open", || false);
    let expiry_year = cx.local_model_keyed("expiry_year", || None::<Arc<str>>);
    let expiry_year_open = cx.local_model_keyed("expiry_year_open", || false);
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));

    let expiry_row = ui::h_flex(|cx| {
        vec![
            shadcn::Field::new([
                shadcn::FieldLabel::new("Month")
                    .for_control("ui-gallery-field-demo-expiry-month")
                    .into_element(cx),
                shadcn::Select::new(expiry_month, expiry_month_open)
                    .control_id("ui-gallery-field-demo-expiry-month")
                    .a11y_label("Month")
                    .value(shadcn::SelectValue::new().placeholder("MM"))
                    .items([
                        shadcn::SelectItem::new("01", "01"),
                        shadcn::SelectItem::new("02", "02"),
                        shadcn::SelectItem::new("03", "03"),
                        shadcn::SelectItem::new("04", "04"),
                        shadcn::SelectItem::new("05", "05"),
                        shadcn::SelectItem::new("06", "06"),
                        shadcn::SelectItem::new("07", "07"),
                        shadcn::SelectItem::new("08", "08"),
                        shadcn::SelectItem::new("09", "09"),
                        shadcn::SelectItem::new("10", "10"),
                        shadcn::SelectItem::new("11", "11"),
                        shadcn::SelectItem::new("12", "12"),
                    ])
                    .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("Year")
                    .for_control("ui-gallery-field-demo-expiry-year")
                    .into_element(cx),
                shadcn::Select::new(expiry_year, expiry_year_open)
                    .control_id("ui-gallery-field-demo-expiry-year")
                    .a11y_label("Year")
                    .value(shadcn::SelectValue::new().placeholder("YYYY"))
                    .items([
                        shadcn::SelectItem::new("2026", "2026"),
                        shadcn::SelectItem::new("2027", "2027"),
                        shadcn::SelectItem::new("2028", "2028"),
                        shadcn::SelectItem::new("2029", "2029"),
                        shadcn::SelectItem::new("2030", "2030"),
                        shadcn::SelectItem::new("2031", "2031"),
                    ])
                    .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
            shadcn::Field::new([
                shadcn::FieldLabel::new("CVV")
                    .for_control("ui-gallery-field-demo-cvv")
                    .into_element(cx),
                shadcn::Input::new(cvv)
                    .control_id("ui-gallery-field-demo-cvv")
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
            shadcn::FieldLegend::new("Payment Method").into_element(cx),
            shadcn::FieldDescription::new("All transactions are secure and encrypted")
                .into_element(cx),
            shadcn::FieldGroup::new([
                shadcn::Field::new([
                    shadcn::FieldLabel::new("Name on Card")
                        .for_control("ui-gallery-field-demo-card-name")
                        .into_element(cx),
                    shadcn::Input::new(card_name)
                        .control_id("ui-gallery-field-demo-card-name")
                        .placeholder("Evil Rabbit")
                        .a11y_label("Name on Card")
                        .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Field::new([
                    shadcn::FieldLabel::new("Card Number")
                        .for_control("ui-gallery-field-demo-card-number")
                        .into_element(cx),
                    shadcn::Input::new(card_number)
                        .control_id("ui-gallery-field-demo-card-number")
                        .placeholder("1234 5678 9012 3456")
                        .a11y_label("Card Number")
                        .into_element(cx),
                    shadcn::FieldDescription::new("Enter your 16-digit card number")
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
            shadcn::FieldLegend::new("Billing Address").into_element(cx),
            shadcn::FieldDescription::new(
                "The billing address associated with your payment method",
            )
            .into_element(cx),
            shadcn::FieldGroup::new([shadcn::Field::new([
                shadcn::Checkbox::new(same_as_shipping)
                    .control_id("ui-gallery-field-demo-same-as-shipping")
                    .a11y_label("Same as shipping address")
                    .into_element(cx),
                shadcn::FieldLabel::new("Same as shipping address")
                    .for_control("ui-gallery-field-demo-same-as-shipping")
                    .into_element(cx),
            ])
            .orientation(shadcn::FieldOrientation::Horizontal)
            .into_element(cx)])
            .checkbox_group()
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::FieldSet::new([shadcn::FieldGroup::new([shadcn::Field::new([
            shadcn::FieldLabel::new("Comments")
                .for_control("ui-gallery-field-demo-comments")
                .into_element(cx),
            shadcn::Textarea::new(comments)
                .control_id("ui-gallery-field-demo-comments")
                .placeholder("Add any additional comments")
                .a11y_label("Comments")
                .refine_layout(LayoutRefinement::default().h_px(Px(96.0)))
                .into_element(cx),
        ])
        .into_element(cx)])
        .into_element(cx)])
        .into_element(cx),
        shadcn::Field::new([
            shadcn::Button::new("Submit").into_element(cx),
            shadcn::Button::new("Cancel")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .into_element(cx),
    ])
    .refine_layout(max_w_md)
    .into_element(cx)
    .test_id("ui-gallery-field-demo")
}
// endregion: example
