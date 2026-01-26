//! Material 3 theme helpers.
//!
//! Note: this is intentionally a thin, crate-local abstraction for now. The expected direction
//! is to drive Fret's global `ThemeConfig` from Material tokens (including non-Px scalar values)
//! once the runtime theme surface is extended via ADR.

use fret_core::Color;

/// A minimal Material 3 color role set (stub).
///
/// This will expand to match `md.sys.color.*` roles (and expressive variants) as we wire up the
/// token import + resolution pipeline.
#[derive(Debug, Clone, Copy)]
pub struct Material3Colors {
    pub primary: Color,
    pub on_primary: Color,
    pub surface: Color,
    pub on_surface: Color,
}

impl Default for Material3Colors {
    fn default() -> Self {
        Self {
            primary: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            on_primary: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            surface: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            on_surface: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
        }
    }
}
