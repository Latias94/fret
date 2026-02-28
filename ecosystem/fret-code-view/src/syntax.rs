use fret_core::Color;
use fret_ui::Theme;

pub(crate) fn syntax_color(theme: &Theme, highlight: &str) -> Option<Color> {
    theme.syntax_color(highlight)
}
