use fret_core::Px;
use fret_ui::Theme;

use super::{ColorFallback, ColorRef, MetricRef, Radius, SignedMetricRef, Space};
use crate::Corners4;

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
    pub corner_radii: Option<CornerRadiiRefinement>,
    pub shadow: Option<ShadowPreset>,
    pub border_width: Option<MetricRef>,
    pub background: Option<ColorRef>,
    pub border_color: Option<ColorRef>,
    pub text_color: Option<ColorRef>,
}

#[derive(Debug, Clone, Default)]
pub struct CornerRadiiRefinement {
    pub top_left: Option<MetricRef>,
    pub top_right: Option<MetricRef>,
    pub bottom_right: Option<MetricRef>,
    pub bottom_left: Option<MetricRef>,
}

impl CornerRadiiRefinement {
    pub fn merge(mut self, other: CornerRadiiRefinement) -> Self {
        if other.top_left.is_some() {
            self.top_left = other.top_left;
        }
        if other.top_right.is_some() {
            self.top_right = other.top_right;
        }
        if other.bottom_right.is_some() {
            self.bottom_right = other.bottom_right;
        }
        if other.bottom_left.is_some() {
            self.bottom_left = other.bottom_left;
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowPreset {
    None,
    Xs,
    Sm,
    Md,
    Lg,
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
        if let Some(r) = other.corner_radii {
            self.corner_radii = Some(self.corner_radii.unwrap_or_default().merge(r));
        }
        if other.shadow.is_some() {
            self.shadow = other.shadow;
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

    pub fn border_width(mut self, width: impl Into<MetricRef>) -> Self {
        self.border_width = Some(width.into());
        self
    }

    pub fn radius(mut self, radius: impl Into<MetricRef>) -> Self {
        self.radius = Some(radius.into());
        self
    }

    pub fn rounded(self, radius: Radius) -> Self {
        self.radius(MetricRef::radius(radius))
    }

    pub fn corner_radii(mut self, radii: impl Into<Corners4<MetricRef>>) -> Self {
        let radii = radii.into();
        self.corner_radii = Some(CornerRadiiRefinement {
            top_left: Some(radii.top_left),
            top_right: Some(radii.top_right),
            bottom_right: Some(radii.bottom_right),
            bottom_left: Some(radii.bottom_left),
        });
        self
    }

    pub fn rounded_tl(mut self, radius: Radius) -> Self {
        let mut radii = self.corner_radii.unwrap_or_default();
        radii.top_left = Some(MetricRef::radius(radius));
        self.corner_radii = Some(radii);
        self
    }

    pub fn rounded_tr(mut self, radius: Radius) -> Self {
        let mut radii = self.corner_radii.unwrap_or_default();
        radii.top_right = Some(MetricRef::radius(radius));
        self.corner_radii = Some(radii);
        self
    }

    pub fn rounded_br(mut self, radius: Radius) -> Self {
        let mut radii = self.corner_radii.unwrap_or_default();
        radii.bottom_right = Some(MetricRef::radius(radius));
        self.corner_radii = Some(radii);
        self
    }

    pub fn rounded_bl(mut self, radius: Radius) -> Self {
        let mut radii = self.corner_radii.unwrap_or_default();
        radii.bottom_left = Some(MetricRef::radius(radius));
        self.corner_radii = Some(radii);
        self
    }

    pub fn shadow(mut self, shadow: ShadowPreset) -> Self {
        self.shadow = Some(shadow);
        self
    }

    pub fn shadow_none(self) -> Self {
        self.shadow(ShadowPreset::None)
    }

    pub fn shadow_xs(self) -> Self {
        self.shadow(ShadowPreset::Xs)
    }

    pub fn shadow_sm(self) -> Self {
        self.shadow(ShadowPreset::Sm)
    }

    pub fn shadow_md(self) -> Self {
        self.shadow(ShadowPreset::Md)
    }

    pub fn shadow_lg(self) -> Self {
        self.shadow(ShadowPreset::Lg)
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

    pub fn border_1(self) -> Self {
        self.border_width(Px(1.0))
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

    pub fn focused_border(self) -> Self {
        self.border_1().border_color(ColorRef::Token {
            key: "ring",
            fallback: ColorFallback::ThemeFocusRing,
        })
    }

    pub fn debug_border(self, color: ColorRef) -> Self {
        if cfg!(debug_assertions) {
            self.border_1().border_color(color)
        } else {
            self
        }
    }

    pub fn debug_border_primary(self) -> Self {
        self.debug_border(ColorRef::Token {
            key: "primary",
            fallback: ColorFallback::ThemeAccent,
        })
    }

    pub fn debug_border_destructive(self) -> Self {
        self.debug_border(ColorRef::Token {
            key: "destructive",
            fallback: ColorFallback::Color(fret_core::Color {
                r: 0.937,
                g: 0.267,
                b: 0.267,
                a: 1.0,
            }),
        })
    }

    pub fn debug_border_ring(self) -> Self {
        self.debug_border(ColorRef::Token {
            key: "ring",
            fallback: ColorFallback::ThemeFocusRing,
        })
    }
}
