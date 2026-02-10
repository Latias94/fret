//! Controllable vs uncontrolled state helpers (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/use-controllable-state/src/use-controllable-state.tsx`
//!
//! Radix components often support both:
//! - controlled state (`prop` provided by the caller), and
//! - uncontrolled state (`defaultProp` stored internally).
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
///
/// Notes:
/// - This helper intentionally does not provide an `on_change` callback. In Fret, consumers can
///   observe models via `ModelWatchExt` / `observe_model` and react to updates.
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

    let model = cx.with_state(UncontrolledModelState::<T>::default, |st| st.model.clone());
    let model = if let Some(model) = model {
        model
    } else {
        let model = cx.app.models_mut().insert(default_value());
        cx.with_state(UncontrolledModelState::<T>::default, |st| {
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;

    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, Px, Rect, Size, SvgId, SvgService, TextBlobId, TextConstraints, TextInput,
        TextMetrics, TextService,
    };
    use fret_runtime::{FrameId, TickId};
    use fret_ui::UiTree;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    fn bump_frame(app: &mut App) {
        app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    #[test]
    fn use_controllable_model_prefers_controlled_and_does_not_call_default() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let controlled = app.models_mut().insert(123u32);
        let called = Cell::new(0);

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let out = use_controllable_model(cx, Some(controlled.clone()), || {
                called.set(called.get() + 1);
                7u32
            });
            assert!(out.is_controlled());
            assert_eq!(out.model(), controlled);
        });

        assert_eq!(called.get(), 0);
    }

    #[test]
    fn use_controllable_model_creates_one_internal_model_and_reuses_it() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let b = bounds();

        let called = Cell::new(0);
        let model_id_out = Cell::new(None);
        let mut services = FakeServices::default();

        let render = |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices| {
            bump_frame(app);
            let called = &called;
            let model_id_out = &model_id_out;

            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                b,
                "controllable-state-test",
                |cx| {
                    vec![cx.keyed("controllable", |cx| {
                        let out = use_controllable_model(cx, None::<Model<u32>>, || {
                            called.set(called.get() + 1);
                            42u32
                        });
                        model_id_out.set(Some(out.model().id()));
                        cx.spacer(Default::default())
                    })]
                },
            );
            ui.set_root(root);
            ui.layout_all(app, services, b, 1.0);
        };

        render(&mut ui, &mut app, &mut services);
        let first = model_id_out.get().expect("model id after first render");
        assert_eq!(called.get(), 1);

        render(&mut ui, &mut app, &mut services);
        let second = model_id_out.get().expect("model id after second render");
        assert_eq!(called.get(), 1);
        assert_eq!(first, second);
    }
}
