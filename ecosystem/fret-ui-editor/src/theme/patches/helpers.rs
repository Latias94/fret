use fret_ui::ThemeConfig;

pub(crate) fn metric(cfg: &mut ThemeConfig, key: &str, value: f32) {
    cfg.metrics.insert(key.to_string(), value);
}

pub(crate) fn color(cfg: &mut ThemeConfig, key: &str, value: &str) {
    cfg.colors.insert(key.to_string(), value.to_string());
}
