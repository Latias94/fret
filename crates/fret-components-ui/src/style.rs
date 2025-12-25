use fret_core::{Color, Px};
use fret_ui::Theme;
use fret_ui::element::PositionStyle;

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

    fn fallback_metric(self) -> MetricFallback {
        match self {
            Self::N0 => MetricFallback::Px(Px(0.0)),
            Self::N0p5 => MetricFallback::ThemePaddingSmMulDiv { mul: 1, div: 4 },
            Self::N1 => MetricFallback::ThemePaddingSmMulDiv { mul: 1, div: 2 },
            Self::N1p5 => MetricFallback::ThemePaddingSmMulDiv { mul: 3, div: 4 },
            Self::N2 => MetricFallback::ThemePaddingSm,
            // This is intentionally tied to the baseline `metric.padding.md` token to avoid value
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
    fn resolve(&self, theme: &Theme) -> Px {
        match *self {
            Self::Px(px) => px,
            Self::ThemeRadiusSm => theme
                .metric_by_key("metric.radius.sm")
                .unwrap_or(theme.metrics.radius_sm),
            Self::ThemeRadiusMd => theme
                .metric_by_key("metric.radius.md")
                .unwrap_or(theme.metrics.radius_md),
            Self::ThemeRadiusLg => theme
                .metric_by_key("metric.radius.lg")
                .unwrap_or(theme.metrics.radius_lg),
            Self::ThemePaddingSm => theme
                .metric_by_key("metric.padding.sm")
                .unwrap_or(theme.metrics.padding_sm),
            Self::ThemePaddingMd => theme
                .metric_by_key("metric.padding.md")
                .unwrap_or(theme.metrics.padding_md),
            Self::ThemePaddingSmMulDiv { mul, div } => {
                if div == 0 {
                    return Px(0.0);
                }
                let base = theme
                    .metric_by_key("metric.padding.sm")
                    .unwrap_or(theme.metrics.padding_sm);
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
pub struct PaddingRefinement {
    pub top: Option<MetricRef>,
    pub right: Option<MetricRef>,
    pub bottom: Option<MetricRef>,
    pub left: Option<MetricRef>,
}

impl PaddingRefinement {
    pub fn merge(mut self, other: PaddingRefinement) -> Self {
        if other.top.is_some() {
            self.top = other.top;
        }
        if other.right.is_some() {
            self.right = other.right;
        }
        if other.bottom.is_some() {
            self.bottom = other.bottom;
        }
        if other.left.is_some() {
            self.left = other.left;
        }
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct MarginRefinement {
    pub top: Option<MetricRef>,
    pub right: Option<MetricRef>,
    pub bottom: Option<MetricRef>,
    pub left: Option<MetricRef>,
}

impl MarginRefinement {
    pub fn merge(mut self, other: MarginRefinement) -> Self {
        if other.top.is_some() {
            self.top = other.top;
        }
        if other.right.is_some() {
            self.right = other.right;
        }
        if other.bottom.is_some() {
            self.bottom = other.bottom;
        }
        if other.left.is_some() {
            self.left = other.left;
        }
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct InsetRefinement {
    pub top: Option<MetricRef>,
    pub right: Option<MetricRef>,
    pub bottom: Option<MetricRef>,
    pub left: Option<MetricRef>,
}

impl InsetRefinement {
    pub fn merge(mut self, other: InsetRefinement) -> Self {
        if other.top.is_some() {
            self.top = other.top;
        }
        if other.right.is_some() {
            self.right = other.right;
        }
        if other.bottom.is_some() {
            self.bottom = other.bottom;
        }
        if other.left.is_some() {
            self.left = other.left;
        }
        self
    }
}

/// Control chrome style patches (colors, padding, borders, radius, etc).
///
/// This intentionally does **not** include layout-affecting fields like margin or absolute
/// positioning. Those live in `LayoutRefinement` and apply only in the declarative authoring path.
#[derive(Debug, Clone, Default)]
pub struct ChromeRefinement {
    pub padding: Option<PaddingRefinement>,
    pub min_height: Option<MetricRef>,
    pub radius: Option<MetricRef>,
    pub border_width: Option<MetricRef>,
    pub background: Option<ColorRef>,
    pub border_color: Option<ColorRef>,
    pub text_color: Option<ColorRef>,
}

impl ChromeRefinement {
    pub fn merge(mut self, other: ChromeRefinement) -> Self {
        if let Some(p) = other.padding {
            self.padding = Some(self.padding.unwrap_or_default().merge(p));
        }
        if other.min_height.is_some() {
            self.min_height = other.min_height;
        }
        if other.radius.is_some() {
            self.radius = other.radius;
        }
        if other.border_width.is_some() {
            self.border_width = other.border_width;
        }
        if other.background.is_some() {
            self.background = other.background;
        }
        if other.border_color.is_some() {
            self.border_color = other.border_color;
        }
        if other.text_color.is_some() {
            self.text_color = other.text_color;
        }
        self
    }

    pub fn px(mut self, space: Space) -> Self {
        let m = MetricRef::space(space);
        let mut padding = self.padding.unwrap_or_default();
        padding.left = Some(m.clone());
        padding.right = Some(m);
        self.padding = Some(padding);
        self
    }

    pub fn py(mut self, space: Space) -> Self {
        let m = MetricRef::space(space);
        let mut padding = self.padding.unwrap_or_default();
        padding.top = Some(m.clone());
        padding.bottom = Some(m);
        self.padding = Some(padding);
        self
    }

    pub fn p(mut self, space: Space) -> Self {
        let m = MetricRef::space(space);
        self.padding = Some(PaddingRefinement {
            top: Some(m.clone()),
            right: Some(m.clone()),
            bottom: Some(m.clone()),
            left: Some(m),
        });
        self
    }

    pub fn pt(mut self, space: Space) -> Self {
        let mut padding = self.padding.unwrap_or_default();
        padding.top = Some(MetricRef::space(space));
        self.padding = Some(padding);
        self
    }

    pub fn pr(mut self, space: Space) -> Self {
        let mut padding = self.padding.unwrap_or_default();
        padding.right = Some(MetricRef::space(space));
        self.padding = Some(padding);
        self
    }

    pub fn pb(mut self, space: Space) -> Self {
        let mut padding = self.padding.unwrap_or_default();
        padding.bottom = Some(MetricRef::space(space));
        self.padding = Some(padding);
        self
    }

    pub fn pl(mut self, space: Space) -> Self {
        let mut padding = self.padding.unwrap_or_default();
        padding.left = Some(MetricRef::space(space));
        self.padding = Some(padding);
        self
    }

    pub fn rounded(mut self, radius: Radius) -> Self {
        self.radius = Some(MetricRef::radius(radius));
        self
    }

    // Tailwind-like spacing scale, backed by namespaced tokens.
    pub fn px_1(self) -> Self {
        self.px(Space::N1)
    }

    pub fn px_0p5(self) -> Self {
        self.px(Space::N0p5)
    }

    pub fn px_1p5(self) -> Self {
        self.px(Space::N1p5)
    }

    pub fn px_2(self) -> Self {
        self.px(Space::N2)
    }

    pub fn px_2p5(self) -> Self {
        self.px(Space::N2p5)
    }

    pub fn px_3(self) -> Self {
        self.px(Space::N3)
    }

    pub fn px_4(self) -> Self {
        self.px(Space::N4)
    }

    pub fn py_1(self) -> Self {
        self.py(Space::N1)
    }

    pub fn py_0p5(self) -> Self {
        self.py(Space::N0p5)
    }

    pub fn py_1p5(self) -> Self {
        self.py(Space::N1p5)
    }

    pub fn py_2(self) -> Self {
        self.py(Space::N2)
    }

    pub fn py_2p5(self) -> Self {
        self.py(Space::N2p5)
    }

    pub fn py_3(self) -> Self {
        self.py(Space::N3)
    }

    pub fn py_4(self) -> Self {
        self.py(Space::N4)
    }

    pub fn p_1(self) -> Self {
        self.p(Space::N1)
    }

    pub fn p_0p5(self) -> Self {
        self.p(Space::N0p5)
    }

    pub fn p_1p5(self) -> Self {
        self.p(Space::N1p5)
    }

    pub fn p_2(self) -> Self {
        self.p(Space::N2)
    }

    pub fn p_2p5(self) -> Self {
        self.p(Space::N2p5)
    }

    pub fn p_3(self) -> Self {
        self.p(Space::N3)
    }

    pub fn p_4(self) -> Self {
        self.p(Space::N4)
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

#[derive(Debug, Clone, Default)]
pub enum LengthRefinement {
    #[default]
    Auto,
    Px(MetricRef),
    Fill,
}

#[derive(Debug, Clone, Default)]
pub struct SizeRefinement {
    pub width: Option<LengthRefinement>,
    pub height: Option<LengthRefinement>,
    pub min_width: Option<MetricRef>,
    pub min_height: Option<MetricRef>,
    pub max_width: Option<MetricRef>,
    pub max_height: Option<MetricRef>,
}

#[derive(Debug, Clone, Default)]
pub struct FlexItemRefinement {
    pub grow: Option<f32>,
    pub shrink: Option<f32>,
    pub basis: Option<LengthRefinement>,
}

/// Layout-affecting style patches (margin, positioning, size constraints, flex/grid).
///
/// These apply only in the declarative authoring path (or via explicit wrappers). Retained widgets
/// must not silently accept these fields, as that would create Tailwind-like APIs that appear to
/// work but are actually no-ops.
#[derive(Debug, Clone, Default)]
pub struct LayoutRefinement {
    pub aspect_ratio: Option<f32>,
    pub margin: Option<MarginRefinement>,
    pub position: Option<PositionStyle>,
    pub inset: Option<InsetRefinement>,
    pub size: Option<SizeRefinement>,
    pub flex_item: Option<FlexItemRefinement>,
}

impl LayoutRefinement {
    pub fn merge(mut self, other: LayoutRefinement) -> Self {
        if other.aspect_ratio.is_some() {
            self.aspect_ratio = other.aspect_ratio;
        }
        if let Some(m) = other.margin {
            self.margin = Some(self.margin.unwrap_or_default().merge(m));
        }
        if other.position.is_some() {
            self.position = other.position;
        }
        if let Some(i) = other.inset {
            self.inset = Some(self.inset.unwrap_or_default().merge(i));
        }
        if let Some(s) = other.size {
            self.size = Some(s);
        }
        if let Some(f) = other.flex_item {
            self.flex_item = Some(f);
        }
        self
    }

    pub fn aspect_ratio(mut self, ratio: f32) -> Self {
        self.aspect_ratio = Some(ratio);
        self
    }

    pub fn relative(mut self) -> Self {
        self.position = Some(PositionStyle::Relative);
        self
    }

    pub fn absolute(mut self) -> Self {
        self.position = Some(PositionStyle::Absolute);
        self
    }

    pub fn inset(mut self, space: Space) -> Self {
        let m = MetricRef::space(space);
        self.inset = Some(InsetRefinement {
            top: Some(m.clone()),
            right: Some(m.clone()),
            bottom: Some(m.clone()),
            left: Some(m),
        });
        self
    }

    pub fn top(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.top = Some(MetricRef::space(space));
        self.inset = Some(inset);
        self
    }

    pub fn right(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.right = Some(MetricRef::space(space));
        self.inset = Some(inset);
        self
    }

    pub fn bottom(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.bottom = Some(MetricRef::space(space));
        self.inset = Some(inset);
        self
    }

    pub fn left(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.left = Some(MetricRef::space(space));
        self.inset = Some(inset);
        self
    }

    pub fn m(mut self, space: Space) -> Self {
        let m = MetricRef::space(space);
        self.margin = Some(MarginRefinement {
            top: Some(m.clone()),
            right: Some(m.clone()),
            bottom: Some(m.clone()),
            left: Some(m),
        });
        self
    }

    pub fn mx(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        let m = MetricRef::space(space);
        margin.left = Some(m.clone());
        margin.right = Some(m);
        self.margin = Some(margin);
        self
    }

    pub fn my(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        let m = MetricRef::space(space);
        margin.top = Some(m.clone());
        margin.bottom = Some(m);
        self.margin = Some(margin);
        self
    }

    pub fn mt(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.top = Some(MetricRef::space(space));
        self.margin = Some(margin);
        self
    }

    pub fn mr(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.right = Some(MetricRef::space(space));
        self.margin = Some(margin);
        self
    }

    pub fn mb(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.bottom = Some(MetricRef::space(space));
        self.margin = Some(margin);
        self
    }

    pub fn ml(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.left = Some(MetricRef::space(space));
        self.margin = Some(margin);
        self
    }
}

pub fn component_color(key: &'static str, fallback: ColorFallback) -> ColorRef {
    ColorRef::Token { key, fallback }
}

pub fn component_metric(key: &'static str, fallback: MetricFallback) -> MetricRef {
    MetricRef::Token { key, fallback }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_ui::ThemeConfig;

    #[test]
    fn space_falls_back_to_theme_padding_scale() {
        let mut app = fret_app::App::default();

        let cfg = ThemeConfig {
            name: "Test".to_string(),
            metrics: std::collections::HashMap::from([
                ("metric.padding.sm".to_string(), 12.0),
                ("metric.padding.md".to_string(), 14.0),
            ]),
            ..ThemeConfig::default()
        };
        Theme::global_mut(&mut app).apply_config(&cfg);

        let theme = Theme::global(&app);
        assert_eq!(MetricRef::space(Space::N2).resolve(theme), Px(12.0));
        assert_eq!(MetricRef::space(Space::N2p5).resolve(theme), Px(14.0));
        assert_eq!(MetricRef::space(Space::N1).resolve(theme), Px(6.0));
        assert_eq!(MetricRef::space(Space::N0p5).resolve(theme), Px(3.0));
        assert_eq!(MetricRef::space(Space::N11).resolve(theme), Px(66.0));
    }

    #[test]
    fn space_token_overrides_theme_fallback() {
        let mut app = fret_app::App::default();

        let cfg = ThemeConfig {
            name: "Test".to_string(),
            metrics: std::collections::HashMap::from([
                ("metric.padding.sm".to_string(), 12.0),
                ("component.space.2".to_string(), 20.0),
            ]),
            ..ThemeConfig::default()
        };
        Theme::global_mut(&mut app).apply_config(&cfg);

        let theme = Theme::global(&app);
        assert_eq!(MetricRef::space(Space::N2).resolve(theme), Px(20.0));
    }

    #[test]
    fn radius_falls_back_to_baseline_metric_tokens() {
        let mut app = fret_app::App::default();

        let cfg = ThemeConfig {
            name: "Test".to_string(),
            metrics: std::collections::HashMap::from([
                ("metric.radius.sm".to_string(), 11.0),
                ("metric.radius.md".to_string(), 9.0),
                ("component.radius.md".to_string(), 12.0),
            ]),
            ..ThemeConfig::default()
        };
        Theme::global_mut(&mut app).apply_config(&cfg);

        let theme = Theme::global(&app);
        assert_eq!(MetricRef::radius(Radius::Md).resolve(theme), Px(12.0));
        assert_eq!(MetricRef::radius(Radius::Sm).resolve(theme), Px(11.0));
    }
}
