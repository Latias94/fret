use fret_core::Px;
use fret_ui::element::{AnyElement, LayoutStyle};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use super::layout::{
    PropertyRowResolvedLayout, apply_property_row_min_height, resolve_property_row_layout,
    resolve_property_row_layout_variant,
};
use super::{PropertyRowLayoutVariant, PropertyRowOptions, PropertyRowReset};

mod column;
mod row;

use column::{PropertyRowColumnElementOptions, property_row_column_element};
use row::{PropertyRowRowElementOptions, property_row_row_element};

#[cfg(test)]
pub(crate) const PROPERTY_ROW_VALUE_SLOT: &str = "fret-ui-editor.property-row.value";

#[cfg(test)]
fn mark_property_row_value_slot(element: AnyElement) -> AnyElement {
    element.component_slot(PROPERTY_ROW_VALUE_SLOT)
}

#[cfg(not(test))]
fn mark_property_row_value_slot(element: AnyElement) -> AnyElement {
    element
}

fn kind_layout_mut(kind: &mut fret_ui::element::ElementKind) -> Option<&mut LayoutStyle> {
    use fret_ui::element::ElementKind;

    match kind {
        ElementKind::Container(props) => Some(&mut props.layout),
        ElementKind::Semantics(props) => Some(&mut props.layout),
        ElementKind::SemanticFlex(props) => Some(&mut props.flex.layout),
        ElementKind::Pressable(props) => Some(&mut props.layout),
        ElementKind::PointerRegion(props) => Some(&mut props.layout),
        ElementKind::TextInputRegion(props) => Some(&mut props.layout),
        ElementKind::InternalDragRegion(props) => Some(&mut props.layout),
        ElementKind::Opacity(props) => Some(&mut props.layout),
        ElementKind::InteractivityGate(props) => Some(&mut props.layout),
        ElementKind::VisualTransform(props) => Some(&mut props.layout),
        ElementKind::RenderTransform(props) => Some(&mut props.layout),
        ElementKind::FractionalRenderTransform(props) => Some(&mut props.layout),
        ElementKind::Anchored(props) => Some(&mut props.layout),
        ElementKind::Column(props) => Some(&mut props.layout),
        ElementKind::Row(props) => Some(&mut props.layout),
        ElementKind::Stack(props) => Some(&mut props.layout),
        ElementKind::Flex(props) => Some(&mut props.layout),
        ElementKind::Grid(props) => Some(&mut props.layout),
        ElementKind::Text(props) => Some(&mut props.layout),
        ElementKind::StyledText(props) => Some(&mut props.layout),
        ElementKind::SelectableText(props) => Some(&mut props.layout),
        ElementKind::TextInput(props) => Some(&mut props.layout),
        ElementKind::TextArea(props) => Some(&mut props.layout),
        ElementKind::Image(props) => Some(&mut props.layout),
        ElementKind::Canvas(props) => Some(&mut props.layout),
        ElementKind::SvgIcon(props) => Some(&mut props.layout),
        ElementKind::SvgImage(props) => Some(&mut props.layout),
        ElementKind::Spinner(props) => Some(&mut props.layout),
        ElementKind::Scroll(props) => Some(&mut props.layout),
        ElementKind::Scrollbar(props) => Some(&mut props.layout),
        ElementKind::Spacer(props) => Some(&mut props.layout),
        ElementKind::HoverRegion(props) => Some(&mut props.layout),
        ElementKind::WheelRegion(props) => Some(&mut props.layout),
        ElementKind::EffectLayer(props) => Some(&mut props.layout),
        ElementKind::FocusScope(props) => Some(&mut props.layout),
        ElementKind::RovingFlex(props) => Some(&mut props.flex.layout),
        ElementKind::VirtualList(props) => Some(&mut props.layout),
        ElementKind::ResizablePanelGroup(props) => Some(&mut props.layout),
        ElementKind::ViewportSurface(props) => Some(&mut props.layout),
        ElementKind::ViewCache(props) => Some(&mut props.layout),
        ElementKind::ManagedSurface(props) => Some(&mut props.layout),
        _ => None,
    }
}

pub(super) fn property_row_land_child<H, Build, Patch, Fallback>(
    cx: &mut ElementContext<'_, H>,
    build: Build,
    patch: Patch,
    fallback: Fallback,
) -> AnyElement
where
    H: UiHost,
    Build: FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    Patch: FnOnce(&mut LayoutStyle),
    Fallback: FnOnce(&mut ElementContext<'_, H>, AnyElement) -> AnyElement,
{
    let mut element = build(cx);
    if let Some(layout) = kind_layout_mut(&mut element.kind) {
        patch(layout);
        element
    } else {
        fallback(cx, element)
    }
}

pub(super) fn property_row_element<H, Label, Value, Actions>(
    cx: &mut ElementContext<'_, H>,
    options: PropertyRowOptions,
    reset: Option<PropertyRowReset>,
    label: Label,
    value: Value,
    actions: Actions,
) -> AnyElement
where
    H: UiHost,
    Label: FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    Value: FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    Actions: FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>,
{
    let bounds = if matches!(options.variant, PropertyRowLayoutVariant::Auto) {
        cx.layout_query_bounds(cx.root_id(), Invalidation::Layout)
    } else {
        None
    };

    let has_reset_slot = reset.as_ref().is_some_and(|reset| reset.options.enabled);

    let PropertyRowResolvedLayout {
        density,
        affordance_extent,
        gap,
        trailing_gap,
        reset_fg,
        auto_below,
        label_w,
        value_max_w,
        status_slot_w,
        reset_slot_w,
    } = resolve_property_row_layout(Theme::global(&*cx.app), &options, has_reset_slot);

    let variant = resolve_property_row_layout_variant(options.variant, bounds, auto_below);

    let mut layout = options.layout;
    apply_property_row_min_height(&mut layout, density.row_height);

    let actions_el = actions(cx);
    let has_action_slot = actions_el.is_some();
    let status_slot_w = if has_action_slot {
        status_slot_w
    } else {
        Px(0.0)
    };
    let reset_slot_w = if has_reset_slot {
        reset_slot_w
    } else {
        Px(0.0)
    };

    let row = match variant {
        PropertyRowLayoutVariant::Row => property_row_row_element(
            cx,
            PropertyRowRowElementOptions {
                layout,
                density,
                affordance_extent,
                gap,
                trailing_gap,
                reset_fg,
                label_w,
                value_max_w,
                status_slot_w,
                reset_slot_w,
                has_reset_slot,
                reset,
                actions_el,
            },
            label,
            value,
        ),
        PropertyRowLayoutVariant::Column => property_row_column_element(
            cx,
            PropertyRowColumnElementOptions {
                layout,
                density,
                affordance_extent,
                trailing_gap,
                reset_fg,
                value_max_w,
                status_slot_w,
                reset_slot_w,
                has_reset_slot,
                reset,
                actions_el,
            },
            label,
            value,
        ),
        PropertyRowLayoutVariant::Auto => unreachable!("auto is resolved above"),
    };

    if let Some(test_id) = options.test_id.as_ref() {
        row.test_id(test_id.clone())
    } else {
        row
    }
}
