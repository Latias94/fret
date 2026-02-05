use std::any::Any;
use std::panic::Location;

use fret_runtime::Model;
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::Selector;

/// UI sugar for memoized derived state.
///
/// Important: the `deps` closure must **observe** the dependencies it encodes (models/globals)
/// every frame. The selector only decides whether to recompute the expensive `compute` closure.
pub trait SelectorElementContextExt {
    #[track_caller]
    fn use_selector<Deps, TValue>(
        &mut self,
        deps: impl FnOnce(&mut Self) -> Deps,
        compute: impl FnOnce(&mut Self) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone;
}

impl<'a, H: UiHost> SelectorElementContextExt for ElementContext<'a, H> {
    #[track_caller]
    fn use_selector<Deps, TValue>(
        &mut self,
        deps: impl FnOnce(&mut Self) -> Deps,
        compute: impl FnOnce(&mut Self) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        let loc = Location::caller();
        let key = (loc.file(), loc.line(), loc.column());

        self.keyed(key, |cx| {
            let deps_value = deps(cx);

            let cached = cx.with_state(Selector::<Deps, TValue>::new, |selector| {
                selector.get_if_deps(&deps_value).cloned()
            });

            if let Some(value) = cached {
                return value;
            }

            let value = compute(cx);
            cx.with_state(Selector::<Deps, TValue>::new, |selector| {
                selector.set(deps_value, value.clone());
            });
            value
        })
    }
}

pub fn observed_model_revision<T: Any, H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: &Model<T>,
    invalidation: Invalidation,
) -> Option<u64> {
    cx.observe_model(model, invalidation);
    cx.app.models().revision(model)
}
