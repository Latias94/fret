//! Material 3 motion scheme helpers.
//!
//! Material Web v30 provides system motion spring tokens under:
//! - `md.sys.motion.spring.{default|fast|slow}.{spatial|effects}.{damping|stiffness}`
//!
//! Compose Material3 wraps these concepts behind a `MotionScheme` that offers 6 canonical specs:
//! `{default, fast, slow} × {spatial, effects}`.

use fret_ui::Theme;

use crate::foundation::context::{
    MaterialMotionScheme, resolved_motion_scheme, theme_default_motion_scheme,
};
use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::motion::SpringSpec;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionSchemeKey {
    DefaultSpatial,
    FastSpatial,
    SlowSpatial,
    DefaultEffects,
    FastEffects,
    SlowEffects,
}

fn sys_spring_for_scheme(
    theme: &Theme,
    scheme: MaterialMotionScheme,
    key: MotionSchemeKey,
) -> SpringSpec {
    let tokens = MaterialTokenResolver::new(theme);

    let (damping_key, stiffness_key, fallback) = match (scheme, key) {
        (MaterialMotionScheme::Standard, MotionSchemeKey::DefaultSpatial) => (
            "md.sys.motion.spring.default.spatial.damping",
            "md.sys.motion.spring.default.spatial.stiffness",
            SpringSpec::new(0.9, 700.0),
        ),
        (MaterialMotionScheme::Standard, MotionSchemeKey::FastSpatial) => (
            "md.sys.motion.spring.fast.spatial.damping",
            "md.sys.motion.spring.fast.spatial.stiffness",
            SpringSpec::new(0.9, 1400.0),
        ),
        (MaterialMotionScheme::Standard, MotionSchemeKey::SlowSpatial) => (
            "md.sys.motion.spring.slow.spatial.damping",
            "md.sys.motion.spring.slow.spatial.stiffness",
            SpringSpec::new(0.9, 300.0),
        ),
        (MaterialMotionScheme::Standard, MotionSchemeKey::DefaultEffects) => (
            "md.sys.motion.spring.default.effects.damping",
            "md.sys.motion.spring.default.effects.stiffness",
            SpringSpec::new(1.0, 1600.0),
        ),
        (MaterialMotionScheme::Standard, MotionSchemeKey::FastEffects) => (
            "md.sys.motion.spring.fast.effects.damping",
            "md.sys.motion.spring.fast.effects.stiffness",
            SpringSpec::new(1.0, 3800.0),
        ),
        (MaterialMotionScheme::Standard, MotionSchemeKey::SlowEffects) => (
            "md.sys.motion.spring.slow.effects.damping",
            "md.sys.motion.spring.slow.effects.stiffness",
            SpringSpec::new(1.0, 800.0),
        ),
        // Compose baseline (ExpressiveMotionTokens) differs only for spatial springs today; keep
        // effects aligned with the standard scheme.
        (MaterialMotionScheme::Expressive, MotionSchemeKey::DefaultSpatial) => (
            "md.sys.fret.material.motion.spring.default.spatial.damping",
            "md.sys.fret.material.motion.spring.default.spatial.stiffness",
            SpringSpec::new(0.8, 380.0),
        ),
        (MaterialMotionScheme::Expressive, MotionSchemeKey::FastSpatial) => (
            "md.sys.fret.material.motion.spring.fast.spatial.damping",
            "md.sys.fret.material.motion.spring.fast.spatial.stiffness",
            SpringSpec::new(0.6, 800.0),
        ),
        (MaterialMotionScheme::Expressive, MotionSchemeKey::SlowSpatial) => (
            "md.sys.fret.material.motion.spring.slow.spatial.damping",
            "md.sys.fret.material.motion.spring.slow.spatial.stiffness",
            SpringSpec::new(0.8, 200.0),
        ),
        (MaterialMotionScheme::Expressive, MotionSchemeKey::DefaultEffects) => (
            "md.sys.fret.material.motion.spring.default.effects.damping",
            "md.sys.fret.material.motion.spring.default.effects.stiffness",
            SpringSpec::new(1.0, 1600.0),
        ),
        (MaterialMotionScheme::Expressive, MotionSchemeKey::FastEffects) => (
            "md.sys.fret.material.motion.spring.fast.effects.damping",
            "md.sys.fret.material.motion.spring.fast.effects.stiffness",
            SpringSpec::new(1.0, 3800.0),
        ),
        (MaterialMotionScheme::Expressive, MotionSchemeKey::SlowEffects) => (
            "md.sys.fret.material.motion.spring.slow.effects.damping",
            "md.sys.fret.material.motion.spring.slow.effects.stiffness",
            SpringSpec::new(1.0, 800.0),
        ),
    };

    SpringSpec {
        damping: tokens.number_sys(damping_key, fallback.damping),
        stiffness: tokens.number_sys(stiffness_key, fallback.stiffness),
    }
}

pub fn sys_spring_in_scope<H: fret_ui::UiHost>(
    cx: &fret_ui::elements::ElementContext<'_, H>,
    theme: &Theme,
    key: MotionSchemeKey,
) -> SpringSpec {
    let default_scheme = theme_default_motion_scheme(theme);
    let scheme = resolved_motion_scheme(cx, default_scheme);
    sys_spring_for_scheme(theme, scheme, key)
}

#[cfg(test)]
mod tests {
    use super::{MotionSchemeKey, sys_spring_for_scheme};
    use crate::foundation::context::MaterialMotionScheme;
    use crate::tokens::v30::{TypographyOptions, theme_config};
    use fret_app::App;
    use fret_ui::Theme;

    #[test]
    fn sys_motion_spring_tokens_are_available_in_v30_theme() {
        let cfg = theme_config(TypographyOptions::default());
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&cfg);
        });
        let theme = Theme::global(&app);

        let spatial = sys_spring_for_scheme(
            &theme,
            MaterialMotionScheme::Standard,
            MotionSchemeKey::DefaultSpatial,
        );
        assert!(spatial.damping > 0.0);
        assert!(spatial.stiffness > 0.0);

        let effects = sys_spring_for_scheme(
            &theme,
            MaterialMotionScheme::Standard,
            MotionSchemeKey::FastEffects,
        );
        assert!(effects.damping > 0.0);
        assert!(effects.stiffness > 0.0);
    }

    #[test]
    fn expressive_motion_spatial_springs_match_compose_baseline() {
        let cfg = theme_config(TypographyOptions::default());
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&cfg);
        });
        let theme = Theme::global(&app);

        let default_spatial = sys_spring_for_scheme(
            &theme,
            MaterialMotionScheme::Expressive,
            MotionSchemeKey::DefaultSpatial,
        );
        assert!(
            (default_spatial.damping - 0.8).abs() < 0.0001,
            "expected Expressive default spatial damping to match Compose ExpressiveMotionTokens"
        );
        assert!(
            (default_spatial.stiffness - 380.0).abs() < 0.0001,
            "expected Expressive default spatial stiffness to match Compose ExpressiveMotionTokens"
        );
    }
}
