use std::any::Any;

pub use fret_selector::Selector;
use fret_selector::ui::SelectorElementContextExt as _;
pub use fret_selector::ui::observed_model_revision;
use fret_ui::{ElementContext, UiHost};

use crate::UiWriter;

/// Memoized derived state helpers for authoring frontends.
///
/// This forwards to `fret_selector::ui::SelectorElementContextExt` using `UiWriter::with_cx_mut`
/// to avoid exposing `ElementContext`'s lifetime parameters in widget surfaces.
pub trait UiWriterSelectorExt<H: UiHost>: UiWriter<H> {
    fn use_selector<Deps, TValue>(
        &mut self,
        deps: impl FnOnce(&mut ElementContext<'_, H>) -> Deps,
        compute: impl FnOnce(&mut ElementContext<'_, H>) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone;
}

impl<H: UiHost, W: UiWriter<H> + ?Sized> UiWriterSelectorExt<H> for W {
    fn use_selector<Deps, TValue>(
        &mut self,
        deps: impl FnOnce(&mut ElementContext<'_, H>) -> Deps,
        compute: impl FnOnce(&mut ElementContext<'_, H>) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        self.with_cx_mut(|cx| cx.use_selector(deps, compute))
    }
}
