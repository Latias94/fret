//! Radix Select `item-aligned` positioning math.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/select/src/select.tsx` (`SelectItemAlignedPosition`)

use fret_core::{LayoutDirection, Px, Rect};

/// Matches Radix `CONTENT_MARGIN` (px).
pub const SELECT_ITEM_ALIGNED_CONTENT_MARGIN: Px = Px(10.0);

#[derive(Debug, Clone, Copy)]
pub struct SelectItemAlignedInputs {
    pub direction: LayoutDirection,

    pub window: Rect,
    pub trigger: Rect,

    /// The last known content panel rect (used to keep width stable across repositioning).
    pub content: Rect,

    /// Trigger value node rect (the text node that displays the selected value).
    pub value_node: Rect,

    /// Selected item's text rect inside the content.
    pub selected_item_text: Rect,

    /// Selected item (row) rect inside the content.
    pub selected_item: Rect,

    /// The listbox/viewport rect that establishes scroll offsets for items.
    pub viewport: Rect,

    pub content_border_top: Px,
    pub content_padding_top: Px,
    pub content_border_bottom: Px,
    pub content_padding_bottom: Px,

    pub viewport_padding_top: Px,
    pub viewport_padding_bottom: Px,

    /// Whether the alignment item is the first selectable item (Radix `items[0]`).
    pub selected_item_is_first: bool,
    /// Whether the alignment item is the last selectable item (Radix `items[items.length - 1]`).
    pub selected_item_is_last: bool,

    /// Scrollable height of all items (Radix `viewport.scrollHeight`).
    pub items_height: Px,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectItemAlignedOutputs {
    pub left: Option<Px>,
    pub right: Option<Px>,
    pub top: Option<Px>,
    pub bottom: Option<Px>,
    pub width: Px,
    pub min_width: Px,
    pub height: Px,
    pub min_height: Px,
    pub max_height: Px,
    /// Desired scroll offset to maintain trigger<->selected alignment when clamping to top.
    pub scroll_to_y: Option<Px>,
}

fn clamp(px: Px, min: Px, max: Px) -> Px {
    Px(px.0.clamp(min.0, max.0))
}

fn midpoint_y(rect: Rect) -> Px {
    Px(rect.origin.y.0 + rect.size.height.0 / 2.0)
}

pub fn select_item_aligned_position(inputs: SelectItemAlignedInputs) -> SelectItemAlignedOutputs {
    let margin = SELECT_ITEM_ALIGNED_CONTENT_MARGIN;

    let window_left = inputs.window.origin.x;
    let window_top = inputs.window.origin.y;
    let window_right = Px(inputs.window.origin.x.0 + inputs.window.size.width.0);
    let left_edge = Px(window_left.0 + margin.0);
    let right_edge = Px(window_right.0 - margin.0);

    // -----------------------------------------------------------------------------------------
    // Horizontal positioning
    // -----------------------------------------------------------------------------------------
    let item_text_offset_ltr = Px(inputs.selected_item_text.origin.x.0 - inputs.content.origin.x.0);
    let item_text_offset_rtl = Px((inputs.content.origin.x.0 + inputs.content.size.width.0)
        - (inputs.selected_item_text.origin.x.0 + inputs.selected_item_text.size.width.0));

    let (left, right, min_width, width) = match inputs.direction {
        LayoutDirection::Ltr => {
            let left = Px(inputs.value_node.origin.x.0 - item_text_offset_ltr.0);
            let left_delta = Px(inputs.trigger.origin.x.0 - left.0);
            let min_width = Px(inputs.trigger.size.width.0 + left_delta.0);
            let width = Px(min_width.0.max(inputs.content.size.width.0));

            let max_left = Px((right_edge.0 - width.0).max(left_edge.0));
            let clamped_left = clamp(left, left_edge, max_left);

            (Some(clamped_left), None, min_width, width)
        }
        LayoutDirection::Rtl => {
            let right = Px((window_right.0
                - inputs.value_node.origin.x.0
                - inputs.value_node.size.width.0)
                - item_text_offset_rtl.0);
            let right_delta = Px((window_right.0
                - inputs.trigger.origin.x.0
                - inputs.trigger.size.width.0)
                - right.0);
            let min_width = Px(inputs.trigger.size.width.0 + right_delta.0);
            let width = Px(min_width.0.max(inputs.content.size.width.0));

            let max_right = Px((right_edge.0 - width.0).max(left_edge.0));
            let clamped_right = clamp(right, left_edge, max_right);

            (None, Some(clamped_right), min_width, width)
        }
    };

    // -----------------------------------------------------------------------------------------
    // Vertical positioning
    // -----------------------------------------------------------------------------------------
    let available_height = Px((inputs.window.size.height.0 - margin.0 * 2.0).max(0.0));

    let selected_item_half_h = Px(inputs.selected_item.size.height.0 / 2.0);

    let viewport_origin_y = inputs.viewport.origin.y;
    // Radix uses `selectedItem.offsetTop`, measured from the viewport's padding edge.
    //
    // In Fret, the viewport rect typically reflects the viewport's outer box, and child layout
    // already includes padding in its origin, so `selected_item.origin - viewport.origin` behaves
    // like `offsetTop` (padding-inclusive). Keep the solver aligned with that coordinate system.
    let selected_item_mid_offset =
        Px((inputs.selected_item.origin.y.0 - viewport_origin_y.0) + selected_item_half_h.0);

    let full_content_h = Px(inputs.content_border_top.0
        + inputs.content_padding_top.0
        + inputs.items_height.0
        + inputs.content_padding_bottom.0
        + inputs.content_border_bottom.0);

    let min_height = Px((selected_item_half_h.0 * 10.0).min(full_content_h.0));

    let top_edge_to_trigger_mid = Px(midpoint_y(inputs.trigger).0 - margin.0 - window_top.0);
    // Radix constrains `maxHeight` to `window.innerHeight - CONTENT_MARGIN*2`, but when that would
    // fall below the `minHeight` (5 rows) it can overflow the collision boundary and still clamp
    // the origin to the top edge. Model this by relaxing the effective max height in that case.
    let max_height = if available_height.0 < min_height.0 {
        inputs.window.size.height
    } else {
        available_height
    };

    let trigger_mid_to_bottom_edge = Px(max_height.0 - top_edge_to_trigger_mid.0);

    let content_top_to_item_mid =
        Px(inputs.content_border_top.0 + inputs.content_padding_top.0 + selected_item_mid_offset.0);
    let item_mid_to_content_bottom = Px(full_content_h.0 - content_top_to_item_mid.0);

    let will_align_without_top_overflow = content_top_to_item_mid.0 <= top_edge_to_trigger_mid.0;

    if will_align_without_top_overflow {
        // Match Radix:
        // `viewportOffsetBottom = content.clientHeight - viewport.offsetTop - viewport.offsetHeight`.
        //
        // `clientHeight` excludes borders, and `offsetTop` is measured from the content's padding
        // edge. Compute the same offsets in window space by switching to the content inner box.
        let content_inner_bottom = Px(inputs.content.origin.y.0 + inputs.content.size.height.0
            - inputs.content_padding_bottom.0
            - inputs.content_border_bottom.0);
        let viewport_offset_bottom =
            Px(content_inner_bottom.0
                - (inputs.viewport.origin.y.0 + inputs.viewport.size.height.0));
        let viewport_padding_bottom = if inputs.selected_item_is_last {
            inputs.viewport_padding_bottom
        } else {
            Px(0.0)
        };
        let clamped_trigger_mid_to_bottom_edge = Px(trigger_mid_to_bottom_edge.0.max(
            selected_item_half_h.0
                + viewport_padding_bottom.0
                + viewport_offset_bottom.0
                + inputs.content_border_bottom.0,
        ));
        let height = Px(
            (content_top_to_item_mid.0 + clamped_trigger_mid_to_bottom_edge.0).min(max_height.0),
        );
        let height = Px(height.0.min(full_content_h.0));

        SelectItemAlignedOutputs {
            left,
            right,
            top: None,
            bottom: Some(Px(0.0)),
            width,
            min_width,
            height,
            min_height: Px(min_height.0.min(height.0)),
            max_height,
            scroll_to_y: None,
        }
    } else {
        // Match Radix:
        // `viewport.offsetTop` is measured from the content's padding edge (border excluded).
        let content_inner_top = Px(inputs.content.origin.y.0
            + inputs.content_border_top.0
            + inputs.content_padding_top.0);
        let viewport_offset_top = Px(inputs.viewport.origin.y.0 - content_inner_top.0);
        let viewport_padding_top = if inputs.selected_item_is_first {
            inputs.viewport_padding_top
        } else {
            Px(0.0)
        };
        let clamped_top_edge_to_trigger_mid = Px(top_edge_to_trigger_mid.0.max(
            inputs.content_border_top.0
                + viewport_offset_top.0
                + viewport_padding_top.0
                + selected_item_half_h.0,
        ));
        let height = Px(
            (clamped_top_edge_to_trigger_mid.0 + item_mid_to_content_bottom.0).min(max_height.0),
        );
        let height = Px(height.0.min(full_content_h.0));

        let scroll_to =
            Px(content_top_to_item_mid.0 - top_edge_to_trigger_mid.0 + viewport_offset_top.0);

        SelectItemAlignedOutputs {
            left,
            right,
            top: Some(Px(0.0)),
            bottom: None,
            width,
            min_width,
            height,
            min_height: Px(min_height.0.min(height.0)),
            max_height,
            scroll_to_y: Some(scroll_to),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Px, Rect, Size};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn ltr_computes_min_width_and_clamps_left() {
        let out = select_item_aligned_position(SelectItemAlignedInputs {
            direction: LayoutDirection::Ltr,
            window: rect(0.0, 0.0, 300.0, 200.0),
            trigger: rect(100.0, 40.0, 80.0, 24.0),
            content: rect(0.0, 0.0, 120.0, 100.0),
            value_node: rect(110.0, 44.0, 60.0, 16.0),
            selected_item_text: rect(20.0, 60.0, 60.0, 16.0),
            selected_item: rect(10.0, 56.0, 100.0, 24.0),
            viewport: rect(10.0, 50.0, 120.0, 100.0),
            content_border_top: Px(1.0),
            content_padding_top: Px(0.0),
            content_border_bottom: Px(1.0),
            content_padding_bottom: Px(0.0),
            viewport_padding_top: Px(4.0),
            viewport_padding_bottom: Px(4.0),
            selected_item_is_first: false,
            selected_item_is_last: false,
            items_height: Px(200.0),
        });

        assert!(out.left.is_some());
        assert_eq!(out.right, None);
        assert!(out.min_width.0 >= 80.0);
        assert!(out.width.0 >= out.min_width.0);
        assert!(out.left.unwrap().0 >= SELECT_ITEM_ALIGNED_CONTENT_MARGIN.0);
    }

    #[test]
    fn vertical_prefers_bottom_when_alignment_fits() {
        let out = select_item_aligned_position(SelectItemAlignedInputs {
            direction: LayoutDirection::Ltr,
            window: rect(0.0, 0.0, 300.0, 200.0),
            trigger: rect(20.0, 40.0, 80.0, 24.0),
            content: rect(0.0, 0.0, 120.0, 100.0),
            value_node: rect(30.0, 44.0, 60.0, 16.0),
            selected_item_text: rect(20.0, 60.0, 60.0, 16.0),
            selected_item: rect(10.0, 56.0, 100.0, 24.0),
            viewport: rect(10.0, 50.0, 120.0, 100.0),
            content_border_top: Px(1.0),
            content_padding_top: Px(0.0),
            content_border_bottom: Px(1.0),
            content_padding_bottom: Px(0.0),
            viewport_padding_top: Px(4.0),
            viewport_padding_bottom: Px(4.0),
            selected_item_is_first: false,
            selected_item_is_last: false,
            items_height: Px(200.0),
        });

        assert_eq!(out.bottom, Some(Px(0.0)));
        assert_eq!(out.top, None);
        assert_eq!(out.scroll_to_y, None);
    }
}
