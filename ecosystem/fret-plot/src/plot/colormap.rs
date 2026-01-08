use fret_core::scene::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorMapId {
    /// A simple blue -> cyan -> green -> yellow -> red ramp (portable and predictable).
    Spectrum,
    /// Google's "Turbo" colormap (smooth and high-contrast).
    Turbo,
    /// A Viridis-like ramp (perceptually uniform-ish).
    Viridis,
    /// Grayscale (black -> white).
    Gray,
}

impl Default for ColorMapId {
    fn default() -> Self {
        Self::Turbo
    }
}

impl ColorMapId {
    pub fn key(self) -> u8 {
        match self {
            Self::Spectrum => 1,
            Self::Turbo => 2,
            Self::Viridis => 3,
            Self::Gray => 4,
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: lerp(a.a, b.a, t),
    }
}

fn sample_stops(stops: &[(f32, Color)], t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    if stops.is_empty() {
        return Color::TRANSPARENT;
    }
    if stops.len() == 1 {
        return stops[0].1;
    }

    for w in stops.windows(2) {
        let (t0, c0) = w[0];
        let (t1, c1) = w[1];
        if t <= t1 {
            let denom = (t1 - t0).max(1.0e-12);
            let u = ((t - t0) / denom).clamp(0.0, 1.0);
            return lerp_color(c0, c1, u);
        }
    }

    stops[stops.len() - 1].1
}

fn turbo_channel(t: f32, c0: f32, c1: f32, c2: f32, c3: f32, c4: f32, c5: f32) -> f32 {
    // Polynomial approximation used by the original Turbo implementation.
    // Source: https://ai.googleblog.com/2019/08/turbo-improved-rainbow-colormap-for.html
    c0 + t * (c1 + t * (c2 + t * (c3 + t * (c4 + t * c5))))
}

fn sample_turbo(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let r = turbo_channel(
        t,
        0.135_721_38,
        4.615_392_60,
        -42.660_322_58,
        132.131_082_34,
        -152.942_393_96,
        59.286_379_43,
    );
    let g = turbo_channel(
        t,
        0.091_402_61,
        2.194_188_39,
        4.842_966_58,
        -14.185_033_33,
        4.277_298_57,
        2.829_566_04,
    );
    let b = turbo_channel(
        t,
        0.106_673_30,
        12.641_946_08,
        -60.582_048_36,
        110.362_767_71,
        -89.903_109_12,
        27.348_249_73,
    );

    Color {
        r: r.clamp(0.0, 1.0),
        g: g.clamp(0.0, 1.0),
        b: b.clamp(0.0, 1.0),
        a: 1.0,
    }
}

pub fn sample(id: ColorMapId, t: f32) -> Color {
    match id {
        ColorMapId::Spectrum => {
            // Matches the old heatmap ramp: blue -> cyan -> green -> yellow -> red.
            let t = t.clamp(0.0, 1.0);
            let (r, g, b) = if t < 0.25 {
                let u = t / 0.25;
                (0.0, lerp(0.1, 1.0, u), 1.0)
            } else if t < 0.50 {
                let u = (t - 0.25) / 0.25;
                (0.0, 1.0, lerp(1.0, 0.0, u))
            } else if t < 0.75 {
                let u = (t - 0.50) / 0.25;
                (lerp(0.0, 1.0, u), 1.0, 0.0)
            } else {
                let u = (t - 0.75) / 0.25;
                (1.0, lerp(1.0, 0.0, u), 0.0)
            };
            Color { r, g, b, a: 1.0 }
        }
        ColorMapId::Turbo => sample_turbo(t),
        ColorMapId::Viridis => {
            // A small-stop approximation. We can replace this with a LUT later if needed.
            const STOPS: &[(f32, Color)] = &[
                (
                    0.0,
                    Color {
                        r: 0.267_004,
                        g: 0.004_874,
                        b: 0.329_415,
                        a: 1.0,
                    },
                ),
                (
                    0.25,
                    Color {
                        r: 0.229_739,
                        g: 0.322_361,
                        b: 0.545_706,
                        a: 1.0,
                    },
                ),
                (
                    0.50,
                    Color {
                        r: 0.127_568,
                        g: 0.566_949,
                        b: 0.550_556,
                        a: 1.0,
                    },
                ),
                (
                    0.75,
                    Color {
                        r: 0.369_214,
                        g: 0.788_888,
                        b: 0.382_914,
                        a: 1.0,
                    },
                ),
                (
                    1.0,
                    Color {
                        r: 0.993_248,
                        g: 0.906_157,
                        b: 0.143_936,
                        a: 1.0,
                    },
                ),
            ];
            sample_stops(STOPS, t)
        }
        ColorMapId::Gray => {
            let t = t.clamp(0.0, 1.0);
            Color {
                r: t,
                g: t,
                b: t,
                a: 1.0,
            }
        }
    }
}
