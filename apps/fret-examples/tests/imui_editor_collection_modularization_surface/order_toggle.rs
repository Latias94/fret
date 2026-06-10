pub(super) fn assert_order_toggle_owner_split(collection_source: &str, order_toggle_source: &str) {
    for needle in [
        "pub(super) fn render_collection_order_toggle(",
        "reverse_order_model: &Model<bool>",
        "if reverse_order {",
        "\"Show folder order\"",
        "\"Reverse visible order\"",
        "ui.button_with_options(",
        "kit::ButtonOptions {",
        "\"imui-editor-proof.authoring.imui.collection.order-toggle\"",
        "if !order_toggle.clicked()",
        ".update(reverse_order_model, |value| *value = !*value)",
        "!reverse_order",
    ] {
        assert!(
            order_toggle_source.contains(needle),
            "the demo-local collection order-toggle owner should keep reverse-order button logic explicit; missing `{needle}`"
        );
    }

    for needle in [
        "\"Show folder order\"",
        "\"Reverse visible order\"",
        "\"imui-editor-proof.authoring.imui.collection.order-toggle\"",
        "ui.button_with_options(",
        "kit::ButtonOptions {",
        ".update(&collection_reverse_order_model, |value| *value = !*value)",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route reverse-order button UI through collection/order_toggle.rs; unexpected `{needle}`"
        );
    }
}
