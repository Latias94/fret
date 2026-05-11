use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use super::*;

use fret_mechanism_harness::{
    MechanismCase, MechanismHarness, MechanismSuite, ObservedTree, ScenarioObserveError,
};
use fret_runtime::GlobalsHost;
use serde::Deserialize;

const ENVIRONMENT_VIEW_CACHE_INVALIDATION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/declarative/tests/fixtures/environment_view_cache_invalidation_v1.json"
));

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum EnvironmentScenario {
    EnvironmentViewCache {
        query: EnvironmentQueryFixture,
        invalidation: InvalidationFixture,
        source: EnvironmentSource,
        initial: EnvironmentValue,
        changed: EnvironmentValue,
    },
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EnvironmentQueryFixture {
    AccentColor,
    ColorScheme,
    ContrastPreference,
    ForcedColorsMode,
    OcclusionInsets,
    PrefersReducedMotion,
    PrefersReducedTransparency,
    SafeAreaInsets,
    TextScaleFactor,
    ViewportWidth,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum InvalidationFixture {
    Layout,
    Paint,
}

impl From<InvalidationFixture> for Invalidation {
    fn from(value: InvalidationFixture) -> Self {
        match value {
            InvalidationFixture::Layout => Invalidation::Layout,
            InvalidationFixture::Paint => Invalidation::Paint,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EnvironmentSource {
    RenderBounds,
    WindowMetrics,
}

#[derive(Debug, Clone, Deserialize)]
struct EnvironmentValue {
    #[serde(default)]
    bool: Option<bool>,
    #[serde(default)]
    color: Option<String>,
    #[serde(rename = "enum", default)]
    enum_value: Option<String>,
    #[serde(default)]
    edges: Option<f32>,
    #[serde(default)]
    float: Option<f32>,
    #[serde(default)]
    width: Option<f32>,
    #[serde(default)]
    height: Option<f32>,
}

#[test]
fn mechanism_harness_environment_view_cache_invalidation_matches_oracles() {
    let suite: MechanismSuite<EnvironmentScenario> =
        MechanismSuite::from_json_str(ENVIRONMENT_VIEW_CACHE_INVALIDATION)
            .expect("environment view-cache invalidation fixture suite");

    let mut observer: fn(
        &MechanismCase<EnvironmentScenario>,
    ) -> Result<ObservedTree, ScenarioObserveError> = observe_case;
    MechanismHarness::new().assert_suite_passes(&suite, &mut observer);
}

fn observe_case(
    case: &MechanismCase<EnvironmentScenario>,
) -> Result<ObservedTree, ScenarioObserveError> {
    match &case.scenario {
        EnvironmentScenario::EnvironmentViewCache {
            query,
            invalidation,
            source,
            initial,
            changed,
        } => observe_environment_view_cache_case(*query, *invalidation, *source, initial, changed),
    }
}

fn observe_environment_view_cache_case(
    query: EnvironmentQueryFixture,
    invalidation: InvalidationFixture,
    source: EnvironmentSource,
    initial: &EnvironmentValue,
    changed: &EnvironmentValue,
) -> Result<ObservedTree, ScenarioObserveError> {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    ui.set_view_cache_enabled(true);

    let mut services = FakeTextService::default();
    let renders_env = Arc::new(AtomicUsize::new(0));
    let renders_plain = Arc::new(AtomicUsize::new(0));
    let observed_values = Arc::new(Mutex::new(Vec::<i64>::new()));

    for frame in 0..4 {
        let value = if frame < 2 { initial } else { changed };
        let bounds = bounds_for_value(value)?;
        apply_environment_value(&mut app, window, query, source, value)?;

        let renders_env_for_closure = Arc::clone(&renders_env);
        let renders_plain_for_closure = Arc::clone(&renders_plain);
        let observed_values_for_closure = Arc::clone(&observed_values);

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "mechanism-harness-environment-view-cache",
            move |cx| {
                let env_cache =
                    cx.view_cache(crate::element::ViewCacheProps::default(), move |cx| {
                        renders_env_for_closure.fetch_add(1, Ordering::SeqCst);
                        let observed = observe_environment_query(cx, query, invalidation.into());
                        observed_values_for_closure
                            .lock()
                            .expect("observed values")
                            .push(observed);
                        vec![cx.text("env")]
                    });

                let plain_cache = cx.view_cache(
                    crate::element::ViewCacheProps {
                        cache_key: 1,
                        ..Default::default()
                    },
                    move |cx| {
                        renders_plain_for_closure.fetch_add(1, Ordering::SeqCst);
                        vec![cx.text("plain")]
                    },
                );

                vec![env_cache, plain_cache]
            },
        );
        if frame == 0 {
            ui.set_root(root);
        }

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        app.advance_frame();
    }

    let observed = observed_values.lock().expect("observed values");
    let unique_values = observed.iter().copied().collect::<HashSet<_>>().len();

    let mut tree = ObservedTree::new(bounds_for_value(changed)?);
    tree.set_metric(
        "renders.env_cache",
        renders_env.load(Ordering::SeqCst) as f32,
    );
    tree.set_metric(
        "renders.plain_cache",
        renders_plain.load(Ordering::SeqCst) as f32,
    );
    tree.set_metric("observed.env_values", observed.len() as f32);
    tree.set_metric("observed.env_unique_values", unique_values as f32);
    Ok(tree)
}

fn apply_environment_value(
    app: &mut TestHost,
    window: AppWindowId,
    query: EnvironmentQueryFixture,
    source: EnvironmentSource,
    value: &EnvironmentValue,
) -> Result<(), ScenarioObserveError> {
    match source {
        EnvironmentSource::RenderBounds => Ok(()),
        EnvironmentSource::WindowMetrics => app.with_global_mut_untracked(
            fret_core::window::WindowMetricsService::default,
            |svc: &mut fret_core::window::WindowMetricsService, _| match query {
                EnvironmentQueryFixture::AccentColor => {
                    svc.set_accent_color(window, Some(color_value(value)?));
                    Ok(())
                }
                EnvironmentQueryFixture::ColorScheme => {
                    svc.set_color_scheme(window, Some(color_scheme_value(value)?));
                    Ok(())
                }
                EnvironmentQueryFixture::ContrastPreference => {
                    svc.set_contrast_preference(window, Some(contrast_value(value)?));
                    Ok(())
                }
                EnvironmentQueryFixture::ForcedColorsMode => {
                    svc.set_forced_colors_mode(window, Some(forced_colors_value(value)?));
                    Ok(())
                }
                EnvironmentQueryFixture::OcclusionInsets => {
                    svc.set_occlusion_insets(window, Some(edges_value(value)?));
                    Ok(())
                }
                EnvironmentQueryFixture::PrefersReducedMotion => {
                    svc.set_prefers_reduced_motion(window, Some(bool_value(value)?));
                    Ok(())
                }
                EnvironmentQueryFixture::PrefersReducedTransparency => {
                    svc.set_prefers_reduced_transparency(window, Some(bool_value(value)?));
                    Ok(())
                }
                EnvironmentQueryFixture::SafeAreaInsets => {
                    svc.set_safe_area_insets(window, Some(edges_value(value)?));
                    Ok(())
                }
                EnvironmentQueryFixture::TextScaleFactor => {
                    svc.set_text_scale_factor(window, Some(float_value(value)?));
                    Ok(())
                }
                EnvironmentQueryFixture::ViewportWidth => Ok(()),
            },
        ),
    }
}

fn observe_environment_query(
    cx: &mut ElementContext<'_, TestHost>,
    query: EnvironmentQueryFixture,
    invalidation: Invalidation,
) -> i64 {
    match query {
        EnvironmentQueryFixture::AccentColor => cx
            .environment_accent_color(invalidation)
            .map(color_token)
            .unwrap_or(-1),
        EnvironmentQueryFixture::ColorScheme => match cx.environment_color_scheme(invalidation) {
            Some(fret_core::ColorScheme::Light) => 0,
            Some(fret_core::ColorScheme::Dark) => 1,
            None => -1,
        },
        EnvironmentQueryFixture::ContrastPreference => {
            match cx.environment_prefers_contrast(invalidation) {
                Some(fret_core::ContrastPreference::NoPreference) => 0,
                Some(fret_core::ContrastPreference::More) => 1,
                Some(fret_core::ContrastPreference::Less) => 2,
                Some(fret_core::ContrastPreference::Custom) => 3,
                None => -1,
            }
        }
        EnvironmentQueryFixture::ForcedColorsMode => {
            match cx.environment_forced_colors_mode(invalidation) {
                Some(fret_core::ForcedColorsMode::None) => 0,
                Some(fret_core::ForcedColorsMode::Active) => 1,
                None => -1,
            }
        }
        EnvironmentQueryFixture::OcclusionInsets => cx
            .environment_occlusion_insets(invalidation)
            .map(edges_token)
            .unwrap_or(-1),
        EnvironmentQueryFixture::PrefersReducedMotion => {
            option_bool_token(cx.environment_prefers_reduced_motion(invalidation))
        }
        EnvironmentQueryFixture::PrefersReducedTransparency => {
            option_bool_token(cx.environment_prefers_reduced_transparency(invalidation))
        }
        EnvironmentQueryFixture::SafeAreaInsets => cx
            .environment_safe_area_insets(invalidation)
            .map(edges_token)
            .unwrap_or(-1),
        EnvironmentQueryFixture::TextScaleFactor => cx
            .environment_text_scale_factor(invalidation)
            .map(|value| (value * 1000.0).round() as i64)
            .unwrap_or(-1),
        EnvironmentQueryFixture::ViewportWidth => {
            cx.environment_viewport_width(invalidation).0.round() as i64
        }
    }
}

fn bounds_for_value(value: &EnvironmentValue) -> Result<Rect, ScenarioObserveError> {
    Ok(Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(
            Px(value.width.unwrap_or(240.0)),
            Px(value.height.unwrap_or(120.0)),
        ),
    ))
}

fn bool_value(value: &EnvironmentValue) -> Result<bool, ScenarioObserveError> {
    value
        .bool
        .ok_or_else(|| ScenarioObserveError::new("environment value missing bool"))
}

fn float_value(value: &EnvironmentValue) -> Result<f32, ScenarioObserveError> {
    value
        .float
        .ok_or_else(|| ScenarioObserveError::new("environment value missing float"))
}

fn color_scheme_value(
    value: &EnvironmentValue,
) -> Result<fret_core::ColorScheme, ScenarioObserveError> {
    match enum_value(value)? {
        "light" => Ok(fret_core::ColorScheme::Light),
        "dark" => Ok(fret_core::ColorScheme::Dark),
        other => Err(ScenarioObserveError::new(format!(
            "unknown color scheme {other:?}"
        ))),
    }
}

fn contrast_value(
    value: &EnvironmentValue,
) -> Result<fret_core::ContrastPreference, ScenarioObserveError> {
    match enum_value(value)? {
        "no_preference" => Ok(fret_core::ContrastPreference::NoPreference),
        "more" => Ok(fret_core::ContrastPreference::More),
        "less" => Ok(fret_core::ContrastPreference::Less),
        "custom" => Ok(fret_core::ContrastPreference::Custom),
        other => Err(ScenarioObserveError::new(format!(
            "unknown contrast preference {other:?}"
        ))),
    }
}

fn forced_colors_value(
    value: &EnvironmentValue,
) -> Result<fret_core::ForcedColorsMode, ScenarioObserveError> {
    match enum_value(value)? {
        "none" => Ok(fret_core::ForcedColorsMode::None),
        "active" => Ok(fret_core::ForcedColorsMode::Active),
        other => Err(ScenarioObserveError::new(format!(
            "unknown forced colors mode {other:?}"
        ))),
    }
}

fn enum_value(value: &EnvironmentValue) -> Result<&str, ScenarioObserveError> {
    value
        .enum_value
        .as_deref()
        .ok_or_else(|| ScenarioObserveError::new("environment value missing enum"))
}

fn edges_value(value: &EnvironmentValue) -> Result<fret_core::Edges, ScenarioObserveError> {
    let edge = value
        .edges
        .ok_or_else(|| ScenarioObserveError::new("environment value missing edges"))?;
    Ok(fret_core::Edges::all(Px(edge)))
}

fn color_value(value: &EnvironmentValue) -> Result<fret_core::Color, ScenarioObserveError> {
    let raw = value
        .color
        .as_deref()
        .ok_or_else(|| ScenarioObserveError::new("environment value missing color"))?;
    let hex = raw
        .strip_prefix('#')
        .ok_or_else(|| ScenarioObserveError::new(format!("expected #RRGGBB color, got {raw:?}")))?;
    let value = u32::from_str_radix(hex, 16)
        .map_err(|err| ScenarioObserveError::new(format!("invalid color {raw:?}: {err}")))?;
    Ok(fret_core::Color::from_srgb_hex_rgb(value))
}

fn option_bool_token(value: Option<bool>) -> i64 {
    match value {
        Some(false) => 0,
        Some(true) => 1,
        None => -1,
    }
}

fn edges_token(edges: fret_core::Edges) -> i64 {
    (edges.top.0 * 1000.0).round() as i64
}

fn color_token(color: fret_core::Color) -> i64 {
    ((color.r * 1000.0).round() as i64) * 1_000_000
        + ((color.g * 1000.0).round() as i64) * 1_000
        + (color.b * 1000.0).round() as i64
}
