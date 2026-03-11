//! Editor density policy resolved from `editor.density.*` tokens.
//!
//! This is intentionally small and stringly-typed: editor components should share the same
//! "hand feel" defaults without coupling to a specific theme pack.

use fret_core::Px;
use fret_ui::Theme;

use super::EditorTokenKeys;

#[derive(Debug, Clone, Copy)]
pub struct EditorDensity {
    pub row_height: Px,
    pub padding_x: Px,
    pub padding_y: Px,
    pub hit_thickness: Px,
    pub icon_size: Px,
}

impl Default for EditorDensity {
    fn default() -> Self {
        Self {
            row_height: Px(20.0),
            padding_x: Px(6.0),
            padding_y: Px(2.0),
            hit_thickness: Px(16.0),
            icon_size: Px(14.0),
        }
    }
}

impl EditorDensity {
    pub fn affordance_extent(self) -> Px {
        Px(self.row_height.0.max(self.hit_thickness.0))
    }

    pub fn resolve(theme: &Theme) -> Self {
        let mut out = Self::default();

        out.row_height = theme
            .metric_by_key(EditorTokenKeys::DENSITY_ROW_HEIGHT)
            .or_else(|| theme.metric_by_key("component.list.row_height"))
            .unwrap_or(out.row_height);
        out.padding_x = theme
            .metric_by_key(EditorTokenKeys::DENSITY_PADDING_X)
            .unwrap_or(out.padding_x);
        out.padding_y = theme
            .metric_by_key(EditorTokenKeys::DENSITY_PADDING_Y)
            .unwrap_or(out.padding_y);
        out.hit_thickness = theme
            .metric_by_key(EditorTokenKeys::DENSITY_HIT_THICKNESS)
            .unwrap_or(out.hit_thickness);
        out.icon_size = theme
            .metric_by_key(EditorTokenKeys::DENSITY_ICON_SIZE)
            .unwrap_or(out.icon_size);

        out.row_height = Px(out.row_height.0.max(0.0));
        out.padding_x = Px(out.padding_x.0.max(0.0));
        out.padding_y = Px(out.padding_y.0.max(0.0));
        out.hit_thickness = Px(out.hit_thickness.0.max(0.0));
        out.icon_size = Px(out.icon_size.0.max(0.0));

        out
    }
}

#[cfg(test)]
mod tests {
    use fret_core::Px;

    use super::EditorDensity;

    #[test]
    fn affordance_extent_prefers_row_height_when_visual_hit_is_smaller() {
        let density = EditorDensity {
            row_height: Px(24.0),
            hit_thickness: Px(20.0),
            ..Default::default()
        };

        assert_eq!(density.affordance_extent(), Px(24.0));
    }

    #[test]
    fn affordance_extent_preserves_larger_hit_targets() {
        let density = EditorDensity {
            row_height: Px(22.0),
            hit_thickness: Px(28.0),
            ..Default::default()
        };

        assert_eq!(density.affordance_extent(), Px(28.0));
    }
}
