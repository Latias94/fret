//! MenuSubContent helpers (Radix-aligned outcomes).
//!
//! Radix submenu content coordinates focus transfer:
//! - when a submenu opens via keyboard, focus moves into the submenu
//! - ArrowLeft closes the submenu and restores focus to its trigger
//!
//! These helpers keep wrappers from duplicating the per-item wiring.

use fret_core::{Rect, SemanticsRole};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle,
    SemanticsProps, SizeStyle,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::menu::sub;

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
    cx.semantics(
        SemanticsProps {
            layout: LayoutStyle::default(),
            role: SemanticsRole::Menu,
            ..Default::default()
        },
        move |cx| {
            let layout = LayoutStyle {
                position: PositionStyle::Absolute,
                inset: InsetStyle {
                    left: Some(placed.origin.x),
                    top: Some(placed.origin.y),
                    ..Default::default()
                },
                size: SizeStyle {
                    width: Length::Px(placed.size.width),
                    height: Length::Px(placed.size.height),
                    ..Default::default()
                },
                overflow: Overflow::Clip,
                ..Default::default()
            };
            vec![cx.container(build_container(layout), f)]
        },
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
