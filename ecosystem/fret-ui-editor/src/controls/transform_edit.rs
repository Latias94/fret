//! Transform editor (position / rotation / scale) built from vec and numeric primitives.
//!
//! This is intentionally an ecosystem-level policy control:
//! - it composes `Vec3Edit` for the numeric editing surface,
//! - it optionally provides a "link scale" toggle,
//! - it can (best-effort) keep scale axes in sync while linked.

use std::sync::Arc;

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::{LayoutStyle, Length, SizeStyle};

use crate::controls::{AxisDragValueOutcome, VecEditAxis};

mod element;
mod model;
mod sections;
mod sync;

pub use model::{TransformEdit, TransformEditPresentations};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransformEditLayoutVariant {
    #[default]
    Column,
    Row,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformEditSection {
    Position,
    Rotation,
    Scale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformEditAxisOutcome {
    section: TransformEditSection,
    axis: VecEditAxis,
    outcome: AxisDragValueOutcome,
}

impl TransformEditAxisOutcome {
    pub(crate) fn new(
        section: TransformEditSection,
        axis: VecEditAxis,
        outcome: AxisDragValueOutcome,
    ) -> Self {
        Self {
            section,
            axis,
            outcome,
        }
    }

    pub fn section(self) -> TransformEditSection {
        self.section
    }

    pub fn axis(self) -> VecEditAxis {
        self.axis
    }

    pub fn outcome(self) -> AxisDragValueOutcome {
        self.outcome
    }
}

pub type OnTransformEditAxisOutcome =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, TransformEditAxisOutcome) + 'static>;

#[derive(Debug, Clone)]
pub struct TransformEditOptions {
    pub layout: LayoutStyle,
    pub variant: TransformEditLayoutVariant,
    pub section_gap: Px,
    pub show_link_scale_toggle: bool,
    pub position_prefix: Option<Arc<str>>,
    pub position_suffix: Option<Arc<str>>,
    pub rotation_prefix: Option<Arc<str>>,
    pub rotation_suffix: Option<Arc<str>>,
    pub scale_prefix: Option<Arc<str>>,
    pub scale_suffix: Option<Arc<str>>,
    /// If `None`, an internal per-element model is used.
    pub linked_scale: Option<Model<bool>>,
    pub default_linked_scale: bool,
    /// Explicit identity source for internal state (linked-scale model, uniform-scale memory).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when a helper function builds multiple transform edits from the same callsite and
    /// you need stable, per-instance state separation.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub link_test_id: Option<Arc<str>>,
}

impl Default for TransformEditOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            variant: TransformEditLayoutVariant::default(),
            section_gap: Px(6.0),
            show_link_scale_toggle: true,
            position_prefix: None,
            position_suffix: None,
            rotation_prefix: None,
            rotation_suffix: None,
            scale_prefix: None,
            scale_suffix: None,
            linked_scale: None,
            default_linked_scale: false,
            id_source: None,
            test_id: None,
            link_test_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TransformEditAxisOutcome, TransformEditSection};
    use crate::controls::VecEditAxis;
    use crate::primitives::EditSessionOutcome;

    #[test]
    fn transform_edit_axis_outcome_exposes_read_only_signals() {
        let outcome = TransformEditAxisOutcome::new(
            TransformEditSection::Scale,
            VecEditAxis::Y,
            EditSessionOutcome::Canceled,
        );

        assert_eq!(outcome.section(), TransformEditSection::Scale);
        assert_eq!(outcome.axis(), VecEditAxis::Y);
        assert_eq!(outcome.outcome(), EditSessionOutcome::Canceled);
    }
}
