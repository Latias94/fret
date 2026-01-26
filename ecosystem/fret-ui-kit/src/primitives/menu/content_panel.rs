//! Menu content panel helpers (Radix-aligned outcomes).
//!
//! This module provides a small, reusable skeleton for positioning menu content panels:
//! - `role=menu` semantics
//! - absolute-positioned panel container clipped to its rect
//!
//! Wrappers (DropdownMenu, ContextMenu, Menubar, etc) should provide styling and inner structure
//! (scroll, roving focus group, items) via closures.

use fret_core::{Point, Px, Rect, SemanticsRole};
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle,
    SemanticsProps, SizeStyle,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

/// Render a menu content semantics wrapper (`role=menu`) and return its stable element id.
///
/// This is intended for `aria-controls`-style trigger relationships (`controls_element`).
pub fn menu_content_semantics_with_id<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutStyle,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> (GlobalElementId, AnyElement)
where
    I: IntoIterator<Item = AnyElement>,
{
    menu_content_semantics_with_id_props(
        cx,
        SemanticsProps {
            layout,
            role: SemanticsRole::Menu,
            ..Default::default()
        },
        f,
    )
}

/// Render a menu content semantics wrapper with explicit semantics props.
///
/// Callers may set relationship fields like `labelled_by_element` to mirror Radix `aria-*`
/// outcomes, while `role` is forced to `SemanticsRole::Menu`.
pub fn menu_content_semantics_with_id_props<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    mut props: SemanticsProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> (GlobalElementId, AnyElement)
where
    I: IntoIterator<Item = AnyElement>,
{
    props.role = SemanticsRole::Menu;
    let element = cx.semantics_with_id(props, move |cx, _id| f(cx));
    (element.id, element)
}

/// Returns a stable menu content element id for a given overlay root name.
///
/// This mirrors the `aria-controls`/`controls_element` outcome used by Radix triggers: the trigger
/// can reference the menu content element even while it is not mounted.
pub fn menu_content_semantics_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    overlay_root_name: &str,
) -> GlobalElementId {
    cx.with_root_name(overlay_root_name, |cx| {
        menu_content_semantics_with_id(cx, LayoutStyle::default(), |_cx| Vec::new()).0
    })
}

/// Render the menu panel container at `placed`, without adding a semantics wrapper.
///
/// This is useful when a wrapper already provides a `SemanticsRole::Menu` element and only wants
/// to reuse the absolute-positioned container layout.
pub fn menu_panel_container_at<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    placed: Rect,
    build_container: impl FnOnce(LayoutStyle) -> ContainerProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
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
pub fn menu_panel_at<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    placed: Rect,
    build_container: impl FnOnce(LayoutStyle) -> ContainerProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
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

    let local_placed = Rect::new(Point::new(Px(0.0), Px(0.0)), placed.size);
    menu_content_semantics_with_id(cx, layout, move |cx| {
        vec![menu_panel_container_at(
            cx,
            local_placed,
            build_container,
            f,
        )]
    })
    .1
}

/// Render a menu panel at `placed` with `role=menu` semantics and an optional `labelled_by`
/// relationship.
pub fn menu_panel_at_with_labelled_by_element<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    placed: Rect,
    labelled_by_element: Option<GlobalElementId>,
    build_container: impl FnOnce(LayoutStyle) -> ContainerProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
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

    let local_placed = Rect::new(Point::new(Px(0.0), Px(0.0)), placed.size);
    menu_content_semantics_with_id_props(
        cx,
        SemanticsProps {
            layout,
            labelled_by_element: labelled_by_element.map(|id| id.0),
            ..Default::default()
        },
        move |cx| {
            vec![menu_panel_container_at(
                cx,
                local_placed,
                build_container,
                f,
            )]
        },
    )
    .1
}
