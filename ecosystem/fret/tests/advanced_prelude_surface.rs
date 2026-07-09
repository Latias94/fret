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

fn advanced_public_slice() -> &'static str {
    let advanced_start = FRET_LIB_RS
        .find("pub mod advanced {")
        .expect("advanced module marker should exist");
    let advanced_slice = &FRET_LIB_RS[advanced_start..];
    let advanced_end = advanced_slice
        .find("\n}\n\n#[derive(Debug, thiserror::Error)]")
        .expect("advanced module end marker should exist");
    &advanced_slice[..advanced_end]
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
    let advanced_public = advanced_public_slice();
    assert!(advanced_prelude.contains("pub use crate::AppRenderCx;"));
    assert!(advanced_prelude_exports_symbol("AppRenderCx"));
    assert!(advanced_prelude.contains("pub use crate::AppComponentCx;"));
    assert!(advanced_prelude.contains("pub use crate::{AppUi, Ui};"));
    assert!(advanced_prelude.contains("pub use crate::advanced::KernelApp;"));
    assert!(advanced_prelude.contains("pub use crate::advanced::driver::{"));
    assert!(advanced_prelude.contains("pub use crate::advanced::interop::embedded_viewport::{"));
    assert!(advanced_prelude.contains("pub use crate::advanced::kernel;"));
    assert!(advanced_public.contains("LocalStateRawModelExt"));
    assert!(advanced_public.contains("LocalStateModelStoreExt"));
    assert!(advanced_public.contains("LocalStateElementContextExt"));
    assert!(advanced_prelude.contains("pub use fret_app::Effect;"));
    assert!(advanced_prelude.contains("pub use fret_core::{AppWindowId, Event, UiServices};"));
    assert!(advanced_prelude.contains("pub use fret_runtime::{ActionId, TypedAction};"));
    assert!(advanced_prelude.contains("pub use fret_ui::{ElementContext, ThemeSnapshot};"));
    assert!(!advanced_prelude_exports_symbol("UiTree"));
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
        "ModelStore",
        "TrackedModelExt",
        "OverlayController",
        "OverlayRequest",
        "OverlayPresence",
    ] {
        assert!(
            !advanced_prelude_exports_symbol(forbidden),
            "advanced prelude should not re-export component noun `{forbidden}`",
        );
    }
    assert!(!advanced_prelude_slice().contains("TrackedModelExt"));
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
    assert!(
        ASYNC_PLAYGROUND_DEMO.contains("use fret::app::prelude::*;"),
        "async_playground_demo should stay on the default app lane for app-owned state and view code",
    );
    assert!(
        ASYNC_PLAYGROUND_DEMO.contains("use fret::app::{")
            && ASYNC_PLAYGROUND_DEMO.contains("AppElement, AppRenderContext, LocalState"),
        "async_playground_demo should name extra app helper seams explicitly",
    );
    assert!(
        !ASYNC_PLAYGROUND_DEMO.contains("advanced::prelude::*")
            && !ASYNC_PLAYGROUND_DEMO.contains("component::prelude::*"),
        "async_playground_demo should not reacquire broad advanced/component preludes",
    );

    assert!(
        IMUI_EDITOR_PROOF_DEMO.contains("use fret::advanced::KernelApp;"),
        "imui_editor_proof_demo should name the kernel app seam explicitly",
    );
    assert!(
        IMUI_EDITOR_PROOF_DEMO.contains("use fret::advanced::driver::{UiAppDriver, ViewElements};"),
        "imui_editor_proof_demo should import driver seams from the advanced driver module",
    );
    assert!(
        IMUI_EDITOR_PROOF_DEMO.contains("use fret::app::{ElementContextAccess, View};"),
        "imui_editor_proof_demo should import app-facing view seams explicitly",
    );
    assert!(
        !IMUI_EDITOR_PROOF_DEMO.contains("advanced::prelude::*")
            && !IMUI_EDITOR_PROOF_DEMO.contains("component::prelude::*"),
        "imui_editor_proof_demo should not reacquire broad advanced/component preludes",
    );

    assert!(
        !ACTION_FIRST_VIEW.contains("advanced::prelude::*"),
        "action_first_view should stay on the default app-facing lane"
    );
    assert!(
        !ACTION_FIRST_VIEW.contains("advanced::raw"),
        "action_first_view should not teach raw model/store seams"
    );
    assert!(
        ACTION_FIRST_VIEW.contains("fret::app::view_child_with(")
            && ACTION_FIRST_VIEW.contains("move |view: &mut ActionFirstViewRuntimeDemo|"),
        "action_first_view should use the app-facing embedded View helper"
    );
}
