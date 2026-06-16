use std::sync::Arc;

use fret_core::Px;
use fret_ui::element::{LayoutStyle, Length, SizeStyle};

use super::layout::PropertyRowLayoutVariant;

#[derive(Debug, Clone)]
pub struct PropertyRowOptions {
    pub layout: LayoutStyle,
    pub label_width: Option<Px>,
    pub gap: Option<Px>,
    pub trailing_gap: Option<Px>,
    pub value_max_width: Option<Px>,
    pub status_slot_width: Option<Px>,
    pub reset_slot_width: Option<Px>,
    pub variant: PropertyRowLayoutVariant,
    pub auto_stack_below: Option<Px>,
    /// Explicit identity source for internal policy state (auto layout heuristics).
    ///
    /// This is the editor-composite equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when building rows in a loop where the callsite is not unique per row.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for PropertyRowOptions {
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
            label_width: None,
            gap: None,
            trailing_gap: None,
            value_max_width: None,
            status_slot_width: None,
            reset_slot_width: None,
            variant: PropertyRowLayoutVariant::Row,
            auto_stack_below: None,
            id_source: None,
            test_id: None,
        }
    }
}

impl PropertyRowOptions {
    pub(crate) fn with_grid_defaults(mut self, defaults: &Self) -> Self {
        if self.label_width.is_none() {
            self.label_width = defaults.label_width;
        }
        if self.gap.is_none() {
            self.gap = defaults.gap;
        }
        if self.trailing_gap.is_none() {
            self.trailing_gap = defaults.trailing_gap;
        }
        if self.value_max_width.is_none() {
            self.value_max_width = defaults.value_max_width;
        }
        if self.status_slot_width.is_none() {
            self.status_slot_width = defaults.status_slot_width;
        }
        if self.reset_slot_width.is_none() {
            self.reset_slot_width = defaults.reset_slot_width;
        }
        if self.auto_stack_below.is_none() {
            self.auto_stack_below = defaults.auto_stack_below;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{PropertyRowLayoutVariant, PropertyRowOptions};
    use fret_core::Px;

    #[test]
    fn with_grid_defaults_preserves_explicit_variant_and_fills_missing_values() {
        let defaults = PropertyRowOptions {
            label_width: Some(Px(104.0)),
            gap: Some(Px(8.0)),
            trailing_gap: Some(Px(4.0)),
            value_max_width: Some(Px(240.0)),
            status_slot_width: Some(Px(12.0)),
            reset_slot_width: Some(Px(12.0)),
            auto_stack_below: Some(Px(320.0)),
            ..Default::default()
        };
        let row = PropertyRowOptions {
            variant: PropertyRowLayoutVariant::Auto,
            ..Default::default()
        };

        let merged = row.with_grid_defaults(&defaults);

        assert_eq!(merged.variant, PropertyRowLayoutVariant::Auto);
        assert_eq!(merged.label_width, defaults.label_width);
        assert_eq!(merged.gap, defaults.gap);
        assert_eq!(merged.trailing_gap, defaults.trailing_gap);
        assert_eq!(merged.value_max_width, defaults.value_max_width);
        assert_eq!(merged.status_slot_width, defaults.status_slot_width);
        assert_eq!(merged.reset_slot_width, defaults.reset_slot_width);
        assert_eq!(merged.auto_stack_below, defaults.auto_stack_below);
    }
}
