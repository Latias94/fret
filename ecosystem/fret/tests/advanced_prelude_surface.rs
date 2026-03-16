const FRET_LIB_RS: &str = include_str!("../src/lib.rs");
const ASYNC_PLAYGROUND_DEMO: &str =
    include_str!("../../../apps/fret-examples/src/async_playground_demo.rs");
const IMUI_EDITOR_PROOF_DEMO: &str =
    include_str!("../../../apps/fret-examples/src/imui_editor_proof_demo.rs");
const ACTION_FIRST_VIEW: &str =
    include_str!("../../../apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs");

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
fn advanced_prelude_stays_advanced_only_instead_of_smuggling_component_surface() {
    let advanced_prelude = advanced_prelude_slice();
    assert!(!advanced_prelude.contains("pub use crate::component::prelude::*;"));
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
fn advanced_prelude_does_not_reexport_component_authoring_nouns() {
    let advanced_prelude = advanced_prelude_slice();
    for forbidden in [
        "UiBuilder",
        "UiPatchTarget",
        "IntoUiElement",
        "UiHost",
        "AnyElement",
        "Model",
        "OverlayController",
        "OverlayRequest",
        "OverlayPresence",
    ] {
        assert!(
            !advanced_prelude.contains(forbidden),
            "advanced prelude should not re-export component noun `{forbidden}`",
        );
    }
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

#[test]
fn advanced_call_sites_import_component_prelude_explicitly_when_needed() {
    for (label, source) in [
        ("async_playground_demo", ASYNC_PLAYGROUND_DEMO),
        ("imui_editor_proof_demo", IMUI_EDITOR_PROOF_DEMO),
        ("action_first_view", ACTION_FIRST_VIEW),
    ] {
        assert!(
            source.contains("advanced::prelude::*"),
            "{label} should stay on the explicit advanced lane",
        );
        assert!(
            source.contains("component::prelude::*"),
            "{label} should add an explicit component lane import when it needs component authoring vocabulary",
        );
    }
}
