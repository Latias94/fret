//! Menu content panel helpers (Radix-aligned outcomes).
//!
//! This module provides a small, reusable skeleton for positioning menu content panels:
//! - `role=menu` semantics
//! - absolute-positioned panel container clipped to its rect
//!
//! Wrappers (DropdownMenu, ContextMenu, Menubar, etc) should provide styling and inner structure
//! (scroll, roving focus group, items) via closures.

use fret_core::{Rect, SemanticsRole};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle,
    SemanticsProps, SizeStyle,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

/// Render a menu content semantics wrapper (`role=menu`) and return its stable element id.
///
/// This is intended for `aria-controls`-style trigger relationships (`controls_element`).
#[track_caller]
pub fn menu_content_semantics_with_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutStyle,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> (GlobalElementId, AnyElement) {
    let element = cx.semantics_with_id(
        SemanticsProps {
            layout,
            role: SemanticsRole::Menu,
            ..Default::default()
        },
        move |cx, _id| f(cx),
    );
    (element.id, element)
}

/// Returns a stable menu content element id for a given overlay root name.
///
/// This mirrors the `aria-controls`/`controls_element` outcome used by Radix triggers: the trigger
/// can reference the menu content element even while it is not mounted.
#[track_caller]
pub fn menu_content_semantics_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    overlay_root_name: &str,
) -> GlobalElementId {
    cx.with_root_name(overlay_root_name, |cx| {
        menu_content_semantics_with_id::<H>(cx, LayoutStyle::default(), |_cx| Vec::new()).0
    })
}

/// Render the menu panel container at `placed`, without adding a semantics wrapper.
///
/// This is useful when a wrapper already provides a `SemanticsRole::Menu` element and only wants
/// to reuse the absolute-positioned container layout.
pub fn menu_panel_container_at<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    placed: Rect,
    build_container: impl FnOnce(LayoutStyle) -> ContainerProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
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
    cx.container(build_container(layout), f)
}

/// Render a menu panel at `placed` with `role=menu` semantics.
pub fn menu_panel_at<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    placed: Rect,
    build_container: impl FnOnce(LayoutStyle) -> ContainerProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    menu_content_semantics_with_id(cx, LayoutStyle::default(), move |cx| {
        vec![menu_panel_container_at(cx, placed, build_container, f)]
    })
    .1
}
