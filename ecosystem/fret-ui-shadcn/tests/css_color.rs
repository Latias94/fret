#![allow(dead_code)]

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub(crate) struct Rgba {
    pub(crate) r: f32,
    pub(crate) g: f32,
    pub(crate) b: f32,
    pub(crate) a: f32,
}

pub(crate) fn color_to_rgba(c: fret_core::Color) -> Rgba {
    Rgba {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

pub(crate) fn parse_css_color(s: &str) -> Option<Rgba> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("transparent") {
        return Some(Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        });
    }
    parse_rgb(s)
        .or_else(|| parse_lab(s))
        .or_else(|| parse_oklab(s))
        .or_else(|| parse_oklch(s))
}

fn srgb_f32_to_linear(c_srgb: f32) -> f32 {
    if c_srgb <= 0.04045 {
        c_srgb / 12.92
    } else {
        ((c_srgb + 0.055) / 1.055).powf(2.4)
    }
}

fn parse_rgb(s: &str) -> Option<Rgba> {
    let s = s.trim();
    let inner = if let Some(v) = s.strip_prefix("rgba(").and_then(|v| v.strip_suffix(')')) {
        (v, true)
    } else if let Some(v) = s.strip_prefix("rgb(").and_then(|v| v.strip_suffix(')')) {
        (v, false)
    } else {
        return None;
    };

    let parts: Vec<&str> = inner.0.split(',').map(|p| p.trim()).collect();
    if parts.len() < 3 {
        return None;
    }

    let r_srgb: f32 = parts[0].parse::<f32>().ok()? / 255.0;
    let g_srgb: f32 = parts[1].parse::<f32>().ok()? / 255.0;
    let b_srgb: f32 = parts[2].parse::<f32>().ok()? / 255.0;
    let a: f32 = if inner.1 {
        parts
            .get(3)
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(1.0)
    } else {
        1.0
    };

    Some(Rgba {
        r: srgb_f32_to_linear(r_srgb.clamp(0.0, 1.0)),
        g: srgb_f32_to_linear(g_srgb.clamp(0.0, 1.0)),
        b: srgb_f32_to_linear(b_srgb.clamp(0.0, 1.0)),
        a,
    })
}

fn parse_oklch(s: &str) -> Option<Rgba> {
    let s = s.trim();
    let inner = s.strip_prefix("oklch(")?.strip_suffix(')')?.trim();

    let (main, alpha_part) = if let Some((l, r)) = inner.split_once('/') {
        (l.trim(), Some(r.trim()))
    } else {
        (inner, None)
    };

    let parts: Vec<&str> = main
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() != 3 {
        return None;
    }

    let l: f32 = parts[0].trim_end_matches('%').parse().ok()?;
    let c: f32 = parts[1].parse().ok()?;
    let h_raw = parts[2].trim();
    let h: f32 = h_raw
        .trim_end_matches("deg")
        .trim_end_matches("rad")
        .trim_end_matches("turn")
        .parse()
        .ok()?;

    let alpha = if let Some(a) = alpha_part {
        if let Some(pct) = a.trim_end_matches('%').parse::<f32>().ok()
            && a.trim_end().ends_with('%')
        {
            (pct / 100.0).clamp(0.0, 1.0)
        } else {
            a.parse::<f32>().ok()?.clamp(0.0, 1.0)
        }
    } else {
        1.0
    };

    // Assume degrees when unitless. (Our extracted goldens currently use unitless degrees.)
    let h_rad = h.to_radians();
    let a_ = c * h_rad.cos();
    let b_ = c * h_rad.sin();

    // OKLab -> linear sRGB (Björn Ottosson).
    let l_ = l + 0.396_337_78 * a_ + 0.215_803_76 * b_;
    let m_ = l - 0.105_561_346 * a_ - 0.063_854_17 * b_;
    let s_ = l - 0.089_484_18 * a_ - 1.291_485_5 * b_;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    let r = 4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_94 * s3;
    let g = -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_4 * s3;
    let b = -0.004_196_086_3 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3;

    Some(Rgba {
        r: r.clamp(0.0, 1.0),
        g: g.clamp(0.0, 1.0),
        b: b.clamp(0.0, 1.0),
        a: alpha,
    })
}

fn parse_oklab(s: &str) -> Option<Rgba> {
    let s = s.trim();
    let inner = s.strip_prefix("oklab(")?.strip_suffix(')')?.trim();

    let (main, alpha_part) = if let Some((l, r)) = inner.split_once('/') {
        (l.trim(), Some(r.trim()))
    } else {
        (inner, None)
    };

    let parts: Vec<&str> = main
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() != 3 {
        return None;
    }

    let l_raw = parts[0].trim();
    let l: f32 = if l_raw.ends_with('%') {
        l_raw.trim_end_matches('%').parse::<f32>().ok()? / 100.0
    } else {
        l_raw.parse().ok()?
    };
    let a_: f32 = parts[1].parse().ok()?;
    let b_: f32 = parts[2].parse().ok()?;

    let alpha = if let Some(a) = alpha_part {
        if let Some(pct) = a.trim_end_matches('%').parse::<f32>().ok()
            && a.trim_end().ends_with('%')
        {
            (pct / 100.0).clamp(0.0, 1.0)
        } else {
            a.parse::<f32>().ok()?.clamp(0.0, 1.0)
        }
    } else {
        1.0
    };

    // OKLab -> linear sRGB (Björn Ottosson).
    let l_ = l + 0.396_337_78 * a_ + 0.215_803_76 * b_;
    let m_ = l - 0.105_561_346 * a_ - 0.063_854_17 * b_;
    let s_ = l - 0.089_484_18 * a_ - 1.291_485_5 * b_;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    let r = 4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_94 * s3;
    let g = -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_4 * s3;
    let b = -0.004_196_086_3 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3;

    Some(Rgba {
        r: r.clamp(0.0, 1.0),
        g: g.clamp(0.0, 1.0),
        b: b.clamp(0.0, 1.0),
        a: alpha,
    })
}

fn parse_lab(s: &str) -> Option<Rgba> {
    let s = s.trim();
    let inner = s.strip_prefix("lab(")?.strip_suffix(')')?.trim();

    let (main, alpha_part) = if let Some((l, r)) = inner.split_once('/') {
        (l.trim(), Some(r.trim()))
    } else {
        (inner, None)
    };

    let parts: Vec<&str> = main
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|p| !p.is_empty())
        .collect();
    if parts.len() != 3 {
        return None;
    }

    let l_star: f32 = parts[0].trim_end_matches('%').parse().ok()?;
    let a_star: f32 = parts[1].parse().ok()?;
    let b_star: f32 = parts[2].parse().ok()?;

    let alpha = if let Some(a) = alpha_part {
        if let Some(pct) = a.trim_end_matches('%').parse::<f32>().ok()
            && a.trim_end().ends_with('%')
        {
            (pct / 100.0).clamp(0.0, 1.0)
        } else {
            a.parse::<f32>().ok()?.clamp(0.0, 1.0)
        }
    } else {
        1.0
    };

    // Convert Lab(D50) -> XYZ(D50) -> XYZ(D65) (Bradford adaptation) -> linear sRGB.
    let fy = (l_star + 16.0) / 116.0;
    let fx = fy + a_star / 500.0;
    let fz = fy - b_star / 200.0;

    let eps = 216.0 / 24_389.0;
    let kappa = 24_389.0 / 27.0;

    let fx3 = fx * fx * fx;
    let fz3 = fz * fz * fz;
    let xr = if fx3 > eps {
        fx3
    } else {
        (116.0 * fx - 16.0) / kappa
    };
    let yr = if l_star > kappa * eps {
        fy * fy * fy
    } else {
        l_star / kappa
    };
    let zr = if fz3 > eps {
        fz3
    } else {
        (116.0 * fz - 16.0) / kappa
    };

    // D50 whitepoint.
    let x_d50 = xr * 0.96422;
    let y_d50 = yr * 1.00000;
    let z_d50 = zr * 0.82521;

    // Bradford adaptation matrix (D50 -> D65).
    let x = 0.955_576_6 * x_d50 + -0.023_039_3 * y_d50 + 0.063_163_6 * z_d50;
    let y = -0.028_289_5 * x_d50 + 1.009_941_6 * y_d50 + 0.021_007_7 * z_d50;
    let z = 0.012_298_2 * x_d50 + -0.020_483 * y_d50 + 1.329_909_8 * z_d50;

    // XYZ(D65) -> linear sRGB.
    let r = 3.240_454_2 * x + -1.537_138_5 * y + -0.498_531_4 * z;
    let g = -0.969_266 * x + 1.876_010_8 * y + 0.041_556 * z;
    let b = 0.055_643_4 * x + -0.204_025_9 * y + 1.057_225_2 * z;

    Some(Rgba {
        r: r.clamp(0.0, 1.0),
        g: g.clamp(0.0, 1.0),
        b: b.clamp(0.0, 1.0),
        a: alpha,
    })
}
