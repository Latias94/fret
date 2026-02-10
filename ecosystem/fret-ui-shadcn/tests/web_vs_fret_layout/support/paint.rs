use super::*;

pub(crate) fn paint_solid_color(paint: fret_core::Paint) -> Option<fret_core::Color> {
    match paint {
        fret_core::Paint::Solid(c) => Some(c),
        _ => None,
    }
}

pub(crate) fn paint_representative_color(paint: fret_core::Paint) -> fret_core::Color {
    match paint {
        fret_core::Paint::Solid(c) => c,
        fret_core::Paint::LinearGradient(g) => {
            let n = usize::from(g.stop_count).min(g.stops.len());
            let mut best = fret_core::Color::TRANSPARENT;
            let mut best_a = -1.0f32;
            for s in g.stops.iter().take(n) {
                let a = s.color.a;
                if a > best_a {
                    best_a = a;
                    best = s.color;
                }
            }
            best
        }
        fret_core::Paint::RadialGradient(g) => {
            let n = usize::from(g.stop_count).min(g.stops.len());
            let mut best = fret_core::Color::TRANSPARENT;
            let mut best_a = -1.0f32;
            for s in g.stops.iter().take(n) {
                let a = s.color.a;
                if a > best_a {
                    best_a = a;
                    best = s.color;
                }
            }
            best
        }
        fret_core::Paint::Material { .. } => fret_core::Color::TRANSPARENT,
    }
}

pub(crate) fn paint_to_rgba(paint: fret_core::Paint) -> Rgba {
    color_to_rgba(paint_representative_color(paint))
}

pub(crate) fn paint_max_alpha(paint: fret_core::Paint) -> f32 {
    match paint {
        fret_core::Paint::Solid(c) => c.a,
        fret_core::Paint::LinearGradient(g) => {
            let n = usize::from(g.stop_count).min(g.stops.len());
            g.stops
                .iter()
                .take(n)
                .map(|s| s.color.a)
                .fold(0.0f32, f32::max)
        }
        fret_core::Paint::RadialGradient(g) => {
            let n = usize::from(g.stop_count).min(g.stops.len());
            g.stops
                .iter()
                .take(n)
                .map(|s| s.color.a)
                .fold(0.0f32, f32::max)
        }
        fret_core::Paint::Material { .. } => 1.0,
    }
}
