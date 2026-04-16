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

fn advanced_prelude_exports_symbol(symbol: &str) -> bool {
    advanced_prelude_slice()
        .split(';')
        .filter(|statement| statement.contains("pub use "))
        .any(|statement| statement_exports_symbol(statement, symbol))
}

fn statement_exports_symbol(statement: &str, symbol: &str) -> bool {
    let Some(pub_use_start) = statement.find("pub use ") else {
        return false;
    };
    let statement = &statement[pub_use_start + "pub use ".len()..];

    if let Some((_, items)) = statement.rsplit_once("::{") {
        let items = items.trim_end_matches('}');
        return items
            .split(',')
            .filter_map(exported_symbol_name)
            .any(|exported| exported == symbol);
    }

    exported_symbol_name(statement).is_some_and(|exported| exported == symbol)
}

fn exported_symbol_name(item: &str) -> Option<&str> {
    let item = item.trim();
    if item.is_empty() {
        return None;
    }

    if let Some((_, alias)) = item.rsplit_once(" as ") {
        let alias = alias.trim();
        return (alias != "_").then_some(alias);
    }

    let exported = item.rsplit("::").next()?.trim();
    (exported != "_").then_some(exported)
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
    assert!(advanced_prelude.contains("pub use crate::AppRenderCx;"));
    assert!(advanced_prelude_exports_symbol("AppRenderCx"));
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
            !advanced_prelude_exports_symbol(forbidden),
            "advanced prelude should not re-export component noun `{forbidden}`",
        );
    }
    assert!(advanced_prelude_slice().contains("TrackedModelExt as _;"));
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
