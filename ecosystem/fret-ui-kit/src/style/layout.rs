use fret_core::Px;
use fret_ui::element::PositionStyle;

use super::{
    InsetRefinement, MarginEdgeRefinement, MarginRefinement, MetricRef, OverflowRefinement,
    SignedMetricRef, Space,
};

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

    pub fn inset_px(mut self, px: Px) -> Self {
        let m = SignedMetricRef::pos(px.into());
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

    pub fn top_px(mut self, px: Px) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.top = Some(SignedMetricRef::pos(px.into()));
        self.inset = Some(inset);
        self
    }

    pub fn top_neg(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.top = Some(SignedMetricRef::neg(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn top_neg_px(mut self, px: impl Into<MetricRef>) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.top = Some(SignedMetricRef::neg(px.into()));
        self.inset = Some(inset);
        self
    }

    pub fn right(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.right = Some(SignedMetricRef::pos(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn right_px(mut self, px: Px) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.right = Some(SignedMetricRef::pos(px.into()));
        self.inset = Some(inset);
        self
    }

    pub fn right_neg(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.right = Some(SignedMetricRef::neg(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn right_neg_px(mut self, px: impl Into<MetricRef>) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.right = Some(SignedMetricRef::neg(px.into()));
        self.inset = Some(inset);
        self
    }

    pub fn bottom(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.bottom = Some(SignedMetricRef::pos(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn bottom_px(mut self, px: Px) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.bottom = Some(SignedMetricRef::pos(px.into()));
        self.inset = Some(inset);
        self
    }

    pub fn bottom_neg(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.bottom = Some(SignedMetricRef::neg(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn bottom_neg_px(mut self, px: impl Into<MetricRef>) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.bottom = Some(SignedMetricRef::neg(px.into()));
        self.inset = Some(inset);
        self
    }

    pub fn left(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.left = Some(SignedMetricRef::pos(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn left_px(mut self, px: Px) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.left = Some(SignedMetricRef::pos(px.into()));
        self.inset = Some(inset);
        self
    }

    pub fn left_neg(mut self, space: Space) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.left = Some(SignedMetricRef::neg(MetricRef::space(space)));
        self.inset = Some(inset);
        self
    }

    pub fn left_neg_px(mut self, px: impl Into<MetricRef>) -> Self {
        let mut inset = self.inset.unwrap_or_default();
        inset.left = Some(SignedMetricRef::neg(px.into()));
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

    pub fn m_px(mut self, px: Px) -> Self {
        let m = MarginEdgeRefinement::Px(SignedMetricRef::pos(px.into()));
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

    pub fn mx_px(mut self, px: Px) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        let m = MarginEdgeRefinement::Px(SignedMetricRef::pos(px.into()));
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

    pub fn my_px(mut self, px: Px) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        let m = MarginEdgeRefinement::Px(SignedMetricRef::pos(px.into()));
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

    pub fn mt_px(mut self, px: Px) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.top = Some(MarginEdgeRefinement::Px(SignedMetricRef::pos(px.into())));
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

    pub fn mr_px(mut self, px: Px) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.right = Some(MarginEdgeRefinement::Px(SignedMetricRef::pos(px.into())));
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

    pub fn mb_px(mut self, px: Px) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.bottom = Some(MarginEdgeRefinement::Px(SignedMetricRef::pos(px.into())));
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

    pub fn ml_px(mut self, px: Px) -> Self {
        let mut margin = self.margin.unwrap_or_default();
        margin.left = Some(MarginEdgeRefinement::Px(SignedMetricRef::pos(px.into())));
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

    pub fn min_w(mut self, width: impl Into<MetricRef>) -> Self {
        self.ensure_size_mut().min_width = Some(width.into());
        self
    }

    pub fn min_w_space(self, width: Space) -> Self {
        self.min_w(MetricRef::space(width))
    }

    pub fn min_h(mut self, height: impl Into<MetricRef>) -> Self {
        self.ensure_size_mut().min_height = Some(height.into());
        self
    }

    pub fn min_h_space(self, height: Space) -> Self {
        self.min_h(MetricRef::space(height))
    }

    /// Shorthand for `min-width: 0px`.
    ///
    /// This is especially important for text-heavy children in a horizontal flex row: without it,
    /// the flex item may refuse to shrink (web-like `min-width: auto` behavior), causing overflow.
    pub fn min_w_0(self) -> Self {
        self.min_w(Px(0.0))
    }

    pub fn w(mut self, width: LengthRefinement) -> Self {
        self.ensure_size_mut().width = Some(width);
        self
    }

    pub fn h(mut self, height: LengthRefinement) -> Self {
        self.ensure_size_mut().height = Some(height);
        self
    }

    pub fn w_px(self, width: impl Into<MetricRef>) -> Self {
        self.w(LengthRefinement::Px(width.into()))
    }

    pub fn w_space(self, width: Space) -> Self {
        self.w_px(MetricRef::space(width))
    }

    pub fn h_px(self, height: impl Into<MetricRef>) -> Self {
        self.h(LengthRefinement::Px(height.into()))
    }

    pub fn h_space(self, height: Space) -> Self {
        self.h_px(MetricRef::space(height))
    }

    /// Shorthand for `width: 100%` of the containing block (percent sizing).
    ///
    /// This is **not** the same as Tailwind `flex-1`. For "take the remaining space as a flex
    /// item", use [`Self::flex_1`] (and typically [`Self::min_w_0`] for text).
    pub fn w_full(self) -> Self {
        self.w(LengthRefinement::Fill)
    }

    /// Shorthand for `height: 100%` of the containing block (percent sizing).
    pub fn h_full(self) -> Self {
        self.h(LengthRefinement::Fill)
    }

    pub fn size_full(self) -> Self {
        self.w_full().h_full()
    }

    pub fn max_w(mut self, width: impl Into<MetricRef>) -> Self {
        self.ensure_size_mut().max_width = Some(width.into());
        self
    }

    pub fn max_w_space(self, width: Space) -> Self {
        self.max_w(MetricRef::space(width))
    }

    pub fn max_h(mut self, height: impl Into<MetricRef>) -> Self {
        self.ensure_size_mut().max_height = Some(height.into());
        self
    }

    pub fn max_h_space(self, height: Space) -> Self {
        self.max_h(MetricRef::space(height))
    }

    pub fn basis(mut self, basis: LengthRefinement) -> Self {
        self.ensure_flex_item_mut().basis = Some(basis);
        self
    }

    pub fn basis_px(self, basis: impl Into<MetricRef>) -> Self {
        self.basis(LengthRefinement::Px(basis.into()))
    }

    pub fn basis_0(self) -> Self {
        self.basis_px(Px(0.0))
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
    ///
    /// Tip: combine with [`Self::min_w_0`] when the child contains text that should be allowed to
    /// shrink/wrap instead of overflowing.
    pub fn flex_1(mut self) -> Self {
        {
            let f = self.ensure_flex_item_mut();
            f.grow = Some(1.0);
            f.shrink = Some(1.0);
            f.basis = Some(LengthRefinement::Px(Px(0.0).into()));
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
