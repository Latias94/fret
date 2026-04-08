use std::collections::BTreeMap;

use fret_core::{
    Color, Corners, DrawOrder, Edges, Px, Rect, SceneOp, TextBlobId, TextConstraints, TextOverflow,
    TextStyle, TextWrap,
};
use fret_ui::{UiHost, retained_bridge::PaintCx};

use crate::core::{Symbol, SymbolId};
use crate::ui::style::NodeGraphStyle;

use super::blackboard_layout::{BlackboardLayout, BlackboardRowLayout};
use super::blackboard_policy::{BlackboardAction, blackboard_action_button_label};
use super::panel_button_paint::{paint_panel_button, paint_panel_label};
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

pub(super) fn paint_blackboard_overlay<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    text_blobs: &mut Vec<TextBlobId>,
    style: &NodeGraphStyle,
    layout: &BlackboardLayout,
    symbols: &BTreeMap<SymbolId, Symbol>,
    state: BlackboardPaintState,
) {
    let bg = style.paint.context_menu_background;
    let border = style.paint.context_menu_border;
    let text_color = style.paint.context_menu_text;
    let corner = style.paint.context_menu_corner_radius;

    cx.scene.push(SceneOp::Quad {
        order: PANEL_ORDER,
        rect: layout.panel,
        background: fret_core::Paint::Solid(bg).into(),

        border: Edges::all(Px(1.0)),
        border_paint: fret_core::Paint::Solid(border).into(),

        corner_radii: Corners::all(Px(corner)),
    });

    let text_style = blackboard_text_style(style);
    let constraints = blackboard_text_constraints(cx.scale_factor);

    paint_panel_label(
        cx,
        text_blobs,
        layout.header,
        "Symbols",
        &text_style,
        constraints,
        text_color,
        LABEL_PADDING_PX,
        BUTTON_BACKGROUND_ORDER,
    );

    paint_blackboard_action_button(
        cx,
        text_blobs,
        style,
        layout.add_button,
        BlackboardAction::AddSymbol,
        blackboard_action_button_label(BlackboardAction::AddSymbol),
        &text_style,
        constraints,
        state,
    );

    for row in &layout.rows {
        paint_blackboard_row(
            cx,
            text_blobs,
            style,
            row,
            blackboard_row_name(symbols, row.symbol),
            &text_style,
            constraints,
            state,
        );
    }
}

fn paint_blackboard_row<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    text_blobs: &mut Vec<TextBlobId>,
    style: &NodeGraphStyle,
    row: &BlackboardRowLayout,
    name: &str,
    text_style: &TextStyle,
    constraints: TextConstraints,
    state: BlackboardPaintState,
) {
    paint_blackboard_action_button(
        cx,
        text_blobs,
        style,
        row.insert_ref,
        BlackboardAction::InsertRef { symbol: row.symbol },
        blackboard_action_button_label(BlackboardAction::InsertRef { symbol: row.symbol }),
        text_style,
        constraints,
        state,
    );
    paint_blackboard_action_button(
        cx,
        text_blobs,
        style,
        row.rename,
        BlackboardAction::Rename { symbol: row.symbol },
        blackboard_action_button_label(BlackboardAction::Rename { symbol: row.symbol }),
        text_style,
        constraints,
        state,
    );
    paint_blackboard_action_button(
        cx,
        text_blobs,
        style,
        row.delete,
        BlackboardAction::Delete { symbol: row.symbol },
        blackboard_action_button_label(BlackboardAction::Delete { symbol: row.symbol }),
        text_style,
        constraints,
        state,
    );
    paint_panel_label(
        cx,
        text_blobs,
        row.label,
        name,
        text_style,
        constraints,
        style.paint.context_menu_text,
        LABEL_PADDING_PX,
        TEXT_ORDER,
    );
}

fn paint_blackboard_action_button<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    text_blobs: &mut Vec<TextBlobId>,
    style: &NodeGraphStyle,
    rect: Rect,
    action: BlackboardAction,
    label: &str,
    text_style: &TextStyle,
    constraints: TextConstraints,
    state: BlackboardPaintState,
) {
    paint_panel_button(
        cx,
        text_blobs,
        rect,
        label,
        text_style,
        constraints,
        blackboard_action_background(style, action, state),
        style.paint.context_menu_text,
        style.paint.context_menu_corner_radius,
        BUTTON_BACKGROUND_ORDER,
        TEXT_ORDER,
    );
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
    use super::{BlackboardPaintState, blackboard_action_background, blackboard_row_name};
    use crate::core::{Symbol, SymbolId};
    use crate::ui::NodeGraphStyle;
    use crate::ui::overlays::blackboard_policy::BlackboardAction;
    use fret_core::Color;
    use std::collections::BTreeMap;

    #[test]
    fn blackboard_action_background_uses_hover_fill_for_any_active_state() {
        let style = NodeGraphStyle::default();
        let symbol = SymbolId::new();
        let action = BlackboardAction::Rename { symbol };

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
            assert_eq!(
                blackboard_action_background(&style, action, state),
                style.paint.context_menu_hover_background
            );
        }
    }

    #[test]
    fn blackboard_action_background_stays_transparent_when_idle() {
        let style = NodeGraphStyle::default();
        let symbol = SymbolId::new();
        let action = BlackboardAction::Delete { symbol };
        let state = BlackboardPaintState {
            hovered: None,
            pressed: None,
            keyboard_active: None,
        };

        assert_eq!(
            blackboard_action_background(&style, action, state),
            Color::TRANSPARENT
        );
    }

    #[test]
    fn blackboard_row_name_falls_back_for_missing_symbol() {
        let symbol = SymbolId::new();
        let mut symbols = BTreeMap::new();
        symbols.insert(
            SymbolId::new(),
            Symbol {
                name: "Other".to_string(),
                ty: None,
                default_value: None,
                meta: serde_json::Value::Null,
            },
        );

        assert_eq!(blackboard_row_name(&symbols, symbol), "<missing>");
    }
}
