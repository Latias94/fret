#[test]
fn data_table_selection_snippets_keep_action_first_surface() {
    for source in [
        include_str!("../src/ui/snippets/data_table/basic_demo.rs"),
        include_str!("../src/ui/snippets/data_table/guide_demo.rs"),
    ] {
        assert!(
            source.contains("cx.actions().models::<act::ToggleAllPageRows>"),
            "data_table snippets should register header select-all through the action-first models surface"
        );
        assert!(
            source.contains("cx.actions().payload_models::<act::ToggleRowSelected>"),
            "data_table snippets should register row selection through the payload_models surface"
        );
        assert!(
            source.contains(".action(act::ToggleAllPageRows)"),
            "data_table snippets should bind the header checkbox through `.action(...)`"
        );
        assert!(
            source.contains(".action(act::ToggleRowSelected)"),
            "data_table snippets should bind row checkboxes through `.action(...)`"
        );
        assert!(
            source.contains(".action_payload("),
            "data_table row checkboxes should keep row identity on `.action_payload(...)`"
        );
        assert!(
            !source.contains("command_on_command_for("),
            "data_table selection snippets should not teach a root command handler for selection"
        );
        assert!(
            !source.contains(".on_click(CommandId::new("),
            "data_table selection snippets should not fall back to command-id click handlers for selection"
        );
    }
}

#[test]
fn data_table_row_action_menus_keep_action_first_surface() {
    let basic = include_str!("../src/ui/snippets/data_table/basic_demo.rs");
    assert!(
        basic.contains(".action(act::CopyPaymentId)"),
        "basic data_table row actions should bind copy through `.action(...)`"
    );
    assert!(
        basic.contains(".action(act::ViewCustomer)"),
        "basic data_table row actions should bind customer view through `.action(...)`"
    );
    assert!(
        basic.contains(".action(act::ViewPaymentDetails)"),
        "basic data_table row actions should bind payment details through `.action(...)`"
    );
    assert!(
        !basic.contains(".on_select(CommandId::new("),
        "basic data_table row actions should not fall back to command-id menu handlers"
    );

    let guide = include_str!("../src/ui/snippets/data_table/guide_demo.rs");
    assert!(
        guide.contains(".action(act::EditRow)"),
        "guide data_table row actions should bind edit through `.action(...)`"
    );
    assert!(
        guide.contains(".action(act::CopyRow)"),
        "guide data_table row actions should bind copy through `.action(...)`"
    );
    assert!(
        guide.contains(".action(act::DeleteRow)"),
        "guide data_table row actions should bind delete through `.action(...)`"
    );
    assert!(
        !guide.contains(".on_select(CommandId::new("),
        "guide data_table row actions should not fall back to command-id menu handlers"
    );

    let rtl = include_str!("../src/ui/snippets/data_table/rtl_demo.rs");
    assert!(
        rtl.contains(".action(act::CopyPaymentId)"),
        "rtl data_table row actions should bind copy through `.action(...)`"
    );
    assert!(
        rtl.contains(".action(act::ViewCustomer)"),
        "rtl data_table row actions should bind customer view through `.action(...)`"
    );
    assert!(
        rtl.contains(".action(act::ViewPaymentDetails)"),
        "rtl data_table row actions should bind payment details through `.action(...)`"
    );
    assert!(
        !rtl.contains(".on_select(CommandId::new("),
        "rtl data_table row actions should not fall back to command-id menu handlers"
    );
}
