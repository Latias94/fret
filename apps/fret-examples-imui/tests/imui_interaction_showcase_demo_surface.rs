#[test]
fn imui_interaction_showcase_demo_uses_local_state_first_bindings() {
    let source = include_str!("../src/imui_interaction_showcase_demo.rs");
    let compact_source = source.split_whitespace().collect::<String>();

    for needle in [
        "app::AppLocalStateTxnExt as _",
        "local_state_txn(|tx| tx.update(&pulse_count, |value| *value += 1))",
        "bookmark_slot.layout_value(ui.cx_mut())",
        "switch_model(\"Autosave snapshots\", &autosave_enabled)",
        "slider_f32_model_with_options(\"Exposure bias\", &exposure_value,",
        "combo_model_with_options(\"imui-showcase.review-mode\",\"Review mode\",&review_mode,",
        "input_text_model_with_options(&draft_note,",
        ".selected_model(&selected_tab)",
        "tx.update(timeline, |events|",
        "tx.set(inspector, ShowcaseInspectorState",
    ] {
        assert!(
            compact_source.contains(&needle.split_whitespace().collect::<String>()),
            "imui_interaction_showcase_demo should stay on the LocalState-first IMUI surface; missing `{needle}`"
        );
    }

    for forbidden in [
        "fret::advanced::raw",
        "LocalStateElementContextExt",
        "LocalStateModelStoreExt",
        "LocalStateRawModelExt",
        "layout_value_in",
        "paint_value_in",
        ".update_in(",
        ".set_in(",
        ".model()",
        "models_mut(",
    ] {
        assert!(
            !source.contains(forbidden),
            "imui_interaction_showcase_demo should not reopen the raw LocalState bridge: `{forbidden}`"
        );
    }
}
