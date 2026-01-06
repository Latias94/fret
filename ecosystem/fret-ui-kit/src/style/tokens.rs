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

    pub(super) fn fallback_metric(self) -> MetricFallback {
        match self {
            Self::N0 => MetricFallback::Px(Px(0.0)),
            Self::N0p5 => MetricFallback::ThemePaddingSmMulDiv { mul: 1, div: 4 },
            Self::N1 => MetricFallback::ThemePaddingSmMulDiv { mul: 1, div: 2 },
            Self::N1p5 => MetricFallback::ThemePaddingSmMulDiv { mul: 3, div: 4 },
            Self::N2 => MetricFallback::ThemePaddingSm,
            // This is intentionally tied to the baseline `fret.padding.md` token to avoid value
            // duplication drift when themes omit `component.space.*` (ADR 0032 / ADR 0050).
            Self::N2p5 => MetricFallback::ThemePaddingMd,
            Self::N3 => MetricFallback::ThemePaddingSmMulDiv { mul: 3, div: 2 },
            Self::N3p5 => MetricFallback::ThemePaddingSmMulDiv { mul: 7, div: 4 },
            Self::N4 => MetricFallback::ThemePaddingSmMulDiv { mul: 2, div: 1 },
            Self::N5 => MetricFallback::ThemePaddingSmMulDiv { mul: 5, div: 2 },
            Self::N6 => MetricFallback::ThemePaddingSmMulDiv { mul: 3, div: 1 },
            Self::N8 => MetricFallback::ThemePaddingSmMulDiv { mul: 4, div: 1 },
            Self::N10 => MetricFallback::ThemePaddingSmMulDiv { mul: 5, div: 1 },
            Self::N11 => MetricFallback::ThemePaddingSmMulDiv { mul: 11, div: 2 },
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
    ThemePaddingSmMulDiv { mul: u32, div: u32 },
}

impl MetricFallback {
    pub(super) fn resolve(&self, theme: &Theme) -> Px {
        match *self {
            Self::Px(px) => px,
            Self::ThemeRadiusSm => theme.metrics.radius_sm,
            Self::ThemeRadiusMd => theme.metrics.radius_md,
            Self::ThemeRadiusLg => theme.metrics.radius_lg,
            Self::ThemePaddingSm => theme.metrics.padding_sm,
            Self::ThemePaddingMd => theme.metrics.padding_md,
            Self::ThemePaddingSmMulDiv { mul, div } => {
                if div == 0 {
                    return Px(0.0);
                }
                let base = theme.metrics.padding_sm;
                Px(base.0 * (mul as f32) / (div as f32))
            }
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
    pub(super) fn resolve(&self, theme: &Theme) -> Color {
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
