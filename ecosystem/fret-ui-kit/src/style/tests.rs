use super::*;
use fret_core::Px;
use fret_ui::Theme;
use fret_ui::ThemeConfig;

#[test]
fn space_falls_back_to_theme_padding_scale() {
    let mut app = fret_app::App::default();

    let cfg = ThemeConfig {
        name: "Test".to_string(),
        metrics: std::collections::HashMap::from([
            ("metric.padding.sm".to_string(), 12.0),
            ("metric.padding.md".to_string(), 14.0),
        ]),
        ..ThemeConfig::default()
    };
    Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

    let theme = Theme::global(&app);
    assert_eq!(MetricRef::space(Space::N2).resolve(theme), Px(12.0));
    assert_eq!(MetricRef::space(Space::N2p5).resolve(theme), Px(14.0));
    assert_eq!(MetricRef::space(Space::N1).resolve(theme), Px(6.0));
    assert_eq!(MetricRef::space(Space::N0p5).resolve(theme), Px(3.0));
    assert_eq!(MetricRef::space(Space::N11).resolve(theme), Px(66.0));
}

#[test]
fn space_token_overrides_theme_fallback() {
    let mut app = fret_app::App::default();

    let cfg = ThemeConfig {
        name: "Test".to_string(),
        metrics: std::collections::HashMap::from([
            ("metric.padding.sm".to_string(), 12.0),
            ("component.space.2".to_string(), 20.0),
        ]),
        ..ThemeConfig::default()
    };
    Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

    let theme = Theme::global(&app);
    assert_eq!(MetricRef::space(Space::N2).resolve(theme), Px(20.0));
}

#[test]
fn radius_falls_back_to_baseline_metric_tokens() {
    let mut app = fret_app::App::default();

    let cfg = ThemeConfig {
        name: "Test".to_string(),
        metrics: std::collections::HashMap::from([
            ("metric.radius.sm".to_string(), 11.0),
            ("metric.radius.md".to_string(), 9.0),
            ("component.radius.md".to_string(), 12.0),
        ]),
        ..ThemeConfig::default()
    };
    Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

    let theme = Theme::global(&app);
    assert_eq!(MetricRef::radius(Radius::Md).resolve(theme), Px(12.0));
    assert_eq!(MetricRef::radius(Radius::Sm).resolve(theme), Px(11.0));
}

#[test]
fn widget_state_property_last_match_wins() {
    let prop = WidgetStateProperty::new(1)
        .when(WidgetStates::HOVERED, 2)
        .when(WidgetStates::ACTIVE, 3);

    assert_eq!(*prop.resolve(WidgetStates::empty()), 1);
    assert_eq!(*prop.resolve(WidgetStates::HOVERED), 2);
    assert_eq!(
        *prop.resolve(WidgetStates::HOVERED | WidgetStates::ACTIVE),
        3
    );
}

#[test]
fn resolve_override_slot_falls_back_when_override_is_none() {
    let defaults = WidgetStateProperty::new(10).when(WidgetStates::HOVERED, 20);
    let overrides = WidgetStateProperty::new(Some(99)).when(WidgetStates::HOVERED, None);

    assert_eq!(
        super::resolve_override_slot(Some(&overrides), &defaults, WidgetStates::empty()),
        99
    );
    assert_eq!(
        super::resolve_override_slot(Some(&overrides), &defaults, WidgetStates::HOVERED),
        20
    );
}

#[test]
fn resolve_override_slot_opt_returns_override_when_present() {
    let defaults = WidgetStateProperty::new(Some(10)).when(WidgetStates::HOVERED, Some(20));
    let overrides = WidgetStateProperty::new(None).when(WidgetStates::HOVERED, Some(30));

    assert_eq!(
        super::resolve_override_slot_opt(Some(&overrides), &defaults, WidgetStates::empty()),
        Some(10)
    );
    assert_eq!(
        super::resolve_override_slot_opt(Some(&overrides), &defaults, WidgetStates::HOVERED),
        Some(30)
    );
}

#[test]
fn color_fallback_theme_token_alpha_mul_derives_from_base_token() {
    let mut app = fret_app::App::default();

    let cfg = ThemeConfig {
        name: "Test".to_string(),
        colors: std::collections::HashMap::from([("primary".to_string(), "#000000FF".to_string())]),
        ..ThemeConfig::default()
    };
    Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

    let theme = Theme::global(&app);
    let base = theme.color_required("primary");

    let derived = ColorRef::Token {
        key: "primary.hover.background",
        fallback: ColorFallback::ThemeTokenAlphaMul {
            key: "primary",
            mul: 0.5,
        },
    }
    .resolve(theme);

    assert!((derived.a - (base.a * 0.5)).abs() < 1e-6);
}

#[test]
fn state_specific_token_overrides_fallback_derivation() {
    let mut app = fret_app::App::default();

    let cfg = ThemeConfig {
        name: "Test".to_string(),
        colors: std::collections::HashMap::from([
            ("primary".to_string(), "#000000FF".to_string()),
            (
                "primary.hover.background".to_string(),
                "#00000080".to_string(),
            ),
        ]),
        ..ThemeConfig::default()
    };
    Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

    let theme = Theme::global(&app);
    let expected = theme.color_required("primary.hover.background");

    let resolved = ColorRef::Token {
        key: "primary.hover.background",
        fallback: ColorFallback::ThemeTokenAlphaMul {
            key: "primary",
            mul: 0.5,
        },
    }
    .resolve(theme);

    assert_eq!(resolved, expected);
}
