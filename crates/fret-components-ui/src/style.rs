use fret_core::{Color, Px};
use fret_ui::Theme;

#[derive(Debug, Clone)]
pub enum MetricFallback {
    Px(Px),
    ThemeRadiusSm,
    ThemeRadiusMd,
    ThemeRadiusLg,
    ThemePaddingSm,
    ThemePaddingMd,
}

impl MetricFallback {
    fn resolve(&self, theme: &Theme) -> Px {
        match *self {
            Self::Px(px) => px,
            Self::ThemeRadiusSm => theme.metrics.radius_sm,
            Self::ThemeRadiusMd => theme.metrics.radius_md,
            Self::ThemeRadiusLg => theme.metrics.radius_lg,
            Self::ThemePaddingSm => theme.metrics.padding_sm,
            Self::ThemePaddingMd => theme.metrics.padding_md,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ColorFallback {
    Color(Color),
    ThemeSurfaceBackground,
    ThemePanelBackground,
    ThemePanelBorder,
    ThemeTextPrimary,
    ThemeTextMuted,
    ThemeTextDisabled,
    ThemeAccent,
    ThemeHoverBackground,
    ThemeSelectionBackground,
    ThemeFocusRing,
}

impl ColorFallback {
    fn resolve(&self, theme: &Theme) -> Color {
        match *self {
            Self::Color(c) => c,
            Self::ThemeSurfaceBackground => theme.colors.surface_background,
            Self::ThemePanelBackground => theme.colors.panel_background,
            Self::ThemePanelBorder => theme.colors.panel_border,
            Self::ThemeTextPrimary => theme.colors.text_primary,
            Self::ThemeTextMuted => theme.colors.text_muted,
            Self::ThemeTextDisabled => theme.colors.text_disabled,
            Self::ThemeAccent => theme.colors.accent,
            Self::ThemeHoverBackground => theme.colors.hover_background,
            Self::ThemeSelectionBackground => theme.colors.selection_background,
            Self::ThemeFocusRing => theme.colors.focus_ring,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MetricRef {
    Px(Px),
    Token {
        key: &'static str,
        fallback: MetricFallback,
    },
}

impl MetricRef {
    pub fn resolve(&self, theme: &Theme) -> Px {
        match self {
            Self::Px(px) => *px,
            Self::Token { key, fallback } => theme
                .metric_by_key(key)
                .unwrap_or_else(|| fallback.resolve(theme)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ColorRef {
    Color(Color),
    Token {
        key: &'static str,
        fallback: ColorFallback,
    },
}

impl ColorRef {
    pub fn resolve(&self, theme: &Theme) -> Color {
        match self {
            Self::Color(c) => *c,
            Self::Token { key, fallback } => theme
                .color_by_key(key)
                .unwrap_or_else(|| fallback.resolve(theme)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StyleRefinement {
    pub padding_x: Option<MetricRef>,
    pub padding_y: Option<MetricRef>,
    pub min_height: Option<MetricRef>,
    pub radius: Option<MetricRef>,
    pub border_width: Option<MetricRef>,
    pub background: Option<ColorRef>,
    pub border_color: Option<ColorRef>,
    pub text_color: Option<ColorRef>,
}

impl StyleRefinement {
    // Tailwind-like spacing scale, backed by namespaced tokens.
    pub fn px_2(mut self) -> Self {
        self.padding_x = Some(MetricRef::Token {
            key: "component.space.2",
            fallback: MetricFallback::Px(Px(8.0)),
        });
        self
    }

    pub fn px_3(mut self) -> Self {
        self.padding_x = Some(MetricRef::Token {
            key: "component.space.3",
            fallback: MetricFallback::Px(Px(12.0)),
        });
        self
    }

    pub fn py_1(mut self) -> Self {
        self.padding_y = Some(MetricRef::Token {
            key: "component.space.1",
            fallback: MetricFallback::Px(Px(4.0)),
        });
        self
    }

    pub fn rounded_md(mut self) -> Self {
        self.radius = Some(MetricRef::Token {
            key: "component.radius.md",
            fallback: MetricFallback::ThemeRadiusMd,
        });
        self
    }

    pub fn border_1(mut self) -> Self {
        self.border_width = Some(MetricRef::Px(Px(1.0)));
        self
    }

    pub fn bg(mut self, color: ColorRef) -> Self {
        self.background = Some(color);
        self
    }

    pub fn border_color(mut self, color: ColorRef) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn text_color(mut self, color: ColorRef) -> Self {
        self.text_color = Some(color);
        self
    }
}

pub fn component_color(key: &'static str, fallback: ColorFallback) -> ColorRef {
    ColorRef::Token { key, fallback }
}

pub fn component_metric(key: &'static str, fallback: MetricFallback) -> MetricRef {
    MetricRef::Token { key, fallback }
}
