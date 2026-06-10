use fret_core::{Color, Px, TextStyle};
use fret_ui::Theme;

use crate::style::MetricRef;
use crate::{Size, Space};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone)]
pub struct MenuListRowChrome {
    pub padding_x: Px,
    pub padding_y: Px,
    pub row_height: Px,
    pub separator_height: Px,
    pub text_style: TextStyle,
    pub text_color: Color,
    pub disabled_text_color: Color,
    pub row_hover: Color,
    pub row_selected: Color,
}

pub fn resolve_menu_list_row_chrome(theme: &Theme, size: Size) -> MenuListRowChrome {
    let text_px = size.control_text_px(theme);

    let padding_x = theme
        .metric_by_key("component.menu.padding_x")
        .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme));
    let padding_y = theme
        .metric_by_key("component.menu.padding_y")
        .unwrap_or_else(|| MetricRef::space(Space::N1).resolve(theme));
    let row_height = theme
        .metric_by_key("component.menu.row_h")
        .unwrap_or_else(|| size.list_row_h(theme));
    let separator_height = theme
        .metric_by_key("component.menu.separator_h")
        .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme));

    let disabled_text_color = alpha_mul(
        theme
            .color_by_key("muted-foreground")
            .unwrap_or_else(|| theme.color_token("muted-foreground")),
        0.5,
    );

    MenuListRowChrome {
        padding_x,
        padding_y,
        row_height,
        separator_height,
        text_style: TextStyle {
            font: fret_core::FontId::default(),
            size: text_px,
            ..Default::default()
        },
        text_color: theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground")),
        disabled_text_color,
        row_hover: theme
            .color_by_key("list.hover.background")
            .or_else(|| theme.color_by_key("accent"))
            .unwrap_or_else(|| theme.color_token("accent")),
        row_selected: theme
            .color_by_key("list.active.background")
            .or_else(|| theme.color_by_key("accent"))
            .unwrap_or_else(|| theme.color_token("accent")),
    }
}
