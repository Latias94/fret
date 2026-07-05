#[test]
fn imui_floating_windows_demo_uses_local_state_first_bindings() {
    let source = include_str!("../src/imui_floating_windows_demo.rs");
    let compact_source = source.split_whitespace().collect::<String>();

    for needle in [
        ".with_open(&open_a_state)",
        "host.local_state_txn(|tx| tx.set(&clicked_state, true))",
        "a_overlap_clicked_state.paint(cx).value_or_default()",
        "combo_model_with_options(\"imui-float-demo.select.popup\",\"Mode\",&select_mode_state,",
    ] {
        assert!(
            compact_source.contains(&needle.split_whitespace().collect::<String>()),
            "imui_floating_windows_demo should stay on the LocalState-first IMUI surface; missing `{needle}`"
        );
    }

    for forbidden in [
        "fret::advanced::raw",
        "LocalStateElementContextExt",
        "LocalStateRawModelExt",
        "clone_model",
        "models_mut(",
        ".paint_value_in(",
        ".model()",
    ] {
        assert!(
            !source.contains(forbidden),
            "imui_floating_windows_demo should not reopen the raw LocalState bridge: `{forbidden}`"
        );
    }
}
