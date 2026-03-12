pub const SOURCE: &str = include_str!("checkbox.rs");

// region: example
use fret_core::Px;
use fret_runtime::CommandId;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct MenubarCheckboxState {
    view_bookmarks_bar: bool,
    view_full_urls: bool,
    format_strikethrough: bool,
    format_code: bool,
    format_superscript: bool,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();
    let state = cx.local_model(|| MenubarCheckboxState {
        view_bookmarks_bar: false,
        view_full_urls: true,
        format_strikethrough: true,
        format_code: false,
        format_superscript: false,
    });
    let state_now = cx.watch_model(&state).layout().cloned().unwrap_or_default();

    let view = shadcn::MenubarMenu::new("View").entries([
        shadcn::MenubarEntry::CheckboxItem(
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
            .action(CommandId::new(
                "ui_gallery.menubar.checkbox.view_bookmarks_bar",
            )),
        ),
        shadcn::MenubarEntry::CheckboxItem(
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
            shadcn::MenubarCheckboxItem::from_checked(
                state_now.format_strikethrough,
                "Strikethrough",
            )
            .on_checked_change({
                let state = state.clone();
                move |host, _action_cx, checked| {
                    let _ = host
                        .models_mut()
                        .update(&state, |st| st.format_strikethrough = checked);
                }
            })
            .action(CommandId::new(
                "ui_gallery.menubar.checkbox.format_strikethrough",
            )),
        ),
        shadcn::MenubarEntry::CheckboxItem(
            shadcn::MenubarCheckboxItem::from_checked(state_now.format_code, "Code")
                .on_checked_change({
                    let state = state.clone();
                    move |host, _action_cx, checked| {
                        let _ = host
                            .models_mut()
                            .update(&state, |st| st.format_code = checked);
                    }
                })
                .action(CommandId::new("ui_gallery.menubar.checkbox.format_code")),
        ),
        shadcn::MenubarEntry::CheckboxItem(
            shadcn::MenubarCheckboxItem::from_checked(state_now.format_superscript, "Superscript")
                .on_checked_change({
                    let state = state.clone();
                    move |host, _action_cx, checked| {
                        let _ = host
                            .models_mut()
                            .update(&state, |st| st.format_superscript = checked);
                    }
                })
                .action(CommandId::new(
                    "ui_gallery.menubar.checkbox.format_superscript",
                )),
        ),
    ]);

    shadcn::Menubar::new([view, format])
        .refine_layout(width)
        .into_element(cx)
}
// endregion: example
