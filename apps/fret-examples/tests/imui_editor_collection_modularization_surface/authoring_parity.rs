pub(super) fn assert_authoring_parity_owner_split(
    demo_source: &str,
    authoring_parity_source: &str,
    authoring_parity_models_source: &str,
    authoring_parity_surface_source: &str,
    authoring_parity_common_source: &str,
    authoring_parity_declarative_source: &str,
    authoring_parity_imui_source: &str,
    authoring_parity_shared_state_source: &str,
) {
    for needle in [
        "mod common;",
        "mod declarative;",
        "mod imui;",
        "mod models;",
        "mod shared_state;",
        "mod surface;",
        "pub(super) use models::{",
        "AuthoringParityModels",
        "shared_models",
        "drag_assets",
        "outliner_items_model",
        "pub(super) use shared_state::render_shared_state;",
        "pub(super) use surface::render_surface;",
    ] {
        assert!(
            authoring_parity_source.contains(needle),
            "the demo-local authoring parity hub should re-export split owner surfaces; missing `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super) struct AuthoringParityModels {",
        "pub(in super::super) fn shared_models<H: UiHost>(",
        "name: name_model(cx),",
        "gradient_next_id: gradient_next_id_model(cx),",
        "pub(in super::super) fn drag_assets() -> Arc<[ProofDragAsset]> {",
        "super::super::collection::authoring_parity_collection_assets()",
        "pub(in super::super) fn outliner_items() -> Arc<[ProofOutlinerItem]> {",
        "pub(in super::super) fn outliner_items_model<H: UiHost>(",
    ] {
        assert!(
            authoring_parity_models_source.contains(needle),
            "the demo-local authoring parity model owner should own shared proof fixtures; missing `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super) fn render_surface(",
        "fn render_authoring_parity_declarative_group(",
        "fn render_authoring_parity_imui_group",
        "fn build_authoring_parity_gradient_editor(",
        "fn render_authoring_parity_imui_host",
        "fn authoring_parity_shading_items() -> Arc<[EnumSelectItem]>",
        "let asset_chips = drag_assets();",
        "collection::render_collection_first_asset_browser_proof(ui);",
    ] {
        assert!(
            authoring_parity_surface_source.contains(needle),
            "the demo-local authoring parity surface router should own cross-owner wiring; missing `{needle}`"
        );
        assert!(
            !demo_source.contains(needle),
            "imui_editor_proof_demo should delegate authoring parity wiring to split owners; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn authoring_parity_shading_items() -> Arc<[EnumSelectItem]>",
        "pub(super) fn build_authoring_parity_gradient_editor(",
        "pub(super) fn render_authoring_parity_imui_host",
    ] {
        assert!(
            authoring_parity_common_source.contains(needle),
            "the demo-local authoring parity common owner should own shared helpers; missing `{needle}`"
        );
        assert!(
            !demo_source.contains(needle),
            "imui_editor_proof_demo should not own authoring parity common helpers; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn render_authoring_parity_declarative_group(",
        "PropertyGroup::new(\"Declarative authoring\")",
        "build_authoring_parity_gradient_editor(",
    ] {
        assert!(
            authoring_parity_declarative_source.contains(needle),
            "the demo-local authoring parity declarative owner should own declarative composition; missing `{needle}`"
        );
        assert!(
            !demo_source.contains(needle),
            "imui_editor_proof_demo should not own declarative authoring composition; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn render_authoring_parity_imui_group",
        "render_collection_browser(ui);",
        "sortable_row(ui, row.response(), payload)",
        "publish_cross_window_drag_preview_ghost_with_options(",
    ] {
        assert!(
            authoring_parity_imui_source.contains(needle),
            "the demo-local authoring parity imui owner should own IMUI composition; missing `{needle}`"
        );
        assert!(
            !demo_source.contains(needle),
            "imui_editor_proof_demo should not own IMUI authoring composition; unexpected `{needle}`"
        );
    }

    for (name, source, expected_import) in [
        (
            "authoring parity common",
            authoring_parity_common_source,
            "use fret_ui_kit::IntoUiElement;",
        ),
        (
            "authoring parity declarative",
            authoring_parity_declarative_source,
            "use fret_ui_kit::IntoUiElement;",
        ),
        (
            "authoring parity imui",
            authoring_parity_imui_source,
            "use fret_ui_kit::IntoUiElement;",
        ),
        (
            "authoring parity shared state",
            authoring_parity_shared_state_source,
            "use fret_ui_kit::IntoUiElement;",
        ),
        (
            "authoring parity surface",
            authoring_parity_surface_source,
            "use fret_ui_kit::IntoUiElement;",
        ),
    ] {
        assert!(
            source.contains(expected_import),
            "{name} should import the UI element landing capability explicitly; missing `{expected_import}`"
        );

        for forbidden in [
            "use fret::component::prelude::*;",
            "use fret::advanced::prelude::*;",
        ] {
            assert!(
                !source.contains(forbidden),
                "{name} should not reintroduce broad prelude imports: `{forbidden}`"
            );
        }
    }

    assert!(
        authoring_parity_shared_state_source.contains("use fret::app::AppRenderDataExt as _;"),
        "authoring parity shared state should use the app render-data extension surface"
    );
    assert!(
        !authoring_parity_shared_state_source
            .contains("use fret::advanced::view::AppRenderDataExt as _;"),
        "authoring parity shared state should not use the advanced view render-data extension"
    );
}
