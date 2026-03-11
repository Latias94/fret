pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_runtime::CommandId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    view_bookmarks_bar: Option<Model<bool>>,
    view_full_urls: Option<Model<bool>>,
    profile: Option<Model<Option<Arc<str>>>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());

    let view_bookmarks_bar = match state.view_bookmarks_bar {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.view_bookmarks_bar = Some(model.clone())
            });
            model
        }
    };

    let view_full_urls = match state.view_full_urls {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(Models::default, |st| {
                st.view_full_urls = Some(model.clone())
            });
            model
        }
    };

    let profile = match state.profile {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("benoit")));
            cx.with_state(Models::default, |st| st.profile = Some(model.clone()));
            model
        }
    };

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
                shadcn::MenubarCheckboxItem::new(
                    view_bookmarks_bar.clone(),
                    "Always Show Bookmarks Bar",
                )
                .into(),
                shadcn::MenubarCheckboxItem::new(view_full_urls.clone(), "Always Show Full URLs")
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
            shadcn::MenubarRadioGroup::new(profile)
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
