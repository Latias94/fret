use fret_core::Px;
use fret_ui::element::PositionStyle;

use super::{
    InsetEdgeRefinement, InsetRefinement, MarginEdgeRefinement, MarginRefinement, MetricRef,
    OverflowRefinement, SignedMetricRef, Space,
};

#[derive(Debug, Clone, Default)]
pub enum LengthRefinement {
    #[default]
    Auto,
    Px(MetricRef),
    /// Fraction of the containing block size (percent sizing).
    ///
    /// Expressed as a ratio (e.g. `0.5` for 50%).
    Fraction(f32),
    Fill,
}

#[derive(Debug, Clone, Default)]
pub struct SizeRefinement {
    pub width: Option<LengthRefinement>,
    pub height: Option<LengthRefinement>,
    pub min_width: Option<LengthRefinement>,
    pub min_height: Option<LengthRefinement>,
    pub max_width: Option<LengthRefinement>,
    pub max_height: Option<LengthRefinement>,
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
    pub order: Option<i32>,
    pub grow: Option<f32>,
    pub shrink: Option<f32>,
    pub basis: Option<LengthRefinement>,
}

impl FlexItemRefinement {
    pub fn merge(mut self, other: FlexItemRefinement) -> Self {
        if other.order.is_some() {
            self.order = other.order;
        }
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
    fn ensure_position_for_inset(&mut self) {
        if self.position.is_none() {
            self.position = Some(PositionStyle::Relative);
        }
    }

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
        self.ensure_position_for_inset();
        let m = InsetEdgeRefinement::Px(SignedMetricRef::pos(MetricRef::space(space)));
        self.inset = Some(InsetRefinement {
            top: Some(m.clone()).into(),
            right: Some(m.clone()).into(),
            bottom: Some(m.clone()).into(),
            left: Some(m).into(),
        });
        self
    }

    pub fn inset_px(mut self, px: Px) -> Self {
        self.ensure_position_for_inset();
        let m = InsetEdgeRefinement::Px(SignedMetricRef::pos(px.into()));
        self.inset = Some(InsetRefinement {
            top: Some(m.clone()).into(),
            right: Some(m.clone()).into(),
            bottom: Some(m.clone()).into(),
            left: Some(m).into(),
        });
        self
    }

    /// Shorthand for `inset: 100%` of the containing block (percent sizing).
    pub fn inset_full(mut self) -> Self {
        self.ensure_position_for_inset();
        let m = InsetEdgeRefinement::Fill;
        self.inset = Some(InsetRefinement {
            top: Some(m.clone()).into(),
            right: Some(m.clone()).into(),
            bottom: Some(m.clone()).into(),
            left: Some(m).into(),
        });
        self
    }

    /// Shorthand for `inset: <fraction>` of the containing block.
    ///
    /// Example: `inset_fraction(0.5)` == 50%.
    pub fn inset_fraction(mut self, fraction: f32) -> Self {
        self.ensure_position_for_inset();
        let m = InsetEdgeRefinement::Fraction(fraction);
        self.inset = Some(InsetRefinement {
            top: Some(m.clone()).into(),
            right: Some(m.clone()).into(),
            bottom: Some(m.clone()).into(),
            left: Some(m).into(),
        });
        self
    }

    /// Shorthand for `inset: <percent>%` of the containing block.
    pub fn inset_percent(self, percent: f32) -> Self {
        self.inset_fraction(percent / 100.0)
    }

    pub fn top(mut self, space: Space) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.top = Some(InsetEdgeRefinement::Px(SignedMetricRef::pos(
            MetricRef::space(space),
        )));
        self.inset = Some(inset);
        self
    }

    pub fn top_px(mut self, px: Px) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.top = Some(InsetEdgeRefinement::Px(SignedMetricRef::pos(px.into())));
        self.inset = Some(inset);
        self
    }

    /// Shorthand for `top: 100%` of the containing block (percent sizing).
    pub fn top_full(mut self) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.top = Some(InsetEdgeRefinement::Fill);
        self.inset = Some(inset);
        self
    }

    /// Shorthand for `top: <fraction>` of the containing block.
    pub fn top_fraction(mut self, fraction: f32) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.top = Some(InsetEdgeRefinement::Fraction(fraction));
        self.inset = Some(inset);
        self
    }

    /// Shorthand for `top: <percent>%` of the containing block.
    pub fn top_percent(self, percent: f32) -> Self {
        self.top_fraction(percent / 100.0)
    }

    pub fn top_neg(mut self, space: Space) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.top = Some(InsetEdgeRefinement::Px(SignedMetricRef::neg(
            MetricRef::space(space),
        )));
        self.inset = Some(inset);
        self
    }

    pub fn top_neg_px(mut self, px: impl Into<MetricRef>) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.top = Some(InsetEdgeRefinement::Px(SignedMetricRef::neg(px.into())));
        self.inset = Some(inset);
        self
    }

    pub fn right(mut self, space: Space) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.right = Some(InsetEdgeRefinement::Px(SignedMetricRef::pos(
            MetricRef::space(space),
        )));
        self.inset = Some(inset);
        self
    }

    pub fn right_px(mut self, px: Px) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.right = Some(InsetEdgeRefinement::Px(SignedMetricRef::pos(px.into())));
        self.inset = Some(inset);
        self
    }

    /// Shorthand for `right: 100%` of the containing block (percent sizing).
    pub fn right_full(mut self) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.right = Some(InsetEdgeRefinement::Fill);
        self.inset = Some(inset);
        self
    }

    /// Shorthand for `right: <fraction>` of the containing block.
    pub fn right_fraction(mut self, fraction: f32) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.right = Some(InsetEdgeRefinement::Fraction(fraction));
        self.inset = Some(inset);
        self
    }

    /// Shorthand for `right: <percent>%` of the containing block.
    pub fn right_percent(self, percent: f32) -> Self {
        self.right_fraction(percent / 100.0)
    }

    pub fn right_neg(mut self, space: Space) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.right = Some(InsetEdgeRefinement::Px(SignedMetricRef::neg(
            MetricRef::space(space),
        )));
        self.inset = Some(inset);
        self
    }

    pub fn right_neg_px(mut self, px: impl Into<MetricRef>) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.right = Some(InsetEdgeRefinement::Px(SignedMetricRef::neg(px.into())));
        self.inset = Some(inset);
        self
    }

    pub fn bottom(mut self, space: Space) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.bottom = Some(InsetEdgeRefinement::Px(SignedMetricRef::pos(
            MetricRef::space(space),
        )));
        self.inset = Some(inset);
        self
    }

    pub fn bottom_px(mut self, px: Px) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.bottom = Some(InsetEdgeRefinement::Px(SignedMetricRef::pos(px.into())));
        self.inset = Some(inset);
        self
    }

    /// Shorthand for `bottom: 100%` of the containing block (percent sizing).
    pub fn bottom_full(mut self) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.bottom = Some(InsetEdgeRefinement::Fill);
        self.inset = Some(inset);
        self
    }

    /// Shorthand for `bottom: <fraction>` of the containing block.
    pub fn bottom_fraction(mut self, fraction: f32) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.bottom = Some(InsetEdgeRefinement::Fraction(fraction));
        self.inset = Some(inset);
        self
    }

    /// Shorthand for `bottom: <percent>%` of the containing block.
    pub fn bottom_percent(self, percent: f32) -> Self {
        self.bottom_fraction(percent / 100.0)
    }

    pub fn bottom_neg(mut self, space: Space) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.bottom = Some(InsetEdgeRefinement::Px(SignedMetricRef::neg(
            MetricRef::space(space),
        )));
        self.inset = Some(inset);
        self
    }

    pub fn bottom_neg_px(mut self, px: impl Into<MetricRef>) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.bottom = Some(InsetEdgeRefinement::Px(SignedMetricRef::neg(px.into())));
        self.inset = Some(inset);
        self
    }

    pub fn left(mut self, space: Space) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.left = Some(InsetEdgeRefinement::Px(SignedMetricRef::pos(
            MetricRef::space(space),
        )));
        self.inset = Some(inset);
        self
    }

    pub fn left_px(mut self, px: Px) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.left = Some(InsetEdgeRefinement::Px(SignedMetricRef::pos(px.into())));
        self.inset = Some(inset);
        self
    }

    /// Shorthand for `left: 100%` of the containing block (percent sizing).
    pub fn left_full(mut self) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.left = Some(InsetEdgeRefinement::Fill);
        self.inset = Some(inset);
        self
    }

    /// Shorthand for `left: <fraction>` of the containing block.
    pub fn left_fraction(mut self, fraction: f32) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.left = Some(InsetEdgeRefinement::Fraction(fraction));
        self.inset = Some(inset);
        self
    }

    /// Shorthand for `left: <percent>%` of the containing block.
    pub fn left_percent(self, percent: f32) -> Self {
        self.left_fraction(percent / 100.0)
    }

    pub fn left_neg(mut self, space: Space) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.left = Some(InsetEdgeRefinement::Px(SignedMetricRef::neg(
            MetricRef::space(space),
        )));
        self.inset = Some(inset);
        self
    }

    pub fn left_neg_px(mut self, px: impl Into<MetricRef>) -> Self {
        self.ensure_position_for_inset();
        let mut inset = self.inset.unwrap_or_default();
        inset.left = Some(InsetEdgeRefinement::Px(SignedMetricRef::neg(px.into())));
        self.inset = Some(inset);
        self
    }

    pub fn m(mut self, space: Space) -> Self {
        let m = MarginEdgeRefinement::Px(SignedMetricRef::pos(MetricRef::space(space)));
        self.margin = Some(MarginRefinement {
            top: Some(m.clone()).into(),
            right: Some(m.clone()).into(),
            bottom: Some(m.clone()).into(),
            left: Some(m).into(),
        });
        self
    }

    pub fn m_px(mut self, px: Px) -> Self {
        let m = MarginEdgeRefinement::Px(SignedMetricRef::pos(px.into()));
        self.margin = Some(MarginRefinement {
            top: Some(m.clone()).into(),
            right: Some(m.clone()).into(),
            bottom: Some(m.clone()).into(),
            left: Some(m).into(),
        });
        self
    }

    pub fn m_neg(mut self, space: Space) -> Self {
        let m = MarginEdgeRefinement::Px(SignedMetricRef::neg(MetricRef::space(space)));
        self.margin = Some(MarginRefinement {
            top: Some(m.clone()).into(),
            right: Some(m.clone()).into(),
            bottom: Some(m.clone()).into(),
            left: Some(m).into(),
        });
        self
    }

    pub fn m_auto(mut self) -> Self {
        let a = MarginEdgeRefinement::Auto;
        self.margin = Some(MarginRefinement {
            top: Some(a.clone()).into(),
            right: Some(a.clone()).into(),
            bottom: Some(a.clone()).into(),
            left: Some(a).into(),
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

    /// Shorthand for `min-width: <px>`.
    pub fn min_w(mut self, width: impl Into<MetricRef>) -> Self {
        self.ensure_size_mut().min_width = Some(LengthRefinement::Px(width.into()));
        self
    }

    pub fn min_w_space(self, width: Space) -> Self {
        self.min_w(MetricRef::space(width))
    }

    /// Shorthand for `min-width: 100%` of the containing block.
    pub fn min_w_full(mut self) -> Self {
        self.ensure_size_mut().min_width = Some(LengthRefinement::Fill);
        self
    }

    /// Shorthand for `min-width: <fraction>` of the containing block.
    pub fn min_w_fraction(mut self, fraction: f32) -> Self {
        self.ensure_size_mut().min_width = Some(LengthRefinement::Fraction(fraction));
        self
    }

    /// Shorthand for `min-width: <percent>%` of the containing block.
    pub fn min_w_percent(self, percent: f32) -> Self {
        self.min_w_fraction(percent / 100.0)
    }

    /// Shorthand for `min-height: <px>`.
    pub fn min_h(mut self, height: impl Into<MetricRef>) -> Self {
        self.ensure_size_mut().min_height = Some(LengthRefinement::Px(height.into()));
        self
    }

    pub fn min_h_space(self, height: Space) -> Self {
        self.min_h(MetricRef::space(height))
    }

    /// Shorthand for `min-height: 100%` of the containing block.
    pub fn min_h_full(mut self) -> Self {
        self.ensure_size_mut().min_height = Some(LengthRefinement::Fill);
        self
    }

    /// Shorthand for `min-height: <fraction>` of the containing block.
    pub fn min_h_fraction(mut self, fraction: f32) -> Self {
        self.ensure_size_mut().min_height = Some(LengthRefinement::Fraction(fraction));
        self
    }

    /// Shorthand for `min-height: <percent>%` of the containing block.
    pub fn min_h_percent(self, percent: f32) -> Self {
        self.min_h_fraction(percent / 100.0)
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

    /// Shorthand for `width: <fraction>` of the containing block.
    ///
    /// Example: `w_fraction(0.5)` == 50% width.
    pub fn w_fraction(self, fraction: f32) -> Self {
        self.w(LengthRefinement::Fraction(fraction))
    }

    /// Shorthand for `width: <percent>%` of the containing block.
    pub fn w_percent(self, percent: f32) -> Self {
        self.w_fraction(percent / 100.0)
    }

    /// Shorthand for `height: 100%` of the containing block (percent sizing).
    pub fn h_full(self) -> Self {
        self.h(LengthRefinement::Fill)
    }

    /// Shorthand for `height: <fraction>` of the containing block.
    ///
    /// Example: `h_fraction(0.5)` == 50% height.
    pub fn h_fraction(self, fraction: f32) -> Self {
        self.h(LengthRefinement::Fraction(fraction))
    }

    /// Shorthand for `height: <percent>%` of the containing block.
    pub fn h_percent(self, percent: f32) -> Self {
        self.h_fraction(percent / 100.0)
    }

    pub fn size_full(self) -> Self {
        self.w_full().h_full()
    }

    /// Shorthand for `max-width: <px>`.
    pub fn max_w(mut self, width: impl Into<MetricRef>) -> Self {
        self.ensure_size_mut().max_width = Some(LengthRefinement::Px(width.into()));
        self
    }

    pub fn max_w_space(self, width: Space) -> Self {
        self.max_w(MetricRef::space(width))
    }

    /// Shorthand for `max-width: 100%` of the containing block.
    pub fn max_w_full(mut self) -> Self {
        self.ensure_size_mut().max_width = Some(LengthRefinement::Fill);
        self
    }

    /// Shorthand for `max-width: <fraction>` of the containing block.
    pub fn max_w_fraction(mut self, fraction: f32) -> Self {
        self.ensure_size_mut().max_width = Some(LengthRefinement::Fraction(fraction));
        self
    }

    /// Shorthand for `max-width: <percent>%` of the containing block.
    pub fn max_w_percent(self, percent: f32) -> Self {
        self.max_w_fraction(percent / 100.0)
    }

    /// Shorthand for `max-height: <px>`.
    pub fn max_h(mut self, height: impl Into<MetricRef>) -> Self {
        self.ensure_size_mut().max_height = Some(LengthRefinement::Px(height.into()));
        self
    }

    pub fn max_h_space(self, height: Space) -> Self {
        self.max_h(MetricRef::space(height))
    }

    /// Shorthand for `max-height: 100%` of the containing block.
    pub fn max_h_full(mut self) -> Self {
        self.ensure_size_mut().max_height = Some(LengthRefinement::Fill);
        self
    }

    /// Shorthand for `max-height: <fraction>` of the containing block.
    pub fn max_h_fraction(mut self, fraction: f32) -> Self {
        self.ensure_size_mut().max_height = Some(LengthRefinement::Fraction(fraction));
        self
    }

    /// Shorthand for `max-height: <percent>%` of the containing block.
    pub fn max_h_percent(self, percent: f32) -> Self {
        self.max_h_fraction(percent / 100.0)
    }

    pub fn basis(mut self, basis: LengthRefinement) -> Self {
        self.ensure_flex_item_mut().basis = Some(basis);
        self
    }

    /// Shorthand for `flex-basis: <fraction>` of the containing block size (main axis).
    pub fn basis_fraction(self, fraction: f32) -> Self {
        self.basis(LengthRefinement::Fraction(fraction))
    }

    /// Shorthand for `flex-basis: <percent>%` of the containing block size (main axis).
    pub fn basis_percent(self, percent: f32) -> Self {
        self.basis_fraction(percent / 100.0)
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

    pub fn order(mut self, order: i32) -> Self {
        self.ensure_flex_item_mut().order = Some(order);
        self
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
