#![recursion_limit = "256"]

use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_genui_core::render::{GenUiRuntime, render_spec};
use fret_genui_core::validate::ValidationMode;
use fret_genui_shadcn::catalog::shadcn_catalog_v1;
use fret_genui_shadcn::resolver::ShadcnResolver;
use fret_ui::element::{AnyElement, ElementKind, TextProps};
use fret_ui_shadcn::facade::themes as shadcn_themes;
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

    shadcn_themes::apply_shadcn_new_york(
        &mut app,
        shadcn_themes::ShadcnBaseColor::Slate,
        shadcn_themes::ShadcnColorScheme::Light,
    );

    let spec: fret_genui_core::spec::SpecV1 = serde_json::from_value(json!({
        "schema_version": 1,
        "root": "root",
        "elements": {
            "root": { "type": "VStack", "props": { "gap": "N3" }, "children": ["title", "compat_heading", "compat_stack", "compat_avatar", "overlay_title", "overlay_row", "nav_title", "pagination", "charts_title", "bar_chart", "line_chart", "form_title", "form", "card", "grid_title", "grid", "tabs_title", "tabs", "accordion_title", "accordion", "table_title", "table", "feedback_title", "feedback_box"] },
            "title": { "type": "Text", "props": { "text": "GenUI Dashboard Smoke" }, "children": [] },

            "compat_heading": { "type": "Heading", "props": { "text": "Dashboard", "level": "h1" }, "children": [] },
            "compat_stack": { "type": "Stack", "props": { "direction": "horizontal", "gap": "md" }, "children": ["stack_a", "stack_b"] },
            "stack_a": { "type": "Badge", "props": { "text": "A", "variant": "outline" }, "children": [] },
            "stack_b": { "type": "Badge", "props": { "label": "B", "variant": "secondary" }, "children": [] },
            "compat_avatar": { "type": "Avatar", "props": { "fallback": "AL", "src": null, "alt": null }, "children": [] },

            "overlay_title": { "type": "Text", "props": { "text": "Overlays" }, "children": [] },
            "overlay_row": { "type": "HStack", "props": { "gap": "N2", "wrap": true }, "children": ["tooltip", "popover", "dropdown", "dialog", "drawer"] },
            "tooltip": { "type": "Tooltip", "props": { "content": "Tooltip content" }, "children": ["tooltip_trigger"] },
            "tooltip_trigger": { "type": "Button", "props": { "label": "Hover me", "variant": "outline" }, "children": [] },
            "popover": { "type": "Popover", "props": { "trigger": "Open Popover" }, "children": ["popover_text"] },
            "popover_text": { "type": "Text", "props": { "text": "Popover body" }, "children": [] },
            "dropdown": { "type": "DropdownMenu", "props": { "trigger": "Menu", "items": [
                { "label": "Refresh", "action": "openUrl", "actionParams": { "url": "https://example.com" } },
                { "type": "separator", "label": "---", "action": null, "actionParams": null },
                { "label": "Delete", "action": "removeState", "params": { "statePath": "/customers", "index": 0 }, "variant": "destructive" }
            ] }, "children": [] },
            "dialog": { "type": "Dialog", "props": { "trigger": "Open Dialog", "title": "Dialog Title", "description": "Dialog description" }, "children": ["dialog_text"] },
            "dialog_text": { "type": "Text", "props": { "text": "Dialog body" }, "children": [] },
            "drawer": { "type": "Drawer", "props": { "trigger": "Open Drawer", "title": "Drawer Title", "description": "Drawer description", "side": "right" }, "children": ["drawer_text"] },
            "drawer_text": { "type": "Text", "props": { "text": "Drawer body" }, "children": [] },

            "nav_title": { "type": "Text", "props": { "text": "Pagination" }, "children": [] },
            "pagination": { "type": "Pagination", "props": { "currentPage": 10, "totalPages": 20, "onPageChange": "openUrl" }, "children": [] },

            "charts_title": { "type": "Text", "props": { "text": "Charts" }, "children": [] },
            "bar_chart": { "type": "BarChart", "props": { "title": "Sales", "data": [{ "day": "Mon", "value": 10 }], "xKey": "day", "yKey": "value", "aggregate": null, "color": null, "height": 120 }, "children": [] },
            "line_chart": { "type": "LineChart", "props": { "title": "Revenue", "data": [{ "day": "Mon", "value": 20 }], "xKey": "day", "yKey": "value", "aggregate": null, "color": null, "height": 120 }, "children": [] },

            "form_title": { "type": "Text", "props": { "text": "Form" }, "children": [] },
            "form": { "type": "Form", "props": {}, "on": { "submit": { "action": "formSubmit", "params": { "formName": "smoke" } } }, "children": ["form_input", "form_button"] },
            "form_input": { "type": "Input", "props": { "label": "Email", "value": "you@example.com", "placeholder": "you@example.com", "type": "email", "wFull": true }, "children": [] },
            "form_button": { "type": "Button", "props": { "label": "Submit", "variant": "default" }, "on": { "press": { "action": "formSubmit", "params": { "formName": "smoke" } } }, "children": [] },

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
            "g6t": { "type": "Text", "props": { "text": "Card 6" }, "children": [] },

            "tabs_title": { "type": "Text", "props": { "text": "Tabs" }, "children": [] },
            "tabs": {
                "type": "Tabs",
                "props": {
                    "defaultValue": "a",
                    "tabs": [
                        { "value": "a", "label": "Tab A" },
                        { "value": "b", "label": "Tab B" }
                    ]
                },
                "children": ["tab_a", "tab_b"]
            },
            "tab_a": { "type": "TabContent", "props": { "value": "a" }, "children": ["tab_a_text"] },
            "tab_a_text": { "type": "Text", "props": { "text": "Panel A" }, "children": [] },
            "tab_b": { "type": "TabContent", "props": { "value": "b" }, "children": ["tab_b_text"] },
            "tab_b_text": { "type": "Text", "props": { "text": "Panel B" }, "children": [] },

            "accordion_title": { "type": "Text", "props": { "text": "Accordion" }, "children": [] },
            "accordion": {
                "type": "Accordion",
                "props": { "type": "single", "collapsible": true, "defaultValue": "one" },
                "children": ["acc_one", "acc_two"]
            },
            "acc_one": {
                "type": "AccordionItem",
                "props": { "value": "one", "title": "First" },
                "children": ["acc_one_text"]
            },
            "acc_one_text": { "type": "Text", "props": { "text": "Accordion body 1" }, "children": [] },
            "acc_two": {
                "type": "AccordionItem",
                "props": { "value": "two", "title": "Second" },
                "children": ["acc_two_text"]
            },
            "acc_two_text": { "type": "Text", "props": { "text": "Accordion body 2" }, "children": [] },

            "table_title": { "type": "Text", "props": { "text": "Table" }, "children": [] },
            "table": {
                "type": "Table",
                "props": {
                    "data": { "$state": "/customers" },
                    "dataPath": "/customers",
                    "columns": [
                        { "key": "name", "label": "Name" },
                        { "key": "email", "label": "Email" },
                        { "key": "status", "label": "Status" }
                    ],
                    "rowActions": [
                        {
                            "label": "Delete",
                            "variant": "destructive",
                            "action": "removeState",
                            "params": { "statePath": "/customers", "index": { "$index": true } }
                        }
                    ],
                    "emptyMessage": "No customers"
                },
                "children": []
            },

            "feedback_title": { "type": "Text", "props": { "text": "Feedback" }, "children": [] },
            "feedback_box": {
                "type": "Box",
                "props": { "p": "N3", "wFull": true, "minW0": true },
                "children": ["feedback_stack"]
            },
            "feedback_stack": {
                "type": "VStack",
                "props": { "gap": "N2", "wFull": true, "minW0": true },
                "children": ["alert", "progress", "spinner", "skeleton"]
            },
            "alert": {
                "type": "Alert",
                "props": { "variant": "destructive", "title": "Heads up", "description": "This is a GenUI alert." },
                "children": []
            },
            "progress": {
                "type": "Progress",
                "props": { "value": { "$state": "/progress" }, "min": 0, "max": 100, "wFull": true, "mirrorInRtl": true },
                "children": []
            },
            "spinner": { "type": "Spinner", "props": { "sizePx": 16 }, "children": [] },
            "skeleton": { "type": "Skeleton", "props": { "hPx": 16, "wFull": true }, "children": [] }
        },
        "state": {
            "count": 1,
            "progress": 42,
            "customers": [
                { "id": "c1", "name": "Ada Lovelace", "email": "ada@example.com", "status": "active" },
                { "id": "c2", "name": "Grace Hopper", "email": "grace@example.com", "status": "inactive" }
            ]
        }
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
        assert!(joined.contains("Dashboard"));
        assert!(joined.contains("Hover me"));
        assert!(joined.contains("Open Popover"));
        assert!(joined.contains("Menu"));
        assert!(joined.contains("Open Dialog"));
        assert!(joined.contains("Open Drawer"));
        assert!(joined.contains("Prev"));
        assert!(joined.contains("Next"));
        assert!(joined.contains("First"));
        assert!(joined.contains("Last"));
        assert!(joined.contains("…"));
        assert!(joined.contains("BarChart placeholder"));
        assert!(joined.contains("Tab A"));
        assert!(joined.contains("Tab B"));
        assert!(joined.contains("Panel A"));
        assert!(joined.contains("Panel B"));
        assert!(joined.contains("First"));
        assert!(joined.contains("Second"));
        assert!(joined.contains("Ada Lovelace"));
        assert!(joined.contains("grace@example.com"));
        assert!(joined.contains("Delete"));
        assert!(
            !joined.contains("Unknown GenUI component"),
            "unexpected unknown component placeholder text; got:\n{joined}"
        );
    });
}
