const FRET_LIB_RS: &str = include_str!("../src/lib.rs");

fn advanced_prelude_slice() -> &'static str {
    let advanced_start = FRET_LIB_RS
        .find("pub mod advanced {")
        .expect("advanced module marker should exist");
    let advanced_slice = &FRET_LIB_RS[advanced_start..];
    let prelude_start = advanced_slice
        .find("pub mod prelude {")
        .expect("advanced prelude marker should exist");
    let advanced_end = advanced_slice
        .find("\n}\n\n#[derive(Debug, thiserror::Error)]")
        .expect("advanced module end marker should exist");
    &advanced_slice[prelude_start..advanced_end]
}

#[test]
fn advanced_prelude_builds_on_component_surface_instead_of_ui_kit_blanket_exports() {
    let advanced_prelude = advanced_prelude_slice();
    assert!(advanced_prelude.contains("pub use crate::component::prelude::*;"));
    assert!(!advanced_prelude.contains("pub use fret_ui_kit::prelude::*;"));
}

#[test]
fn advanced_prelude_keeps_manual_assembly_seams_explicit() {
    let advanced_prelude = advanced_prelude_slice();
    assert!(advanced_prelude.contains("pub use crate::advanced::*;"));
    assert!(advanced_prelude.contains("pub use crate::{AppUi, Ui, UiCx};"));
    assert!(advanced_prelude.contains("pub use fret_app::Effect;"));
    assert!(advanced_prelude.contains("pub use fret_core::{AppWindowId, Event, UiServices};"));
    assert!(advanced_prelude.contains("pub use fret_runtime::{ActionId, TypedAction};"));
    assert!(advanced_prelude.contains("pub use fret_ui::{ElementContext, ThemeSnapshot, UiTree};"));
    assert!(advanced_prelude.contains(
        "pub use fret_ui::element::{HoverRegionProps, Length, SemanticsProps, TextProps};",
    ));
}

#[test]
fn advanced_prelude_omits_broad_ui_kit_internals() {
    let advanced_prelude = advanced_prelude_slice();
    for forbidden in [
        "ColorFallback",
        "MarginEdge",
        "SignedMetricRef",
        "WidgetState",
        "CachedSubtreeProps",
        "ImageSamplingHint",
        "merge_override_slot",
        "merge_slot",
        "resolve_override_slot",
        "resolve_slot",
    ] {
        assert!(
            !advanced_prelude.contains(forbidden),
            "advanced prelude should not re-export `{forbidden}` transitively",
        );
    }
}
