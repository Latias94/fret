pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_runtime::CommandId;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct MenubarRtlState {
    view_bookmarks_bar: bool,
    view_full_urls: bool,
    profile: Option<Arc<str>>,
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();
    let state = cx.local_model(|| MenubarRtlState {
        view_bookmarks_bar: false,
        view_full_urls: true,
        profile: Some(Arc::<str>::from("benoit")),
    });
    let state_now = cx.watch_model(&state).layout().cloned().unwrap_or_default();

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let file = shadcn::MenubarTrigger::new("ملف")
            .test_id("ui-gallery-menubar-rtl-file")
            .into_menu()
            .entries_parts(
                shadcn::MenubarContent::new(),
                [
                    shadcn::MenubarGroup::new([
                        shadcn::MenubarItem::new("علامة تبويب جديدة")
                            .action(CommandId::new("ui_gallery.menubar.rtl.new_tab"))
                            .test_id("ui-gallery-menubar-rtl-new-tab")
                            .trailing(shadcn::MenubarShortcut::new("⌘T").into_element(cx))
                            .into(),
                        shadcn::MenubarItem::new("نافذة جديدة")
                            .action(CommandId::new("ui_gallery.menubar.rtl.new_window"))
                            .test_id("ui-gallery-menubar-rtl-new-window")
                            .trailing(shadcn::MenubarShortcut::new("⌘N").into_element(cx))
                            .into(),
                        shadcn::MenubarItem::new("نافذة التصفح المتخفي الجديدة")
                            .disabled(true)
                            .into(),
                    ])
                    .into(),
                    shadcn::MenubarSeparator::new().into(),
                    shadcn::MenubarGroup::new([shadcn::MenubarSub::new(
                        shadcn::MenubarSubTrigger::new("مشاركة")
                            .refine(|item| item.test_id("ui-gallery-menubar-rtl-more")),
                        shadcn::MenubarSubContent::new([shadcn::MenubarGroup::new([
                            shadcn::MenubarItem::new("رابط البريد الإلكتروني")
                                .action(CommandId::new("ui_gallery.menubar.rtl.share.email_link"))
                                .test_id("ui-gallery-menubar-rtl-sub-alpha")
                                .into(),
                            shadcn::MenubarItem::new("الرسائل")
                                .action(CommandId::new("ui_gallery.menubar.rtl.share.messages"))
                                .test_id("ui-gallery-menubar-rtl-sub-beta")
                                .into(),
                            shadcn::MenubarItem::new("الملاحظات")
                                .action(CommandId::new("ui_gallery.menubar.rtl.share.notes"))
                                .into(),
                        ])
                        .into()]),
                    )
                    .into()])
                    .into(),
                    shadcn::MenubarSeparator::new().into(),
                    shadcn::MenubarGroup::new([shadcn::MenubarItem::new("طباعة...")
                        .action(CommandId::new("ui_gallery.menubar.rtl.print"))
                        .test_id("ui-gallery-menubar-rtl-print")
                        .trailing(shadcn::MenubarShortcut::new("⌘P").into_element(cx))
                        .into()])
                    .into(),
                ],
            );

        let edit = shadcn::MenubarTrigger::new("تعديل")
            .into_menu()
            .entries_parts(
                shadcn::MenubarContent::new(),
                [
                    shadcn::MenubarGroup::new([
                        shadcn::MenubarItem::new("تراجع")
                            .action(CommandId::new("ui_gallery.menubar.rtl.undo"))
                            .trailing(shadcn::MenubarShortcut::new("⌘Z").into_element(cx))
                            .into(),
                        shadcn::MenubarItem::new("إعادة")
                            .action(CommandId::new("ui_gallery.menubar.rtl.redo"))
                            .trailing(shadcn::MenubarShortcut::new("⇧⌘Z").into_element(cx))
                            .into(),
                    ])
                    .into(),
                    shadcn::MenubarSeparator::new().into(),
                    shadcn::MenubarGroup::new([shadcn::MenubarSub::new(
                        shadcn::MenubarSubTrigger::new("بحث"),
                        shadcn::MenubarSubContent::new([
                            shadcn::MenubarGroup::new([shadcn::MenubarItem::new(
                                "البحث على الويب",
                            )
                            .action(CommandId::new("ui_gallery.menubar.rtl.find.search"))
                            .into()])
                            .into(),
                            shadcn::MenubarSeparator::new().into(),
                            shadcn::MenubarGroup::new([
                                shadcn::MenubarItem::new("بحث...")
                                    .action(CommandId::new("ui_gallery.menubar.rtl.find.find"))
                                    .into(),
                                shadcn::MenubarItem::new("البحث التالي")
                                    .action(CommandId::new("ui_gallery.menubar.rtl.find.next"))
                                    .into(),
                                shadcn::MenubarItem::new("البحث السابق")
                                    .action(CommandId::new("ui_gallery.menubar.rtl.find.previous"))
                                    .into(),
                            ])
                            .into(),
                        ]),
                    )
                    .into()])
                    .into(),
                    shadcn::MenubarSeparator::new().into(),
                    shadcn::MenubarGroup::new([
                        shadcn::MenubarItem::new("قص")
                            .action(CommandId::new("ui_gallery.menubar.rtl.cut"))
                            .into(),
                        shadcn::MenubarItem::new("نسخ")
                            .action(CommandId::new("ui_gallery.menubar.rtl.copy"))
                            .into(),
                        shadcn::MenubarItem::new("لصق")
                            .action(CommandId::new("ui_gallery.menubar.rtl.paste"))
                            .into(),
                    ])
                    .into(),
                ],
            );

        let view = shadcn::MenubarTrigger::new("عرض")
            .into_menu()
            .entries_parts(
                shadcn::MenubarContent::new().min_width(Px(176.0)),
                [
                    shadcn::MenubarGroup::new([
                        shadcn::MenubarCheckboxItem::from_checked(
                            state_now.view_bookmarks_bar,
                            "شريط الإشارات المرجعية",
                        )
                        .on_checked_change({
                            let state = state.clone();
                            move |host, _action_cx, checked| {
                                let _ = host
                                    .models_mut()
                                    .update(&state, |st| st.view_bookmarks_bar = checked);
                            }
                        })
                        .into(),
                        shadcn::MenubarCheckboxItem::from_checked(
                            state_now.view_full_urls,
                            "عناوين URL الكاملة",
                        )
                        .on_checked_change({
                            let state = state.clone();
                            move |host, _action_cx, checked| {
                                let _ = host
                                    .models_mut()
                                    .update(&state, |st| st.view_full_urls = checked);
                            }
                        })
                        .into(),
                    ])
                    .into(),
                    shadcn::MenubarSeparator::new().into(),
                    shadcn::MenubarGroup::new([
                        shadcn::MenubarItem::new("إعادة تحميل")
                            .inset(true)
                            .trailing(shadcn::MenubarShortcut::new("⌘R").into_element(cx))
                            .into(),
                        shadcn::MenubarItem::new("إعادة تحميل قسري")
                            .disabled(true)
                            .inset(true)
                            .trailing(shadcn::MenubarShortcut::new("⇧⌘R").into_element(cx))
                            .into(),
                    ])
                    .into(),
                    shadcn::MenubarSeparator::new().into(),
                    shadcn::MenubarGroup::new([shadcn::MenubarItem::new("تبديل وضع ملء الشاشة")
                        .inset(true)
                        .into()])
                    .into(),
                    shadcn::MenubarSeparator::new().into(),
                    shadcn::MenubarGroup::new([shadcn::MenubarItem::new("إخفاء الشريط الجانبي")
                        .inset(true)
                        .into()])
                    .into(),
                ],
            );

        let profiles = shadcn::MenubarTrigger::new("الملفات الشخصية")
            .into_menu()
            .entries_parts(
                shadcn::MenubarContent::new(),
                [
                    shadcn::MenubarRadioGroup::from_value(state_now.profile.clone())
                        .on_value_change({
                            let state = state.clone();
                            move |host, _action_cx, value| {
                                let _ = host
                                    .models_mut()
                                    .update(&state, |st| st.profile = Some(value));
                            }
                        })
                        .item(shadcn::MenubarRadioItemSpec::new("andy", "Andy"))
                        .item(shadcn::MenubarRadioItemSpec::new("benoit", "Benoit"))
                        .item(shadcn::MenubarRadioItemSpec::new("luis", "Luis"))
                        .into(),
                    shadcn::MenubarSeparator::new().into(),
                    shadcn::MenubarGroup::new([shadcn::MenubarItem::new("تعديل...")
                        .inset(true)
                        .into()])
                    .into(),
                    shadcn::MenubarSeparator::new().into(),
                    shadcn::MenubarGroup::new([shadcn::MenubarItem::new("إضافة ملف شخصي...")
                        .inset(true)
                        .into()])
                    .into(),
                ],
            );

        shadcn::Menubar::new([file, edit, view, profiles])
            .refine_layout(width.clone())
            .into_element(cx)
    })
}
// endregion: example
