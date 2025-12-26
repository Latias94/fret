use crate::{PathService, TextService};

/// UI runtime services passed to widgets during layout/paint/event handling.
///
/// This is intentionally a single `&mut` handle so runtimes can pass a single renderer-owned
/// service object (similar to how GPUI passes a `Window`/context that provides multiple facilities).
pub trait UiServices: TextService + PathService {}

impl<T> UiServices for T where T: TextService + PathService {}

impl dyn UiServices + '_ {
    pub fn text(&mut self) -> &mut dyn TextService {
        self
    }

    pub fn path(&mut self) -> &mut dyn PathService {
        self
    }
}
