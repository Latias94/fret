//! External drag-and-drop payload retrieval contracts.
//!
//! Fret uses a token-based model for external drops so the UI/runtime can stay portable:
//! the backend captures an OS drop payload and assigns an opaque `ExternalDropToken`.
//! The UI can later request reading the payload via effects.

pub use fret_core::ExternalDropReadLimits;

use fret_core::{ExternalDropDataEvent, ExternalDropToken};

pub trait ExternalDropProvider {
    fn read_all(
        &mut self,
        token: ExternalDropToken,
        limits: ExternalDropReadLimits,
    ) -> Option<ExternalDropDataEvent>;

    fn release(&mut self, token: ExternalDropToken);
}
