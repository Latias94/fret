use std::time::Duration;

use fret_ui::{Theme, UiHost};
use fret_ui_headless::motion::spring::SpringDescription;

const THEME_DURATION_SHADCN_SPRING_DRAWER_SETTLE: &str =
    "duration.shadcn.motion.spring.drawer.settle";
const THEME_NUMBER_SHADCN_SPRING_DRAWER_SETTLE_BOUNCE: &str =
    "number.shadcn.motion.spring.drawer.settle.bounce";

const THEME_DURATION_SHADCN_SPRING_DRAWER_INERTIA_BOUNCE: &str =
    "duration.shadcn.motion.spring.drawer.inertia_bounce";
const THEME_NUMBER_SHADCN_SPRING_DRAWER_INERTIA_BOUNCE_BOUNCE: &str =
    "number.shadcn.motion.spring.drawer.inertia_bounce.bounce";

fn duration_from_theme_ms<H: UiHost>(app: &H, key: &str) -> Option<Duration> {
    let theme = Theme::global(app);
    let ms = theme.duration_ms_by_key(key)?;
    if ms == 0 {
        return None;
    }
    Some(Duration::from_millis(ms as u64))
}

fn bounce_from_theme_number<H: UiHost>(app: &H, key: &str) -> Option<f64> {
    let theme = Theme::global(app);
    let v = theme.number_by_key(key)? as f64;
    if !v.is_finite() {
        return None;
    }
    Some(v)
}

fn spring_from_duration_and_bounce(duration: Duration, bounce: f64) -> SpringDescription {
    let duration = if duration > Duration::ZERO {
        duration
    } else {
        Duration::from_millis(1)
    };
    // `with_duration_and_bounce` requires `bounce > -1.0`. Clamp to avoid theme configs producing
    // panics when experimenting with values.
    let bounce = bounce.max(-0.999);
    SpringDescription::with_duration_and_bounce(duration, bounce)
}

pub fn shadcn_drawer_settle_spring_description<H: UiHost>(app: &H) -> SpringDescription {
    let default_duration = Duration::from_millis(240);
    let default_bounce = 0.0;

    let duration = duration_from_theme_ms(app, THEME_DURATION_SHADCN_SPRING_DRAWER_SETTLE)
        .unwrap_or(default_duration);
    let bounce = bounce_from_theme_number(app, THEME_NUMBER_SHADCN_SPRING_DRAWER_SETTLE_BOUNCE)
        .unwrap_or(default_bounce);

    spring_from_duration_and_bounce(duration, bounce)
}

pub fn shadcn_drawer_inertia_bounce_spring_description<H: UiHost>(app: &H) -> SpringDescription {
    let default_duration = Duration::from_millis(240);
    let default_bounce = 0.25;

    let duration = duration_from_theme_ms(app, THEME_DURATION_SHADCN_SPRING_DRAWER_INERTIA_BOUNCE)
        .unwrap_or(default_duration);
    let bounce =
        bounce_from_theme_number(app, THEME_NUMBER_SHADCN_SPRING_DRAWER_INERTIA_BOUNCE_BOUNCE)
            .unwrap_or(default_bounce);

    spring_from_duration_and_bounce(duration, bounce)
}
