//! Shared authoring contracts for ecosystem-level UI frontends.
//!
//! This crate defines small, policy-light traits that allow ecosystem crates to expose authoring
//! helpers without coupling to a specific frontend (e.g. `fret-imui`).
//!
//! Design constraints:
//! - Keep dependencies minimal (`fret-ui` / `fret-core` only).
//! - Do not introduce a second UI runtime: authoring must compile down to the declarative element
//!   taxonomy mounted into `UiTree` (ADR 0028).

use std::hash::Hash;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

/// Minimal authoring surface for immediate-style composition.
///
/// This is intended for ecosystem crates that want to expose helpers that work across multiple
/// authoring frontends while staying policy-light.
pub trait UiWriter<H: UiHost> {
    /// Execute a closure with access to the underlying element context (escape hatch).
    ///
    /// This avoids leaking `ElementContext`'s lifetime parameter through the trait surface while
    /// still enabling advanced integrations when needed.
    fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R;

    /// Append a single element to the current output list.
    fn add(&mut self, element: AnyElement);

    /// Append an iterator of elements to the current output list.
    fn extend<I>(&mut self, elements: I)
    where
        I: IntoIterator<Item = AnyElement>,
    {
        for element in elements {
            self.add(element);
        }
    }

    /// Embed an existing declarative builder into the current output list.
    fn mount<I>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> I)
    where
        I: IntoIterator<Item = AnyElement>,
    {
        let elements: Vec<AnyElement> = self.with_cx_mut(|cx| f(cx).into_iter().collect());
        self.extend(elements);
    }

    /// Create a keyed identity scope.
    ///
    /// This delegates to the runtime's canonical keyed identity mechanism (`ElementContext::keyed`)
    /// so hashing and stable ID rules remain consistent across frontends.
    fn keyed<K: Hash, R>(&mut self, key: K, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {
        self.with_cx_mut(|cx| cx.keyed(key, f))
    }
}

#[cfg(test)]
mod tests {
    use super::UiWriter;
    use fret_ui::UiHost;
    use fret_ui::element::AnyElement;

    // Compile-level smoke: this crate exists primarily to define shared signatures across
    // ecosystem crates. The body is never executed; it only ensures the surface stays usable.
    #[allow(dead_code)]
    fn writer_surface_smoke<H: UiHost>(ui: &mut impl UiWriter<H>) {
        ui.with_cx_mut(|_cx| ());
        ui.extend(Vec::<AnyElement>::new());
        ui.mount(|_cx| Vec::<AnyElement>::new());
        ui.keyed("key", |_cx| ());
        ui.keyed(123_u64, |_cx| ());
    }

    #[test]
    fn ui_writer_compiles() {}
}
