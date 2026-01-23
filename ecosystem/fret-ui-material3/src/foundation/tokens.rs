use fret_core::Color;
use fret_ui::Theme;

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct MaterialTokens<'a> {
    theme: &'a Theme,
}

#[allow(dead_code)]
impl<'a> MaterialTokens<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }

    pub fn theme(&self) -> &'a Theme {
        self.theme
    }

    pub fn sys_primary(&self) -> Color {
        self.theme.color_required("md.sys.color.primary")
    }

    pub fn sys_on_surface(&self) -> Color {
        self.theme.color_required("md.sys.color.on-surface")
    }

    pub fn sys_on_surface_variant(&self) -> Color {
        self.theme.color_required("md.sys.color.on-surface-variant")
    }

    pub fn sys_surface_container(&self) -> Color {
        self.theme.color_required("md.sys.color.surface-container")
    }

    pub fn sys_outline(&self) -> Color {
        self.theme.color_required("md.sys.color.outline")
    }

    pub fn sys_outline_variant(&self) -> Color {
        self.theme.color_required("md.sys.color.outline-variant")
    }

    pub fn sys_surface(&self) -> Color {
        self.theme.color_required("md.sys.color.surface")
    }

    pub fn sys_background(&self) -> Color {
        self.theme.color_required("md.sys.color.background")
    }

    pub fn color_comp_or_sys(&self, comp: &str, sys: &str) -> Color {
        self.theme
            .color_by_key(comp)
            .or_else(|| self.theme.color_by_key(sys))
            .unwrap_or_else(|| self.theme.color_required(sys))
    }
}
