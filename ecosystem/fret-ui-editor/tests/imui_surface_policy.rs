#![cfg(feature = "imui")]

const IMUI_RS: &str = include_str!("../src/imui.rs");

fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

#[test]
fn imui_module_stays_a_thin_into_element_adapter_layer() {
    let normalized = normalize_ws(IMUI_RS);

    let required_markers = [
        "Optional immediate-mode authoring facade adapters.",
        "This must remain a thin adapter over the declarative, single source-of-truth implementation.",
        "Do not introduce a parallel widget implementation here.",
        "fn add_editor_element<H: UiHost + 'static>(",
        "pub fn text_field<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: TextField) {",
        "pub fn checkbox<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: Checkbox) {",
        "pub fn color_edit<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: ColorEdit) {",
        "pub fn drag_value<H, T>(ui: &mut impl UiWriter<H>, control: DragValue<T>)",
        "pub fn axis_drag_value<H, T>(ui: &mut impl UiWriter<H>, control: AxisDragValue<T>)",
        "pub fn numeric_input<H, T>(ui: &mut impl UiWriter<H>, control: NumericInput<T>)",
        "pub fn slider<H, T>(ui: &mut impl UiWriter<H>, control: Slider<T>)",
        "pub fn enum_select<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: EnumSelect) {",
        "pub fn mini_search_box<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: MiniSearchBox) {",
        "pub fn text_assist_field<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: TextAssistField) {",
        "pub fn icon_button<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: IconButton) {",
        "pub fn field_status_badge<H: UiHost + 'static>(",
        "pub fn vec2_edit<H, T>(ui: &mut impl UiWriter<H>, control: Vec2Edit<T>)",
        "pub fn vec3_edit<H, T>(ui: &mut impl UiWriter<H>, control: Vec3Edit<T>)",
        "pub fn vec4_edit<H, T>(ui: &mut impl UiWriter<H>, control: Vec4Edit<T>)",
        "pub fn transform_edit<H: UiHost + 'static>(ui: &mut impl UiWriter<H>, control: TransformEdit) {",
        "pub fn property_group<H: UiHost + 'static>(",
        "pub fn property_grid<H: UiHost + 'static>(",
        "pub fn gradient_editor<H: UiHost + 'static>(",
        "pub fn property_grid_virtualized<H: UiHost + 'static>(",
        "pub fn inspector_panel<H: UiHost + 'static>(",
    ];
    let forbidden_markers = [
        "pub struct ",
        "pub enum ",
        "Model<",
        "LocalState",
        "ActionCx",
        "OnActivate",
        "fret_ui_kit",
        "fret_ui_shadcn",
        "selector_model",
        "watch(",
    ];

    for marker in required_markers {
        let marker = normalize_ws(marker);
        assert!(
            normalized.contains(&marker),
            "imui.rs should keep the promoted editor adapter surface explicit and auditable"
        );
    }

    for marker in forbidden_markers {
        assert!(
            !IMUI_RS.contains(marker),
            "imui.rs should stay free of declarative control internals and adapter-local state/policy"
        );
    }

    assert_eq!(
        count_occurrences(
            IMUI_RS,
            "add_editor_element(ui, move |cx| control.into_element(cx));",
        ),
        16,
        "imui.rs should keep each promoted control adapter as a one-hop `into_element` forwarder",
    );

    assert_eq!(
        count_occurrences(
            &normalized,
            &normalize_ws(
                "add_editor_element(ui, move |cx| {
                    composite.into_element(cx, header_actions, contents)
                });",
            ),
        ),
        1,
        "property_group should stay a one-hop `into_element` forwarder",
    );

    assert_eq!(
        count_occurrences(
            &normalized,
            &normalize_ws("add_editor_element(ui, move |cx| composite.into_element(cx, rows));"),
        ),
        1,
        "property_grid should stay a one-hop `into_element` forwarder",
    );

    assert_eq!(
        count_occurrences(
            &normalized,
            &normalize_ws("add_editor_element(ui, move |cx| composite.into_element(cx));"),
        ),
        1,
        "gradient_editor should stay a one-hop `into_element` forwarder",
    );

    assert_eq!(
        count_occurrences(
            &normalized,
            &normalize_ws(
                "add_editor_element(ui, move |cx| {
                    composite.into_element(cx, len, key_at, row_at)
                });",
            ),
        ),
        1,
        "property_grid_virtualized should stay a one-hop `into_element` forwarder",
    );

    assert_eq!(
        count_occurrences(
            &normalized,
            &normalize_ws(
                "add_editor_element(ui, move |cx| composite.into_element(cx, toolbar, contents));",
            ),
        ),
        1,
        "inspector_panel should stay a one-hop `into_element` forwarder",
    );
}
