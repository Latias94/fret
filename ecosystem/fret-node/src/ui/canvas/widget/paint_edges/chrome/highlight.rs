use super::*;

fn highlight_hint(
    interaction_hint: crate::ui::InteractionChromeHint,
    edge_selected: bool,
    edge_hovered: bool,
) -> Option<crate::ui::WireHighlightHint> {
    if edge_hovered {
        interaction_hint.wire_highlight_hovered
    } else if edge_selected {
        interaction_hint.wire_highlight_selected
    } else {
        None
    }
}

fn normalized_width_mul(width_mul: f32) -> f32 {
    if width_mul.is_finite() {
        width_mul.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn normalized_alpha_mul(alpha_mul: f32) -> f32 {
    if alpha_mul.is_finite() {
        alpha_mul.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn default_highlight_color(color: Color) -> Color {
    let t = 0.45;
    Color {
        r: color.r + (1.0 - color.r) * t,
        g: color.g + (1.0 - color.g) * t,
        b: color.b + (1.0 - color.b) * t,
        a: color.a,
    }
}

pub(super) fn resolve_edge_highlight(
    interaction_hint: crate::ui::InteractionChromeHint,
    edge_selected: bool,
    edge_hovered: bool,
    color: Color,
    width: f32,
    highlight_budget: &mut WorkBudget,
    highlight_budget_skipped: &mut u32,
) -> Option<WireHighlightPaint> {
    let hint = highlight_hint(interaction_hint, edge_selected, edge_hovered)?;

    let width_mul = normalized_width_mul(hint.width_mul);
    let alpha_mul = normalized_alpha_mul(hint.alpha_mul);
    let highlight_width = width * width_mul;
    if !highlight_width.is_finite() || highlight_width <= 1.0e-3 || alpha_mul <= 1.0e-6 {
        return None;
    }
    if !highlight_budget.try_consume(1) {
        *highlight_budget_skipped = highlight_budget_skipped.saturating_add(1);
        return None;
    }

    let mut highlight_color = hint.color.unwrap_or_else(|| default_highlight_color(color));
    highlight_color.a = (highlight_color.a * alpha_mul).clamp(0.0, 1.0);
    (highlight_color.a > 0.0).then_some(WireHighlightPaint {
        width: highlight_width,
        color: highlight_color,
    })
}
