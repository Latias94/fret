pub const SOURCE: &str = include_str!("checkbox.rs");

// region: example
use fret_core::Px;
use fret_runtime::CommandId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();
    let view_bookmarks_bar = cx.local_model_keyed("view_bookmarks_bar", || false);
    let view_full_urls = cx.local_model_keyed("view_full_urls", || true);
    let format_strikethrough = cx.local_model_keyed("format_strikethrough", || true);
    let format_code = cx.local_model_keyed("format_code", || false);
    let format_superscript = cx.local_model_keyed("format_superscript", || false);

    let view = shadcn::MenubarMenu::new("View").entries([
        shadcn::MenubarEntry::CheckboxItem(
            shadcn::MenubarCheckboxItem::new(
                view_bookmarks_bar.clone(),
                "Always Show Bookmarks Bar",
            )
            .action(CommandId::new(
                "ui_gallery.menubar.checkbox.view_bookmarks_bar",
            )),
        ),
        shadcn::MenubarEntry::CheckboxItem(
            shadcn::MenubarCheckboxItem::new(view_full_urls.clone(), "Always Show Full URLs")
                .action(CommandId::new("ui_gallery.menubar.checkbox.view_full_urls")),
        ),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Reload")
                .action(CommandId::new("ui_gallery.menubar.checkbox.reload"))
                .inset(true)
                .trailing(shadcn::MenubarShortcut::new("⌘R").into_element(cx)),
        ),
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Force Reload")
                .action(CommandId::new("ui_gallery.menubar.checkbox.force_reload"))
                .disabled(true)
                .inset(true)
                .trailing(shadcn::MenubarShortcut::new("⇧⌘R").into_element(cx)),
        ),
    ]);

    let format = shadcn::MenubarMenu::new("Format").entries([
        shadcn::MenubarEntry::CheckboxItem(
            shadcn::MenubarCheckboxItem::new(format_strikethrough.clone(), "Strikethrough").action(
                CommandId::new("ui_gallery.menubar.checkbox.format_strikethrough"),
            ),
        ),
        shadcn::MenubarEntry::CheckboxItem(
            shadcn::MenubarCheckboxItem::new(format_code.clone(), "Code")
                .action(CommandId::new("ui_gallery.menubar.checkbox.format_code")),
        ),
        shadcn::MenubarEntry::CheckboxItem(
            shadcn::MenubarCheckboxItem::new(format_superscript.clone(), "Superscript").action(
                CommandId::new("ui_gallery.menubar.checkbox.format_superscript"),
            ),
        ),
    ]);

    shadcn::Menubar::new([view, format])
        .refine_layout(width)
        .into_element(cx)
}
// endregion: example
