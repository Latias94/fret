use fret_core::Color;
use fret_ui::Theme;

#[derive(Clone, Copy)]
pub struct MaterialTokenResolver<'a> {
    theme: &'a Theme,
}

impl<'a> MaterialTokenResolver<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }

    pub fn color_sys(&self, sys_key: &str) -> Color {
        debug_assert!(
            sys_key.starts_with("md.sys."),
            "expected md.sys.* key, got: {sys_key}"
        );
        self.theme
            .color_by_key(sys_key)
            .unwrap_or_else(|| fallback_color_for_sys(sys_key))
    }

    pub fn color_comp_or_sys(&self, comp_key: &str, sys_key: &str) -> Color {
        debug_assert!(
            comp_key.starts_with("md.comp."),
            "expected md.comp.* key, got: {comp_key}"
        );
        debug_assert!(
            sys_key.starts_with("md.sys."),
            "expected md.sys.* key, got: {sys_key}"
        );
        self.theme
            .color_by_key(comp_key)
            .or_else(|| self.theme.color_by_key(sys_key))
            .unwrap_or_else(|| fallback_color_for_sys(sys_key))
    }

    pub fn number_sys(&self, sys_key: &str, fallback: f32) -> f32 {
        debug_assert!(
            sys_key.starts_with("md.sys."),
            "expected md.sys.* key, got: {sys_key}"
        );
        self.theme.number_by_key(sys_key).unwrap_or(fallback)
    }

    pub fn number_comp_or_sys(&self, comp_key: &str, sys_key: &str, fallback: f32) -> f32 {
        debug_assert!(
            comp_key.starts_with("md.comp."),
            "expected md.comp.* key, got: {comp_key}"
        );
        debug_assert!(
            sys_key.starts_with("md.sys."),
            "expected md.sys.* key, got: {sys_key}"
        );
        self.theme
            .number_by_key(comp_key)
            .or_else(|| self.theme.number_by_key(sys_key))
            .unwrap_or(fallback)
    }
}

fn fallback_color_for_sys(sys_key: &str) -> Color {
    match sys_key {
        "md.sys.color.primary" => Color {
            r: 0.403,
            g: 0.314,
            b: 0.643,
            a: 1.0,
        },
        "md.sys.color.on-primary" => Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
        "md.sys.color.surface" => Color {
            r: 0.11,
            g: 0.11,
            b: 0.12,
            a: 1.0,
        },
        "md.sys.color.surface-container" => Color {
            r: 0.16,
            g: 0.16,
            b: 0.17,
            a: 1.0,
        },
        "md.sys.color.surface-container-highest" => Color {
            r: 0.2,
            g: 0.2,
            b: 0.21,
            a: 1.0,
        },
        "md.sys.color.on-surface" => Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
        "md.sys.color.on-surface-variant" => Color {
            r: 0.75,
            g: 0.75,
            b: 0.78,
            a: 1.0,
        },
        "md.sys.color.outline" => Color {
            r: 0.55,
            g: 0.55,
            b: 0.58,
            a: 1.0,
        },
        "md.sys.color.outline-variant" => Color {
            r: 0.35,
            g: 0.35,
            b: 0.38,
            a: 1.0,
        },
        _ => Color {
            r: 1.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        },
    }
}
