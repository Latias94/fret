//! Open state helpers (Radix-aligned outcomes).
//!
//! Many Radix primitives expose a controlled/uncontrolled `open` surface:
//! - `open` (controlled)
//! - `defaultOpen` (uncontrolled initial value)
//!
//! In Fret, "controlled" maps to a caller-provided `Model<bool>`, while "uncontrolled" maps to an
//! internal `Model<bool>` stored in element state and initialized once.

use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

/// Returns an open-state model that behaves like Radix `useControllableState` (`open` /
/// `defaultOpen`).
pub fn open_use_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<bool>>,
    default_open: impl FnOnce() -> bool,
) -> crate::primitives::controllable_state::ControllableModel<bool> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_open)
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
    fn open_use_model_creates_one_internal_model_and_reuses_it() {
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
                "open-state-test",
                |cx| {
                    vec![cx.keyed("open", |cx| {
                        let out = open_use_model(cx, None::<Model<bool>>, || {
                            called.set(called.get() + 1);
                            true
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
