use fret_ui_headless::table::TanStackTableState;

#[test]
fn tanstack_v8_state_json_semantics_parity_omitted_vs_explicit_defaults() {
    struct Case {
        id: &'static str,
        input: serde_json::Value,
        expected: serde_json::Value,
    }

    fn roundtrip_with_shape(input: &serde_json::Value) -> serde_json::Value {
        let source = TanStackTableState::from_json(input).expect("tanstack state json");
        let state = source.to_table_state().expect("to_table_state");
        let exported = TanStackTableState::from_table_state_with_shape(&state, &source);
        exported.to_json().expect("to_json")
    }

    let cases = [
        Case {
            id: "empty_object_omits_all",
            input: serde_json::json!({}),
            expected: serde_json::json!({}),
        },
        Case {
            id: "explicit_empty_arrays_are_preserved",
            input: serde_json::json!({
                "sorting": [],
                "columnFilters": [],
                "grouping": [],
                "columnOrder": [],
            }),
            expected: serde_json::json!({
                "sorting": [],
                "columnFilters": [],
                "grouping": [],
                "columnOrder": [],
            }),
        },
        Case {
            id: "explicit_empty_maps_and_structs_are_preserved",
            input: serde_json::json!({
                "columnSizing": {},
                "columnVisibility": {},
                "rowSelection": {},
                "columnPinning": { "left": [], "right": [] },
                "rowPinning": { "top": [], "bottom": [] },
                "expanded": {},
            }),
            expected: serde_json::json!({
                "columnSizing": {},
                "columnVisibility": {},
                "rowSelection": {},
                "columnPinning": { "left": [], "right": [] },
                "rowPinning": { "top": [], "bottom": [] },
                "expanded": {},
            }),
        },
        Case {
            id: "explicit_null_option_fields_are_preserved",
            input: serde_json::json!({
                "globalFilter": null,
                "pagination": null,
                "expanded": null,
                "rowPinning": null,
                "rowSelection": null,
                "columnPinning": null,
                "columnVisibility": null,
                "columnSizingInfo": null,
            }),
            expected: serde_json::json!({
                "globalFilter": null,
                "pagination": null,
                "expanded": null,
                "rowPinning": null,
                "rowSelection": null,
                "columnPinning": null,
                "columnVisibility": null,
                "columnSizingInfo": null,
            }),
        },
        Case {
            id: "column_sizing_info_is_canonicalized_when_present",
            input: serde_json::json!({
                "columnSizingInfo": {
                    "isResizingColumn": false
                }
            }),
            expected: serde_json::json!({
                "columnSizingInfo": {
                    "columnSizingStart": [],
                    "deltaOffset": null,
                    "deltaPercentage": null,
                    "isResizingColumn": false,
                    "startOffset": null,
                    "startSize": null
                }
            }),
        },
        Case {
            id: "non_default_values_are_emitted_even_when_source_omits",
            input: serde_json::json!({}),
            expected: serde_json::json!({
                "sorting": [{ "id": "cpu", "desc": true }]
            }),
        },
    ];

    for case in cases {
        let actual = if case.id == "non_default_values_are_emitted_even_when_source_omits" {
            let source = TanStackTableState::from_json(&case.input).expect("tanstack state json");
            let mut state = source.to_table_state().expect("to_table_state");
            state.sorting.push(fret_ui_headless::table::SortSpec {
                column: std::sync::Arc::<str>::from("cpu"),
                desc: true,
            });
            let exported = TanStackTableState::from_table_state_with_shape(&state, &source);
            exported.to_json().expect("to_json")
        } else {
            roundtrip_with_shape(&case.input)
        };

        assert_eq!(actual, case.expected, "case {} mismatch", case.id);
    }
}
