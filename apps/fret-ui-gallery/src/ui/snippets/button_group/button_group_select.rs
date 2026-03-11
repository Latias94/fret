pub const SOURCE: &str = include_str!("button_group_select.rs");

// region: example
use fret_core::{FontId, Px};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct Models {
    currency_value: Option<Model<Option<Arc<str>>>>,
    currency_open: Option<Model<bool>>,
    amount_value: Option<Model<String>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (currency_value, currency_open, amount_value) = cx.with_state(Models::default, |st| {
        (
            st.currency_value.clone(),
            st.currency_open.clone(),
            st.amount_value.clone(),
        )
    });

    let currency_value = match currency_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("$")));
            cx.with_state(Models::default, |st| {
                st.currency_value = Some(model.clone())
            });
            model
        }
    };

    let currency_open = match currency_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.currency_open = Some(model.clone()));
            model
        }
    };

    let amount_value = match amount_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.amount_value = Some(model.clone()));
            model
        }
    };

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
            .into_element_parts(
                cx,
                |_cx| {
                    shadcn::SelectTrigger::new()
                        .font(FontId::monospace())
                        .label_policy(shadcn::SelectTriggerLabelPolicy::Value)
                },
                |_cx| shadcn::SelectValue::new(),
                move |_cx| {
                    shadcn::SelectContent::new()
                        .position(fret_ui_shadcn::select::SelectPosition::Popper)
                        .align(shadcn::SelectAlign::Start)
                        .with_entries(entries)
                },
            )
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
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .max_w(Px(760.0)),
    )
    .into_element(cx)
    .test_id("ui-gallery-button-group-select")
}

// endregion: example
