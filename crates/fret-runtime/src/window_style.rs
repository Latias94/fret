use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowRole {
    Main,
    #[default]
    Auxiliary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskbarVisibility {
    Show,
    Hide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationPolicy {
    Activates,
    NonActivating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowZLevel {
    Normal,
    AlwaysOnTop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowHitTestRequestV1 {
    /// Normal OS hit testing (default).
    Normal,
    /// Window ignores pointer hit testing (click-through).
    PassthroughAll,
    /// Window is passthrough by default, but interactive within the union of regions.
    PassthroughRegions { regions: Vec<WindowHitTestRegionV1> },
}

/// A hit-test region in window *client* coordinates (logical pixels).
///
/// The region union defines where the window should remain interactive when
/// `WindowHitTestRequestV1::PassthroughRegions` is effective.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WindowHitTestRegionV1 {
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    /// Rounded-rect with a single radius (applied to all corners).
    RRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowHitTestRegionsSignatureV1 {
    pub fingerprint64: u64,
}

impl WindowHitTestRegionV1 {
    fn canonicalize_f32(v: f32) -> f32 {
        if !v.is_finite() {
            return 0.0;
        }
        if v == 0.0 {
            // Canonicalize -0.0.
            return 0.0;
        }
        v
    }

    fn quantize_64(v: f32) -> i64 {
        let v = Self::canonicalize_f32(v);
        (v * 64.0).round() as i64
    }

    fn canonical_key(&self) -> (u8, i64, i64, i64, i64, i64) {
        match *self {
            WindowHitTestRegionV1::Rect {
                x,
                y,
                width,
                height,
            } => (
                0,
                Self::quantize_64(x),
                Self::quantize_64(y),
                Self::quantize_64(width),
                Self::quantize_64(height),
                0,
            ),
            WindowHitTestRegionV1::RRect {
                x,
                y,
                width,
                height,
                radius,
            } => (
                1,
                Self::quantize_64(x),
                Self::quantize_64(y),
                Self::quantize_64(width),
                Self::quantize_64(height),
                Self::quantize_64(radius),
            ),
        }
    }
}

pub fn canonicalize_hit_test_regions_v1(
    mut regions: Vec<WindowHitTestRegionV1>,
) -> Vec<WindowHitTestRegionV1> {
    fn canonical(v: f32) -> f32 {
        if !v.is_finite() {
            return 0.0;
        }
        if v == 0.0 {
            return 0.0;
        }
        v
    }

    regions.retain_mut(|r| match r {
        WindowHitTestRegionV1::Rect {
            x,
            y,
            width,
            height,
        } => {
            *x = canonical(*x);
            *y = canonical(*y);
            *width = canonical(*width).max(0.0);
            *height = canonical(*height).max(0.0);
            *width > 0.0 && *height > 0.0
        }
        WindowHitTestRegionV1::RRect {
            x,
            y,
            width,
            height,
            radius,
        } => {
            *x = canonical(*x);
            *y = canonical(*y);
            *width = canonical(*width).max(0.0);
            *height = canonical(*height).max(0.0);
            let w = *width;
            let h = *height;
            if w <= 0.0 || h <= 0.0 {
                return false;
            }
            let max_r = 0.5 * w.min(h);
            *radius = canonical(*radius).clamp(0.0, max_r);
            true
        }
    });

    regions.sort_by_key(|r| r.canonical_key());
    regions
}

pub fn hit_test_regions_signature_v1(
    regions: &[WindowHitTestRegionV1],
) -> (String, WindowHitTestRegionsSignatureV1) {
    fn fnv1a_64(bytes: &[u8]) -> u64 {
        const FNV1A_OFFSET: u64 = 0xcbf29ce484222325;
        const FNV1A_PRIME: u64 = 0x00000100000001B3;
        let mut hash = FNV1A_OFFSET;
        for &b in bytes {
            hash ^= b as u64;
            hash = hash.wrapping_mul(FNV1A_PRIME);
        }
        hash
    }

    let mut sig = String::from("hit_test_regions_v1;u64px=1/64;");
    for r in regions {
        match *r {
            WindowHitTestRegionV1::Rect {
                x,
                y,
                width,
                height,
            } => {
                sig.push_str(&format!(
                    "rect(x={},y={},w={},h={});",
                    WindowHitTestRegionV1::quantize_64(x),
                    WindowHitTestRegionV1::quantize_64(y),
                    WindowHitTestRegionV1::quantize_64(width),
                    WindowHitTestRegionV1::quantize_64(height),
                ));
            }
            WindowHitTestRegionV1::RRect {
                x,
                y,
                width,
                height,
                radius,
            } => {
                sig.push_str(&format!(
                    "rrect(x={},y={},w={},h={},r={});",
                    WindowHitTestRegionV1::quantize_64(x),
                    WindowHitTestRegionV1::quantize_64(y),
                    WindowHitTestRegionV1::quantize_64(width),
                    WindowHitTestRegionV1::quantize_64(height),
                    WindowHitTestRegionV1::quantize_64(radius),
                ));
            }
        }
    }

    let fingerprint64 = fnv1a_64(sig.as_bytes());
    (sig, WindowHitTestRegionsSignatureV1 { fingerprint64 })
}

/// Global window opacity hint (best-effort).
///
/// This is not per-pixel transparency. The value is expressed as an 8-bit alpha where:
/// - `0` = fully transparent (may be treated as hidden on some platforms),
/// - `255` = fully opaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WindowOpacity(pub u8);

impl WindowOpacity {
    pub fn from_f32(opacity: f32) -> Self {
        let a = opacity.clamp(0.0, 1.0);
        let byte = (255.0 * a).round().clamp(0.0, 255.0) as u8;
        Self(byte)
    }

    pub fn as_f32(self) -> f32 {
        (self.0 as f32) / 255.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowDecorationsRequest {
    /// Platform default decorations.
    System,
    /// Request a frameless window (client-drawn).
    None,
    /// Request server-side decorations (Wayland only; best-effort).
    Server,
    /// Request client-side decorations (Wayland only; best-effort).
    Client,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowBackgroundMaterialRequest {
    /// Explicitly disable OS-provided background materials (opaque/default backdrop).
    None,
    /// Request platform default material for a utility window class, if any.
    SystemDefault,
    /// Windows 11-style Mica (best-effort).
    Mica,
    /// Acrylic/blurred translucent backdrop (best-effort).
    Acrylic,
    /// macOS vibrancy-style backdrop (best-effort).
    Vibrancy,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WindowStyleRequest {
    pub taskbar: Option<TaskbarVisibility>,
    pub activation: Option<ActivationPolicy>,
    pub z_level: Option<WindowZLevel>,
    pub decorations: Option<WindowDecorationsRequest>,
    pub resizable: Option<bool>,
    /// Requests a transparent composited window background (best-effort).
    pub transparent: Option<bool>,
    /// Optional request for OS-provided background materials (best-effort).
    pub background_material: Option<WindowBackgroundMaterialRequest>,
    /// Optional request for window-level pointer hit testing (best-effort).
    pub hit_test: Option<WindowHitTestRequestV1>,
    /// Request global window opacity (not per-pixel transparency), best-effort.
    pub opacity: Option<WindowOpacity>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalize_regions_clamps_and_sorts() {
        let regions = vec![
            WindowHitTestRegionV1::RRect {
                x: 10.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
                radius: 999.0,
            },
            WindowHitTestRegionV1::Rect {
                x: f32::NAN,
                y: 0.0,
                width: 0.0,
                height: 10.0,
            },
            WindowHitTestRegionV1::Rect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
        ];

        let out = canonicalize_hit_test_regions_v1(regions);
        assert_eq!(out.len(), 2);

        // Sorted: Rect first.
        assert!(matches!(out[0], WindowHitTestRegionV1::Rect { .. }));

        match out[1] {
            WindowHitTestRegionV1::RRect { radius, .. } => {
                // radius clamped to min(w,h)/2 = 5
                assert_eq!(radius, 5.0);
            }
            _ => panic!("expected rrect"),
        }

        let (_sig, fp) = hit_test_regions_signature_v1(&out);
        assert_ne!(fp.fingerprint64, 0);
    }
}
