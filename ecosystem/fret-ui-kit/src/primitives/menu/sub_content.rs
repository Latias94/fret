//! MenuSubContent helpers (Radix-aligned outcomes).
//!
//! Radix submenu content coordinates focus transfer:
//! - when a submenu opens via keyboard, focus moves into the submenu
//! - ArrowLeft closes the submenu and restores focus to its trigger
//!
//! These helpers keep wrappers from duplicating the per-item wiring.

use fret_core::Rect;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use std::sync::Arc;

use crate::primitives::menu::content_panel;
use crate::primitives::menu::sub;
use crate::primitives::menu::{content, content::RovingFlexProps, content::TypeaheadPolicy};

/// Render a submenu content panel anchored at the submenu geometry's floating rect.
///
/// This keeps wrappers from duplicating the "role=menu + absolute positioned panel container"
/// skeleton while still allowing each wrapper (DropdownMenu, Menubar, etc) to provide its own
/// styling and inner content structure.
pub fn submenu_panel_at<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    placed: Rect,
    build_container: impl FnOnce(LayoutStyle) -> ContainerProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    content_panel::menu_panel_at(cx, placed, build_container, f)
}

/// Render a submenu roving group with APG-aligned keyboard navigation and prefix typeahead.
pub fn submenu_roving_group_apg_prefix_typeahead<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: RovingFlexProps,
    labels: Arc<[Arc<str>]>,
    timeout_ticks: u64,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    content::menu_roving_group_apg(
        cx,
        props,
        TypeaheadPolicy::Prefix {
            labels,
            timeout_ticks,
        },
        f,
    )
}

/// Wire submenu-content behavior for a single submenu item.
///
/// Intended to be called inside the submenu panel pressable closure.
pub fn wire_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    item_id: GlobalElementId,
    disabled: bool,
    models: &sub::MenuSubmenuModels,
) {
    sub::focus_first_available_on_open(cx, models, item_id, disabled);
    cx.key_on_key_down_for(
        item_id,
        sub::submenu_item_arrow_left_handler(models.clone()),
    );
}
