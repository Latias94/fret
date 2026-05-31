//! Shared default matrices for Material 3 FAB token fallbacks.

use fret_core::Px;

use crate::fab::FabSize;

pub(crate) fn disabled_container_opacity() -> f32 {
    0.12
}

pub(crate) fn disabled_content_opacity() -> f32 {
    0.38
}

pub(crate) fn disabled_container_elevation() -> Px {
    Px(0.0)
}

pub(crate) fn icon_container_size(size: FabSize) -> Px {
    match size {
        FabSize::Small => Px(40.0),
        FabSize::Regular => Px(56.0),
        FabSize::Medium => Px(80.0),
        FabSize::Large => Px(96.0),
    }
}

pub(crate) fn icon_size(size: FabSize) -> Px {
    match size {
        FabSize::Small | FabSize::Regular => Px(24.0),
        FabSize::Medium => Px(28.0),
        FabSize::Large => Px(36.0),
    }
}

pub(crate) fn icon_container_shape_system_key(size: FabSize) -> &'static str {
    match size {
        FabSize::Small => "md.sys.shape.corner.medium",
        FabSize::Regular => "md.sys.shape.corner.large",
        FabSize::Medium => "md.sys.shape.corner.large-increased",
        FabSize::Large => "md.sys.shape.corner.extra-large",
    }
}

pub(crate) fn icon_container_shape_radius(size: FabSize) -> Px {
    match size {
        FabSize::Small => Px(12.0),
        FabSize::Regular => Px(16.0),
        FabSize::Medium => Px(20.0),
        FabSize::Large => Px(28.0),
    }
}

pub(crate) fn extended_container_height(size: FabSize) -> Px {
    match size {
        FabSize::Small | FabSize::Regular => Px(56.0),
        FabSize::Medium => Px(80.0),
        FabSize::Large => Px(96.0),
    }
}

pub(crate) fn extended_min_width(size: FabSize, resolved_height: Px) -> Px {
    match size {
        FabSize::Regular => Px(80.0),
        FabSize::Small | FabSize::Medium | FabSize::Large => resolved_height,
    }
}

pub(crate) fn extended_icon_size(size: FabSize) -> Px {
    icon_size(size)
}

pub(crate) fn extended_container_shape_system_key(size: FabSize) -> &'static str {
    match size {
        FabSize::Small | FabSize::Regular => "md.sys.shape.corner.large",
        FabSize::Medium => "md.sys.shape.corner.large-increased",
        FabSize::Large => "md.sys.shape.corner.extra-large",
    }
}

pub(crate) fn extended_container_shape_radius(size: FabSize) -> Px {
    match size {
        FabSize::Small | FabSize::Regular => Px(16.0),
        FabSize::Medium => Px(20.0),
        FabSize::Large => Px(28.0),
    }
}

pub(crate) fn extended_leading_space(size: FabSize) -> Px {
    match size {
        FabSize::Small | FabSize::Regular => Px(16.0),
        FabSize::Medium => Px(26.0),
        FabSize::Large => Px(28.0),
    }
}

pub(crate) fn extended_trailing_space(size: FabSize) -> Px {
    match size {
        FabSize::Regular => Px(20.0),
        FabSize::Small => Px(16.0),
        FabSize::Medium => Px(26.0),
        FabSize::Large => Px(28.0),
    }
}

pub(crate) fn extended_icon_label_space(size: FabSize) -> Px {
    match size {
        FabSize::Small => Px(8.0),
        FabSize::Regular | FabSize::Medium => Px(12.0),
        FabSize::Large => Px(16.0),
    }
}

pub(crate) fn extended_label_text_source(size: FabSize) -> &'static str {
    match size {
        FabSize::Small => "md.sys.typescale.title-medium",
        FabSize::Regular => "md.comp.extended-fab.label-text",
        FabSize::Medium => "md.sys.typescale.title-large",
        FabSize::Large => "md.sys.typescale.headline-small",
    }
}

pub(crate) fn hovered_state_layer_opacity() -> f32 {
    0.08
}

pub(crate) fn focused_state_layer_opacity() -> f32 {
    0.1
}

pub(crate) fn pressed_state_layer_opacity() -> f32 {
    0.1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_fab_size_defaults_match_material_matrix() {
        assert_eq!(icon_container_size(FabSize::Small), Px(40.0));
        assert_eq!(icon_container_size(FabSize::Regular), Px(56.0));
        assert_eq!(icon_container_size(FabSize::Medium), Px(80.0));
        assert_eq!(icon_container_size(FabSize::Large), Px(96.0));

        assert_eq!(icon_size(FabSize::Small), Px(24.0));
        assert_eq!(icon_size(FabSize::Regular), Px(24.0));
        assert_eq!(icon_size(FabSize::Medium), Px(28.0));
        assert_eq!(icon_size(FabSize::Large), Px(36.0));
    }

    #[test]
    fn shape_defaults_keep_size_specific_system_fallbacks() {
        assert_eq!(
            icon_container_shape_system_key(FabSize::Small),
            "md.sys.shape.corner.medium"
        );
        assert_eq!(icon_container_shape_radius(FabSize::Small), Px(12.0));
        assert_eq!(
            extended_container_shape_system_key(FabSize::Medium),
            "md.sys.shape.corner.large-increased"
        );
        assert_eq!(extended_container_shape_radius(FabSize::Large), Px(28.0));
    }

    #[test]
    fn extended_fab_spacing_defaults_match_material_matrix() {
        assert_eq!(extended_container_height(FabSize::Regular), Px(56.0));
        assert_eq!(extended_min_width(FabSize::Regular, Px(72.0)), Px(80.0));
        assert_eq!(extended_min_width(FabSize::Small, Px(56.0)), Px(56.0));
        assert_eq!(extended_leading_space(FabSize::Medium), Px(26.0));
        assert_eq!(extended_trailing_space(FabSize::Regular), Px(20.0));
        assert_eq!(extended_icon_label_space(FabSize::Large), Px(16.0));
    }

    #[test]
    fn opacity_defaults_match_material_state_matrix() {
        assert_eq!(disabled_container_opacity(), 0.12);
        assert_eq!(disabled_content_opacity(), 0.38);
        assert_eq!(hovered_state_layer_opacity(), 0.08);
        assert_eq!(focused_state_layer_opacity(), 0.1);
        assert_eq!(pressed_state_layer_opacity(), 0.1);
    }
}
