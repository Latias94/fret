use crate::{MaterialService, PathService, SvgService, TextService};

/// UI runtime services passed to widgets during layout/paint/event handling.
///
/// This is intentionally a single `&mut` handle so runtimes can pass a single renderer-owned
/// service object (similar to how GPUI passes a `Window`/context that provides multiple facilities).
pub trait UiServices: TextService + PathService + SvgService + MaterialService {}

impl<T> UiServices for T where T: TextService + PathService + SvgService + MaterialService {}

impl dyn UiServices + '_ {
    pub fn text(&mut self) -> &mut dyn TextService {
        self
    }

    pub fn path(&mut self) -> &mut dyn PathService {
        self
    }

    pub fn svg(&mut self) -> &mut dyn SvgService {
        self
    }

    pub fn materials(&mut self) -> &mut dyn MaterialService {
        self
    }
}
