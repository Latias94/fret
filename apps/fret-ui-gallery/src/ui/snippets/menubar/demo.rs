pub const SOURCE: &str = include_str!("demo.rs");

// region: example
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

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.local_model(|| MenubarDemoState {
        view_bookmarks_bar: false,
        view_full_urls: true,
        profile: Some(Arc::<str>::from("benoit")),
    });
    let state_now = cx.watch_model(&state).layout().cloned().unwrap_or_default();

    let file = shadcn::MenubarMenu::new("File").entries([
        shadcn::MenubarEntry::Group(shadcn::MenubarGroup::new([
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("New Tab")
                    .action(CommandId::new("ui_gallery.menubar.demo.new_tab"))
                    .trailing(shadcn::MenubarShortcut::new("⌘T").into_element(cx)),
            ),
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("New Window")
                    .action(CommandId::new("ui_gallery.menubar.demo.new_window")),
            ),
        ])),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Group(shadcn::MenubarGroup::new([
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Share")
                    .close_on_select(false)
                    .action(CommandId::new("ui_gallery.menubar.demo.share")),
            ),
            shadcn::MenubarEntry::Item(
                shadcn::MenubarItem::new("Print")
                    .close_on_select(false)
                    .action(CommandId::new("ui_gallery.menubar.demo.print")),
            ),
        ])),
    ]);

    let edit = shadcn::MenubarMenu::new("Edit").entries([
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Undo")
                .trailing(shadcn::MenubarShortcut::new("⌘Z").into_element(cx)),
        ),
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Redo")
                .trailing(shadcn::MenubarShortcut::new("⇧⌘Z").into_element(cx)),
        ),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Submenu(shadcn::MenubarItem::new("Find").submenu([
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Search the web")),
            shadcn::MenubarEntry::Separator,
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Find...")),
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Find Next")),
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Find Previous")),
        ])),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Cut")),
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Copy")),
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Paste")),
    ]);

    let view = shadcn::MenubarTrigger::new("View")
        .into_menu()
        .entries_parts(
            shadcn::MenubarContent::new().min_width(Px(256.0)),
            [
                shadcn::MenubarCheckboxItem::from_checked(
                    state_now.view_bookmarks_bar,
                    "Always Show Bookmarks Bar",
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
                    "Always Show Full URLs",
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
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarItem::new("Reload")
                    .inset(true)
                    .trailing(shadcn::MenubarShortcut::new("⌘R").into_element(cx))
                    .into(),
                shadcn::MenubarItem::new("Force Reload")
                    .disabled(true)
                    .inset(true)
                    .trailing(shadcn::MenubarShortcut::new("⇧⌘R").into_element(cx))
                    .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarItem::new("Toggle Fullscreen")
                    .inset(true)
                    .into(),
                shadcn::MenubarSeparator::new().into(),
                shadcn::MenubarItem::new("Hide Sidebar").inset(true).into(),
            ],
        );

    let profiles = shadcn::MenubarMenu::new("Profiles").entries([
        shadcn::MenubarEntry::RadioGroup(
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
                .item(shadcn::MenubarRadioItemSpec::new("luis", "Luis")),
        ),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Edit...").inset(true)),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Add Profile...").inset(true)),
    ]);

    shadcn::Menubar::new([file, edit, view, profiles]).into_element(cx)
}
// endregion: example
