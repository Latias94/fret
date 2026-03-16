use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_genui_core::render::{GenUiRuntime, render_spec};
use fret_genui_core::validate::ValidationMode;
use fret_genui_shadcn::catalog::shadcn_catalog_v1;
use fret_genui_shadcn::resolver::ShadcnResolver;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ElementKind, InteractivityGateProps, TextProps};
use fret_ui_shadcn::facade::themes as shadcn_themes;
use serde_json::{Value, json};

fn collect_visible_text_nodes(el: &AnyElement, out: &mut Vec<Arc<str>>) {
    match &el.kind {
        ElementKind::InteractivityGate(InteractivityGateProps { present, .. }) => {
            if !present {
                return;
            }
        }
        ElementKind::Text(TextProps { text, .. }) => out.push(text.clone()),
        _ => {}
    }
    for child in el.children.iter() {
        collect_visible_text_nodes(child, out);
    }
}

fn render_root(
    app: &mut App,
    state: &Model<Value>,
    spec: &fret_genui_core::spec::SpecV1,
) -> AnyElement {
    let catalog = Arc::new(shadcn_catalog_v1());
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(980.0), Px(720.0)),
    );

    let mut out_root: Option<AnyElement> = None;
    fret_ui::elements::with_element_cx(
        app,
        window,
        bounds,
        "genui-validation-issues-smoke",
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
                render_spec(cx, spec, &runtime, &mut resolver).expect("render should succeed");
            assert!(
                out.issues.is_empty(),
                "expected no spec issues under strict catalog validation, got: {:?}",
                out.issues
            );
            out_root = Some(out.roots.into_iter().next().expect("root"));
        },
    );
    out_root.expect("root captured")
}

#[test]
fn genui_validation_issue_presentation_smoke_repeat_and_visible_filtering() {
    let mut app = App::new();

    shadcn_themes::apply_shadcn_new_york(
        &mut app,
        shadcn_themes::ShadcnBaseColor::Slate,
        shadcn_themes::ShadcnColorScheme::Light,
    );

    let spec: fret_genui_core::spec::SpecV1 = serde_json::from_value(json!({
        "schema_version": 1,
        "root": "root",
        "elements": {
            "root": {
                "type": "VStack",
                "props": { "gap": "N2", "p": "N3", "wFull": true, "minW0": true },
                "children": ["title", "errors"]
            },
            "title": { "type": "Text", "props": { "text": "Validation issues", "variant": "h4" }, "children": [] },
            "errors": {
                "type": "VStack",
                "props": { "gap": "N1" },
                "repeat": { "statePath": "/validation/issues" },
                "children": ["error_item"]
            },
            "error_item": {
                "type": "Badge",
                "props": { "label": { "$item": "message" }, "variant": "destructive" },
                "visible": { "$item": "path", "eq": "/form/email" },
                "children": []
            }
        }
    }))
    .unwrap();

    let state = app
        .models_mut()
        .insert(json!({ "validation": { "issues": [] } }));

    // Empty: no error text should be visible.
    {
        let root = render_root(&mut app, &state, &spec);
        let mut texts = Vec::new();
        collect_visible_text_nodes(&root, &mut texts);
        let joined = texts
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !joined.contains("Email is required."),
            "unexpected text:\n{joined}"
        );
    }

    // With mixed issues: only the email issue should be visible (path filter).
    let _ = app.models_mut().update(&state, |v| {
        let issues = Value::Array(vec![
            json!({ "path": "/form/email", "code": "required", "message": "Email is required." }),
            json!({ "path": "/name", "code": "required", "message": "Name is required." }),
        ]);
        let _ = fret_genui_core::json_pointer::set(v, "/validation/issues", issues);
    });

    {
        let root = render_root(&mut app, &state, &spec);
        let mut texts = Vec::new();
        collect_visible_text_nodes(&root, &mut texts);
        let joined = texts
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            joined.contains("Email is required."),
            "missing email issue:\n{joined}"
        );
        assert!(
            !joined.contains("Name is required."),
            "unexpected non-email issue visible:\n{joined}"
        );
        assert!(
            !joined.contains("Unknown GenUI component"),
            "unexpected unknown component placeholder text:\n{joined}"
        );
    }

    // Cleared: error disappears deterministically.
    let _ = app.models_mut().update(&state, |v| {
        let _ = fret_genui_core::json_pointer::set(v, "/validation/issues", Value::Array(vec![]));
    });

    {
        let root = render_root(&mut app, &state, &spec);
        let mut texts = Vec::new();
        collect_visible_text_nodes(&root, &mut texts);
        let joined = texts
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !joined.contains("Email is required."),
            "unexpected text:\n{joined}"
        );
    }
}
