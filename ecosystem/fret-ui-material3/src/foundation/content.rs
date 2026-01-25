use fret_core::Color;
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

#[derive(Debug, Clone, Copy)]
pub struct MaterialContentDefaults {
    pub content_color: Color,
    pub disabled_opacity: f32,
}

impl MaterialContentDefaults {
    pub fn on_surface(theme: &Theme) -> Self {
        let tokens = MaterialTokenResolver::new(theme);
        Self {
            content_color: tokens.color_sys("md.sys.color.on-surface"),
            disabled_opacity: tokens.number_sys("md.sys.state.disabled.state-layer-opacity", 0.38),
        }
    }

    pub fn on_surface_variant(theme: &Theme) -> Self {
        let tokens = MaterialTokenResolver::new(theme);
        Self {
            content_color: tokens.color_sys("md.sys.color.on-surface-variant"),
            disabled_opacity: tokens.number_sys("md.sys.state.disabled.state-layer-opacity", 0.38),
        }
    }
}
