/// Color space metadata for image resources.
///
/// This is a portable, contract-level value type used across the runtime/renderer boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageColorSpace {
    Srgb,
    Linear,
}
