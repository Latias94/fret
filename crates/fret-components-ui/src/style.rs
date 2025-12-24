use fret_core::{Color, Px};
use fret_ui::Theme;

/// Tailwind-like spacing scale for component libraries.
///
/// This is intentionally small and opinionated. It is used by component-level style refinements
/// and can be overridden via theme extension metrics (ADR 0050).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Space {
    N0,
    N0p5,
    N1,
    N1p5,
    N2,
    N2p5,
    N3,
    N3p5,
    N4,
    N5,
    N6,
    N8,
    N10,
    N11,
}

impl Space {
    pub fn token_key(self) -> &'static str {
        match self {
            Self::N0 => "component.space.0",
            Self::N0p5 => "component.space.0p5",
            Self::N1 => "component.space.1",
            Self::N1p5 => "component.space.1p5",
            Self::N2 => "component.space.2",
            Self::N2p5 => "component.space.2p5",
            Self::N3 => "component.space.3",
            Self::N3p5 => "component.space.3p5",
            Self::N4 => "component.space.4",
            Self::N5 => "component.space.5",
            Self::N6 => "component.space.6",
            Self::N8 => "component.space.8",
            Self::N10 => "component.space.10",
            Self::N11 => "component.space.11",
        }
    }

    pub fn fallback_px(self) -> Px {
        match self {
            Self::N0 => Px(0.0),
            Self::N0p5 => Px(2.0),
            Self::N1 => Px(4.0),
            Self::N1p5 => Px(6.0),
            Self::N2 => Px(8.0),
            Self::N2p5 => Px(10.0),
            Self::N3 => Px(12.0),
            Self::N3p5 => Px(14.0),
            Self::N4 => Px(16.0),
            Self::N5 => Px(20.0),
            Self::N6 => Px(24.0),
            Self::N8 => Px(32.0),
            Self::N10 => Px(40.0),
            Self::N11 => Px(44.0),
        }
    }
}

/// Tailwind-like border radius presets for component libraries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Radius {
    Sm,
    Md,
    Lg,
    Full,
}

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
    pub fn space(space: Space) -> Self {
        MetricRef::Token {
            key: space.token_key(),
            fallback: MetricFallback::Px(space.fallback_px()),
        }
    }

    pub fn radius(radius: Radius) -> Self {
        match radius {
            Radius::Sm => MetricRef::Token {
                key: "component.radius.sm",
                fallback: MetricFallback::ThemeRadiusSm,
            },
            Radius::Md => MetricRef::Token {
                key: "component.radius.md",
                fallback: MetricFallback::ThemeRadiusMd,
            },
            Radius::Lg => MetricRef::Token {
                key: "component.radius.lg",
                fallback: MetricFallback::ThemeRadiusLg,
            },
            Radius::Full => MetricRef::Token {
                key: "component.radius.full",
                fallback: MetricFallback::Px(Px(999.0)),
            },
        }
    }

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
    pub fn px(mut self, space: Space) -> Self {
        self.padding_x = Some(MetricRef::space(space));
        self
    }

    pub fn py(mut self, space: Space) -> Self {
        self.padding_y = Some(MetricRef::space(space));
        self
    }

    pub fn p(mut self, space: Space) -> Self {
        self.padding_x = Some(MetricRef::space(space));
        self.padding_y = Some(MetricRef::space(space));
        self
    }

    pub fn rounded(mut self, radius: Radius) -> Self {
        self.radius = Some(MetricRef::radius(radius));
        self
    }

    // Tailwind-like spacing scale, backed by namespaced tokens.
    pub fn px_2(self) -> Self {
        self.px(Space::N2)
    }

    pub fn px_3(self) -> Self {
        self.px(Space::N3)
    }

    pub fn py_1(self) -> Self {
        self.py(Space::N1)
    }

    pub fn rounded_md(self) -> Self {
        self.rounded(Radius::Md)
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
