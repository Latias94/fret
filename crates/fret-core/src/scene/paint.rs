use crate::MaterialId;
use crate::geometry::{Point, Size};

use super::Color;

pub const MAX_STOPS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileMode {
    Clamp,
    Repeat,
    Mirror,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Srgb,
    Oklab,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    pub offset: f32,
    pub color: Color,
}

impl GradientStop {
    pub const fn new(offset: f32, color: Color) -> Self {
        Self { offset, color }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearGradient {
    pub start: Point,
    pub end: Point,
    pub tile_mode: TileMode,
    pub color_space: ColorSpace,
    pub stop_count: u8,
    pub stops: [GradientStop; MAX_STOPS],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadialGradient {
    pub center: Point,
    pub radius: Size,
    pub tile_mode: TileMode,
    pub color_space: ColorSpace,
    pub stop_count: u8,
    pub stops: [GradientStop; MAX_STOPS],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialParams {
    pub vec4s: [[f32; 4]; 4],
}

impl MaterialParams {
    pub const ZERO: Self = Self {
        vec4s: [[0.0; 4]; 4],
    };

    pub fn sanitize(self) -> Self {
        let mut out = self;
        for v in &mut out.vec4s {
            for x in v {
                if !x.is_finite() {
                    *x = 0.0;
                }
            }
        }
        out
    }

    pub fn is_finite(self) -> bool {
        self.vec4s.iter().flatten().all(|&x| x.is_finite())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Paint {
    Solid(Color),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
    Material {
        id: MaterialId,
        params: MaterialParams,
    },
}

impl From<Color> for Paint {
    fn from(value: Color) -> Self {
        Paint::Solid(value)
    }
}

impl Paint {
    pub const TRANSPARENT: Self = Self::Solid(Color::TRANSPARENT);

    pub fn sanitize(self) -> Self {
        fn color_is_finite(c: Color) -> bool {
            c.r.is_finite() && c.g.is_finite() && c.b.is_finite() && c.a.is_finite()
        }

        fn point_is_finite(p: Point) -> bool {
            p.x.0.is_finite() && p.y.0.is_finite()
        }

        fn size_is_finite(s: Size) -> bool {
            s.width.0.is_finite() && s.height.0.is_finite()
        }

        fn stops_all_finite(count: u8, stops: &[GradientStop; MAX_STOPS]) -> bool {
            let n = usize::from(count).min(MAX_STOPS);
            for i in 0..n {
                let s = stops[i];
                if !s.offset.is_finite() || !color_is_finite(s.color) {
                    return false;
                }
            }
            true
        }

        fn clamp01(x: f32) -> f32 {
            x.clamp(0.0, 1.0)
        }

        fn sort_stops(
            count: u8,
            mut stops: [GradientStop; MAX_STOPS],
        ) -> [GradientStop; MAX_STOPS] {
            let n = usize::from(count).min(MAX_STOPS);

            for i in 0..n {
                stops[i].offset = clamp01(stops[i].offset);
            }

            // Stable in-place sort (no heap) for small fixed arrays.
            for i in 1..n {
                let key = stops[i];
                let mut j = i;
                while j > 0 && stops[j - 1].offset > key.offset {
                    stops[j] = stops[j - 1];
                    j -= 1;
                }
                stops[j] = key;
            }

            stops
        }

        fn normalize_stop_count(count: u8) -> u8 {
            count.min(MAX_STOPS as u8)
        }

        fn degrade_tile_mode(tile_mode: TileMode) -> TileMode {
            match tile_mode {
                TileMode::Clamp => TileMode::Clamp,
                TileMode::Repeat | TileMode::Mirror => TileMode::Clamp,
            }
        }

        fn degrade_color_space(color_space: ColorSpace) -> ColorSpace {
            match color_space {
                ColorSpace::Srgb => ColorSpace::Srgb,
                ColorSpace::Oklab => ColorSpace::Srgb,
            }
        }

        fn maybe_solid_from_degenerate(
            count: u8,
            stops: &[GradientStop; MAX_STOPS],
        ) -> Option<Paint> {
            let n = usize::from(count).min(MAX_STOPS);
            if n == 0 {
                return Some(Paint::TRANSPARENT);
            }
            if n == 1 {
                return Some(Paint::Solid(stops[0].color));
            }
            let first = stops[0].offset;
            let all_same = (1..n).all(|i| stops[i].offset == first);
            if all_same {
                return Some(Paint::Solid(stops[n - 1].color));
            }
            None
        }

        match self {
            Paint::Solid(c) => {
                if !color_is_finite(c) {
                    Paint::TRANSPARENT
                } else {
                    Paint::Solid(c)
                }
            }
            Paint::LinearGradient(mut g) => {
                g.stop_count = normalize_stop_count(g.stop_count);
                g.tile_mode = degrade_tile_mode(g.tile_mode);
                g.color_space = degrade_color_space(g.color_space);

                if !point_is_finite(g.start)
                    || !point_is_finite(g.end)
                    || !stops_all_finite(g.stop_count, &g.stops)
                {
                    return Paint::TRANSPARENT;
                }

                g.stops = sort_stops(g.stop_count, g.stops);
                if let Some(solid) = maybe_solid_from_degenerate(g.stop_count, &g.stops) {
                    return solid;
                }

                Paint::LinearGradient(g)
            }
            Paint::RadialGradient(mut g) => {
                g.stop_count = normalize_stop_count(g.stop_count);
                g.tile_mode = degrade_tile_mode(g.tile_mode);
                g.color_space = degrade_color_space(g.color_space);

                if !point_is_finite(g.center)
                    || !size_is_finite(g.radius)
                    || !stops_all_finite(g.stop_count, &g.stops)
                {
                    return Paint::TRANSPARENT;
                }

                g.stops = sort_stops(g.stop_count, g.stops);
                if let Some(solid) = maybe_solid_from_degenerate(g.stop_count, &g.stops) {
                    return solid;
                }

                Paint::RadialGradient(g)
            }
            Paint::Material { id, params } => Paint::Material {
                id,
                params: params.sanitize(),
            },
        }
    }
}
