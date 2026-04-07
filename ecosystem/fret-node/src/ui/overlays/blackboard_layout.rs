use fret_core::{Point, Px, Rect, Size};

use crate::core::SymbolId;
use crate::ui::screen_space_placement::{AxisAlign, rect_in_bounds};
use crate::ui::style::NodeGraphStyle;

use super::blackboard_policy::BlackboardAction;

const PANEL_MARGIN_PX: f32 = 12.0;
const BUTTON_GAP_PX: f32 = 6.0;

#[derive(Debug, Clone)]
pub(super) struct BlackboardRowLayout {
    pub(super) symbol: SymbolId,
    pub(super) label: Rect,
    pub(super) insert_ref: Rect,
    pub(super) rename: Rect,
    pub(super) delete: Rect,
}

#[derive(Debug, Clone)]
pub(super) struct BlackboardLayout {
    pub(super) panel: Rect,
    pub(super) header: Rect,
    pub(super) add_button: Rect,
    pub(super) rows: Vec<BlackboardRowLayout>,
}

pub(super) fn blackboard_panel_size(style: &NodeGraphStyle, rows: usize) -> Size {
    let pad = style.paint.context_menu_padding.max(0.0);
    let row_h = blackboard_row_height(style);
    let w = style.paint.context_menu_width.max(120.0);
    let h = (row_h + rows as f32 * row_h + 2.0 * pad).max(24.0);
    Size::new(Px(w), Px(h))
}

pub(super) fn compute_blackboard_layout<I>(
    style: &NodeGraphStyle,
    bounds: Rect,
    symbols: I,
) -> BlackboardLayout
where
    I: IntoIterator<Item = SymbolId>,
{
    let symbols: Vec<_> = symbols.into_iter().collect();
    let size = blackboard_panel_size(style, symbols.len());
    let panel = rect_in_bounds(
        bounds,
        size,
        AxisAlign::Start,
        AxisAlign::Start,
        PANEL_MARGIN_PX,
        Point::new(Px(0.0), Px(0.0)),
    );

    let pad = style.paint.context_menu_padding.max(0.0);
    let row_h = blackboard_row_height(style);
    let inner_x = panel.origin.x.0 + pad;
    let inner_y = panel.origin.y.0 + pad;
    let inner_w = (panel.size.width.0 - 2.0 * pad).max(0.0);

    let header = Rect::new(
        Point::new(Px(inner_x), Px(inner_y)),
        Size::new(Px(inner_w), Px(row_h)),
    );

    let button_w = row_h.max(18.0);
    let add_button = Rect::new(
        Point::new(
            Px(header.origin.x.0 + (header.size.width.0 - button_w).max(0.0)),
            header.origin.y,
        ),
        Size::new(Px(button_w), header.size.height),
    );

    let mut rows = Vec::with_capacity(symbols.len());
    let mut y = inner_y + header.size.height.0;
    for symbol in symbols {
        let row = Rect::new(
            Point::new(Px(inner_x), Px(y)),
            Size::new(Px(inner_w), Px(row_h)),
        );

        let delete = Rect::new(
            Point::new(
                Px(row.origin.x.0 + (row.size.width.0 - button_w).max(0.0)),
                row.origin.y,
            ),
            Size::new(Px(button_w), Px(row_h)),
        );
        let rename = Rect::new(
            Point::new(
                Px(delete.origin.x.0 - BUTTON_GAP_PX - button_w),
                row.origin.y,
            ),
            Size::new(Px(button_w), Px(row_h)),
        );
        let insert_ref = Rect::new(
            Point::new(
                Px(rename.origin.x.0 - BUTTON_GAP_PX - button_w),
                row.origin.y,
            ),
            Size::new(Px(button_w), Px(row_h)),
        );
        let label = Rect::new(
            row.origin,
            Size::new(
                Px((insert_ref.origin.x.0 - row.origin.x.0 - BUTTON_GAP_PX).max(0.0)),
                row.size.height,
            ),
        );

        rows.push(BlackboardRowLayout {
            symbol,
            label,
            insert_ref,
            rename,
            delete,
        });
        y += row_h;
    }

    BlackboardLayout {
        panel,
        header,
        add_button,
        rows,
    }
}

pub(super) fn blackboard_action_at(
    layout: &BlackboardLayout,
    position: Point,
) -> Option<BlackboardAction> {
    if layout.add_button.contains(position) {
        return Some(BlackboardAction::AddSymbol);
    }
    for row in &layout.rows {
        if row.insert_ref.contains(position) {
            return Some(BlackboardAction::InsertRef { symbol: row.symbol });
        }
        if row.rename.contains(position) {
            return Some(BlackboardAction::Rename { symbol: row.symbol });
        }
        if row.delete.contains(position) {
            return Some(BlackboardAction::Delete { symbol: row.symbol });
        }
    }
    None
}

fn blackboard_row_height(style: &NodeGraphStyle) -> f32 {
    style.paint.context_menu_item_height.max(20.0)
}

#[cfg(test)]
mod tests {
    use super::{blackboard_action_at, blackboard_panel_size, compute_blackboard_layout};
    use crate::core::SymbolId;
    use crate::ui::NodeGraphStyle;
    use crate::ui::overlays::blackboard_policy::BlackboardAction;
    use fret_core::{Point, Px, Rect, Size};

    #[test]
    fn blackboard_layout_keeps_header_and_row_actions_in_expected_slots() {
        let style = NodeGraphStyle::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let symbol = SymbolId::new();
        let layout = compute_blackboard_layout(&style, bounds, [symbol]);

        assert_eq!(layout.panel.origin, Point::new(Px(12.0), Px(12.0)));
        assert_eq!(layout.rows.len(), 1);
        assert!(layout.add_button.origin.x.0 > layout.header.origin.x.0);
        assert!(layout.rows[0].insert_ref.origin.x.0 > layout.rows[0].label.origin.x.0);
        assert!(layout.rows[0].rename.origin.x.0 > layout.rows[0].insert_ref.origin.x.0);
        assert!(layout.rows[0].delete.origin.x.0 > layout.rows[0].rename.origin.x.0);
    }

    #[test]
    fn blackboard_action_hit_testing_matches_layout_buttons() {
        let style = NodeGraphStyle::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let symbol = SymbolId::new();
        let layout = compute_blackboard_layout(&style, bounds, [symbol]);

        let add_point = Point::new(
            Px(layout.add_button.origin.x.0 + 1.0),
            Px(layout.add_button.origin.y.0 + 1.0),
        );
        let row = &layout.rows[0];
        let rename_point = Point::new(
            Px(row.rename.origin.x.0 + 1.0),
            Px(row.rename.origin.y.0 + 1.0),
        );

        assert_eq!(
            blackboard_action_at(&layout, add_point),
            Some(BlackboardAction::AddSymbol)
        );
        assert_eq!(
            blackboard_action_at(&layout, rename_point),
            Some(BlackboardAction::Rename { symbol })
        );
        assert_eq!(
            blackboard_action_at(
                &layout,
                Point::new(
                    Px(layout.panel.origin.x.0 + 1.0),
                    Px(layout.panel.origin.y.0 + 1.0)
                )
            ),
            None
        );
    }

    #[test]
    fn blackboard_panel_size_grows_with_rows() {
        let style = NodeGraphStyle::default();
        let empty = blackboard_panel_size(&style, 0);
        let triple = blackboard_panel_size(&style, 3);
        assert!(triple.height.0 > empty.height.0);
        assert_eq!(triple.width, empty.width);
    }
}
