pub const SOURCE: &str = include_str!("rich_items.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{FontId, Px};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || Some(Arc::<str>::from("$")));
    let open = cx.local_model_keyed("open", || false);

    let currencies: &[(&'static str, &'static str)] = &[
        ("$", "US Dollar"),
        ("€", "Euro"),
        ("£", "British Pound"),
        ("¥", "Japanese Yen"),
    ];

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

    shadcn::Select::new(value, open)
        .test_id_prefix("ui-gallery-select-rich-items")
        .trigger(
            shadcn::SelectTrigger::new()
                .refine_layout(LayoutRefinement::default().w_px(Px(220.0)))
                .font(FontId::monospace())
                .label_policy(shadcn::SelectTriggerLabelPolicy::Value),
        )
        .value(shadcn::SelectValue::new().placeholder("Select currency"))
        .content(
            shadcn::SelectContent::new()
                .position(shadcn::raw::select::SelectPosition::Popper)
                .align(shadcn::SelectAlign::Start),
        )
        .entries(entries)
}
// endregion: example
