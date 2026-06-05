use std::cell::Cell;

use fret_app::App;
use fret_core::AppWindowId;

use super::*;

#[test]
fn tabs_use_value_model_prefers_controlled_and_does_not_call_default() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let b = bounds();

    let controlled = app.models_mut().insert(Some(Arc::from("a")));
    let called = Cell::new(0);

    fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
        let out = tabs_use_value_model(cx, Some(controlled.clone()), || {
            called.set(called.get() + 1);
            None
        });
        assert!(out.is_controlled());
        assert_eq!(out.model(), controlled);
    });

    assert_eq!(called.get(), 0);
}
