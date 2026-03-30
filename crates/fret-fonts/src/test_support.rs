use crate::BundledFontFaceSpec;

/// Test-only helper that converts bundled face specs into owned font blobs.
///
/// This exists for deterministic renderer/shaper tests that explicitly inject bundled fonts into a
/// raw-byte API such as `Renderer::add_fonts(...)` or `ParleyShaper::add_fonts(...)`.
///
/// Production/runtime code should stay on the manifest/asset contract instead:
/// - publish `asset_entries()` into the shared runtime asset resolver,
/// - resolve bytes from `asset_request()`,
/// - and keep startup baselines anchored on logical asset identity.
pub fn face_blobs<'a, I>(faces: I) -> impl Iterator<Item = Vec<u8>> + 'a
where
    I: IntoIterator<Item = &'a BundledFontFaceSpec>,
    I::IntoIter: 'a,
{
    faces.into_iter().map(|face| face.bytes.to_vec())
}
