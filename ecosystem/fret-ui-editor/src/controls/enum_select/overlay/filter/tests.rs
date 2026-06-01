use super::filter_enum_select_items;
use crate::controls::enum_select::EnumSelectItem;

fn sample_items() -> Vec<EnumSelectItem> {
    vec![
        EnumSelectItem::new("matcap", "Material / Matcap"),
        EnumSelectItem::new("emission", "Lighting / Emission"),
        EnumSelectItem::new("uv_map", "Geometry / UV Map"),
    ]
}

#[test]
fn enum_select_filter_matches_label_case_insensitively() {
    let filtered = filter_enum_select_items(&sample_items(), " MATCAP ");

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].value.as_ref(), "matcap");
}

#[test]
fn enum_select_filter_matches_value_case_insensitively() {
    let filtered = filter_enum_select_items(&sample_items(), "UV_");

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].label.as_ref(), "Geometry / UV Map");
}

#[test]
fn enum_select_filter_empty_query_keeps_original_order() {
    let filtered = filter_enum_select_items(&sample_items(), "  ");

    assert_eq!(
        filtered
            .iter()
            .map(|item| item.value.as_ref())
            .collect::<Vec<_>>(),
        vec!["matcap", "emission", "uv_map"]
    );
}
