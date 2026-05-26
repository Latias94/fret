use fret_core::scene::ImageSamplingHint;
use fret_core::{Color, SvgFit, ViewportFit};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugDrawImageOptions {
    pub fit: ViewportFit,
    pub sampling: ImageSamplingHint,
    pub opacity: f32,
}

impl Default for DebugDrawImageOptions {
    fn default() -> Self {
        Self {
            fit: ViewportFit::Stretch,
            sampling: ImageSamplingHint::Default,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugDrawImageQuadOptions {
    pub sampling: ImageSamplingHint,
    pub tint: Color,
    pub opacity: f32,
}

impl Default for DebugDrawImageQuadOptions {
    fn default() -> Self {
        Self {
            sampling: ImageSamplingHint::Default,
            tint: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugDrawImageMeshOptions {
    pub sampling: ImageSamplingHint,
    pub opacity: f32,
}

impl Default for DebugDrawImageMeshOptions {
    fn default() -> Self {
        Self {
            sampling: ImageSamplingHint::Default,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugDrawSvgOptions {
    pub fit: SvgFit,
    pub opacity: f32,
}

impl Default for DebugDrawSvgOptions {
    fn default() -> Self {
        Self {
            fit: SvgFit::Stretch,
            opacity: 1.0,
        }
    }
}
