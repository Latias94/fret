use super::*;

#[test]
fn tree_node_leaf_uses_tree_item_semantics() {
    let mut app = App::new();
    fret_ui::elements::with_element_cx(
        &mut app,
        Default::default(),
        Default::default(),
        "test",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };
            let response = tree_node_with_options(
                &mut ui,
                "leaf",
                Arc::from("Leaf"),
                TreeNodeOptions {
                    leaf: true,
                    level: 3,
                    selected: true,
                    ..Default::default()
                },
                |_ui| {},
            );

            assert!(!response.open());
            let pressable = first_pressable(&out[0]).expect("expected pressable row");
            assert_eq!(pressable.a11y.role, Some(SemanticsRole::TreeItem));
            assert_eq!(pressable.a11y.level, Some(3));
            assert!(pressable.a11y.selected);
            assert_eq!(pressable.a11y.expanded, None);
        },
    );
}

#[test]
fn tree_node_default_options_start_at_level_one() {
    let options = TreeNodeOptions::default();
    assert_eq!(options.level, 1);
    assert!(!options.selected);
    assert!(!options.leaf);
}
