#![cfg(feature = "imui")]

use fret_core::{Px, Size};
use fret_runtime::ModelStore;
use fret_ui_kit::imui::{FloatingWindowOptions, FloatingWindowResizeOptions, WindowOptions};

#[test]
fn floating_window_behavior_defaults_compile() {
    let options = FloatingWindowOptions::default();
    assert!(options.movable);
    assert!(options.resizable);
    assert!(options.collapsible);
    assert!(options.closable);
    assert!(options.focus_on_click);
    assert!(options.activate_on_click);
    assert!(options.inputs_enabled);
    assert!(!options.no_inputs);
    assert!(!options.pointer_passthrough);
}

#[test]
fn floating_window_resize_defaults_compile() {
    let options = FloatingWindowResizeOptions::default();
    assert_eq!(options.min_size, Size::new(Px(120.0), Px(72.0)));
    assert!(options.max_size.is_none());
}

#[test]
fn window_option_builders_compile() {
    let mut models = ModelStore::default();
    let open = models.insert(true);
    let resize = FloatingWindowResizeOptions {
        min_size: Size::new(Px(160.0), Px(96.0)),
        max_size: Some(Size::new(Px(640.0), Px(480.0))),
    };
    let behavior = FloatingWindowOptions {
        movable: false,
        resizable: false,
        collapsible: false,
        closable: false,
        focus_on_click: false,
        activate_on_click: false,
        inputs_enabled: false,
        no_inputs: true,
        pointer_passthrough: true,
    };

    let options = WindowOptions::default()
        .with_open(&open)
        .with_size(Size::new(Px(320.0), Px(240.0)))
        .with_resize(resize)
        .with_behavior(behavior);

    assert_eq!(
        options.open.as_ref().map(|model| model.id()),
        Some(open.id())
    );
    assert_eq!(options.size, Some(Size::new(Px(320.0), Px(240.0))));
    assert_eq!(
        options
            .resize
            .map(|resize| (resize.min_size, resize.max_size)),
        Some((resize.min_size, resize.max_size))
    );
    assert!(!options.behavior.movable);
    assert!(!options.behavior.focus_on_click);
    assert!(options.behavior.no_inputs);
    assert!(options.behavior.pointer_passthrough);
}
