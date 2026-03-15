pub const SOURCE: &str = include_str!("button_group_select.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{FontId, Px};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let currency_value = cx.local_model_keyed("currency_value", || Some(Arc::<str>::from("$")));
    let currency_open = cx.local_model_keyed("currency_open", || false);
    let amount_value = cx.local_model_keyed("amount_value", String::new);

    let currencies: &[(&'static str, &'static str)] =
        &[("$", "US Dollar"), ("€", "Euro"), ("£", "British Pound")];

    let currency = {
        let entries: Vec<shadcn::SelectEntry> = vec![
            shadcn::SelectGroup::new(currencies.iter().map(|(value, label)| {
                shadcn::SelectItem::new(*value, *label)
                    .item_text(shadcn::SelectItemText::new([
                        shadcn::SelectTextRun::new(
                            Arc::<str>::from(format!("{value} ")),
                            shadcn::SelectTextTone::Normal,
                        ),
                        shadcn::SelectTextRun::new(*label, shadcn::SelectTextTone::Muted),
                    ]))
                    .into()
            }))
            .into(),
        ];

        shadcn::Select::new(currency_value.clone(), currency_open.clone())
            .trigger_test_id("ui-gallery-button-group-select-currency-trigger")
            .trigger(
                shadcn::SelectTrigger::new()
                    .font(FontId::monospace())
                    .label_policy(shadcn::SelectTriggerLabelPolicy::Value),
            )
            .value(shadcn::SelectValue::new())
            .content(
                shadcn::SelectContent::new()
                    .position(shadcn::raw::select::SelectPosition::Popper)
                    .align(shadcn::SelectAlign::Start),
            )
            .entries(entries)
            .into_element(cx)
    };

    let amount = shadcn::Input::new(amount_value)
        .a11y_label("Amount")
        .placeholder("10.00")
        .test_id("ui-gallery-button-group-select-amount");

    let send = shadcn::Button::new("")
        .a11y_label("Send")
        .size(shadcn::ButtonSize::Icon)
        .variant(shadcn::ButtonVariant::Outline)
        .icon(IconId::new_static("lucide.arrow-right"));

    shadcn::ButtonGroup::new([
        shadcn::ButtonGroup::new([currency.into(), amount.into()])
            .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
            .into(),
        shadcn::ButtonGroup::new([send.into()]).into(),
    ])
    .refine_layout(LayoutRefinement::default().w_px(Px(420.0)).min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-button-group-select")
}

// endregion: example
