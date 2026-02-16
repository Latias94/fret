use std::collections::BTreeMap;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_genui_core::render::{ComponentResolver, GenUiRuntime, RenderLimits, render_spec};
use fret_genui_core::spec::SpecV1;
use fret_genui_core::validate::ValidationMode;
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::{ElementContext, GlobalElementId, UiHost};
use serde_json::{Value, json};

#[derive(Debug, thiserror::Error)]
enum TestResolverError {}

#[derive(Default)]
struct TestResolver;

impl TestResolver {
    fn element_test_id(el: &AnyElement) -> Option<&str> {
        el.semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
    }

    fn collect_test_ids(el: &AnyElement, out: &mut BTreeMap<String, GlobalElementId>) {
        if let Some(id) = Self::element_test_id(el) {
            out.insert(id.to_string(), el.id);
        }
        for child in el.children.iter() {
            Self::collect_test_ids(child, out);
        }
    }
}

impl<H: UiHost> ComponentResolver<H> for TestResolver {
    type Error = TestResolverError;

    fn render_element(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        _key: &fret_genui_core::spec::ElementKey,
        element: &fret_genui_core::spec::ElementV1,
        props: &fret_genui_core::props::ResolvedProps,
        children: Vec<AnyElement>,
        _on_event: &dyn Fn(&str) -> Option<OnActivate>,
    ) -> Result<AnyElement, Self::Error> {
        let mut el = match element.ty.as_str() {
            "VStack" => cx.container(ContainerProps::default(), move |_cx| children),
            "ItemBox" => cx.container(ContainerProps::default(), move |_cx| children),
            other => {
                let msg = format!("unknown test component: {other}");
                cx.container(ContainerProps::default(), move |cx| vec![cx.text(msg)])
            }
        };

        if element.ty == "ItemBox" {
            if let Some(tid) = props.props.get("testId").and_then(|v| v.as_str()) {
                el = el.test_id(tid);
            }
        }

        Ok(el)
    }
}

fn frame_item_ids(
    app: &mut App,
    state: &Model<Value>,
    spec: &SpecV1,
) -> BTreeMap<String, GlobalElementId> {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(400.0)),
    );

    let mut out = BTreeMap::new();
    fret_ui::elements::with_element_cx(app, window, bounds, "genui-repeat-identity", |cx| {
        let runtime = GenUiRuntime {
            state: state.clone(),
            action_queue: None,
            auto_apply_standard_actions: false,
            limits: RenderLimits::default(),
            catalog: None,
            catalog_validation: ValidationMode::Ignore,
        };
        let mut resolver = TestResolver::default();
        let rendered = render_spec(cx, spec, &runtime, &mut resolver).expect("render ok");
        let root = rendered.roots.into_iter().next().expect("root exists");
        TestResolver::collect_test_ids(&root, &mut out);
    });
    out
}

#[test]
fn repeat_with_key_preserves_ids_across_reorder() {
    let mut app = App::new();
    let state = app.models_mut().insert(json!({
        "todos": [
            { "id": "a", "label": "A" },
            { "id": "b", "label": "B" }
        ]
    }));

    let spec: SpecV1 = serde_json::from_value(json!({
        "schema_version": 1,
        "root": "root",
        "elements": {
            "root": { "type": "VStack", "props": {}, "children": ["list"] },
            "list": {
                "type": "VStack",
                "props": {},
                "repeat": { "statePath": "/todos", "key": "id" },
                "children": ["item"]
            },
            "item": {
                "type": "ItemBox",
                "props": { "testId": { "$item": "id" } },
                "children": []
            }
        }
    }))
    .unwrap();

    let ids_frame_1 = frame_item_ids(&mut app, &state, &spec);
    assert!(ids_frame_1.contains_key("a"));
    assert!(ids_frame_1.contains_key("b"));

    app.set_frame_id(fret_runtime::FrameId(app.frame_id().0.saturating_add(1)));
    app.set_tick_id(fret_runtime::TickId(app.tick_id().0.saturating_add(1)));

    let _ = app.models_mut().update(&state, |v| {
        *v = json!({
            "todos": [
                { "id": "b", "label": "B" },
                { "id": "a", "label": "A" }
            ]
        });
    });

    let ids_frame_2 = frame_item_ids(&mut app, &state, &spec);
    assert_eq!(ids_frame_1.get("a"), ids_frame_2.get("a"));
    assert_eq!(ids_frame_1.get("b"), ids_frame_2.get("b"));
}
