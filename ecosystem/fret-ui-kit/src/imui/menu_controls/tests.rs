use std::sync::Arc;

use super::element::menu_item_element_with_pressable_hook_inner;
use super::visual::{menu_item_indicator_text, menu_item_label_text, menu_item_shortcut_text};

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, SemanticsRole, Size, TextOverflow, TextWrap};
use fret_ui::element::{ElementKind, Length};
use fret_ui::elements::{self, GlobalElementId};
use fret_ui::{ElementContext, UiHost};

use crate::imui::{MenuItemOptions, ResponseExt};

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    )
}

fn noop_test_menu_item_pressable_hook<H: UiHost>(
    _cx: &mut ElementContext<'_, H>,
    _state: fret_ui::element::PressableState,
    _item_id: GlobalElementId,
    _enabled: bool,
) {
}

mod root;
mod text_roles;
