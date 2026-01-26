//! Tree-local Material 3 overrides.
//!
//! Compose Material3 uses composition locals for theme-scoped overrides (`LocalContentColor`,
//! `LocalRippleConfiguration`, `LocalMotionScheme`, ...). Fret does not require a dedicated runtime
//! context system to model this outcome: `ElementContext::inherited_state_where` + `with_state`
//! provides a lightweight provider pattern.

#![allow(dead_code)]

use fret_core::Color;
use fret_ui::Theme;
use fret_ui::UiHost;
use fret_ui::elements::ElementContext;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialContentColor {
    /// Mask any inherited content color and fall back to component defaults.
    UseDefault,
    /// Override content color for the subtree.
    Custom(Color),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialRippleConfiguration {
    /// Mask any inherited ripple configuration and fall back to component defaults.
    UseDefault,
    /// Disable ripples for the subtree.
    Disabled,
    /// Override ripple appearance for the subtree.
    Custom {
        color: Option<Color>,
        base_opacity: Option<f32>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialMotionScheme {
    Standard,
    Expressive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialDesignVariant {
    Standard,
    Expressive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialMotionSchemeOverride {
    /// Mask any inherited scheme and fall back to component defaults.
    UseDefault,
    /// Override the motion scheme for the subtree.
    Custom(MaterialMotionScheme),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialDesignVariantOverride {
    /// Mask any inherited variant and fall back to component defaults.
    UseDefault,
    /// Override the design variant for the subtree.
    Custom(MaterialDesignVariant),
}

#[derive(Debug, Default, Clone, Copy)]
struct MaterialContextState {
    content_color: Option<MaterialContentColor>,
    ripple: Option<MaterialRippleConfiguration>,
    motion_scheme: Option<MaterialMotionSchemeOverride>,
    design_variant: Option<MaterialDesignVariantOverride>,
}

pub fn inherited_content_color_policy<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<MaterialContentColor> {
    cx.inherited_state_where::<MaterialContextState>(|st| st.content_color.is_some())
        .and_then(|st| st.content_color)
}

pub fn inherited_content_color<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<Color> {
    match inherited_content_color_policy(cx)? {
        MaterialContentColor::UseDefault => None,
        MaterialContentColor::Custom(color) => Some(color),
    }
}

pub fn inherited_ripple_configuration<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<MaterialRippleConfiguration> {
    cx.inherited_state_where::<MaterialContextState>(|st| st.ripple.is_some())
        .and_then(|st| st.ripple)
}

pub fn inherited_motion_scheme_override<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<MaterialMotionSchemeOverride> {
    cx.inherited_state_where::<MaterialContextState>(|st| st.motion_scheme.is_some())
        .and_then(|st| st.motion_scheme)
}

pub fn inherited_design_variant_override<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<MaterialDesignVariantOverride> {
    cx.inherited_state_where::<MaterialContextState>(|st| st.design_variant.is_some())
        .and_then(|st| st.design_variant)
}

pub fn resolved_motion_scheme<H: UiHost>(
    cx: &ElementContext<'_, H>,
    fallback: MaterialMotionScheme,
) -> MaterialMotionScheme {
    match inherited_motion_scheme_override(cx) {
        Some(MaterialMotionSchemeOverride::Custom(scheme)) => scheme,
        Some(MaterialMotionSchemeOverride::UseDefault) | None => fallback,
    }
}

pub fn theme_default_design_variant(theme: &Theme) -> MaterialDesignVariant {
    // Fret-owned token that tracks the top-level Material "expressive" selection. This lets
    // Material3 components switch to expressive component tokens without requiring a context
    // wrapper at every callsite.
    let expressive = theme
        .number_by_key("md.sys.fret.material.is-expressive")
        .unwrap_or(0.0)
        > 0.5;

    if expressive {
        MaterialDesignVariant::Expressive
    } else {
        MaterialDesignVariant::Standard
    }
}

pub fn resolved_design_variant<H: UiHost>(
    cx: &ElementContext<'_, H>,
    fallback: MaterialDesignVariant,
) -> MaterialDesignVariant {
    match inherited_design_variant_override(cx) {
        Some(MaterialDesignVariantOverride::Custom(variant)) => variant,
        Some(MaterialDesignVariantOverride::UseDefault) | None => fallback,
    }
}

#[track_caller]
pub fn with_material_content_color<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    with_material_content_color_policy(cx, MaterialContentColor::Custom(color), f)
}

#[track_caller]
pub fn with_default_material_content_color<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    with_material_content_color_policy(cx, MaterialContentColor::UseDefault, f)
}

#[track_caller]
pub fn with_material_content_color_policy<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    policy: MaterialContentColor,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(MaterialContextState::default, |st| {
        let prev = st.content_color;
        st.content_color = Some(policy);
        prev
    });
    let out = f(cx);
    cx.with_state(MaterialContextState::default, |st| {
        st.content_color = prev;
    });
    out
}

#[track_caller]
pub fn with_material_ripple_configuration<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    config: MaterialRippleConfiguration,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(MaterialContextState::default, |st| {
        let prev = st.ripple;
        st.ripple = Some(config);
        prev
    });
    let out = f(cx);
    cx.with_state(MaterialContextState::default, |st| {
        st.ripple = prev;
    });
    out
}

#[track_caller]
pub fn with_material_motion_scheme<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    scheme: MaterialMotionScheme,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    with_material_motion_scheme_override(cx, MaterialMotionSchemeOverride::Custom(scheme), f)
}

#[track_caller]
pub fn with_default_material_motion_scheme<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    with_material_motion_scheme_override(cx, MaterialMotionSchemeOverride::UseDefault, f)
}

#[track_caller]
pub fn with_material_motion_scheme_override<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    override_policy: MaterialMotionSchemeOverride,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(MaterialContextState::default, |st| {
        let prev = st.motion_scheme;
        st.motion_scheme = Some(override_policy);
        prev
    });
    let out = f(cx);
    cx.with_state(MaterialContextState::default, |st| {
        st.motion_scheme = prev;
    });
    out
}

#[track_caller]
pub fn with_material_design_variant<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    variant: MaterialDesignVariant,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    with_material_design_variant_override(cx, MaterialDesignVariantOverride::Custom(variant), f)
}

#[track_caller]
pub fn with_default_material_design_variant<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    with_material_design_variant_override(cx, MaterialDesignVariantOverride::UseDefault, f)
}

#[track_caller]
pub fn with_material_design_variant_override<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    override_policy: MaterialDesignVariantOverride,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(MaterialContextState::default, |st| {
        let prev = st.design_variant;
        st.design_variant = Some(override_policy);
        prev
    });
    let out = f(cx);
    cx.with_state(MaterialContextState::default, |st| {
        st.design_variant = prev;
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::elements::with_element_cx;

    fn bounds() -> Rect {
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)))
    }

    #[test]
    fn content_color_inherits_masks_and_restores() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let red = Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let blue = Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        };

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "m3-context-content-color",
            |cx| {
                assert_eq!(inherited_content_color_policy(cx), None);
                assert_eq!(inherited_content_color(cx), None);

                with_material_content_color(cx, red, |cx| {
                    assert_eq!(
                        inherited_content_color_policy(cx),
                        Some(MaterialContentColor::Custom(red))
                    );
                    assert_eq!(inherited_content_color(cx), Some(red));

                    with_default_material_content_color(cx, |cx| {
                        assert_eq!(
                            inherited_content_color_policy(cx),
                            Some(MaterialContentColor::UseDefault)
                        );
                        assert_eq!(inherited_content_color(cx), None);
                    });

                    assert_eq!(inherited_content_color(cx), Some(red));

                    with_material_content_color(cx, blue, |cx| {
                        assert_eq!(inherited_content_color(cx), Some(blue));
                    });

                    assert_eq!(inherited_content_color(cx), Some(red));
                });

                assert_eq!(inherited_content_color(cx), None);
            },
        );
    }

    #[test]
    fn ripple_configuration_inherits_masks_and_restores() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(&mut app, window, bounds(), "m3-context-ripple", |cx| {
            assert_eq!(inherited_ripple_configuration(cx), None);

            with_material_ripple_configuration(cx, MaterialRippleConfiguration::Disabled, |cx| {
                assert_eq!(
                    inherited_ripple_configuration(cx),
                    Some(MaterialRippleConfiguration::Disabled)
                );

                with_material_ripple_configuration(
                    cx,
                    MaterialRippleConfiguration::UseDefault,
                    |cx| {
                        assert_eq!(
                            inherited_ripple_configuration(cx),
                            Some(MaterialRippleConfiguration::UseDefault)
                        );
                    },
                );

                assert_eq!(
                    inherited_ripple_configuration(cx),
                    Some(MaterialRippleConfiguration::Disabled)
                );
            });

            assert_eq!(inherited_ripple_configuration(cx), None);
        });
    }

    #[test]
    fn motion_scheme_inherits_masks_and_restores() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "m3-context-motion-scheme",
            |cx| {
                assert_eq!(inherited_motion_scheme_override(cx), None);
                assert_eq!(
                    resolved_motion_scheme(cx, MaterialMotionScheme::Standard),
                    MaterialMotionScheme::Standard
                );

                with_material_motion_scheme(cx, MaterialMotionScheme::Expressive, |cx| {
                    assert_eq!(
                        inherited_motion_scheme_override(cx),
                        Some(MaterialMotionSchemeOverride::Custom(
                            MaterialMotionScheme::Expressive
                        ))
                    );
                    assert_eq!(
                        resolved_motion_scheme(cx, MaterialMotionScheme::Standard),
                        MaterialMotionScheme::Expressive
                    );

                    with_default_material_motion_scheme(cx, |cx| {
                        assert_eq!(
                            inherited_motion_scheme_override(cx),
                            Some(MaterialMotionSchemeOverride::UseDefault)
                        );
                        assert_eq!(
                            resolved_motion_scheme(cx, MaterialMotionScheme::Expressive),
                            MaterialMotionScheme::Expressive
                        );
                    });

                    assert_eq!(
                        resolved_motion_scheme(cx, MaterialMotionScheme::Standard),
                        MaterialMotionScheme::Expressive
                    );
                });

                assert_eq!(inherited_motion_scheme_override(cx), None);
            },
        );
    }

    #[test]
    fn design_variant_inherits_masks_and_restores() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "m3-context-design-variant",
            |cx| {
                assert_eq!(inherited_design_variant_override(cx), None);
                assert_eq!(
                    resolved_design_variant(cx, MaterialDesignVariant::Standard),
                    MaterialDesignVariant::Standard
                );

                with_material_design_variant(cx, MaterialDesignVariant::Expressive, |cx| {
                    assert_eq!(
                        inherited_design_variant_override(cx),
                        Some(MaterialDesignVariantOverride::Custom(
                            MaterialDesignVariant::Expressive
                        ))
                    );
                    assert_eq!(
                        resolved_design_variant(cx, MaterialDesignVariant::Standard),
                        MaterialDesignVariant::Expressive
                    );

                    with_default_material_design_variant(cx, |cx| {
                        assert_eq!(
                            inherited_design_variant_override(cx),
                            Some(MaterialDesignVariantOverride::UseDefault)
                        );
                        assert_eq!(
                            resolved_design_variant(cx, MaterialDesignVariant::Expressive),
                            MaterialDesignVariant::Expressive
                        );
                    });

                    assert_eq!(
                        resolved_design_variant(cx, MaterialDesignVariant::Standard),
                        MaterialDesignVariant::Expressive
                    );
                });

                assert_eq!(inherited_design_variant_override(cx), None);
                assert_eq!(
                    resolved_design_variant(cx, MaterialDesignVariant::Standard),
                    MaterialDesignVariant::Standard
                );
            },
        );
    }
}
