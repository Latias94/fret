#![cfg(feature = "imui")]

use std::sync::Arc;

use fret_ui_kit::imui::{FloatingAreaContext, FloatingAreaOptions};

#[test]
fn floating_area_option_defaults_compile() {
    let options = FloatingAreaOptions::default();
    assert_eq!(options.test_id_prefix, "imui.float_area.area:");
    assert!(options.test_id.is_none());
    assert!(!options.hit_test_passthrough);
    assert!(!options.no_inputs);
}

#[test]
fn floating_area_options_can_override_pass_through_policy() {
    let options = FloatingAreaOptions {
        test_id_prefix: "custom.float:",
        test_id: Some(Arc::from("floating-area")),
        hit_test_passthrough: true,
        no_inputs: true,
    };

    assert_eq!(options.test_id_prefix, "custom.float:");
    assert_eq!(options.test_id.as_deref(), Some("floating-area"));
    assert!(options.hit_test_passthrough);
    assert!(options.no_inputs);
}

#[allow(dead_code)]
fn floating_area_context_reexport_compiles(_context: FloatingAreaContext) {}
