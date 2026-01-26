//! MenuSubContent helpers (Radix-aligned outcomes).
//!
//! Radix submenu content coordinates focus transfer:
//! - when a submenu opens via keyboard, focus moves into the submenu
//! - ArrowLeft closes the submenu and restores focus to its trigger
//!
//! These helpers keep wrappers from duplicating the per-item wiring.

use fret_core::Rect;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, Overflow, ScrollAxis, ScrollProps, SizeStyle,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use std::sync::Arc;

use crate::primitives::direction::{self as direction_prim, LayoutDirection};
use crate::primitives::menu::content_panel;
use crate::primitives::menu::sub;
use crate::primitives::menu::{content, content::RovingFlexProps, content::TypeaheadPolicy};

fn with_submenu_value_scope<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    open_value: &Arc<str>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    cx.keyed(open_value.clone(), f)
}

fn submenu_content_semantics_id_in_scope<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open_value: &Arc<str>,
) -> GlobalElementId {
    with_submenu_value_scope(cx, open_value, |cx| {
        // Compute the id via the same call path used by the actual mounted submenu panel
        // (`submenu_panel_for_value_at` -> `submenu_panel_at` -> `menu_panel_at`), so callsite-based
        // element ids stay aligned.
        content_panel::menu_panel_at_with_labelled_by_element(
            cx,
            Rect::new(
                fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
                fret_core::Size::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            ),
            None,
            |layout| ContainerProps {
                layout,
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
        .id
    })
}

/// Returns the stable semantics element id for a submenu content panel keyed by its trigger value.
///
/// This mirrors Radix `MenuSubTrigger` / `MenuSubContent` behavior where the trigger advertises a
/// `controls` relationship (`aria-controls`) to the submenu content.
///
/// Callers should use this root-name-scoped helper rather than trying to compute the id inline
/// while rendering menu items: id computation must not advance the current frame's element id
/// counters, otherwise it will desync from the actual mounted submenu content element.
#[track_caller]
pub fn submenu_content_semantics_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    overlay_root_name: &str,
    open_value: &Arc<str>,
) -> GlobalElementId {
    cx.with_root_name(overlay_root_name, |cx| {
        submenu_content_semantics_id_in_scope::<H>(cx, open_value)
    })
}

/// Render a submenu content panel anchored at the submenu geometry's floating rect.
///
/// This keeps wrappers from duplicating the "role=menu + absolute positioned panel container"
/// skeleton while still allowing each wrapper (DropdownMenu, Menubar, etc) to provide its own
/// styling and inner content structure.
pub fn submenu_panel_at<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    placed: Rect,
    labelled_by_element: Option<GlobalElementId>,
    build_container: impl FnOnce(LayoutStyle) -> ContainerProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    content_panel::menu_panel_at_with_labelled_by_element(
        cx,
        placed,
        labelled_by_element,
        build_container,
        f,
    )
}

/// Render a submenu panel and scope its element ids by the submenu trigger value.
///
/// This ensures the submenu content element id is stable and can be referenced by the trigger via
/// `submenu_content_semantics_id`.
pub fn submenu_panel_for_value_at<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    open_value: Arc<str>,
    placed: Rect,
    labelled_by_element: Option<GlobalElementId>,
    build_container: impl FnOnce(LayoutStyle) -> ContainerProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    with_submenu_value_scope(cx, &open_value, |cx| {
        submenu_panel_at(cx, placed, labelled_by_element, build_container, f)
    })
}

fn submenu_scroll_y_fill<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let scroll_layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Fill,
            ..Default::default()
        },
        overflow: Overflow::Clip,
        ..Default::default()
    };
    cx.scroll(
        ScrollProps {
            layout: scroll_layout,
            axis: ScrollAxis::Y,
            ..Default::default()
        },
        f,
    )
}

/// Render a submenu panel that wraps its content in a scroll container (Y axis).
///
/// This matches the Radix Menu pattern of sizing the panel viewport to the available height and
/// scrolling the internal list when content exceeds that viewport.
pub fn submenu_panel_scroll_y_for_value_at<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    open_value: Arc<str>,
    placed: Rect,
    labelled_by_element: Option<GlobalElementId>,
    build_container: impl FnOnce(LayoutStyle) -> ContainerProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    submenu_panel_for_value_at(
        cx,
        open_value,
        placed,
        labelled_by_element,
        build_container,
        move |cx| vec![submenu_scroll_y_fill(cx, f)],
    )
}

/// Render a submenu roving group with APG-aligned keyboard navigation and prefix typeahead.
pub fn submenu_roving_group_apg_prefix_typeahead<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: RovingFlexProps,
    labels: Arc<[Arc<str>]>,
    timeout_ticks: u64,
    models: sub::MenuSubmenuModels,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let dir = direction_prim::use_direction_in_scope(cx, None);
    content::menu_roving_group_apg(
        cx,
        props,
        TypeaheadPolicy::PrefixAlwaysWrap {
            labels,
            timeout_ticks,
        },
        move |cx| {
            let models = models.clone();
            cx.roving_add_on_key_down(Arc::new(move |host, acx, down| {
                use fret_core::KeyCode;

                if down.repeat {
                    return false;
                }
                let is_close_key = match (down.key, dir) {
                    (KeyCode::ArrowLeft, LayoutDirection::Ltr) => true,
                    (KeyCode::ArrowRight, LayoutDirection::Rtl) => true,
                    _ => false,
                };
                if !is_close_key {
                    return false;
                }
                sub::close_and_restore_trigger(host, acx, &models);
                true
            }));
            f(cx)
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
    cx.key_prepend_on_key_down_for(
        item_id,
        sub::submenu_item_close_key_handler(
            models.clone(),
            direction_prim::use_direction_in_scope(cx, None),
        ),
    );
}
