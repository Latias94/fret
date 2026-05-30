use fret_ui::Theme;
use fret_ui_material3::tokens::v30::{
    ColorSchemeOptions, DynamicVariant, SchemeMode, TypographyOptions, theme_config_with_colors,
};

use super::host::TestHost;

pub(crate) fn apply_material_theme(app: &mut TestHost, mode: SchemeMode, variant: DynamicVariant) {
    let mut colors = ColorSchemeOptions::default();
    colors.mode = mode;
    colors.variant = variant;

    let cfg = theme_config_with_colors(TypographyOptions::default(), colors);
    Theme::with_global_mut(app, |theme| theme.apply_config(&cfg));
}

pub(crate) fn apply_material_theme_rtl(
    app: &mut TestHost,
    mode: SchemeMode,
    variant: DynamicVariant,
) {
    let mut colors = ColorSchemeOptions::default();
    colors.mode = mode;
    colors.variant = variant;

    let mut cfg = theme_config_with_colors(TypographyOptions::default(), colors);
    cfg.numbers
        .insert("md.sys.fret.layout.is-rtl".to_string(), 1.0);
    Theme::with_global_mut(app, |theme| theme.apply_config(&cfg));
}
