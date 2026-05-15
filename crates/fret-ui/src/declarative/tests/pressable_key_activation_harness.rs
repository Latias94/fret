use std::sync::Arc;

use serde::Deserialize;

use super::*;

const PRESSABLE_KEY_ACTIVATION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/declarative/tests/fixtures/pressable_key_activation_v1.json"
));

#[derive(Debug, Deserialize)]
struct Suite {
    schema_version: u32,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    pressable: PressableCase,
    key: KeyCase,
    expected: ExpectedCase,
}

#[derive(Debug, Clone, Deserialize)]
struct PressableCase {
    enabled: bool,
    focusable: bool,
    key_activation: KeyActivationCase,
    semantic_disabled: Option<bool>,
    semantic_invokable: Option<bool>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum KeyActivationCase {
    EnterAndSpace,
    EnterOnly,
    None,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum KeyCase {
    Enter,
    Space,
}

#[derive(Debug, Deserialize)]
struct ExpectedCase {
    focus_traversable: bool,
    focus_preserved_after_layout: bool,
    semantics_disabled: bool,
    semantics_focus: bool,
    semantics_invoke: bool,
    activated: bool,
}

#[test]
fn mechanism_harness_pressable_key_activation_matches_oracles() {
    let suite: Suite =
        serde_json::from_str(PRESSABLE_KEY_ACTIVATION).expect("pressable key activation fixture");
    assert_eq!(suite.schema_version, 1);

    for case in &suite.cases {
        run_case(case);
    }
}

fn run_case(case: &Case) {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(180.0), Px(44.0)));
    let mut services = FakeTextService::default();
    let activated = app.models_mut().insert(false);

    let pressable_case = case.pressable.clone();
    let key = case.key;
    let activated_for_render = activated.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-key-activation-harness",
        move |cx| {
            let mut props = crate::element::PressableProps::default();
            props.enabled = pressable_case.enabled;
            props.focusable = pressable_case.focusable;
            props.key_activation = pressable_case
                .key_activation
                .into_pressable_key_activation();
            props.layout.size.width = crate::element::Length::Fill;
            props.layout.size.height = crate::element::Length::Fill;
            props.a11y = crate::element::PressableA11y {
                role: Some(fret_core::SemanticsRole::Button),
                label: Some(Arc::from("Target")),
                test_id: Some(Arc::from("target")),
                ..Default::default()
            };

            let activated = activated_for_render.clone();
            let mut element = cx.pressable(props, move |cx, _state| {
                let activated = activated.clone();
                cx.pressable_on_activate(Arc::new(move |host, _cx, _reason| {
                    let _ = host
                        .models_mut()
                        .update(&activated, |value: &mut bool| *value = true);
                }));
                vec![cx.text("Target")]
            });

            let mut decoration = crate::element::SemanticsDecoration::default();
            if let Some(disabled) = pressable_case.semantic_disabled {
                decoration = decoration.disabled(disabled);
            }
            if let Some(invokable) = pressable_case.semantic_invokable {
                decoration = decoration.invokable(invokable);
            }
            element = element.attach_semantics(decoration);
            vec![element]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    assert_eq!(
        ui.first_focusable_descendant(root) == Some(pressable_node),
        case.expected.focus_traversable,
        "{}: focus traversal mismatch",
        case.id
    );

    ui.set_focus(Some(pressable_node));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(
        ui.focus() == Some(pressable_node),
        case.expected.focus_preserved_after_layout,
        "{}: focus repair mismatch",
        case.id
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snapshot = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snapshot
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("target"))
        .unwrap_or_else(|| panic!("{}: missing target semantics node", case.id));
    assert_eq!(
        node.flags.disabled, case.expected.semantics_disabled,
        "{}: semantics disabled mismatch",
        case.id
    );
    assert_eq!(
        node.actions.focus, case.expected.semantics_focus,
        "{}: semantics focus action mismatch",
        case.id
    );
    assert_eq!(
        node.actions.invoke, case.expected.semantics_invoke,
        "{}: semantics invoke action mismatch",
        case.id
    );

    dispatch_key_pair(&mut ui, &mut app, &mut services, key);
    assert_eq!(
        app.models().get_copied(&activated),
        Some(case.expected.activated),
        "{}: keyboard activation mismatch",
        case.id
    );
}

impl KeyActivationCase {
    fn into_pressable_key_activation(self) -> crate::element::PressableKeyActivation {
        match self {
            Self::EnterAndSpace => crate::element::PressableKeyActivation::EnterAndSpace,
            Self::EnterOnly => crate::element::PressableKeyActivation::EnterOnly,
            Self::None => crate::element::PressableKeyActivation::None,
        }
    }
}

impl KeyCase {
    fn into_key_code(self) -> fret_core::KeyCode {
        match self {
            Self::Enter => fret_core::KeyCode::Enter,
            Self::Space => fret_core::KeyCode::Space,
        }
    }
}

fn dispatch_key_pair(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    key: KeyCase,
) {
    let key = key.into_key_code();
    ui.dispatch_event(
        app,
        services,
        &fret_core::Event::KeyDown {
            key,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        app,
        services,
        &fret_core::Event::KeyUp {
            key,
            modifiers: Modifiers::default(),
        },
    );
}
