use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, SemanticsDecoration, SpacerProps};
use fret_ui::{ElementContext, UiHost};

use super::super::super::visual;

const SUBMENU_INDICATOR: &str = "\u{203A}";

pub(super) fn menu_item_indicator(role: SemanticsRole, checked: Option<bool>) -> Option<Arc<str>> {
    match (role, checked) {
        (SemanticsRole::MenuItemCheckbox, Some(true)) => Some(Arc::from("\u{2713}")),
        (SemanticsRole::MenuItemCheckbox, Some(false)) => Some(Arc::from(" ")),
        (SemanticsRole::MenuItemRadio, Some(true)) => Some(Arc::from("\u{25CF}")),
        (SemanticsRole::MenuItemRadio, Some(false)) => Some(Arc::from(" ")),
        _ => None,
    }
}

pub(super) fn push_menu_item_leading_indicator<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    out: &mut Vec<AnyElement>,
    indicator: Option<Arc<str>>,
) {
    if let Some(indicator) = indicator {
        out.push(visual::menu_item_indicator_text(cx, indicator));
    }
}

pub(super) fn push_menu_item_trailing_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    out: &mut Vec<AnyElement>,
    shortcut: Option<Arc<str>>,
    shortcut_test_id: Option<Arc<str>>,
    submenu: bool,
) {
    if let Some(shortcut) = shortcut {
        out.push(cx.spacer(SpacerProps::default()));

        let mut shortcut = visual::menu_item_shortcut_text(cx, shortcut);
        if let Some(test_id) = shortcut_test_id {
            shortcut = shortcut.attach_semantics(SemanticsDecoration::default().test_id(test_id));
        }
        out.push(shortcut);
    } else if submenu {
        out.push(cx.spacer(SpacerProps::default()));
        out.push(visual::menu_item_indicator_text(
            cx,
            Arc::from(SUBMENU_INDICATOR),
        ));
    }
}
