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
fn genui_forms_layout_smoke_renders_under_strict_catalog_validation() {
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
                "children": ["title", "boxed", "footer"]
            },
            "title": { "type": "Text", "props": { "text": "Forms Smoke", "variant": "h3" }, "children": [] },

            "boxed": {
                "type": "Box",
                "props": { "p": "N3", "wFull": true, "minW0": true },
                "children": ["card"]
            },
            "card": {
                "type": "Card",
                "props": { "wrapContent": false },
                "children": ["card_header", "card_content"]
            },
            "card_header": {
                "type": "CardHeader",
                "props": {},
                "children": ["card_title", "card_desc"]
            },
            "card_title": { "type": "CardTitle", "props": { "text": "Profile" }, "children": [] },
            "card_desc": { "type": "CardDescription", "props": { "text": "Bindings + wrap + alignment" }, "children": [] },
            "card_content": {
                "type": "CardContent",
                "props": {},
                "children": ["form_stack"]
            },
            "form_stack": {
                "type": "VStack",
                "props": { "gap": "N2", "wFull": true, "minW0": true },
                "children": ["row_name", "row_enabled", "row_actions"]
            },

            "row_name": {
                "type": "HStack",
                "props": { "gap": "N2", "wrap": true, "items": "center", "wFull": true, "minW0": true },
                "children": ["name_label", "name_input", "name_value"]
            },
            "name_label": { "type": "Text", "props": { "text": "Name:", "variant": "small" }, "children": [] },
            "name_input": { "type": "Input", "props": { "placeholder": "Type…", "value": { "$bindState": "/name" }, "flex1": true, "minW0": true }, "children": [] },
            "name_value": { "type": "Badge", "props": { "label": { "$state": "/name" }, "variant": "secondary" }, "children": [] },

            "row_enabled": {
                "type": "HStack",
                "props": { "gap": "N2", "wrap": true, "items": "center", "wFull": true },
                "children": ["enabled_label", "enabled_switch", "enabled_value"]
            },
            "enabled_label": { "type": "Text", "props": { "text": "Enabled:", "variant": "small" }, "children": [] },
            "enabled_switch": { "type": "Switch", "props": { "checked": { "$bindState": "/enabled" } }, "children": [] },
            "enabled_value": { "type": "Badge", "props": { "label": { "$state": "/enabled" }, "variant": "outline" }, "children": [] },

            "row_actions": {
                "type": "HStack",
                "props": { "gap": "N2", "wrap": true, "justify": "between", "wFull": true },
                "children": ["reset_btn", "save_btn"]
            },
            "reset_btn": {
                "type": "Button",
                "props": { "label": "Reset" },
                "on": { "press": { "action": "setState", "params": { "statePath": "/name", "value": "" } } },
                "children": []
            },
            "save_btn": {
                "type": "Button",
                "props": { "label": "Save" },
                "on": { "press": { "action": "incrementState", "params": { "statePath": "/saveClicks", "delta": 1 } } },
                "children": []
            },

            "footer": { "type": "Text", "props": { "text": "End.", "variant": "muted" }, "children": [] }
        },
        "state": { "name": "Ada", "enabled": true, "saveClicks": 0 }
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
        "genui-forms-layout-smoke",
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

            assert!(joined.contains("Forms Smoke"));
            assert!(joined.contains("Profile"));
            assert!(joined.contains("Name:"));
            assert!(joined.contains("Enabled:"));
            assert!(joined.contains("Reset"));
            assert!(joined.contains("Save"));
            assert!(
                !joined.contains("Unknown GenUI component"),
                "unexpected unknown component placeholder text; got:\n{joined}"
            );
        },
    );
}
