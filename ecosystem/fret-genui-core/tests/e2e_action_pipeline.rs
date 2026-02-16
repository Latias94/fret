use std::collections::BTreeMap;
use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_genui_core::render::{
    ComponentResolver, GenUiActionQueue, GenUiRuntime, RenderLimits, render_spec,
};
use fret_genui_core::spec::{ElementKey, ElementV1, SpecV1};
use fret_genui_core::validate::ValidationMode;
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHostAdapter};
use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::{ElementContext, GlobalElementId, UiHost};
use serde_json::{Value, json};

#[derive(Debug, thiserror::Error)]
enum TestResolverError {
    #[error("missing button press handler: {label}")]
    MissingHandler { label: String },
}

#[derive(Default)]
struct TestResolver {
    handlers: BTreeMap<String, OnActivate>,
}

impl TestResolver {
    fn take_handler(&mut self, label: &str) -> Result<OnActivate, TestResolverError> {
        self.handlers
            .remove(label)
            .ok_or_else(|| TestResolverError::MissingHandler {
                label: label.to_string(),
            })
    }
}

impl<H: UiHost> ComponentResolver<H> for TestResolver {
    type Error = TestResolverError;

    fn render_element(
        &mut self,
        cx: &mut ElementContext<'_, H>,
        _key: &ElementKey,
        element: &ElementV1,
        props: &fret_genui_core::props::ResolvedProps,
        children: Vec<AnyElement>,
        on_event: &dyn Fn(&str) -> Option<OnActivate>,
    ) -> Result<AnyElement, Self::Error> {
        match element.ty.as_str() {
            "Button" => {
                let label = props
                    .props
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                if let Some(on) = on_event("press") {
                    self.handlers.insert(label, on);
                }

                Ok(cx.container(ContainerProps::default(), move |_cx| children))
            }
            "VStack" => Ok(cx.container(ContainerProps::default(), move |_cx| children)),
            other => {
                let msg = format!("unknown test component: {other}");
                Ok(cx.container(ContainerProps::default(), move |cx| vec![cx.text(msg)]))
            }
        }
    }
}

fn render_and_capture_button(
    app: &mut App,
    state: &Model<Value>,
    queue: Option<Model<GenUiActionQueue>>,
    spec: &SpecV1,
    auto_apply_standard_actions: bool,
    label: &str,
) -> OnActivate {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(400.0)),
    );

    let mut resolver = TestResolver::default();
    fret_ui::elements::with_element_cx(app, window, bounds, "genui-e2e", |cx| {
        let runtime = GenUiRuntime {
            state: state.clone(),
            action_queue: queue.clone(),
            auto_apply_standard_actions,
            limits: RenderLimits::default(),
            catalog: None,
            catalog_validation: ValidationMode::Ignore,
        };
        let rendered = render_spec(cx, spec, &runtime, &mut resolver).expect("render ok");
        assert!(
            rendered.issues.is_empty(),
            "expected no spec issues, got: {:?}",
            rendered.issues
        );
    });

    resolver.take_handler(label).expect("handler exists")
}

fn invoke_press(app: &mut App, on: &OnActivate) {
    let mut host = UiActionHostAdapter { app };
    (on)(
        &mut host,
        ActionCx {
            window: AppWindowId::default(),
            target: GlobalElementId(42),
        },
        ActivateReason::Pointer,
    );
}

fn queue_len(app: &App, queue: &Model<GenUiActionQueue>) -> usize {
    app.models()
        .read(queue, |q| q.invocations.len())
        .unwrap_or(0)
}

fn last_queue_action(app: &App, queue: &Model<GenUiActionQueue>) -> Option<(Arc<str>, Value)> {
    app.models()
        .read(queue, |q| {
            q.invocations
                .last()
                .map(|inv| (inv.action.clone(), inv.params.clone()))
        })
        .ok()
        .flatten()
}

#[test]
fn press_emits_queue_and_optionally_updates_state() {
    let mut app = App::new();
    let state = app.models_mut().insert(json!({ "count": 0 }));
    let queue = app.models_mut().insert(GenUiActionQueue::default());

    let spec: SpecV1 = serde_json::from_value(json!({
        "schema_version": 1,
        "root": "root",
        "elements": {
            "root": { "type": "VStack", "props": {}, "children": ["inc"] },
            "inc": {
                "type": "Button",
                "props": { "label": "Increment" },
                "on": { "press": { "action": "incrementState", "params": { "statePath": "/count", "delta": 1 } } },
                "children": []
            }
        }
    }))
    .unwrap();

    // Queue-only mode: press should enqueue but not mutate state.
    let on = render_and_capture_button(
        &mut app,
        &state,
        Some(queue.clone()),
        &spec,
        false,
        "Increment",
    );
    invoke_press(&mut app, &on);

    assert_eq!(queue_len(&app, &queue), 1);
    assert_eq!(app.models().get_cloned(&state), Some(json!({ "count": 0 })));

    let (action, params) = last_queue_action(&app, &queue).expect("last invocation");
    assert_eq!(action.as_ref(), "incrementState");
    assert_eq!(params, json!({ "delta": 1, "statePath": "/count" }));

    // Auto-apply mode: press should enqueue and mutate state deterministically.
    let _ = app.models_mut().update(&queue, |q| q.invocations.clear());

    let on = render_and_capture_button(
        &mut app,
        &state,
        Some(queue.clone()),
        &spec,
        true,
        "Increment",
    );
    invoke_press(&mut app, &on);
    invoke_press(&mut app, &on);

    assert_eq!(queue_len(&app, &queue), 2);
    assert_eq!(app.models().get_cloned(&state), Some(json!({ "count": 2 })));
}
