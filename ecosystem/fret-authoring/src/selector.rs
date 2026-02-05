use std::any::Any;
use std::hash::Hash;
use std::panic::Location;

pub use fret_selector::DepsSignature;
pub use fret_selector::Selector;
pub use fret_selector::ui::DepsBuilder;
use fret_selector::ui::SelectorElementContextExt as _;
pub use fret_selector::ui::observed_global_token;
pub use fret_selector::ui::observed_model_revision;
use fret_ui::{ElementContext, UiHost};

use crate::UiWriter;

/// Memoized derived state helpers for authoring frontends.
///
/// This forwards to `fret_selector::ui::SelectorElementContextExt` using `UiWriter::with_cx_mut`
/// to avoid exposing `ElementContext`'s lifetime parameters in widget surfaces.
pub trait UiWriterSelectorExt<H: UiHost>: UiWriter<H> {
    #[track_caller]
    fn use_selector<Deps, TValue>(
        &mut self,
        deps: impl FnOnce(&mut ElementContext<'_, H>) -> Deps,
        compute: impl FnOnce(&mut ElementContext<'_, H>) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone;

    #[track_caller]
    fn use_selector_keyed<K: Hash, Deps, TValue>(
        &mut self,
        key: K,
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
        let callsite = Location::caller();
        self.with_cx_mut(|cx| cx.use_selector_at(callsite, deps, compute))
    }

    fn use_selector_keyed<K: Hash, Deps, TValue>(
        &mut self,
        key: K,
        deps: impl FnOnce(&mut ElementContext<'_, H>) -> Deps,
        compute: impl FnOnce(&mut ElementContext<'_, H>) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        let callsite = Location::caller();
        self.with_cx_mut(|cx| cx.use_selector_keyed_at(callsite, key, deps, compute))
    }
}
