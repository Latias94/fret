pub const SOURCE: &str = include_str!("checkbox.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    view_bookmarks_bar: Option<Model<bool>>,
    view_full_urls: Option<Model<bool>>,
    format_strikethrough: Option<Model<bool>>,
    format_code: Option<Model<bool>>,
    format_superscript: Option<Model<bool>>,
    _unused: Option<Model<Option<Arc<str>>>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());
    let width = LayoutRefinement::default().w_px(Px(288.0)).min_w_0();

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

    let format_strikethrough = match state.format_strikethrough {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(Models::default, |st| {
                st.format_strikethrough = Some(model.clone())
            });
            model
        }
    };

    let format_code = match state.format_code {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.format_code = Some(model.clone()));
            model
        }
    };

    let format_superscript = match state.format_superscript {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.format_superscript = Some(model.clone())
            });
            model
        }
    };

    let view = shadcn::MenubarMenu::new("View").entries([
        shadcn::MenubarEntry::CheckboxItem(shadcn::MenubarCheckboxItem::new(
            view_bookmarks_bar.clone(),
            "Always Show Bookmarks Bar",
        )),
        shadcn::MenubarEntry::CheckboxItem(shadcn::MenubarCheckboxItem::new(
            view_full_urls.clone(),
            "Always Show Full URLs",
        )),
        shadcn::MenubarEntry::Separator,
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Reload")
                .inset(true)
                .trailing(shadcn::MenubarShortcut::new("⌘R").into_element(cx)),
        ),
        shadcn::MenubarEntry::Item(
            shadcn::MenubarItem::new("Force Reload")
                .disabled(true)
                .inset(true)
                .trailing(shadcn::MenubarShortcut::new("⇧⌘R").into_element(cx)),
        ),
    ]);

    let format = shadcn::MenubarMenu::new("Format").entries([
        shadcn::MenubarEntry::CheckboxItem(shadcn::MenubarCheckboxItem::new(
            format_strikethrough.clone(),
            "Strikethrough",
        )),
        shadcn::MenubarEntry::CheckboxItem(shadcn::MenubarCheckboxItem::new(
            format_code.clone(),
            "Code",
        )),
        shadcn::MenubarEntry::CheckboxItem(shadcn::MenubarCheckboxItem::new(
            format_superscript.clone(),
            "Superscript",
        )),
    ]);

    shadcn::Menubar::new([view, format])
        .refine_layout(width)
        .into_element(cx)
}
// endregion: example
