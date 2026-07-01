use std::any::Any;
use std::hash::Hash;

use fret_ui::UiHost;

use super::{AppUi, AppUiComponentLaneRequiresExplicitElementsEscapeHatch};

impl<'cx, 'a, H: UiHost> AppUi<'cx, 'a, H> {
    // Lane-sealing barriers for the default app surface.
    //
    // Keep these helpers callable-but-unusable on `AppUi` itself so method resolution stops here
    // instead of falling through the `Deref` to `ElementContext`. Advanced code can still opt into
    // the lower-level substrate explicitly via `cx.elements()`.

    #[doc(hidden)]
    pub fn scope<R>(&mut self, _f: impl FnOnce(&mut Self) -> R)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.keyed(...) or cx.elements().scope(...)")
    }

    #[doc(hidden)]
    pub fn named<R>(&mut self, _name: &str, _f: impl FnOnce(&mut Self) -> R)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.keyed(...) or cx.elements().named(...)")
    }

    #[doc(hidden)]
    pub fn root_state<S: Any, R>(&mut self, _init: impl FnOnce() -> S, _f: impl FnOnce(&mut S) -> R)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.state().local* or cx.elements().root_state(...)")
    }

    #[doc(hidden)]
    pub fn with_state<S: Any, R>(&mut self, _init: impl FnOnce() -> S, _f: impl FnOnce(&mut S) -> R)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.state().local* or cx.elements().root_state(...)")
    }

    #[doc(hidden)]
    pub fn slot_state<S: Any, R>(&mut self, _init: impl FnOnce() -> S, _f: impl FnOnce(&mut S) -> R)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.state().local* or cx.elements().slot_state(...)")
    }

    #[doc(hidden)]
    pub fn keyed_slot_state<K: Hash, S: Any, R>(
        &mut self,
        _key: K,
        _init: impl FnOnce() -> S,
        _f: impl FnOnce(&mut S) -> R,
    ) where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.state().local_keyed(...) or cx.elements().keyed_slot_state(...)")
    }

    #[doc(hidden)]
    pub fn slot_id(&mut self)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.elements().slot_id(...)")
    }

    #[doc(hidden)]
    pub fn keyed_slot_id<K: Hash>(&mut self, _key: K)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.elements().keyed_slot_id(...)")
    }

    #[doc(hidden)]
    pub fn local_model<T: Any>(&mut self, _init: impl FnOnce() -> T)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.state().local_init(...) or cx.elements().local_model(...)")
    }

    #[doc(hidden)]
    pub fn local_model_keyed<K: Hash, T: Any>(&mut self, _key: K, _init: impl FnOnce() -> T)
    where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.state().local_keyed(...) or cx.elements().local_model_keyed(...)")
    }

    #[doc(hidden)]
    pub fn state_for<S: Any, R>(
        &mut self,
        _element: fret_ui::GlobalElementId,
        _init: impl FnOnce() -> S,
        _f: impl FnOnce(&mut S) -> R,
    ) where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.elements().state_for(...)")
    }

    #[doc(hidden)]
    pub fn with_state_for<S: Any, R>(
        &mut self,
        _element: fret_ui::GlobalElementId,
        _init: impl FnOnce() -> S,
        _f: impl FnOnce(&mut S) -> R,
    ) where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.elements().state_for(...)")
    }

    #[doc(hidden)]
    pub fn model_for<T: Any>(
        &mut self,
        _element: fret_ui::GlobalElementId,
        _init: impl FnOnce() -> T,
    ) where
        Self: AppUiComponentLaneRequiresExplicitElementsEscapeHatch,
    {
        unreachable!("use cx.elements().model_for(...)")
    }
}
