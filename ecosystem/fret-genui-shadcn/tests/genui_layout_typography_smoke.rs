use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_genui_core::render::{GenUiRuntime, render_spec};
use fret_genui_core::validate::ValidationMode;
use fret_genui_shadcn::catalog::shadcn_catalog_v1;
use fret_genui_shadcn::resolver::ShadcnResolver;
use fret_ui::element::{AnyElement, ElementKind, TextProps};
use serde_json::{Value, json};

fn collect_text_nodes(el: &AnyElement, out: &mut Vec<Arc<str>>) {
    match &el.kind {
        ElementKind::Text(TextProps { text, .. }) => out.push(text.clone()),
        _ => {}
    }
    for child in el.children.iter() {
        collect_text_nodes(child, out);
    }
}

#[test]
fn genui_layout_and_typography_smoke_renders_under_strict_catalog_validation() {
    let mut app = App::new();

    fret_ui_shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnBaseColor::Slate,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let spec: fret_genui_core::spec::SpecV1 = serde_json::from_value(json!({
        "schema_version": 1,
        "root": "root",
        "elements": {
            "root": {
                "type": "VStack",
                "props": { "gap": "N3", "p": "N3", "wFull": true, "minW0": true },
                "children": ["title", "subtitle", "inline_code", "row", "grid", "stack"]
            },
            "title": { "type": "Text", "props": { "text": "Typography Smoke", "variant": "h3" }, "children": [] },
            "subtitle": { "type": "Text", "props": { "text": "muted + small variants", "variant": "muted" }, "children": [] },
            "inline_code": { "type": "Text", "props": { "text": "let x = 1", "variant": "inlineCode" }, "children": [] },
            "row": {
                "type": "HStack",
                "props": { "gap": "N2", "wrap": true, "items": "center", "justify": "between", "wFull": true },
                "children": ["row_label", "row_value"]
            },
            "row_label": { "type": "Text", "props": { "text": "Count:", "variant": "small" }, "children": [] },
            "row_value": { "type": "Badge", "props": { "label": { "$state": "/count" }, "variant": "secondary" }, "children": [] },
            "grid": {
                "type": "ResponsiveGrid",
                "props": { "gap": "N2", "query": "container", "fillLastRow": true, "columns": { "base": 1, "md": 2 } },
                "children": ["g1", "g2", "g3"]
            },
            "g1": { "type": "Card", "props": {}, "children": ["g1t"] },
            "g1t": { "type": "Text", "props": { "text": "Card 1", "variant": "body" }, "children": [] },
            "g2": { "type": "Card", "props": {}, "children": ["g2t"] },
            "g2t": { "type": "Text", "props": { "text": "Card 2", "variant": "body" }, "children": [] },
            "g3": { "type": "Card", "props": {}, "children": ["g3t"] },
            "g3t": { "type": "Text", "props": { "text": "Card 3", "variant": "body" }, "children": [] },
            "stack": {
                "type": "ResponsiveStack",
                "props": { "gap": "N2", "query": "container", "direction": { "base": "vertical", "lg": "horizontal" } },
                "children": ["s1", "s2"]
            },
            "s1": { "type": "Card", "props": {}, "children": ["s1t"] },
            "s1t": { "type": "Text", "props": { "text": "Stack A", "variant": "body" }, "children": [] },
            "s2": { "type": "Card", "props": {}, "children": ["s2t"] },
            "s2t": { "type": "Text", "props": { "text": "Stack B", "variant": "body" }, "children": [] }
        },
        "state": { "count": 3 }
    }))
    .unwrap();

    let seed = spec.state.clone().unwrap_or(Value::Null);
    let state = app.models_mut().insert(seed);
    let catalog = Arc::new(shadcn_catalog_v1());

    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(980.0), Px(720.0)),
    );

    fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "genui-layout-typography-smoke",
        |cx| {
            let runtime = GenUiRuntime {
                state: state.clone(),
                action_queue: None,
                auto_apply_standard_actions: false,
                limits: Default::default(),
                catalog: Some(catalog.clone()),
                catalog_validation: ValidationMode::Strict,
            };

            let mut resolver = ShadcnResolver::new();
            let out =
                render_spec(cx, &spec, &runtime, &mut resolver).expect("render should succeed");
            assert!(
                out.issues.is_empty(),
                "expected no spec issues under strict catalog validation, got: {:?}",
                out.issues
            );

            let root = out.roots.into_iter().next().expect("root");
            let mut texts = Vec::new();
            collect_text_nodes(&root, &mut texts);
            let joined = texts
                .iter()
                .map(|s| s.as_ref())
                .collect::<Vec<_>>()
                .join("\n");

            assert!(joined.contains("Typography Smoke"));
            assert!(joined.contains("let x = 1"));
            assert!(joined.contains("Stack A"));
            assert!(
                !joined.contains("Unknown GenUI component"),
                "unexpected unknown component placeholder text; got:\n{joined}"
            );
        },
    );
}
