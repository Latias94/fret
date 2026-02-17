use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_menubar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use shadcn::{
        Menubar, MenubarCheckboxItem, MenubarEntry, MenubarGroup, MenubarItem, MenubarMenu,
        MenubarRadioGroup, MenubarRadioItemSpec, MenubarShortcut,
    };

    #[derive(Default)]
    struct MenubarModels {
        view_bookmarks_bar: Option<Model<bool>>,
        view_full_urls: Option<Model<bool>>,
        format_strikethrough: Option<Model<bool>>,
        format_code: Option<Model<bool>>,
        format_superscript: Option<Model<bool>>,
        profile: Option<Model<Option<Arc<str>>>>,
        theme: Option<Model<Option<Arc<str>>>>,
    }

    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();
    let icon = doc_layout::icon;

    let view_bookmarks_bar =
        cx.with_state(MenubarModels::default, |st| st.view_bookmarks_bar.clone());
    let view_bookmarks_bar = match view_bookmarks_bar {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(MenubarModels::default, |st| {
                st.view_bookmarks_bar = Some(model.clone())
            });
            model
        }
    };
    let view_full_urls = cx.with_state(MenubarModels::default, |st| st.view_full_urls.clone());
    let view_full_urls = match view_full_urls {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(MenubarModels::default, |st| {
                st.view_full_urls = Some(model.clone())
            });
            model
        }
    };
    let format_strikethrough =
        cx.with_state(MenubarModels::default, |st| st.format_strikethrough.clone());
    let format_strikethrough = match format_strikethrough {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(MenubarModels::default, |st| {
                st.format_strikethrough = Some(model.clone())
            });
            model
        }
    };
    let format_code = cx.with_state(MenubarModels::default, |st| st.format_code.clone());
    let format_code = match format_code {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(MenubarModels::default, |st| {
                st.format_code = Some(model.clone())
            });
            model
        }
    };
    let format_superscript =
        cx.with_state(MenubarModels::default, |st| st.format_superscript.clone());
    let format_superscript = match format_superscript {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(MenubarModels::default, |st| {
                st.format_superscript = Some(model.clone())
            });
            model
        }
    };
    let profile = cx.with_state(MenubarModels::default, |st| st.profile.clone());
    let profile = match profile {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("benoit")));
            cx.with_state(MenubarModels::default, |st| {
                st.profile = Some(model.clone())
            });
            model
        }
    };
    let theme = cx.with_state(MenubarModels::default, |st| st.theme.clone());
    let theme = match theme {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Some(Arc::<str>::from("system")));
            cx.with_state(MenubarModels::default, |st| st.theme = Some(model.clone()));
            model
        }
    };

    let demo = {
        let file = MenubarMenu::new("File").entries([
            MenubarEntry::Group(MenubarGroup::new([
                MenubarEntry::Item(
                    MenubarItem::new("New Tab")
                        .trailing(MenubarShortcut::new("⌘T").into_element(cx)),
                ),
                MenubarEntry::Item(MenubarItem::new("New Window")),
            ])),
            MenubarEntry::Separator,
            MenubarEntry::Group(MenubarGroup::new([
                MenubarEntry::Item(MenubarItem::new("Share").close_on_select(false)),
                MenubarEntry::Item(MenubarItem::new("Print").close_on_select(false)),
            ])),
        ]);
        Menubar::new([file])
            .refine_layout(width.clone())
            .into_element(cx)
    };

    let checkbox = {
        let view = MenubarMenu::new("View").entries([
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
                view_bookmarks_bar.clone(),
                "Always Show Bookmarks Bar",
            )),
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
                view_full_urls.clone(),
                "Always Show Full URLs",
            )),
            MenubarEntry::Separator,
            MenubarEntry::Item(
                MenubarItem::new("Reload")
                    .inset(true)
                    .trailing(MenubarShortcut::new("⌘R").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("Force Reload")
                    .disabled(true)
                    .inset(true)
                    .trailing(MenubarShortcut::new("⇧⌘R").into_element(cx)),
            ),
        ]);
        let format = MenubarMenu::new("Format").entries([
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
                format_strikethrough.clone(),
                "Strikethrough",
            )),
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(format_code.clone(), "Code")),
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
                format_superscript.clone(),
                "Superscript",
            )),
        ]);
        Menubar::new([view, format])
            .refine_layout(width.clone())
            .into_element(cx)
    };

    let radio = {
        let profiles = MenubarMenu::new("Profiles").entries([
            MenubarEntry::RadioGroup(
                MenubarRadioGroup::new(profile.clone())
                    .item(MenubarRadioItemSpec::new("andy", "Andy"))
                    .item(MenubarRadioItemSpec::new("benoit", "Benoit"))
                    .item(MenubarRadioItemSpec::new("luis", "Luis")),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Edit...").inset(true)),
            MenubarEntry::Item(MenubarItem::new("Add Profile...").inset(true)),
        ]);
        let themes = MenubarMenu::new("Theme").entries([MenubarEntry::RadioGroup(
            MenubarRadioGroup::new(theme.clone())
                .item(MenubarRadioItemSpec::new("light", "Light"))
                .item(MenubarRadioItemSpec::new("dark", "Dark"))
                .item(MenubarRadioItemSpec::new("system", "System")),
        )]);
        Menubar::new([profiles, themes])
            .refine_layout(width.clone())
            .into_element(cx)
    };

    let submenu = {
        let file = MenubarMenu::new("File").entries([
            MenubarEntry::Submenu(MenubarItem::new("Share").submenu([
                MenubarEntry::Item(MenubarItem::new("Email link")),
                MenubarEntry::Item(MenubarItem::new("Messages")),
                MenubarEntry::Item(MenubarItem::new("Notes")),
            ])),
            MenubarEntry::Separator,
            MenubarEntry::Item(
                MenubarItem::new("Print...").trailing(MenubarShortcut::new("⌘P").into_element(cx)),
            ),
        ]);
        let edit = MenubarMenu::new("Edit").entries([
            MenubarEntry::Item(
                MenubarItem::new("Undo").trailing(MenubarShortcut::new("⌘Z").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("Redo").trailing(MenubarShortcut::new("⇧⌘Z").into_element(cx)),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Submenu(MenubarItem::new("Find").submenu([
                MenubarEntry::Item(MenubarItem::new("Find...")),
                MenubarEntry::Item(MenubarItem::new("Find Next")),
                MenubarEntry::Item(MenubarItem::new("Find Previous")),
            ])),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Cut")),
            MenubarEntry::Item(MenubarItem::new("Copy")),
            MenubarEntry::Item(MenubarItem::new("Paste")),
        ]);
        Menubar::new([file, edit])
            .refine_layout(width.clone())
            .into_element(cx)
    };

    let with_icons = {
        let file = MenubarMenu::new("File").entries([
            MenubarEntry::Item(
                MenubarItem::new("New File")
                    .leading(icon(cx, "lucide.file"))
                    .trailing(MenubarShortcut::new("⌘N").into_element(cx)),
            ),
            MenubarEntry::Item(MenubarItem::new("Open Folder").leading(icon(cx, "lucide.folder"))),
            MenubarEntry::Separator,
            MenubarEntry::Item(
                MenubarItem::new("Save")
                    .leading(icon(cx, "lucide.save"))
                    .trailing(MenubarShortcut::new("⌘S").into_element(cx)),
            ),
        ]);
        let more = MenubarMenu::new("More").entries([MenubarEntry::Group(MenubarGroup::new([
            MenubarEntry::Item(MenubarItem::new("Settings").leading(icon(cx, "lucide.settings"))),
            MenubarEntry::Item(MenubarItem::new("Help").leading(icon(cx, "lucide.info"))),
            MenubarEntry::Separator,
            MenubarEntry::Item(
                MenubarItem::new("Delete")
                    .leading(icon(cx, "lucide.trash"))
                    .variant(shadcn::menubar::MenubarItemVariant::Destructive),
            ),
        ]))]);
        Menubar::new([file, more])
            .refine_layout(width.clone())
            .into_element(cx)
    };

    let rtl = {
        doc_layout::rtl(cx, |cx| {
            let file = MenubarMenu::new("ملف").entries([
                MenubarEntry::Item(
                    MenubarItem::new("علامة تبويب جديدة")
                        .trailing(MenubarShortcut::new("⌘T").into_element(cx)),
                ),
                MenubarEntry::Item(
                    MenubarItem::new("نافذة جديدة")
                        .trailing(MenubarShortcut::new("⌘N").into_element(cx)),
                ),
                MenubarEntry::Separator,
                MenubarEntry::Item(
                    MenubarItem::new("طباعة...")
                        .trailing(MenubarShortcut::new("⌘P").into_element(cx)),
                ),
            ]);
            Menubar::new([file])
                .refine_layout(width.clone())
                .into_element(cx)
        })
    };

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Menubar docs order: Demo, Checkbox, Radio, Submenu, With Icons, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(520.0))
                .code(
                    "rust",
                    r#"use shadcn::{Menubar, MenubarEntry, MenubarGroup, MenubarItem, MenubarMenu, MenubarShortcut};

let file = MenubarMenu::new("File").entries([
    MenubarEntry::Group(MenubarGroup::new([
        MenubarEntry::Item(
            MenubarItem::new("New Tab")
                .trailing(MenubarShortcut::new("⌘T").into_element(cx)),
        ),
        MenubarEntry::Item(MenubarItem::new("New Window")),
    ])),
    MenubarEntry::Separator,
    MenubarEntry::Item(MenubarItem::new("Print...")),
]);

Menubar::new([file]).into_element(cx);"#,
                ),
            DocSection::new("Checkbox", checkbox)
                .max_w(Px(520.0))
                .code(
                    "rust",
                    r#"use shadcn::{
    Menubar, MenubarCheckboxItem, MenubarEntry, MenubarItem, MenubarMenu, MenubarShortcut,
};

let show_bookmarks = cx.app.models_mut().insert(false);

let view = MenubarMenu::new("View").entries([
    MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
        show_bookmarks,
        "Always Show Bookmarks Bar",
    )),
    MenubarEntry::Separator,
    MenubarEntry::Item(
        MenubarItem::new("Reload").trailing(MenubarShortcut::new("⌘R").into_element(cx)),
    ),
]);

Menubar::new([view]).into_element(cx);"#,
                ),
            DocSection::new("Radio", radio)
                .max_w(Px(520.0))
                .code(
                    "rust",
                    r#"use shadcn::{
    Menubar, MenubarEntry, MenubarMenu, MenubarRadioGroup, MenubarRadioItemSpec,
};

let theme = cx.app.models_mut().insert(Some(Arc::<str>::from("system")));

let theme_menu = MenubarMenu::new("Theme").entries([MenubarEntry::RadioGroup(
    MenubarRadioGroup::new(theme)
        .item(MenubarRadioItemSpec::new("light", "Light"))
        .item(MenubarRadioItemSpec::new("dark", "Dark"))
        .item(MenubarRadioItemSpec::new("system", "System")),
)]);

Menubar::new([theme_menu]).into_element(cx);"#,
                ),
            DocSection::new("Submenu", submenu)
                .max_w(Px(520.0))
                .code(
                    "rust",
                    r#"use shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

let file = MenubarMenu::new("File").entries([
    MenubarEntry::Submenu(MenubarItem::new("Share").submenu([
        MenubarEntry::Item(MenubarItem::new("Email link")),
        MenubarEntry::Item(MenubarItem::new("Messages")),
        MenubarEntry::Item(MenubarItem::new("Notes")),
    ])),
    MenubarEntry::Separator,
    MenubarEntry::Item(MenubarItem::new("Print...")),
]);

Menubar::new([file]).into_element(cx);"#,
                ),
            DocSection::new("With Icons", with_icons)
                .max_w(Px(520.0))
                .code(
                    "rust",
                    r#"use shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

let icon = |cx: &mut ElementContext<'_, App>, id: &'static str| {
    shadcn::icon::icon(cx, fret_icons::IconId::new_static(id))
};

let file = MenubarMenu::new("File").entries([MenubarEntry::Group(shadcn::MenubarGroup::new([
    MenubarEntry::Item(MenubarItem::new("Help").leading(icon(cx, "lucide.info"))),
    MenubarEntry::Item(
        MenubarItem::new("Delete")
            .leading(icon(cx, "lucide.trash"))
            .variant(shadcn::menubar::MenubarItemVariant::Destructive),
    ),
]))]);

Menubar::new([file]).into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .max_w(Px(520.0))
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| {
        let file = shadcn::MenubarMenu::new("ملف").entries([
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("علامة تبويب جديدة")),
            shadcn::MenubarEntry::Separator,
            shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("طباعة...")),
        ]);

        shadcn::Menubar::new([file]).into_element(cx)
    },
);"#,
                ),
        ],
    );

    vec![body.test_id("ui-gallery-menubar-component")]
}
