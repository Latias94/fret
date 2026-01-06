use fret_core::Px;
use fret_core::scene::Color;
use fret_ui::Theme;

pub(crate) fn color(theme: &Theme, key: &'static str, compat_key: &'static str) -> Option<Color> {
    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key(compat_key))
}

pub(crate) fn metric(theme: &Theme, key: &'static str, compat_key: &'static str) -> Option<Px> {
    theme
        .metric_by_key(key)
        .or_else(|| theme.metric_by_key(compat_key))
}

pub(crate) fn resolve_series_palette(theme: &Theme, base: [Color; 10]) -> [Color; 10] {
    const FRETPLOT_KEYS: [&str; 10] = [
        "fret.plot.palette.0",
        "fret.plot.palette.1",
        "fret.plot.palette.2",
        "fret.plot.palette.3",
        "fret.plot.palette.4",
        "fret.plot.palette.5",
        "fret.plot.palette.6",
        "fret.plot.palette.7",
        "fret.plot.palette.8",
        "fret.plot.palette.9",
    ];
    const COMPAT_KEYS: [&str; 10] = [
        "plot.palette.0",
        "plot.palette.1",
        "plot.palette.2",
        "plot.palette.3",
        "plot.palette.4",
        "plot.palette.5",
        "plot.palette.6",
        "plot.palette.7",
        "plot.palette.8",
        "plot.palette.9",
    ];

    let mut out = base;
    for i in 0..out.len() {
        if let Some(c) = theme
            .color_by_key(FRETPLOT_KEYS[i])
            .or_else(|| theme.color_by_key(COMPAT_KEYS[i]))
        {
            out[i] = c;
        }
    }
    out
}
