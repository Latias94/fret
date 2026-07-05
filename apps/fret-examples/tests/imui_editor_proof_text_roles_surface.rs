#[test]
fn imui_editor_proof_main_fixed_text_uses_shared_roles() {
    let main_source = include_str!("../src/imui_editor_proof_demo.rs");
    let helper_source = include_str!("../src/imui_editor_proof_demo/proof_helpers.rs");
    let authoring_parity_source = include_str!("../src/imui_editor_proof_demo/authoring_parity.rs");
    let authoring_parity_shared_state_source =
        include_str!("../src/imui_editor_proof_demo/authoring_parity/shared_state.rs");

    for needle in [
        "fn proof_imui_section_text(",
        "fn proof_imui_readout_text(",
        "fn proof_imui_compact_paragraph_text(",
        "use fret::AppComponentCx;",
        "use fret::advanced::KernelApp;",
        "use fret::app::AppRenderDataExt as _;",
        "use fret_ui_kit::IntoUiElement;",
        "decl_text::text_section_chrome_label(cx, text)",
        "decl_text::text_control_readout(cx, text)",
        "decl_text::text_compact_paragraph(cx, text)",
    ] {
        assert!(
            helper_source.contains(needle),
            "imui_editor_proof_demo text helper owner should use shared role helpers; missing `{needle}`"
        );
    }

    for forbidden in [
        "use fret::advanced::prelude::*;",
        "use fret::component::prelude::*;",
        "advanced::prelude::*",
        "component::prelude::*",
    ] {
        assert!(
            !helper_source.contains(forbidden),
            "imui_editor_proof_demo text helper owner should not rely on broad prelude imports: `{forbidden}`"
        );
    }

    for needle in [
        "imui editor-grade proof (M7): docking + multi-window + viewport surfaces",
        "single-window mode enabled",
        "authoring parity proof: shared models",
        "shared state readout: each declarative/imui pair",
        "fret-ui-editor (M2): PropertyGroup + PropertyGrid + search assist",
    ] {
        assert!(
            main_source.contains(needle),
            "imui_editor_proof_demo main text should use shared role helpers; missing `{needle}`"
        );
    }

    let combined_source = format!(
        "{main_source}\n{helper_source}\n{authoring_parity_source}\n{authoring_parity_shared_state_source}"
    );
    for needle in [
        "fret_ui_kit::ui::text(",
        "let headline =",
        "let parity_intro =",
        "let parity_state_hint =",
        "let editor_label =",
        ".font_semibold()",
        ".text_xs()",
    ] {
        assert!(
            !combined_source.contains(needle),
            "imui_editor_proof_demo main text should not hand-roll local text styling; unexpected `{needle}`"
        );
    }
}
