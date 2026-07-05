#[test]
fn imui_response_signals_demo_uses_local_state_first_bindings() {
    let source = include_str!("../src/imui_response_signals_demo.rs");
    let compact_source = source.split_whitespace().collect::<String>();

    assert!(
        source.contains("app::AppLocalStateTxnExt as _"),
        "imui_response_signals_demo should import the app-facing LocalState transaction trait"
    );

    for needle in [
        "local_state_txn(|tx| tx.update(&left_clicks, |value| *value += 1))",
        "local_state_txn(|tx| tx.set(&press_holding, click.press_holding()))",
        "checkbox_model(\"Edited checkbox\", &lifecycle_checkbox_value)",
        "slider_f32_model_with_options(\"Edited slider\", &lifecycle_slider_value,",
        "input_text_model_with_options(&lifecycle_text_value,",
        "combo_model_with_options(\"imui-resp-demo.lifecycle-combo-model\",\"Lifecycle combo model\",&lifecycle_combo_model_value,",
        ".selected_model(&trigger_tab_selected)",
        ".test_id(\"imui-resp-demo.trigger-tabs.root\")",
    ] {
        assert!(
            compact_source.contains(&needle.split_whitespace().collect::<String>()),
            "imui_response_signals_demo should stay on the LocalState-first IMUI surface; missing `{needle}`"
        );
    }

    for forbidden in [
        "fret::advanced::raw",
        "LocalStateModelStoreExt",
        "LocalStateRawModelExt",
        ".update_in(",
        ".set_in(",
        ".model()",
    ] {
        assert!(
            !source.contains(forbidden),
            "imui_response_signals_demo should not reopen the raw LocalState bridge: `{forbidden}`"
        );
    }
}
