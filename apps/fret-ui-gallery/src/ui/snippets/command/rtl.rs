pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let query = cx.local_model(String::new);
    let noop: fret_ui::action::OnActivate = Arc::new(|_host, _action_cx, _reason| {});
    let icon_id = fret_icons::IconId::new_static;

    let entries = vec![
        shadcn::CommandGroup::new([
            shadcn::CommandItem::new("التقويم")
                .leading_icon(icon_id("lucide.calendar"))
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("البحث عن الرموز التعبيرية")
                .leading_icon(icon_id("lucide.smile"))
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("الآلة الحاسبة")
                .leading_icon(icon_id("lucide.calculator"))
                .disabled(true),
        ])
        .heading("اقتراحات")
        .into(),
        shadcn::CommandSeparator::new().into(),
        shadcn::CommandGroup::new([
            shadcn::CommandItem::new("الملف الشخصي")
                .leading_icon(icon_id("lucide.user"))
                .shortcut("⌘P")
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("الفوترة")
                .leading_icon(icon_id("lucide.credit-card"))
                .shortcut("⌘B")
                .on_select_action(noop.clone()),
            shadcn::CommandItem::new("الإعدادات")
                .leading_icon(icon_id("lucide.settings"))
                .shortcut("⌘S")
                .on_select_action(noop.clone()),
        ])
        .heading("الإعدادات")
        .into(),
    ];

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::CommandPalette::new(query.clone(), Vec::new())
            .placeholder("اكتب أمرًا أو ابحث...")
            .empty_text("لم يتم العثور على نتائج.")
            .a11y_label("لوحة أوامر باتجاه من اليمين إلى اليسار")
            .entries(entries)
            .refine_layout(
                LayoutRefinement::default()
                    .w_full()
                    .max_w(Px(384.0))
                    .min_w_0(),
            )
            .test_id_input("ui-gallery-command-rtl-input")
            .list_test_id("ui-gallery-command-rtl-listbox")
            .test_id_item_prefix("ui-gallery-command-rtl-item-")
            .into_element(cx)
            .test_id("ui-gallery-command-rtl")
    })
}
// endregion: example
