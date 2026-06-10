use super::MaterialCatalog;

/// App-owned catalog for renderer-registered visual primitives.
///
/// This is intentionally ecosystem-first: it provides a stable place to cache `MaterialId` handles
/// (and future visual IDs) without leaking backend handles into components.
///
/// Storage: app model / app global state.
///
/// Rationale:
/// - `MaterialId` values are renderer-owned and must be registered via `MaterialService`.
/// - keeping the cache app-owned avoids hidden global state and makes lifecycles explicit.
#[derive(Debug, Default)]
pub struct VisualCatalog {
    pub materials: MaterialCatalog,
}
