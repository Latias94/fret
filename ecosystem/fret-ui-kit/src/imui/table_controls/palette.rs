use fret_ui::Theme;

use super::body;

pub(super) fn resolve_table_palette(theme: &Theme) -> body::TablePalette {
    let table_bg = theme
        .color_by_key("table.background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_token("card"));
    let border = theme
        .color_by_key("table.border")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_token("border"));
    let header_bg = theme
        .color_by_key("table.header.background")
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or_else(|| theme.color_token("muted"));
    let mut striped_bg = theme
        .color_by_key("table.row.striped")
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or_else(|| theme.color_token("muted"));
    striped_bg.a *= 0.35;

    body::TablePalette {
        table_bg,
        border,
        header_bg,
        striped_bg,
    }
}
