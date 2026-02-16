use std::collections::BTreeMap;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_genui_core::render::{
    ComponentResolver, GenUiRuntime, RenderLimits, RenderedChildV1, render_spec,
};
use fret_genui_core::spec::{ElementKey, ElementV1, SpecV1};
use fret_genui_core::validate::ValidationMode;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHostAdapter};
use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::{ElementContext, GlobalElementId, UiHost};
use serde_json::{Value, json};

#[derive(Debug, thiserror::Error)]
enum TestResolverError {
    #[error("missing button press handler: {label}")]
    MissingHandler { label: String },
    #[error("missing badge label")]
    MissingBadgeLabel,
}

#[derive(Default)]
struct TestResolver {
    handlers: BTreeMap<String, OnActivate>,
    badge_label: Option<String>,
}

impl TestResolver {
    fn take_handler(&mut self, label: &str) -> Result<OnActivate, TestResolverError> {
        self.handlers
            .remove(label)
            .ok_or_else(|| TestResolverError::MissingHandler {
                label: label.to_string(),
            })
    }

    fn take_badge_label(&mut self) -> Result<String, TestResolverError> {
        self.badge_label
            .take()
            .ok_or(TestResolverError::MissingBadgeLabel)
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
        children: Vec<RenderedChildV1>,
        on_event: &dyn Fn(&str) -> Option<OnActivate>,
    ) -> Result<AnyElement, Self::Error> {
        let children: Vec<AnyElement> = children.into_iter().map(|c| c.rendered).collect();
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
            "Badge" => {
                let label = props.props.get("label").cloned().unwrap_or(Value::Null);
                self.badge_label = Some(match label {
                    Value::String(s) => s,
                    other => other.to_string(),
                });
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

fn render_once(app: &mut App, state: &fret_runtime::Model<Value>, spec: &SpecV1) -> TestResolver {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(500.0), Px(400.0)),
    );

    let mut resolver = TestResolver::default();
    fret_ui::elements::with_element_cx(app, window, bounds, "genui-e2e-dynamic-props", |cx| {
        let runtime = GenUiRuntime {
            state: state.clone(),
            action_queue: None,
            auto_apply_standard_actions: true,
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

    resolver
}

#[test]
fn press_updates_state_and_rerender_resolves_dynamic_props_from_latest_state() {
    let mut app = App::new();
    let state = app.models_mut().insert(json!({ "count": 0 }));

    let spec: SpecV1 = serde_json::from_value(json!({
        "schema_version": 1,
        "root": "root",
        "elements": {
            "root": { "type": "VStack", "props": {}, "children": ["badge", "inc"] },
            "badge": { "type": "Badge", "props": { "label": { "$state": "/count" } }, "children": [] },
            "inc": {
                "type": "Button",
                "props": { "label": "Increment" },
                "on": { "press": { "action": "incrementState", "params": { "statePath": "/count", "delta": 1 } } },
                "children": []
            }
        }
    }))
    .unwrap();

    let mut r1 = render_once(&mut app, &state, &spec);
    assert_eq!(r1.take_badge_label().expect("badge label"), "0");

    let on = r1.take_handler("Increment").expect("press handler");
    invoke_press(&mut app, &on);
    assert_eq!(app.models().get_cloned(&state), Some(json!({ "count": 1 })));

    let mut r2 = render_once(&mut app, &state, &spec);
    assert_eq!(r2.take_badge_label().expect("badge label"), "1");
}
