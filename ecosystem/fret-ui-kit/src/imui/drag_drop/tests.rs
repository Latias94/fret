use super::super::{DragSourceOptions, DropTargetOptions, ResponseExt};
use super::{drag_source_with_options, drop_target_with_options};

use fret_app::App;
use fret_authoring::UiWriter;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

struct TestWriter<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    out: &'cx mut Vec<AnyElement>,
}

impl<'cx, 'a, H: UiHost> UiWriter<H> for TestWriter<'cx, 'a, H> {
    fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {
        f(self.cx)
    }

    fn add(&mut self, element: AnyElement) {
        self.out.push(element);
    }
}

#[test]
fn drag_source_returns_inactive_without_trigger_id() {
    let mut app = App::new();
    fret_ui::elements::with_element_cx(
        &mut app,
        Default::default(),
        Default::default(),
        "test",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };
            let response = drag_source_with_options(
                &mut ui,
                ResponseExt::default(),
                42_u32,
                DragSourceOptions::default(),
            );
            assert!(!response.active());
            assert!(out.is_empty());
        },
    );
}

#[test]
fn drop_target_returns_empty_without_trigger_id() {
    let mut app = App::new();
    fret_ui::elements::with_element_cx(
        &mut app,
        Default::default(),
        Default::default(),
        "test",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };
            let response = drop_target_with_options::<_, _, u32>(
                &mut ui,
                ResponseExt::default(),
                DropTargetOptions::default(),
            );
            assert!(!response.active());
            assert!(!response.over());
            assert!(!response.delivered());
            assert!(response.preview_payload().is_none());
            assert!(response.delivered_payload().is_none());
            assert!(out.is_empty());
        },
    );
}
