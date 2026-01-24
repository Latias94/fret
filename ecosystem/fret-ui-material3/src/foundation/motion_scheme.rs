//! Material 3 motion scheme helpers.
//!
//! Material Web v30 provides system motion spring tokens under:
//! - `md.sys.motion.spring.{default|fast|slow}.{spatial|effects}.{damping|stiffness}`
//!
//! Compose Material3 wraps these concepts behind a `MotionScheme` that offers 6 canonical specs:
//! `{default, fast, slow} × {spatial, effects}`.

use fret_ui::Theme;

use crate::foundation::context::{MaterialMotionScheme, resolved_motion_scheme};
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

pub fn sys_spring(theme: &Theme, key: MotionSchemeKey) -> SpringSpec {
    let tokens = MaterialTokenResolver::new(theme);

    let (damping_key, stiffness_key, fallback) = match key {
        MotionSchemeKey::DefaultSpatial => (
            "md.sys.motion.spring.default.spatial.damping",
            "md.sys.motion.spring.default.spatial.stiffness",
            SpringSpec::new(0.9, 700.0),
        ),
        MotionSchemeKey::FastSpatial => (
            "md.sys.motion.spring.fast.spatial.damping",
            "md.sys.motion.spring.fast.spatial.stiffness",
            SpringSpec::new(0.9, 1400.0),
        ),
        MotionSchemeKey::SlowSpatial => (
            "md.sys.motion.spring.slow.spatial.damping",
            "md.sys.motion.spring.slow.spatial.stiffness",
            SpringSpec::new(0.9, 300.0),
        ),
        MotionSchemeKey::DefaultEffects => (
            "md.sys.motion.spring.default.effects.damping",
            "md.sys.motion.spring.default.effects.stiffness",
            SpringSpec::new(1.0, 1600.0),
        ),
        MotionSchemeKey::FastEffects => (
            "md.sys.motion.spring.fast.effects.damping",
            "md.sys.motion.spring.fast.effects.stiffness",
            SpringSpec::new(1.0, 3800.0),
        ),
        MotionSchemeKey::SlowEffects => (
            "md.sys.motion.spring.slow.effects.damping",
            "md.sys.motion.spring.slow.effects.stiffness",
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
    // Material Web v30 currently ships a single `md.sys.motion.spring.*` set. We still surface the
    // scheme concept so the component layer can converge on a stable API, and we can swap in
    // expressive tokens later without per-component refactors.
    let _scheme = resolved_motion_scheme(cx, MaterialMotionScheme::Standard);
    sys_spring(theme, key)
}

#[cfg(test)]
mod tests {
    use super::{MotionSchemeKey, sys_spring};
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

        let spatial = sys_spring(&theme, MotionSchemeKey::DefaultSpatial);
        assert!(spatial.damping > 0.0);
        assert!(spatial.stiffness > 0.0);

        let effects = sys_spring(&theme, MotionSchemeKey::FastEffects);
        assert!(effects.damping > 0.0);
        assert!(effects.stiffness > 0.0);
    }
}
