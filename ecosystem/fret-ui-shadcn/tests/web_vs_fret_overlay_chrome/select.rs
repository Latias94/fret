use super::*;

#[path = "select/fixtures.rs"]
mod fixtures;

fn build_shadcn_select_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let entries: Vec<SelectEntry> = vec![
        SelectGroup::new(vec![
            SelectLabel::new("Fruits").into(),
            SelectItem::new("apple", "Apple").into(),
            SelectItem::new("banana", "Banana").into(),
            SelectItem::new("blueberry", "Blueberry").into(),
            SelectItem::new("grapes", "Grapes").into(),
            SelectItem::new("pineapple", "Pineapple").into(),
        ])
        .into(),
    ];

    fret_ui_shadcn::Select::new(value, open.clone())
        .a11y_label("Select")
        .placeholder("Select a fruit")
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default().w_px(fret_ui_kit::MetricRef::Px(Px(180.0))),
        )
        .entries(entries)
        .into_element(cx)
}

fn build_shadcn_select_scrollable_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{SelectEntry, SelectGroup, SelectItem, SelectLabel};

    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let entries: Vec<SelectEntry> = vec![
        SelectGroup::new(vec![
            SelectLabel::new("North America").into(),
            SelectItem::new("est", "Eastern Standard Time (EST)").into(),
            SelectItem::new("cst", "Central Standard Time (CST)").into(),
            SelectItem::new("mst", "Mountain Standard Time (MST)").into(),
            SelectItem::new("pst", "Pacific Standard Time (PST)").into(),
            SelectItem::new("akst", "Alaska Standard Time (AKST)").into(),
            SelectItem::new("hst", "Hawaii Standard Time (HST)").into(),
        ])
        .into(),
        SelectGroup::new(vec![
            SelectLabel::new("Europe & Africa").into(),
            SelectItem::new("gmt", "Greenwich Mean Time (GMT)").into(),
            SelectItem::new("cet", "Central European Time (CET)").into(),
            SelectItem::new("eet", "Eastern European Time (EET)").into(),
            SelectItem::new("west", "Western European Summer Time (WEST)").into(),
            SelectItem::new("cat", "Central Africa Time (CAT)").into(),
            SelectItem::new("eat", "East Africa Time (EAT)").into(),
        ])
        .into(),
        SelectGroup::new(vec![
            SelectLabel::new("Asia").into(),
            SelectItem::new("msk", "Moscow Time (MSK)").into(),
            SelectItem::new("ist", "India Standard Time (IST)").into(),
            SelectItem::new("cst_china", "China Standard Time (CST)").into(),
            SelectItem::new("jst", "Japan Standard Time (JST)").into(),
            SelectItem::new("kst", "Korea Standard Time (KST)").into(),
            SelectItem::new("ist_indonesia", "Indonesia Central Standard Time (WITA)").into(),
        ])
        .into(),
        SelectGroup::new(vec![
            SelectLabel::new("Australia & Pacific").into(),
            SelectItem::new("awst", "Australian Western Standard Time (AWST)").into(),
            SelectItem::new("acst", "Australian Central Standard Time (ACST)").into(),
            SelectItem::new("aest", "Australian Eastern Standard Time (AEST)").into(),
            SelectItem::new("nzst", "New Zealand Standard Time (NZST)").into(),
            SelectItem::new("fjt", "Fiji Time (FJT)").into(),
        ])
        .into(),
        SelectGroup::new(vec![
            SelectLabel::new("South America").into(),
            SelectItem::new("art", "Argentina Time (ART)").into(),
            SelectItem::new("bot", "Bolivia Time (BOT)").into(),
            SelectItem::new("brt", "Brasilia Time (BRT)").into(),
            SelectItem::new("clt", "Chile Standard Time (CLT)").into(),
        ])
        .into(),
    ];

    fret_ui_shadcn::Select::new(value, open.clone())
        .a11y_label("Select")
        .placeholder("Select a timezone")
        .refine_layout(
            fret_ui_kit::LayoutRefinement::default().w_px(fret_ui_kit::MetricRef::Px(Px(280.0))),
        )
        .entries(entries)
        .into_element(cx)
}

fn build_shadcn_select_scrollable_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    build_shadcn_select_scrollable_page(cx, open)
}
