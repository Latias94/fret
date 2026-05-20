use std::collections::BTreeMap;

use fret_core::{Color, DrawOrder, Rect, TextConstraints, TextOverflow, TextStyle, TextWrap};

use crate::core::{Symbol, SymbolId};
use crate::ui::style::NodeGraphStyle;

use super::blackboard_layout::{BlackboardLayout, BlackboardRowLayout};
use super::blackboard_policy::{BlackboardAction, blackboard_action_button_label};
use super::panel_item_state::panel_item_visual_state;

const LABEL_PADDING_PX: f32 = 4.0;
const PANEL_ORDER: DrawOrder = DrawOrder(20_900);
const BUTTON_BACKGROUND_ORDER: DrawOrder = DrawOrder(20_901);
const TEXT_ORDER: DrawOrder = DrawOrder(20_902);

#[derive(Debug, Clone, Copy)]
pub(super) struct BlackboardPaintState {
    pub(super) hovered: Option<BlackboardAction>,
    pub(super) pressed: Option<BlackboardAction>,
    pub(super) keyboard_active: Option<BlackboardAction>,
}

#[derive(Clone)]
pub(super) struct BlackboardPaintPlan {
    pub(super) panel: BlackboardPanelPaintPlan,
    pub(super) text_style: TextStyle,
    pub(super) text_constraints: TextConstraints,
    pub(super) items: Vec<BlackboardPaintItem>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BlackboardPanelPaintPlan {
    pub(super) order: DrawOrder,
    pub(super) rect: Rect,
    pub(super) background: Color,
    pub(super) border: Color,
    pub(super) corner_px: f32,
}

#[derive(Debug, Clone)]
pub(super) enum BlackboardPaintItem {
    Label(BlackboardLabelPaintPlan),
    Button(BlackboardButtonPaintPlan),
}

#[derive(Debug, Clone)]
pub(super) struct BlackboardLabelPaintPlan {
    pub(super) rect: Rect,
    pub(super) text: String,
    pub(super) text_color: Color,
    pub(super) padding_px: f32,
    pub(super) text_order: DrawOrder,
}

#[derive(Debug, Clone)]
pub(super) struct BlackboardButtonPaintPlan {
    pub(super) action: BlackboardAction,
    pub(super) rect: Rect,
    pub(super) label: &'static str,
    pub(super) background: Color,
    pub(super) text_color: Color,
    pub(super) corner_px: f32,
    pub(super) background_order: DrawOrder,
    pub(super) text_order: DrawOrder,
}

pub(super) fn blackboard_paint_plan(
    style: &NodeGraphStyle,
    layout: &BlackboardLayout,
    symbols: &BTreeMap<SymbolId, Symbol>,
    state: BlackboardPaintState,
    scale_factor: f32,
) -> BlackboardPaintPlan {
    let mut items = Vec::with_capacity(2 + layout.rows.len() * 4);

    items.push(BlackboardPaintItem::Label(BlackboardLabelPaintPlan {
        rect: layout.header,
        text: "Symbols".to_string(),
        text_color: style.paint.context_menu_text,
        padding_px: LABEL_PADDING_PX,
        text_order: BUTTON_BACKGROUND_ORDER,
    }));
    items.push(BlackboardPaintItem::Button(blackboard_action_button_plan(
        style,
        layout.add_button,
        BlackboardAction::AddSymbol,
        state,
    )));

    for row in &layout.rows {
        append_blackboard_row_paint_plans(&mut items, style, row, symbols, state);
    }

    BlackboardPaintPlan {
        panel: BlackboardPanelPaintPlan {
            order: PANEL_ORDER,
            rect: layout.panel,
            background: style.paint.context_menu_background,
            border: style.paint.context_menu_border,
            corner_px: style.paint.context_menu_corner_radius,
        },
        text_style: blackboard_text_style(style),
        text_constraints: blackboard_text_constraints(scale_factor),
        items,
    }
}

fn append_blackboard_row_paint_plans(
    items: &mut Vec<BlackboardPaintItem>,
    style: &NodeGraphStyle,
    row: &BlackboardRowLayout,
    symbols: &BTreeMap<SymbolId, Symbol>,
    state: BlackboardPaintState,
) {
    items.push(BlackboardPaintItem::Button(blackboard_action_button_plan(
        style,
        row.insert_ref,
        BlackboardAction::InsertRef { symbol: row.symbol },
        state,
    )));
    items.push(BlackboardPaintItem::Button(blackboard_action_button_plan(
        style,
        row.rename,
        BlackboardAction::Rename { symbol: row.symbol },
        state,
    )));
    items.push(BlackboardPaintItem::Button(blackboard_action_button_plan(
        style,
        row.delete,
        BlackboardAction::Delete { symbol: row.symbol },
        state,
    )));
    items.push(BlackboardPaintItem::Label(BlackboardLabelPaintPlan {
        rect: row.label,
        text: blackboard_row_name(symbols, row.symbol).to_string(),
        text_color: style.paint.context_menu_text,
        padding_px: LABEL_PADDING_PX,
        text_order: TEXT_ORDER,
    }));
}

fn blackboard_action_button_plan(
    style: &NodeGraphStyle,
    rect: Rect,
    action: BlackboardAction,
    state: BlackboardPaintState,
) -> BlackboardButtonPaintPlan {
    BlackboardButtonPaintPlan {
        action,
        rect,
        label: blackboard_action_button_label(action),
        background: blackboard_action_background(style, action, state),
        text_color: style.paint.context_menu_text,
        corner_px: style.paint.context_menu_corner_radius,
        background_order: BUTTON_BACKGROUND_ORDER,
        text_order: TEXT_ORDER,
    }
}

fn blackboard_row_name(symbols: &BTreeMap<SymbolId, Symbol>, symbol: SymbolId) -> &str {
    symbols
        .get(&symbol)
        .map(|symbol| symbol.name.as_str())
        .unwrap_or("<missing>")
}

fn blackboard_action_background(
    style: &NodeGraphStyle,
    action: BlackboardAction,
    state: BlackboardPaintState,
) -> Color {
    let visual_state = panel_item_visual_state(
        action,
        state.hovered,
        state.pressed,
        state.keyboard_active,
        true,
        false,
    );
    if visual_state.active() {
        style.paint.context_menu_hover_background
    } else {
        Color::TRANSPARENT
    }
}

fn blackboard_text_style(style: &NodeGraphStyle) -> TextStyle {
    style.geometry.context_menu_text_style.clone()
}

fn blackboard_text_constraints(scale_factor: f32) -> TextConstraints {
    TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use fret_core::{Color, Point, Px, Rect, Size, TextOverflow, TextWrap};

    use crate::core::{Symbol, SymbolId};
    use crate::ui::NodeGraphStyle;
    use crate::ui::overlays::blackboard_layout::compute_blackboard_layout;
    use crate::ui::overlays::blackboard_paint_plan::{
        BlackboardPaintItem, BlackboardPaintState, blackboard_paint_plan,
    };
    use crate::ui::overlays::blackboard_policy::BlackboardAction;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        )
    }

    fn symbol(id: SymbolId, name: &str) -> (SymbolId, Symbol) {
        (
            id,
            Symbol {
                name: name.to_string(),
                ty: None,
                default_value: None,
                meta: serde_json::Value::Null,
            },
        )
    }

    #[test]
    fn blackboard_paint_plan_emits_panel_header_buttons_and_rows() {
        let style = NodeGraphStyle::default();
        let symbol_id = SymbolId::from_u128(42);
        let symbols = BTreeMap::from([symbol(symbol_id, "World")]);
        let layout = compute_blackboard_layout(&style, bounds(), symbols.keys().copied());
        let plan = blackboard_paint_plan(
            &style,
            &layout,
            &symbols,
            BlackboardPaintState {
                hovered: Some(BlackboardAction::AddSymbol),
                pressed: None,
                keyboard_active: None,
            },
            2.0,
        );

        assert_eq!(plan.panel.rect, layout.panel);
        assert_eq!(plan.panel.background, style.paint.context_menu_background);
        assert_eq!(plan.panel.border, style.paint.context_menu_border);
        assert_eq!(plan.text_constraints.scale_factor, 2.0);
        assert_eq!(plan.text_constraints.wrap, TextWrap::None);
        assert_eq!(plan.text_constraints.overflow, TextOverflow::Clip);
        assert_eq!(plan.items.len(), 6);

        let BlackboardPaintItem::Label(header) = &plan.items[0] else {
            panic!("first blackboard paint item should be the header label");
        };
        assert_eq!(header.text, "Symbols");
        assert_eq!(header.rect, layout.header);

        let BlackboardPaintItem::Button(add) = &plan.items[1] else {
            panic!("second blackboard paint item should be the add button");
        };
        assert_eq!(add.action, BlackboardAction::AddSymbol);
        assert_eq!(add.label, "+");
        assert_eq!(add.background, style.paint.context_menu_hover_background);

        let BlackboardPaintItem::Label(row_label) = &plan.items[5] else {
            panic!("last blackboard paint item should be the row label");
        };
        assert_eq!(row_label.text, "World");
        assert_eq!(row_label.rect, layout.rows[0].label);
    }

    #[test]
    fn blackboard_paint_plan_uses_active_background_for_hover_press_and_keyboard() {
        let style = NodeGraphStyle::default();
        let symbol_id = SymbolId::from_u128(7);
        let symbols = BTreeMap::from([symbol(symbol_id, "Value")]);
        let layout = compute_blackboard_layout(&style, bounds(), symbols.keys().copied());
        let action = BlackboardAction::Rename { symbol: symbol_id };

        for state in [
            BlackboardPaintState {
                hovered: Some(action),
                pressed: None,
                keyboard_active: None,
            },
            BlackboardPaintState {
                hovered: None,
                pressed: Some(action),
                keyboard_active: None,
            },
            BlackboardPaintState {
                hovered: None,
                pressed: None,
                keyboard_active: Some(action),
            },
        ] {
            let plan = blackboard_paint_plan(&style, &layout, &symbols, state, 1.0);
            let rename = plan
                .items
                .iter()
                .find_map(|item| match item {
                    BlackboardPaintItem::Button(button) if button.action == action => Some(button),
                    _ => None,
                })
                .expect("rename button paint plan");
            assert_eq!(rename.background, style.paint.context_menu_hover_background);
        }

        let idle = blackboard_paint_plan(
            &style,
            &layout,
            &symbols,
            BlackboardPaintState {
                hovered: None,
                pressed: None,
                keyboard_active: None,
            },
            1.0,
        );
        let rename = idle
            .items
            .iter()
            .find_map(|item| match item {
                BlackboardPaintItem::Button(button) if button.action == action => Some(button),
                _ => None,
            })
            .expect("rename button paint plan");
        assert_eq!(rename.background, Color::TRANSPARENT);
    }

    #[test]
    fn blackboard_paint_plan_falls_back_for_missing_symbol_names() {
        let style = NodeGraphStyle::default();
        let symbol_id = SymbolId::from_u128(9);
        let layout = compute_blackboard_layout(&style, bounds(), [symbol_id]);
        let plan = blackboard_paint_plan(
            &style,
            &layout,
            &BTreeMap::new(),
            BlackboardPaintState {
                hovered: None,
                pressed: None,
                keyboard_active: None,
            },
            1.0,
        );

        let BlackboardPaintItem::Label(row_label) = &plan.items[5] else {
            panic!("last blackboard paint item should be the row label");
        };
        assert_eq!(row_label.text, "<missing>");
    }
}
