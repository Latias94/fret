use fret_core::{Corners, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

const SYS_FOCUS_INDICATOR_THICKNESS: &str = "md.sys.state.focus-indicator.thickness";
const SYS_FOCUS_INDICATOR_OUTER_OFFSET: &str = "md.sys.state.focus-indicator.outer-offset";
const DEFAULT_FOCUS_INDICATOR_THICKNESS: Px = Px(3.0);
const DEFAULT_FOCUS_INDICATOR_OUTER_OFFSET: Px = Px(2.0);

pub(crate) fn material_focus_indicator_color(
    theme: &Theme,
    component_prefix: &str,
) -> fret_core::Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        &format!("{component_prefix}.focus.indicator.color"),
        "md.sys.color.secondary",
    )
}

pub(crate) fn material_focus_indicator_thickness(theme: &Theme, component_prefix: &str) -> Px {
    let component_key = format!("{component_prefix}.focus.indicator.thickness");
    MaterialTokenResolver::new(theme).metric_chain(
        &[component_key.as_str(), SYS_FOCUS_INDICATOR_THICKNESS],
        DEFAULT_FOCUS_INDICATOR_THICKNESS,
    )
}

pub(crate) fn material_focus_indicator_outline_offset(theme: &Theme, component_prefix: &str) -> Px {
    let outline_key = format!("{component_prefix}.focus.indicator.outline.offset");
    let offset_key = format!("{component_prefix}.focus.indicator.offset");
    MaterialTokenResolver::new(theme).metric_chain(
        &[
            outline_key.as_str(),
            offset_key.as_str(),
            SYS_FOCUS_INDICATOR_OUTER_OFFSET,
        ],
        DEFAULT_FOCUS_INDICATOR_OUTER_OFFSET,
    )
}

pub fn material_focus_ring_for_component(
    theme: &Theme,
    component_prefix: &str,
    corner_radii: Corners,
) -> fret_ui::element::RingStyle {
    let mut color = material_focus_indicator_color(theme, component_prefix);
    color.a = 1.0;
    let thickness = material_focus_indicator_thickness(theme, component_prefix);
    let outline_offset = material_focus_indicator_outline_offset(theme, component_prefix);

    let (placement, offset) = if outline_offset.0 < 0.0 {
        (
            fret_ui::element::RingPlacement::Inset,
            Px(outline_offset.0.abs()),
        )
    } else {
        (fret_ui::element::RingPlacement::Outset, outline_offset)
    };

    fret_ui::element::RingStyle {
        placement,
        width: thickness,
        offset,
        color,
        offset_color: None,
        corner_radii,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_ui::{Theme, theme::ThemeConfig};

    fn theme_with_patch(patch: ThemeConfig) -> (App, Theme) {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    #[test]
    fn focus_indicator_metrics_prefer_component_over_system() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert("md.comp.test.focus.indicator.thickness".to_string(), 5.0);
        patch
            .metrics
            .insert(SYS_FOCUS_INDICATOR_THICKNESS.to_string(), 4.0);
        patch.metrics.insert(
            "md.comp.test.focus.indicator.outline.offset".to_string(),
            -1.0,
        );
        patch
            .metrics
            .insert(SYS_FOCUS_INDICATOR_OUTER_OFFSET.to_string(), 6.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            material_focus_indicator_thickness(&theme, "md.comp.test"),
            Px(5.0)
        );
        assert_eq!(
            material_focus_indicator_outline_offset(&theme, "md.comp.test"),
            Px(-1.0)
        );
    }

    #[test]
    fn focus_indicator_metrics_fall_back_to_system_then_defaults() {
        let mut patch = ThemeConfig::default();
        patch
            .metrics
            .insert(SYS_FOCUS_INDICATOR_THICKNESS.to_string(), 4.0);
        patch
            .metrics
            .insert(SYS_FOCUS_INDICATOR_OUTER_OFFSET.to_string(), 6.0);
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            material_focus_indicator_thickness(&theme, "md.comp.missing"),
            Px(4.0)
        );
        assert_eq!(
            material_focus_indicator_outline_offset(&theme, "md.comp.missing"),
            Px(6.0)
        );

        let app = App::new();
        let theme = Theme::global(&app);
        assert_eq!(
            material_focus_indicator_thickness(theme, "md.comp.missing"),
            DEFAULT_FOCUS_INDICATOR_THICKNESS
        );
        assert_eq!(
            material_focus_indicator_outline_offset(theme, "md.comp.missing"),
            DEFAULT_FOCUS_INDICATOR_OUTER_OFFSET
        );
    }
}
