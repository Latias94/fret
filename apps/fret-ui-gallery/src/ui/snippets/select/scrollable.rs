pub const SOURCE: &str = include_str!("scrollable.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
        .refine_layout(LayoutRefinement::default().w_px(Px(280.0)))
        .trigger(shadcn::SelectTrigger::new())
        .value(shadcn::SelectValue::new().placeholder("Select a timezone"))
        .content(shadcn::SelectContent::new())
        .entries([
            shadcn::SelectGroup::new([
                shadcn::SelectLabel::new("North America").into(),
                shadcn::SelectItem::new("est", "Eastern Standard Time (EST)").into(),
                shadcn::SelectItem::new("cst", "Central Standard Time (CST)").into(),
                shadcn::SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
                shadcn::SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
                shadcn::SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
                shadcn::SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
            ])
            .into(),
            shadcn::SelectGroup::new([
                shadcn::SelectLabel::new("Europe & Africa").into(),
                shadcn::SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
                shadcn::SelectItem::new("cet", "Central European Time (CET)").into(),
                shadcn::SelectItem::new("eet", "Eastern European Time (EET)").into(),
                shadcn::SelectItem::new("west", "Western European Summer Time (WEST)").into(),
                shadcn::SelectItem::new("cat", "Central Africa Time (CAT)").into(),
                shadcn::SelectItem::new("eat", "East Africa Time (EAT)").into(),
            ])
            .into(),
            shadcn::SelectGroup::new([
                shadcn::SelectLabel::new("Asia").into(),
                shadcn::SelectItem::new("msk", "Moscow Time (MSK)").into(),
                shadcn::SelectItem::new("ist", "India Standard Time (IST)").into(),
                shadcn::SelectItem::new("cst_china", "China Standard Time (CST)").into(),
                shadcn::SelectItem::new("jst", "Japan Standard Time (JST)").into(),
                shadcn::SelectItem::new("kst", "Korea Standard Time (KST)").into(),
                shadcn::SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (IST)")
                    .into(),
            ])
            .into(),
            shadcn::SelectGroup::new([
                shadcn::SelectLabel::new("Australia & Pacific").into(),
                shadcn::SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
                shadcn::SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
                shadcn::SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
                shadcn::SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
                shadcn::SelectItem::new("fjt", "Fiji Time (FJT)").into(),
            ])
            .into(),
            shadcn::SelectGroup::new([
                shadcn::SelectLabel::new("South America").into(),
                shadcn::SelectItem::new("art", "Argentina Time (ART)").into(),
                shadcn::SelectItem::new("bot", "Bolivia Time (BOT)").into(),
                shadcn::SelectItem::new("brt", "Brasilia Time (BRT)").into(),
                shadcn::SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
            ])
            .into(),
        ])
        .into_element(cx)
        .test_id("ui-gallery-select-scrollable")
}

// endregion: example
