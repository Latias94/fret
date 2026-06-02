use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::imui::MenuItemOptions;

use super::super::visual;

mod content;
mod layout;

pub(super) struct MenuItemVisualRow {
    label: Arc<str>,
    shortcut: Option<Arc<str>>,
    shortcut_test_id: Option<Arc<str>>,
    submenu: bool,
    role: SemanticsRole,
    checked: Option<bool>,
}

impl MenuItemVisualRow {
    pub(super) fn from_options(
        label: Arc<str>,
        options: &MenuItemOptions,
        role: SemanticsRole,
        checked: Option<bool>,
    ) -> Self {
        let test_id = options.test_id.clone();
        let shortcut_test_id = options.shortcut_test_id.clone().or_else(|| {
            test_id
                .as_ref()
                .map(|test_id| Arc::from(format!("{test_id}.shortcut")))
        });
        Self {
            label,
            shortcut: options.shortcut.clone(),
            shortcut_test_id,
            submenu: options.submenu,
            role,
            checked,
        }
    }
}

pub(super) fn render_menu_item_visual_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    visual_row: MenuItemVisualRow,
) -> AnyElement {
    cx.container(layout::menu_item_panel_props(), move |cx| {
        let indicator = content::menu_item_indicator(visual_row.role, visual_row.checked);
        vec![cx.row(layout::menu_item_row_props(), move |cx| {
            let mut out: Vec<AnyElement> = Vec::new();
            content::push_menu_item_leading_indicator(cx, &mut out, indicator.clone());
            out.push(visual::menu_item_label_text(cx, visual_row.label.clone()));
            content::push_menu_item_trailing_content(
                cx,
                &mut out,
                visual_row.shortcut.clone(),
                visual_row.shortcut_test_id.clone(),
                visual_row.submenu,
            );
            out
        })]
    })
}
