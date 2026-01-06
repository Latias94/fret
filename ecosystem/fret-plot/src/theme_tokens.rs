use fret_core::scene::Color;
use fret_ui::Theme;

pub(crate) fn color(theme: &Theme, key: &'static str, compat_key: &'static str) -> Option<Color> {
    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key(compat_key))
}
