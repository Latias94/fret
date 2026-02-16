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
fn genui_dashboard_smoke_renders_with_strict_catalog_validation() {
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
            "root": { "type": "VStack", "props": { "gap": "N3" }, "children": ["title", "card", "grid_title", "grid"] },
            "title": { "type": "Text", "props": { "text": "GenUI Dashboard Smoke" }, "children": [] },
            "card": { "type": "Card", "props": { "wrapContent": false }, "children": ["card_header", "card_content"] },
            "card_header": { "type": "CardHeader", "props": {}, "children": ["card_title", "card_desc"] },
            "card_title": { "type": "CardTitle", "props": { "text": "Stats" }, "children": [] },
            "card_desc": { "type": "CardDescription", "props": { "text": "Strict catalog validation + render smoke." }, "children": [] },
            "card_content": { "type": "CardContent", "props": {}, "children": ["row"] },
            "row": { "type": "HStack", "props": { "gap": "N2" }, "children": ["count_label", "count_badge", "inc_btn"] },
            "count_label": { "type": "Text", "props": { "text": "Count:" }, "children": [] },
            "count_badge": { "type": "Badge", "props": { "label": { "$state": "/count" }, "variant": "secondary" }, "children": [] },
            "inc_btn": {
                "type": "Button",
                "props": { "label": "Increment" },
                "on": { "press": { "action": "incrementState", "params": { "statePath": "/count", "delta": 1 } } },
                "children": []
            },
            "grid_title": { "type": "Text", "props": { "text": "ResponsiveGrid" }, "children": [] },
            "grid": {
                "type": "ResponsiveGrid",
                "props": { "gap": "N2", "query": "container", "fillLastRow": true, "columns": { "base": 1, "md": 2, "lg": 3 } },
                "children": ["g1", "g2", "g3", "g4", "g5", "g6"]
            },
            "g1": { "type": "Card", "props": { "wrapContent": true }, "children": ["g1t"] },
            "g1t": { "type": "Text", "props": { "text": "Card 1" }, "children": [] },
            "g2": { "type": "Card", "props": { "wrapContent": true }, "children": ["g2t"] },
            "g2t": { "type": "Text", "props": { "text": "Card 2" }, "children": [] },
            "g3": { "type": "Card", "props": { "wrapContent": true }, "children": ["g3t"] },
            "g3t": { "type": "Text", "props": { "text": "Card 3" }, "children": [] },
            "g4": { "type": "Card", "props": { "wrapContent": true }, "children": ["g4t"] },
            "g4t": { "type": "Text", "props": { "text": "Card 4" }, "children": [] },
            "g5": { "type": "Card", "props": { "wrapContent": true }, "children": ["g5t"] },
            "g5t": { "type": "Text", "props": { "text": "Card 5" }, "children": [] },
            "g6": { "type": "Card", "props": { "wrapContent": true }, "children": ["g6t"] },
            "g6t": { "type": "Text", "props": { "text": "Card 6" }, "children": [] }
        },
        "state": { "count": 1 }
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

    fret_ui::elements::with_element_cx(&mut app, window, bounds, "genui-dashboard-smoke", |cx| {
        let runtime = GenUiRuntime {
            state: state.clone(),
            action_queue: None,
            auto_apply_standard_actions: false,
            limits: Default::default(),
            catalog: Some(catalog.clone()),
            catalog_validation: ValidationMode::Strict,
        };

        let mut resolver = ShadcnResolver::new();
        let out = render_spec(cx, &spec, &runtime, &mut resolver).expect("render should succeed");
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

        assert!(
            joined.contains("GenUI Dashboard Smoke"),
            "expected title text to be present; got:\n{joined}"
        );
        assert!(
            !joined.contains("Unknown GenUI component"),
            "unexpected unknown component placeholder text; got:\n{joined}"
        );
    });
}
