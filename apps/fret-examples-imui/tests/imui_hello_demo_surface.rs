#[test]
fn imui_hello_demo_uses_local_state_first_bindings() {
    let source = include_str!("../src/imui_hello_demo.rs");
    let compact_source = source.split_whitespace().collect::<String>();

    assert!(
        source.contains("app::AppLocalStateTxnExt as _"),
        "imui_hello_demo should import the app-facing LocalState transaction trait"
    );

    for needle in [
        ".local_state_txn(|tx| tx.update(&count_state, |value| *value += 1))",
        ".checkbox_model(\"Enabled\",&enabled_state)",
        "enabled_state.paint_value(ui.cx_mut())",
    ] {
        assert!(
            compact_source.contains(&needle.split_whitespace().collect::<String>()),
            "imui_hello_demo should stay on the LocalState-first IMUI surface; missing `{needle}`"
        );
    }

    for forbidden in [
        "fret::advanced::raw",
        "LocalStateModelStoreExt",
        "LocalStateRawModelExt",
        "LocalStateElementContextExt",
        ".update_in(",
        ".model()",
        ".paint_value_in(",
    ] {
        assert!(
            !source.contains(forbidden),
            "imui_hello_demo should not reopen the raw LocalState bridge: `{forbidden}`"
        );
    }
}
