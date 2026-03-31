#![cfg(feature = "imui")]

#[test]
fn combo_model_wrapper_does_not_materialize_items_vec_each_frame() {
    let source = include_str!("../src/imui/combo_model_controls.rs");
    assert!(
        !source.contains("let items: Vec<Arc<str>> = items.to_vec();"),
        "combo_model_with_options should keep items borrowed instead of cloning into Vec each frame"
    );
    assert!(
        source.contains("combo_with_options("),
        "combo_model_with_options should reuse the canonical combo helper instead of duplicating popup trigger flow"
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
    let source = include_str!("../src/imui/popup_overlay.rs");
    assert!(
        source.contains("environment_viewport_bounds"),
        "imui popup menu should derive popper outer bounds from committed environment viewport bounds"
    );
    assert!(
        !source.contains("popper_content_layout_sized(cx.bounds"),
        "imui popup menu should not use cx.bounds as the popper outer bounds (bypasses environment query deps)"
    );
}

#[test]
fn imui_virtual_list_wrapper_reuses_runtime_virtual_list_substrate() {
    let source = include_str!("../src/imui/virtual_list_controls.rs");
    assert!(
        source.contains("virtual_list_keyed_with_layout("),
        "imui virtual_list should stay a thin wrapper over the runtime keyed virtual list substrate"
    );
    assert!(
        source.contains("slot_state(VirtualListScrollHandle::new"),
        "imui virtual_list should keep a stable default VirtualListScrollHandle when the caller does not provide one"
    );
}
