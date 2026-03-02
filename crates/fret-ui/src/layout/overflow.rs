use crate::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};
use fret_core::Size;

/// Layout-time overflow budget hints inherited down the layout subtree.
///
/// This is intentionally minimal: it only affects how wrapper widgets construct probe/measurement
/// constraints for their children. It does **not** change paint-time clipping behavior.
///
/// The initial use case is scroll extents (DOM/GPUI parity): scroll roots can install a context
/// indicating the scroll axis should be treated as `MaxContent` for child probes so wrapper-heavy
/// trees can still observe overflow in post-layout geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutOverflowContext {
    /// Override the available-space budget used when building probe constraints for children.
    ///
    /// When `None`, probe constraints default to `AvailableSpace::Definite(size.{width,height})`.
    pub probe_available_override: LayoutSize<Option<AvailableSpace>>,
}

impl LayoutOverflowContext {
    pub const fn new(probe_available_override: LayoutSize<Option<AvailableSpace>>) -> Self {
        Self {
            probe_available_override,
        }
    }

    pub const fn default_for_layout() -> Self {
        Self {
            probe_available_override: LayoutSize::new(None, None),
        }
    }

    pub fn probe_constraints_for_size(self, size: Size) -> LayoutConstraints {
        let mut available = LayoutSize::new(
            AvailableSpace::Definite(size.width),
            AvailableSpace::Definite(size.height),
        );

        if let Some(w) = self.probe_available_override.width {
            available.width = w;
        }
        if let Some(h) = self.probe_available_override.height {
            available.height = h;
        }

        LayoutConstraints::new(LayoutSize::new(None, None), available)
    }
}

impl Default for LayoutOverflowContext {
    fn default() -> Self {
        Self::default_for_layout()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Px;

    #[test]
    fn probe_constraints_default_to_definite() {
        let ctx = LayoutOverflowContext::default();
        let c = ctx.probe_constraints_for_size(Size::new(Px(10.0), Px(20.0)));
        assert_eq!(c.available.width, AvailableSpace::Definite(Px(10.0)));
        assert_eq!(c.available.height, AvailableSpace::Definite(Px(20.0)));
    }

    #[test]
    fn probe_constraints_apply_overrides_per_axis() {
        let ctx = LayoutOverflowContext {
            probe_available_override: LayoutSize::new(Some(AvailableSpace::MaxContent), None),
        };
        let c = ctx.probe_constraints_for_size(Size::new(Px(10.0), Px(20.0)));
        assert_eq!(c.available.width, AvailableSpace::MaxContent);
        assert_eq!(c.available.height, AvailableSpace::Definite(Px(20.0)));
    }
}
