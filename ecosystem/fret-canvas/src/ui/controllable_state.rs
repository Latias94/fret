//! Controllable vs uncontrolled state helpers (Radix-aligned outcomes).
//!
//! In Fret, "controlled" maps to "a caller-provided `Model<T>`", while "uncontrolled" maps to an
//! internal `Model<T>` stored in element state and initialized once from `default_value`.

use std::marker::PhantomData;

use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

#[derive(Debug, Clone)]
pub struct ControllableModel<T> {
    model: Model<T>,
    controlled: bool,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> ControllableModel<T> {
    pub fn model(&self) -> Model<T> {
        self.model.clone()
    }

    pub fn is_controlled(&self) -> bool {
        self.controlled
    }
}

/// Returns a `Model<T>` that behaves like Radix `useControllableState`:
/// - if `controlled` is provided, it is used directly
/// - otherwise an internal model is created (once) using `default_value`
#[track_caller]
pub fn use_controllable_model<T: Clone + 'static, H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<T>>,
    default_value: impl FnOnce() -> T,
) -> ControllableModel<T> {
    if let Some(controlled) = controlled {
        return ControllableModel {
            model: controlled,
            controlled: true,
            _phantom: PhantomData,
        };
    }

    struct UncontrolledModelState<T> {
        model: Option<Model<T>>,
    }

    impl<T> Default for UncontrolledModelState<T> {
        fn default() -> Self {
            Self { model: None }
        }
    }

    let slot = cx.slot_id();
    let model = cx.state_for(slot, UncontrolledModelState::<T>::default, |st| {
        st.model.clone()
    });
    let model = if let Some(model) = model {
        model
    } else {
        let model = cx.app.models_mut().insert(default_value());
        cx.state_for(slot, UncontrolledModelState::<T>::default, |st| {
            st.model = Some(model.clone());
        });
        model
    };

    ControllableModel {
        model,
        controlled: false,
        _phantom: PhantomData,
    }
}
