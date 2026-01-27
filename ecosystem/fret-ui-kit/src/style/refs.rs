use fret_core::{Color, Px};
use fret_ui::Theme;

use super::{ColorFallback, MetricFallback, Radius, Space};

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
            fallback: space.fallback_metric(),
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

impl From<Px> for MetricRef {
    fn from(value: Px) -> Self {
        Self::Px(value)
    }
}

impl From<Space> for MetricRef {
    fn from(value: Space) -> Self {
        Self::space(value)
    }
}

impl From<Radius> for MetricRef {
    fn from(value: Radius) -> Self {
        Self::radius(value)
    }
}

#[derive(Debug, Clone)]
pub enum SignedMetricRef {
    Pos(MetricRef),
    Neg(MetricRef),
}

impl SignedMetricRef {
    pub fn pos(metric: MetricRef) -> Self {
        Self::Pos(metric)
    }

    pub fn neg(metric: MetricRef) -> Self {
        Self::Neg(metric)
    }

    pub fn resolve(&self, theme: &Theme) -> Px {
        match self {
            Self::Pos(m) => m.resolve(theme),
            Self::Neg(m) => {
                let px = m.resolve(theme);
                Px(-px.0)
            }
        }
    }
}

impl From<MetricRef> for SignedMetricRef {
    fn from(value: MetricRef) -> Self {
        Self::Pos(value)
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
