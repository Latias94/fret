#![cfg(feature = "imui")]

#[test]
fn select_wrapper_does_not_materialize_items_vec_each_frame() {
    let source = include_str!("../src/imui.rs");
    assert!(
        !source.contains("let items: Vec<Arc<str>> = items.to_vec();"),
        "select_model_ex should keep items borrowed instead of cloning into Vec each frame"
    );
}

#[test]
fn virtual_list_stress_demo_keeps_keyed_virtualization_path() {
    let source = include_str!("../../../apps/fret-examples/src/virtual_list_stress_demo.rs");
    assert!(
        source.contains("VirtualListOptions::new"),
        "virtual_list_stress_demo should configure VirtualListOptions"
    );
    assert!(
        source.contains("virtual_list_keyed_with_layout"),
        "virtual_list_stress_demo should keep keyed virtualization as default path"
    );
}
