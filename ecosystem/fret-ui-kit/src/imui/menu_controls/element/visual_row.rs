use std::sync::Arc;

use fret_core::{Edges, Px, SemanticsRole};
use fret_ui::element::{
    AnyElement, ContainerProps, Length, RowProps, SemanticsDecoration, SpacerProps, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use crate::imui::MenuItemOptions;

use super::super::visual;

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
    cx.container(menu_item_panel_props(), move |cx| {
        let indicator = menu_item_indicator(visual_row.role, visual_row.checked);
        vec![cx.row(menu_item_row_props(), move |cx| {
            let mut out: Vec<AnyElement> = Vec::new();
            if let Some(indicator) = indicator.clone() {
                out.push(visual::menu_item_indicator_text(cx, indicator));
            }
            out.push(visual::menu_item_label_text(cx, visual_row.label.clone()));

            if let Some(shortcut) = visual_row.shortcut.clone() {
                out.push(cx.spacer(SpacerProps::default()));

                let mut shortcut = visual::menu_item_shortcut_text(cx, shortcut);
                if let Some(test_id) = visual_row.shortcut_test_id.clone() {
                    shortcut =
                        shortcut.attach_semantics(SemanticsDecoration::default().test_id(test_id));
                }
                out.push(shortcut);
            } else if visual_row.submenu {
                out.push(cx.spacer(SpacerProps::default()));
                out.push(visual::menu_item_indicator_text(cx, Arc::from("\u{203A}")));
            }
            out
        })]
    })
}

fn menu_item_panel_props() -> ContainerProps {
    let mut panel = ContainerProps::default();
    panel.layout.size.width = Length::Fill;
    panel.layout.size.height = Length::Auto;
    panel.padding = Edges {
        left: Px(6.0),
        right: Px(6.0),
        top: Px(2.0),
        bottom: Px(2.0),
    }
    .into();
    panel
}

fn menu_item_row_props() -> RowProps {
    let mut row = RowProps::default();
    row.layout.size.width = Length::Fill;
    row.layout.size.height = Length::Auto;
    row.gap = SpacingLength::Px(Px(6.0));
    row
}

fn menu_item_indicator(role: SemanticsRole, checked: Option<bool>) -> Option<Arc<str>> {
    match (role, checked) {
        (SemanticsRole::MenuItemCheckbox, Some(true)) => Some(Arc::from("\u{2713}")),
        (SemanticsRole::MenuItemCheckbox, Some(false)) => Some(Arc::from(" ")),
        (SemanticsRole::MenuItemRadio, Some(true)) => Some(Arc::from("\u{25CF}")),
        (SemanticsRole::MenuItemRadio, Some(false)) => Some(Arc::from(" ")),
        _ => None,
    }
}
