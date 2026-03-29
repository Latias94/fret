#![cfg(feature = "imui")]

use fret_ui::UiHost;
use fret_ui_kit::imui::{CollapsingHeaderOptions, TreeNodeOptions, UiWriterImUiFacadeExt};

#[allow(dead_code)]
fn disclosure_api_compiles<H: UiHost>(ui: &mut impl UiWriterImUiFacadeExt<H>) {
    let _ = ui.collapsing_header("collapsing.basic", "Section", |ui| {
        ui.text("Body");
    });
    let _ = ui.collapsing_header_with_options(
        "collapsing.with_options",
        "Section",
        CollapsingHeaderOptions::default(),
        |ui| {
            ui.text("Body");
        },
    );

    let _ = ui.tree_node("tree.basic", "Scene", |ui| {
        let _ = ui.tree_node_with_options(
            "tree.basic.camera",
            "Camera",
            TreeNodeOptions {
                leaf: true,
                level: 2,
                ..Default::default()
            },
            |_ui| {},
        );
    });
    let _ = ui.tree_node_with_options(
        "tree.with_options",
        "Materials",
        TreeNodeOptions {
            default_open: true,
            selected: true,
            ..Default::default()
        },
        |ui| {
            ui.text("PBR");
        },
    );
}

#[test]
fn disclosure_option_defaults_compile() {
    let collapsing = CollapsingHeaderOptions::default();
    assert!(collapsing.enabled);
    assert!(!collapsing.default_open);
    assert!(collapsing.open.is_none());

    let tree = TreeNodeOptions::default();
    assert!(tree.enabled);
    assert_eq!(tree.level, 1);
    assert!(!tree.selected);
    assert!(!tree.leaf);
}
