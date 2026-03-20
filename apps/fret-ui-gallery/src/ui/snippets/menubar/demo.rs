pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_runtime::CommandId;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct MenubarDemoState {
    view_bookmarks_bar: bool,
    view_full_urls: bool,
    profile: Option<Arc<str>>,
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();
    let state = cx.local_model(|| MenubarDemoState {
        view_bookmarks_bar: false,
        view_full_urls: true,
        profile: Some(Arc::<str>::from("benoit")),
    });
    let state_now = cx.watch_model(&state).layout().cloned().unwrap_or_default();

    let file = shadcn::MenubarTrigger::new("File")
        .into_menu()
        .entries_parts(
            shadcn::MenubarContent::new(),
            [
                shadcn::MenubarGroup::new([
                    shadcn::MenubarItem::new("New Tab")
                        .action(CommandId::new("ui_gallery.menubar.demo.new_tab"))
                        .trailing(shadcn::MenubarShortcut::new("⌘T").into_element(cx))
                        .into(),
                    shadcn::MenubarItem::new("New Window")
                        .action(CommandId::new("ui_gallery.menubar.demo.new_window"))
                        .trailing(shadcn::MenubarShortcut::new("⌘N").into_element(cx))
                        .into(),
                    shadcn::MenubarItem::new("New Incognito Window")
                        .disabled(true)
                        .into(),
                ])
                .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarGroup::new([shadcn::MenubarSub::new(
                    shadcn::MenubarSubTrigger::new("Share"),
                    shadcn::MenubarSubContent::new([shadcn::MenubarGroup::new([
                        shadcn::MenubarItem::new("Email link")
                            .action(CommandId::new("ui_gallery.menubar.demo.share.email_link"))
                            .into(),
                        shadcn::MenubarItem::new("Messages")
                            .action(CommandId::new("ui_gallery.menubar.demo.share.messages"))
                            .into(),
                        shadcn::MenubarItem::new("Notes")
                            .action(CommandId::new("ui_gallery.menubar.demo.share.notes"))
                            .into(),
                    ])
                    .into()]),
                )
                .into()])
                .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarGroup::new([shadcn::MenubarItem::new("Print...")
                    .action(CommandId::new("ui_gallery.menubar.demo.print"))
                    .trailing(shadcn::MenubarShortcut::new("⌘P").into_element(cx))
                    .into()])
                .into(),
            ],
        );

    let edit = shadcn::MenubarTrigger::new("Edit")
        .into_menu()
        .entries_parts(
            shadcn::MenubarContent::new(),
            [
                shadcn::MenubarGroup::new([
                    shadcn::MenubarItem::new("Undo")
                        .action(CommandId::new("ui_gallery.menubar.demo.undo"))
                        .trailing(shadcn::MenubarShortcut::new("⌘Z").into_element(cx))
                        .into(),
                    shadcn::MenubarItem::new("Redo")
                        .action(CommandId::new("ui_gallery.menubar.demo.redo"))
                        .trailing(shadcn::MenubarShortcut::new("⇧⌘Z").into_element(cx))
                        .into(),
                ])
                .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarGroup::new([shadcn::MenubarSub::new(
                    shadcn::MenubarSubTrigger::new("Find"),
                    shadcn::MenubarSubContent::new([
                        shadcn::MenubarGroup::new([shadcn::MenubarItem::new("Search the web")
                            .action(CommandId::new("ui_gallery.menubar.demo.find.search"))
                            .into()])
                        .into(),
                        shadcn::MenubarSeparator::new().into(),
                        shadcn::MenubarGroup::new([
                            shadcn::MenubarItem::new("Find...")
                                .action(CommandId::new("ui_gallery.menubar.demo.find.find"))
                                .into(),
                            shadcn::MenubarItem::new("Find Next")
                                .action(CommandId::new("ui_gallery.menubar.demo.find.next"))
                                .into(),
                            shadcn::MenubarItem::new("Find Previous")
                                .action(CommandId::new("ui_gallery.menubar.demo.find.previous"))
                                .into(),
                        ])
                        .into(),
                    ]),
                )
                .into()])
                .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarGroup::new([
                    shadcn::MenubarItem::new("Cut")
                        .action(CommandId::new("ui_gallery.menubar.demo.cut"))
                        .into(),
                    shadcn::MenubarItem::new("Copy")
                        .action(CommandId::new("ui_gallery.menubar.demo.copy"))
                        .into(),
                    shadcn::MenubarItem::new("Paste")
                        .action(CommandId::new("ui_gallery.menubar.demo.paste"))
                        .into(),
                ])
                .into(),
            ],
        );

    let view = shadcn::MenubarTrigger::new("View")
        .into_menu()
        .entries_parts(
            shadcn::MenubarContent::new().min_width(Px(176.0)),
            [
                shadcn::MenubarGroup::new([
                    shadcn::MenubarCheckboxItem::from_checked(
                        state_now.view_bookmarks_bar,
                        "Bookmarks Bar",
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
                        "Full URLs",
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
                    shadcn::MenubarItem::new("Reload")
                        .inset(true)
                        .trailing(shadcn::MenubarShortcut::new("⌘R").into_element(cx))
                        .into(),
                    shadcn::MenubarItem::new("Force Reload")
                        .disabled(true)
                        .inset(true)
                        .trailing(shadcn::MenubarShortcut::new("⇧⌘R").into_element(cx))
                        .into(),
                ])
                .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarGroup::new([shadcn::MenubarItem::new("Toggle Fullscreen")
                    .inset(true)
                    .into()])
                .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarGroup::new([shadcn::MenubarItem::new("Hide Sidebar")
                    .inset(true)
                    .into()])
                .into(),
            ],
        );

    let profiles = shadcn::MenubarTrigger::new("Profiles")
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
                shadcn::MenubarGroup::new([
                    shadcn::MenubarItem::new("Edit...").inset(true).into(),
                    shadcn::MenubarItem::new("Add Profile...")
                        .inset(true)
                        .into(),
                ])
                .into(),
            ],
        );

    shadcn::Menubar::new([file, edit, view, profiles])
        .refine_layout(width)
        .into_element(cx)
}
// endregion: example
