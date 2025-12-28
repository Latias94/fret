use fret_core::{Color, Px};
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, MainAlign, Overflow, PositionStyle};

/// Tailwind-like `justify-*` vocabulary (component-layer).
///
/// This exists to avoid leaking runtime enums into recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Justify {
    Start,
    Center,
    End,
    Between,
    Around,
    Evenly,
}

impl Justify {
    pub fn to_main_align(self) -> MainAlign {
        match self {
            Self::Start => MainAlign::Start,
            Self::Center => MainAlign::Center,
            Self::End => MainAlign::End,
            Self::Between => MainAlign::SpaceBetween,
            Self::Around => MainAlign::SpaceAround,
            Self::Evenly => MainAlign::SpaceEvenly,
        }
    }
}

/// Tailwind-like `items-*` vocabulary (component-layer).
///
/// This exists to avoid leaking runtime enums into recipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Items {
    Start,
    Center,
    End,
    Stretch,
}

impl Items {
    pub fn to_cross_align(self) -> CrossAlign {
        match self {
            Self::Start => CrossAlign::Start,
            Self::Center => CrossAlign::Center,
            Self::End => CrossAlign::End,
            Self::Stretch => CrossAlign::Stretch,
        }
    }
}

/// Tailwind-like overflow vocabulary (component-layer).
///
/// Note: Fret deliberately separates clipping (`overflow_hidden`) from scrolling (explicit `Scroll`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowRefinement {
    Visible,
    Hidden,
}

impl OverflowRefinement {
    pub fn to_overflow(self) -> Overflow {
        match self {
            Self::Visible => Overflow::Visible,
            Self::Hidden => Overflow::Clip,
        }
    }
}

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
    pub top: Option<MarginEdgeRefinement>,
    pub right: Option<MarginEdgeRefinement>,
    pub bottom: Option<MarginEdgeRefinement>,
    pub left: Option<MarginEdgeRefinement>,
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
    pub top: Option<SignedMetricRef>,
    pub right: Option<SignedMetricRef>,
    pub bottom: Option<SignedMetricRef>,
    pub left: Option<SignedMetricRef>,
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

#[derive(Debug, Clone)]
pub enum MarginEdgeRefinement {
    Px(SignedMetricRef),
    Auto,
}

impl MarginEdgeRefinement {
    pub fn resolve(&self, theme: &Theme) -> fret_ui::element::MarginEdge {
        match self {
            Self::Px(m) => fret_ui::element::MarginEdge::Px(m.resolve(theme)),
            Self::Auto => fret_ui::element::MarginEdge::Auto,
        }
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
    pub fn px_0(self) -> Self {
        self.px(Space::N0)
    }

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

    pub fn py_0(self) -> Self {
        self.py(Space::N0)
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

    pub fn p_0(self) -> Self {
        self.p(Space::N0)
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

impl SizeRefinement {
    pub fn merge(mut self, other: SizeRefinement) -> Self {
        if other.width.is_some() {
            self.width = other.width;
        }
        if other.height.is_some() {
            self.height = other.height;
        }
        if other.min_width.is_some() {
            self.min_width = other.min_width;
        }
        if other.min_height.is_some() {
            self.min_height = other.min_height;
        }
        if other.max_width.is_some() {
            self.max_width = other.max_width;
        }
        if other.max_height.is_some() {
            self.max_height = other.max_height;
        }
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct FlexItemRefinement {
    pub grow: Option<f32>,
    pub shrink: Option<f32>,
    pub basis: Option<LengthRefinement>,
}

impl FlexItemRefinement {
    pub fn merge(mut self, other: FlexItemRefinement) -> Self {
        if other.grow.is_some() {
            self.grow = other.grow;
        }
        if other.shrink.is_some() {
            self.shrink = other.shrink;
        }
        if other.basis.is_some() {
            self.basis = other.basis;
        }
        self
    }
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
    pub overflow: Option<OverflowRefinement>,
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
            self.size = Some(self.size.unwrap_or_default().merge(s));
        }
        if let Some(f) = other.flex_item {
            self.flex_item = Some(self.flex_item.unwrap_or_default().merge(f));
        }
        if other.overflow.is_some() {
            self.overflow = other.overflow;
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

    pub fn overflow_hidden(mut self) -> Self {
        self.overflow = Some(OverflowRefinement::Hidden);
        self
    }

    pub fn overflow_visible(mut self) -> Self {
        self.overflow = Some(OverflowRefinement::Visible);
        self
    }

    pub fn overflow_x_hidden(self) -> Self {
        self.overflow_hidden()
    }

    pub fn overflow_y_hidden(self) -> Self {
        self.overflow_hidden()
    }

    pub fn inset(mut self, space: Space) -> Self {
        let m = SignedMetricRef::pos(MetricRef::space(space));
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
        inset.top = Some(SignedMetricRef::pos(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn top_neg(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.top = Some(SignedMetricRef::neg(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn right(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.right = Some(SignedMetricRef::pos(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn right_neg(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.right = Some(SignedMetricRef::neg(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn bottom(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.bottom = Some(SignedMetricRef::pos(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn bottom_neg(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.bottom = Some(SignedMetricRef::neg(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn left(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.left = Some(SignedMetricRef::pos(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn left_neg(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.left = Some(SignedMetricRef::neg(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn m(mut self, space: Space) -> Self {
        let m = MarginEdgeRefinement::Px(SignedMetricRef::pos(MetricRef::space(space)));
        self.margin = Some(MarginRefinement {
            top: Some(m.clone()),
            right: Some(m.clone()),
            bottom: Some(m.clone()),
            left: Some(m),
        });
        self
    }

    pub fn m_neg(mut self, space: Space) -> Self {
        let m = MarginEdgeRefinement::Px(SignedMetricRef::neg(MetricRef::space(space)));
        self.margin = Some(MarginRefinement {
            top: Some(m.clone()),
            right: Some(m.clone()),
            bottom: Some(m.clone()),
            left: Some(m),
        });
        self
    }

    pub fn m_auto(mut self) -> Self {
        let a = MarginEdgeRefinement::Auto;
        self.margin = Some(MarginRefinement {
            top: Some(a.clone()),
            right: Some(a.clone()),
            bottom: Some(a.clone()),
            left: Some(a),
        });
        self
    }

    pub fn mx(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        let m = MarginEdgeRefinement::Px(SignedMetricRef::pos(MetricRef::space(space)));
        margin.left = Some(m.clone());
        margin.right = Some(m);
        self.margin = Some(margin);
        self
    }

    pub fn mx_neg(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        let m = MarginEdgeRefinement::Px(SignedMetricRef::neg(MetricRef::space(space)));
        margin.left = Some(m.clone());
        margin.right = Some(m);
        self.margin = Some(margin);
        self
    }

    pub fn mx_auto(mut self) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.left = Some(MarginEdgeRefinement::Auto);
        margin.right = Some(MarginEdgeRefinement::Auto);
        self.margin = Some(margin);
        self
    }

    pub fn my(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        let m = MarginEdgeRefinement::Px(SignedMetricRef::pos(MetricRef::space(space)));
        margin.top = Some(m.clone());
        margin.bottom = Some(m);
        self.margin = Some(margin);
        self
    }

    pub fn my_neg(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        let m = MarginEdgeRefinement::Px(SignedMetricRef::neg(MetricRef::space(space)));
        margin.top = Some(m.clone());
        margin.bottom = Some(m);
        self.margin = Some(margin);
        self
    }

    pub fn my_auto(mut self) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.top = Some(MarginEdgeRefinement::Auto);
        margin.bottom = Some(MarginEdgeRefinement::Auto);
        self.margin = Some(margin);
        self
    }

    pub fn mt(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.top = Some(MarginEdgeRefinement::Px(SignedMetricRef::pos(
            MetricRef::space(space),
        )));
        self.margin = Some(margin);
        self
    }

    pub fn mt_neg(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.top = Some(MarginEdgeRefinement::Px(SignedMetricRef::neg(
            MetricRef::space(space),
        )));
        self.margin = Some(margin);
        self
    }

    pub fn mt_auto(mut self) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.top = Some(MarginEdgeRefinement::Auto);
        self.margin = Some(margin);
        self
    }

    pub fn mr(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.right = Some(MarginEdgeRefinement::Px(SignedMetricRef::pos(
            MetricRef::space(space),
        )));
        self.margin = Some(margin);
        self
    }

    pub fn mr_neg(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.right = Some(MarginEdgeRefinement::Px(SignedMetricRef::neg(
            MetricRef::space(space),
        )));
        self.margin = Some(margin);
        self
    }

    pub fn mr_auto(mut self) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.right = Some(MarginEdgeRefinement::Auto);
        self.margin = Some(margin);
        self
    }

    pub fn mb(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.bottom = Some(MarginEdgeRefinement::Px(SignedMetricRef::pos(
            MetricRef::space(space),
        )));
        self.margin = Some(margin);
        self
    }

    pub fn mb_neg(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.bottom = Some(MarginEdgeRefinement::Px(SignedMetricRef::neg(
            MetricRef::space(space),
        )));
        self.margin = Some(margin);
        self
    }

    pub fn mb_auto(mut self) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.bottom = Some(MarginEdgeRefinement::Auto);
        self.margin = Some(margin);
        self
    }

    pub fn ml(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.left = Some(MarginEdgeRefinement::Px(SignedMetricRef::pos(
            MetricRef::space(space),
        )));
        self.margin = Some(margin);
        self
    }

    pub fn ml_neg(mut self, space: Space) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.left = Some(MarginEdgeRefinement::Px(SignedMetricRef::neg(
            MetricRef::space(space),
        )));
        self.margin = Some(margin);
        self
    }

    pub fn ml_auto(mut self) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.left = Some(MarginEdgeRefinement::Auto);
        self.margin = Some(margin);
        self
    }

    fn ensure_size_mut(&mut self) -> &mut SizeRefinement {
        if self.size.is_none() {
            self.size = Some(SizeRefinement::default());
        }
        self.size.as_mut().expect("size exists")
    }

    fn ensure_flex_item_mut(&mut self) -> &mut FlexItemRefinement {
        if self.flex_item.is_none() {
            self.flex_item = Some(FlexItemRefinement::default());
        }
        self.flex_item.as_mut().expect("flex_item exists")
    }

    pub fn min_w(mut self, width: MetricRef) -> Self {
        self.ensure_size_mut().min_width = Some(width);
        self
    }

    pub fn min_w_space(self, width: Space) -> Self {
        self.min_w(MetricRef::space(width))
    }

    pub fn min_h(mut self, height: MetricRef) -> Self {
        self.ensure_size_mut().min_height = Some(height);
        self
    }

    pub fn min_h_space(self, height: Space) -> Self {
        self.min_h(MetricRef::space(height))
    }

    pub fn min_w_0(self) -> Self {
        self.min_w(MetricRef::Px(Px(0.0)))
    }

    pub fn w(mut self, width: LengthRefinement) -> Self {
        self.ensure_size_mut().width = Some(width);
        self
    }

    pub fn h(mut self, height: LengthRefinement) -> Self {
        self.ensure_size_mut().height = Some(height);
        self
    }

    pub fn w_px(self, width: MetricRef) -> Self {
        self.w(LengthRefinement::Px(width))
    }

    pub fn w_space(self, width: Space) -> Self {
        self.w_px(MetricRef::space(width))
    }

    pub fn h_px(self, height: MetricRef) -> Self {
        self.h(LengthRefinement::Px(height))
    }

    pub fn h_space(self, height: Space) -> Self {
        self.h_px(MetricRef::space(height))
    }

    pub fn w_full(self) -> Self {
        self.w(LengthRefinement::Fill)
    }

    pub fn h_full(self) -> Self {
        self.h(LengthRefinement::Fill)
    }

    pub fn size_full(self) -> Self {
        self.w_full().h_full()
    }

    pub fn max_w(mut self, width: MetricRef) -> Self {
        self.ensure_size_mut().max_width = Some(width);
        self
    }

    pub fn max_w_space(self, width: Space) -> Self {
        self.max_w(MetricRef::space(width))
    }

    pub fn max_h(mut self, height: MetricRef) -> Self {
        self.ensure_size_mut().max_height = Some(height);
        self
    }

    pub fn max_h_space(self, height: Space) -> Self {
        self.max_h(MetricRef::space(height))
    }

    pub fn basis(mut self, basis: LengthRefinement) -> Self {
        self.ensure_flex_item_mut().basis = Some(basis);
        self
    }

    pub fn basis_0(self) -> Self {
        self.basis(LengthRefinement::Px(MetricRef::Px(Px(0.0))))
    }

    pub fn flex_grow(mut self, grow: f32) -> Self {
        self.ensure_flex_item_mut().grow = Some(grow);
        self
    }

    pub fn flex_shrink(mut self, shrink: f32) -> Self {
        self.ensure_flex_item_mut().shrink = Some(shrink);
        self
    }

    pub fn flex_shrink_0(self) -> Self {
        self.flex_shrink(0.0)
    }

    /// Tailwind-like `flex-1` shorthand: `grow=1`, `shrink=1`, `basis=0`.
    pub fn flex_1(mut self) -> Self {
        {
            let f = self.ensure_flex_item_mut();
            f.grow = Some(1.0);
            f.shrink = Some(1.0);
            f.basis = Some(LengthRefinement::Px(MetricRef::Px(Px(0.0))));
        }
        self
    }

    /// Tailwind-like `flex-none` shorthand: `grow=0`, `shrink=0`, `basis=auto`.
    pub fn flex_none(mut self) -> Self {
        {
            let f = self.ensure_flex_item_mut();
            f.grow = Some(0.0);
            f.shrink = Some(0.0);
            f.basis = Some(LengthRefinement::Auto);
        }
        self
    }
}

impl LayoutRefinement {
    pub fn w_0(self) -> Self {
        self.w_space(Space::N0)
    }

    pub fn h_0(self) -> Self {
        self.h_space(Space::N0)
    }

    pub fn min_h_0(self) -> Self {
        self.min_h_space(Space::N0)
    }

    pub fn max_w_0(self) -> Self {
        self.max_w_space(Space::N0)
    }

    pub fn max_h_0(self) -> Self {
        self.max_h_space(Space::N0)
    }

    pub fn w_0p5(self) -> Self {
        self.w_space(Space::N0p5)
    }

    pub fn h_0p5(self) -> Self {
        self.h_space(Space::N0p5)
    }

    pub fn min_w_0p5(self) -> Self {
        self.min_w_space(Space::N0p5)
    }

    pub fn min_h_0p5(self) -> Self {
        self.min_h_space(Space::N0p5)
    }

    pub fn max_w_0p5(self) -> Self {
        self.max_w_space(Space::N0p5)
    }

    pub fn max_h_0p5(self) -> Self {
        self.max_h_space(Space::N0p5)
    }

    pub fn w_1(self) -> Self {
        self.w_space(Space::N1)
    }

    pub fn h_1(self) -> Self {
        self.h_space(Space::N1)
    }

    pub fn min_w_1(self) -> Self {
        self.min_w_space(Space::N1)
    }

    pub fn min_h_1(self) -> Self {
        self.min_h_space(Space::N1)
    }

    pub fn max_w_1(self) -> Self {
        self.max_w_space(Space::N1)
    }

    pub fn max_h_1(self) -> Self {
        self.max_h_space(Space::N1)
    }

    pub fn w_1p5(self) -> Self {
        self.w_space(Space::N1p5)
    }

    pub fn h_1p5(self) -> Self {
        self.h_space(Space::N1p5)
    }

    pub fn min_w_1p5(self) -> Self {
        self.min_w_space(Space::N1p5)
    }

    pub fn min_h_1p5(self) -> Self {
        self.min_h_space(Space::N1p5)
    }

    pub fn max_w_1p5(self) -> Self {
        self.max_w_space(Space::N1p5)
    }

    pub fn max_h_1p5(self) -> Self {
        self.max_h_space(Space::N1p5)
    }

    pub fn w_2(self) -> Self {
        self.w_space(Space::N2)
    }

    pub fn h_2(self) -> Self {
        self.h_space(Space::N2)
    }

    pub fn min_w_2(self) -> Self {
        self.min_w_space(Space::N2)
    }

    pub fn min_h_2(self) -> Self {
        self.min_h_space(Space::N2)
    }

    pub fn max_w_2(self) -> Self {
        self.max_w_space(Space::N2)
    }

    pub fn max_h_2(self) -> Self {
        self.max_h_space(Space::N2)
    }

    pub fn w_2p5(self) -> Self {
        self.w_space(Space::N2p5)
    }

    pub fn h_2p5(self) -> Self {
        self.h_space(Space::N2p5)
    }

    pub fn min_w_2p5(self) -> Self {
        self.min_w_space(Space::N2p5)
    }

    pub fn min_h_2p5(self) -> Self {
        self.min_h_space(Space::N2p5)
    }

    pub fn max_w_2p5(self) -> Self {
        self.max_w_space(Space::N2p5)
    }

    pub fn max_h_2p5(self) -> Self {
        self.max_h_space(Space::N2p5)
    }

    pub fn w_3(self) -> Self {
        self.w_space(Space::N3)
    }

    pub fn h_3(self) -> Self {
        self.h_space(Space::N3)
    }

    pub fn min_w_3(self) -> Self {
        self.min_w_space(Space::N3)
    }

    pub fn min_h_3(self) -> Self {
        self.min_h_space(Space::N3)
    }

    pub fn max_w_3(self) -> Self {
        self.max_w_space(Space::N3)
    }

    pub fn max_h_3(self) -> Self {
        self.max_h_space(Space::N3)
    }

    pub fn w_3p5(self) -> Self {
        self.w_space(Space::N3p5)
    }

    pub fn h_3p5(self) -> Self {
        self.h_space(Space::N3p5)
    }

    pub fn min_w_3p5(self) -> Self {
        self.min_w_space(Space::N3p5)
    }

    pub fn min_h_3p5(self) -> Self {
        self.min_h_space(Space::N3p5)
    }

    pub fn max_w_3p5(self) -> Self {
        self.max_w_space(Space::N3p5)
    }

    pub fn max_h_3p5(self) -> Self {
        self.max_h_space(Space::N3p5)
    }

    pub fn w_4(self) -> Self {
        self.w_space(Space::N4)
    }

    pub fn h_4(self) -> Self {
        self.h_space(Space::N4)
    }

    pub fn min_w_4(self) -> Self {
        self.min_w_space(Space::N4)
    }

    pub fn min_h_4(self) -> Self {
        self.min_h_space(Space::N4)
    }

    pub fn max_w_4(self) -> Self {
        self.max_w_space(Space::N4)
    }

    pub fn max_h_4(self) -> Self {
        self.max_h_space(Space::N4)
    }

    pub fn w_5(self) -> Self {
        self.w_space(Space::N5)
    }

    pub fn h_5(self) -> Self {
        self.h_space(Space::N5)
    }

    pub fn min_w_5(self) -> Self {
        self.min_w_space(Space::N5)
    }

    pub fn min_h_5(self) -> Self {
        self.min_h_space(Space::N5)
    }

    pub fn max_w_5(self) -> Self {
        self.max_w_space(Space::N5)
    }

    pub fn max_h_5(self) -> Self {
        self.max_h_space(Space::N5)
    }

    pub fn w_6(self) -> Self {
        self.w_space(Space::N6)
    }

    pub fn h_6(self) -> Self {
        self.h_space(Space::N6)
    }

    pub fn min_w_6(self) -> Self {
        self.min_w_space(Space::N6)
    }

    pub fn min_h_6(self) -> Self {
        self.min_h_space(Space::N6)
    }

    pub fn max_w_6(self) -> Self {
        self.max_w_space(Space::N6)
    }

    pub fn max_h_6(self) -> Self {
        self.max_h_space(Space::N6)
    }

    pub fn w_8(self) -> Self {
        self.w_space(Space::N8)
    }

    pub fn h_8(self) -> Self {
        self.h_space(Space::N8)
    }

    pub fn min_w_8(self) -> Self {
        self.min_w_space(Space::N8)
    }

    pub fn min_h_8(self) -> Self {
        self.min_h_space(Space::N8)
    }

    pub fn max_w_8(self) -> Self {
        self.max_w_space(Space::N8)
    }

    pub fn max_h_8(self) -> Self {
        self.max_h_space(Space::N8)
    }

    pub fn w_10(self) -> Self {
        self.w_space(Space::N10)
    }

    pub fn h_10(self) -> Self {
        self.h_space(Space::N10)
    }

    pub fn min_w_10(self) -> Self {
        self.min_w_space(Space::N10)
    }

    pub fn min_h_10(self) -> Self {
        self.min_h_space(Space::N10)
    }

    pub fn max_w_10(self) -> Self {
        self.max_w_space(Space::N10)
    }

    pub fn max_h_10(self) -> Self {
        self.max_h_space(Space::N10)
    }

    pub fn w_11(self) -> Self {
        self.w_space(Space::N11)
    }

    pub fn h_11(self) -> Self {
        self.h_space(Space::N11)
    }

    pub fn min_w_11(self) -> Self {
        self.min_w_space(Space::N11)
    }

    pub fn min_h_11(self) -> Self {
        self.min_h_space(Space::N11)
    }

    pub fn max_w_11(self) -> Self {
        self.max_w_space(Space::N11)
    }

    pub fn max_h_11(self) -> Self {
        self.max_h_space(Space::N11)
    }
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
