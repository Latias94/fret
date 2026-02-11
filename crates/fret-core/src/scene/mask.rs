use super::{Color, ColorSpace, GradientStop, LinearGradient, MAX_STOPS, RadialGradient, TileMode};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mask {
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
}

impl Mask {
    pub const fn linear_gradient(g: LinearGradient) -> Self {
        Self::LinearGradient(g)
    }

    pub const fn radial_gradient(g: RadialGradient) -> Self {
        Self::RadialGradient(g)
    }

    pub fn sanitize(self) -> Option<Self> {
        fn color_is_finite(c: Color) -> bool {
            c.r.is_finite() && c.g.is_finite() && c.b.is_finite() && c.a.is_finite()
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

        match self {
            Mask::LinearGradient(mut g) => {
                g.stop_count = normalize_stop_count(g.stop_count);
                g.tile_mode = degrade_tile_mode(g.tile_mode);
                g.color_space = degrade_color_space(g.color_space);

                if !g.start.x.0.is_finite()
                    || !g.start.y.0.is_finite()
                    || !g.end.x.0.is_finite()
                    || !g.end.y.0.is_finite()
                    || !stops_all_finite(g.stop_count, &g.stops)
                {
                    return None;
                }

                g.stops = sort_stops(g.stop_count, g.stops);
                Some(Mask::LinearGradient(g))
            }
            Mask::RadialGradient(mut g) => {
                g.stop_count = normalize_stop_count(g.stop_count);
                g.tile_mode = degrade_tile_mode(g.tile_mode);
                g.color_space = degrade_color_space(g.color_space);

                if !g.center.x.0.is_finite()
                    || !g.center.y.0.is_finite()
                    || !g.radius.width.0.is_finite()
                    || !g.radius.height.0.is_finite()
                    || !stops_all_finite(g.stop_count, &g.stops)
                {
                    return None;
                }

                g.stops = sort_stops(g.stop_count, g.stops);
                Some(Mask::RadialGradient(g))
            }
        }
    }
}
