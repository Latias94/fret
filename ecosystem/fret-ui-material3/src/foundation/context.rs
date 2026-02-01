//! Tree-local Material 3 overrides.
//!
//! Compose Material3 uses composition locals for theme-scoped overrides (`LocalContentColor`,
//! `LocalRippleConfiguration`, `LocalMotionScheme`, ...). Fret does not require a dedicated runtime
//! context system to model this outcome: `ElementContext::inherited_state_where` + `with_state`
//! provides a lightweight provider pattern.

#![allow(dead_code)]

use fret_core::{Color, LayoutDirection, Px, TextStyle};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialLayoutDirectionOverride {
    /// Mask any inherited layout direction and fall back to component defaults.
    UseDefault,
    /// Override the layout direction for the subtree.
    Custom(LayoutDirection),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MaterialTextStyleOverride {
    /// Mask any inherited style and fall back to component defaults.
    UseDefault,
    /// Override the default text style for the subtree (Compose `ProvideTextStyle`-like).
    Custom(TextStyle),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialIconSizeOverride {
    /// Mask any inherited size and fall back to component defaults.
    UseDefault,
    /// Override the default icon size for the subtree.
    Custom(Px),
}

#[derive(Debug, Default, Clone)]
struct MaterialContextState {
    content_color: Option<MaterialContentColor>,
    ripple: Option<MaterialRippleConfiguration>,
    motion_scheme: Option<MaterialMotionSchemeOverride>,
    design_variant: Option<MaterialDesignVariantOverride>,
    layout_direction: Option<MaterialLayoutDirectionOverride>,
    text_style: Option<MaterialTextStyleOverride>,
    icon_size: Option<MaterialIconSizeOverride>,
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

pub fn inherited_layout_direction_override<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<MaterialLayoutDirectionOverride> {
    cx.inherited_state_where::<MaterialContextState>(|st| st.layout_direction.is_some())
        .and_then(|st| st.layout_direction)
}

pub fn inherited_text_style_override<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<MaterialTextStyleOverride> {
    cx.inherited_state_where::<MaterialContextState>(|st| st.text_style.is_some())
        .and_then(|st| st.text_style.clone())
}

pub fn inherited_text_style<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<TextStyle> {
    match inherited_text_style_override(cx)? {
        MaterialTextStyleOverride::UseDefault => None,
        MaterialTextStyleOverride::Custom(style) => Some(style),
    }
}

pub fn inherited_icon_size_override<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<MaterialIconSizeOverride> {
    cx.inherited_state_where::<MaterialContextState>(|st| st.icon_size.is_some())
        .and_then(|st| st.icon_size)
}

pub fn inherited_icon_size<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<Px> {
    match inherited_icon_size_override(cx)? {
        MaterialIconSizeOverride::UseDefault => None,
        MaterialIconSizeOverride::Custom(size) => Some(size),
    }
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

pub fn theme_default_motion_scheme(theme: &Theme) -> MaterialMotionScheme {
    match theme_default_design_variant(theme) {
        MaterialDesignVariant::Standard => MaterialMotionScheme::Standard,
        MaterialDesignVariant::Expressive => MaterialMotionScheme::Expressive,
    }
}

pub fn theme_default_layout_direction(theme: &Theme) -> LayoutDirection {
    // Fret-owned token for UI layout direction.
    //
    // This is intentionally not a Material Web token. It represents app-level directionality
    // that can be applied across design systems, but is currently plumbed via Material context
    // to keep the initial change local.
    let rtl = theme
        .number_by_key("md.sys.fret.layout.is-rtl")
        .unwrap_or(0.0)
        > 0.5;

    if rtl {
        LayoutDirection::Rtl
    } else {
        LayoutDirection::Ltr
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

pub fn resolved_layout_direction<H: UiHost>(
    cx: &ElementContext<'_, H>,
    fallback: LayoutDirection,
) -> LayoutDirection {
    match inherited_layout_direction_override(cx) {
        Some(MaterialLayoutDirectionOverride::Custom(direction)) => direction,
        Some(MaterialLayoutDirectionOverride::UseDefault) | None => fallback,
    }
}

pub fn resolved_text_style<H: UiHost>(
    cx: &ElementContext<'_, H>,
    fallback: Option<TextStyle>,
) -> Option<TextStyle> {
    inherited_text_style(cx).or(fallback)
}

pub fn resolved_icon_size<H: UiHost>(cx: &ElementContext<'_, H>, fallback: Px) -> Px {
    inherited_icon_size(cx).unwrap_or(fallback)
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
pub fn with_material_layout_direction<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    direction: LayoutDirection,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    with_material_layout_direction_override(
        cx,
        MaterialLayoutDirectionOverride::Custom(direction),
        f,
    )
}

#[track_caller]
pub fn with_default_material_layout_direction<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    with_material_layout_direction_override(cx, MaterialLayoutDirectionOverride::UseDefault, f)
}

#[track_caller]
pub fn with_material_layout_direction_override<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    override_policy: MaterialLayoutDirectionOverride,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(MaterialContextState::default, |st| {
        let prev = st.layout_direction;
        st.layout_direction = Some(override_policy);
        prev
    });
    let out = f(cx);
    cx.with_state(MaterialContextState::default, |st| {
        st.layout_direction = prev;
    });
    out
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

#[track_caller]
pub fn with_material_text_style<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    style: TextStyle,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    with_material_text_style_override(cx, MaterialTextStyleOverride::Custom(style), f)
}

#[track_caller]
pub fn with_default_material_text_style<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    with_material_text_style_override(cx, MaterialTextStyleOverride::UseDefault, f)
}

#[track_caller]
pub fn with_material_text_style_override<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    override_policy: MaterialTextStyleOverride,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(MaterialContextState::default, |st| {
        let prev = st.text_style.clone();
        st.text_style = Some(override_policy);
        prev
    });
    let out = f(cx);
    cx.with_state(MaterialContextState::default, |st| {
        st.text_style = prev;
    });
    out
}

#[track_caller]
pub fn with_material_icon_size<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    size: Px,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    with_material_icon_size_override(cx, MaterialIconSizeOverride::Custom(size), f)
}

#[track_caller]
pub fn with_default_material_icon_size<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    with_material_icon_size_override(cx, MaterialIconSizeOverride::UseDefault, f)
}

#[track_caller]
pub fn with_material_icon_size_override<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    override_policy: MaterialIconSizeOverride,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(MaterialContextState::default, |st| {
        let prev = st.icon_size;
        st.icon_size = Some(override_policy);
        prev
    });
    let out = f(cx);
    cx.with_state(MaterialContextState::default, |st| {
        st.icon_size = prev;
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size, TextStyle};
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

    #[test]
    fn layout_direction_inherits_masks_and_restores() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(
            &mut app,
            window,
            bounds(),
            "m3-context-layout-direction",
            |cx| {
                assert_eq!(inherited_layout_direction_override(cx), None);
                assert_eq!(
                    resolved_layout_direction(cx, LayoutDirection::Ltr),
                    LayoutDirection::Ltr
                );

                with_material_layout_direction(cx, LayoutDirection::Rtl, |cx| {
                    assert_eq!(
                        inherited_layout_direction_override(cx),
                        Some(MaterialLayoutDirectionOverride::Custom(
                            LayoutDirection::Rtl
                        ))
                    );
                    assert_eq!(
                        resolved_layout_direction(cx, LayoutDirection::Ltr),
                        LayoutDirection::Rtl
                    );

                    with_default_material_layout_direction(cx, |cx| {
                        assert_eq!(
                            inherited_layout_direction_override(cx),
                            Some(MaterialLayoutDirectionOverride::UseDefault)
                        );
                        assert_eq!(
                            resolved_layout_direction(cx, LayoutDirection::Rtl),
                            LayoutDirection::Rtl
                        );
                    });

                    assert_eq!(
                        resolved_layout_direction(cx, LayoutDirection::Ltr),
                        LayoutDirection::Rtl
                    );
                });

                assert_eq!(inherited_layout_direction_override(cx), None);
            },
        );
    }

    #[test]
    fn text_style_inherits_masks_and_restores() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let large = TextStyle {
            size: Px(42.0),
            ..TextStyle::default()
        };
        let small = TextStyle {
            size: Px(10.0),
            ..TextStyle::default()
        };

        with_element_cx(&mut app, window, bounds(), "m3-context-text-style", |cx| {
            assert_eq!(inherited_text_style_override(cx), None);
            assert_eq!(inherited_text_style(cx), None);
            assert_eq!(resolved_text_style(cx, None), None);
            assert_eq!(
                resolved_text_style(cx, Some(small.clone())),
                Some(small.clone())
            );

            with_material_text_style(cx, large.clone(), |cx| {
                assert_eq!(
                    inherited_text_style_override(cx),
                    Some(MaterialTextStyleOverride::Custom(large.clone()))
                );
                assert_eq!(inherited_text_style(cx), Some(large.clone()));
                assert_eq!(resolved_text_style(cx, None), Some(large.clone()));

                with_default_material_text_style(cx, |cx| {
                    assert_eq!(
                        inherited_text_style_override(cx),
                        Some(MaterialTextStyleOverride::UseDefault)
                    );
                    assert_eq!(inherited_text_style(cx), None);
                    assert_eq!(
                        resolved_text_style(cx, Some(small.clone())),
                        Some(small.clone())
                    );
                });

                assert_eq!(inherited_text_style(cx), Some(large.clone()));

                with_material_text_style(cx, small.clone(), |cx| {
                    assert_eq!(inherited_text_style(cx), Some(small.clone()));
                });

                assert_eq!(inherited_text_style(cx), Some(large.clone()));
            });

            assert_eq!(inherited_text_style(cx), None);
        });
    }

    #[test]
    fn icon_size_inherits_masks_and_restores() {
        let window = AppWindowId::default();
        let mut app = App::new();

        with_element_cx(&mut app, window, bounds(), "m3-context-icon-size", |cx| {
            assert_eq!(inherited_icon_size_override(cx), None);
            assert_eq!(inherited_icon_size(cx), None);
            assert_eq!(resolved_icon_size(cx, Px(24.0)), Px(24.0));

            with_material_icon_size(cx, Px(20.0), |cx| {
                assert_eq!(
                    inherited_icon_size_override(cx),
                    Some(MaterialIconSizeOverride::Custom(Px(20.0)))
                );
                assert_eq!(inherited_icon_size(cx), Some(Px(20.0)));
                assert_eq!(resolved_icon_size(cx, Px(24.0)), Px(20.0));

                with_default_material_icon_size(cx, |cx| {
                    assert_eq!(
                        inherited_icon_size_override(cx),
                        Some(MaterialIconSizeOverride::UseDefault)
                    );
                    assert_eq!(inherited_icon_size(cx), None);
                    assert_eq!(resolved_icon_size(cx, Px(24.0)), Px(24.0));
                });

                assert_eq!(inherited_icon_size(cx), Some(Px(20.0)));

                with_material_icon_size(cx, Px(16.0), |cx| {
                    assert_eq!(inherited_icon_size(cx), Some(Px(16.0)));
                });

                assert_eq!(inherited_icon_size(cx), Some(Px(20.0)));
            });

            assert_eq!(inherited_icon_size(cx), None);
        });
    }
}
