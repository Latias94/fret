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

#[test]
fn floating_layer_z_order_does_not_clone_vec_each_frame() {
    let source = include_str!("../src/imui.rs");
    assert!(
        !source.contains("st.order.clone()"),
        "floating_layer should avoid cloning the z-order Vec on every frame"
    );
}

#[test]
fn popup_menu_uses_environment_viewport_bounds_for_popper_outer_bounds() {
    let source = include_str!("../src/imui.rs");
    assert!(
        source.contains("environment_viewport_bounds"),
        "imui popup menu should derive popper outer bounds from committed environment viewport bounds"
    );
    assert!(
        !source.contains("popper_content_layout_sized(cx.bounds"),
        "imui popup menu should not use cx.bounds as the popper outer bounds (bypasses environment query deps)"
    );
}
