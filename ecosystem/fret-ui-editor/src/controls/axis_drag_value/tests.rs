use std::sync::Arc;

use super::AxisDragValue;
use crate::primitives::NumericPresentation;
use crate::primitives::style::EditorStyle;
use fret_app::App;
use fret_core::{AppWindowId, Color, Px, Rect};
use fret_ui::Theme;
use fret_ui::element::{AnyElement, ElementKind, Length};

#[test]
fn axis_drag_value_from_presentation_adopts_format_parse_and_chrome_affixes() {
    let mut app = App::new();
    let model = app.models_mut().insert(1.25f64);
    let presentation = NumericPresentation::<f64>::fixed_decimals(2)
        .with_chrome_prefix("$")
        .with_chrome_suffix("ms");

    let drag_value = AxisDragValue::from_presentation(
        Arc::from("X"),
        Color::from_srgb_hex_rgb(0xf2_59_59),
        model,
        presentation,
    );

    assert_eq!((drag_value.format)(1.25).as_ref(), "1.25");
    assert_eq!((drag_value.parse)("1.25"), Some(1.25));
    assert_eq!(drag_value.options.prefix, Some(Arc::from("$")));
    assert_eq!(drag_value.options.suffix, Some(Arc::from("ms")));
}

#[test]
fn axis_drag_value_uses_stable_session_shell_for_scrub_and_typing_branches() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let model = app.models_mut().insert(1.25f64);
    let element = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        Rect::default(),
        "axis-drag-value-session-shell",
        |cx| {
            AxisDragValue::new(
                Arc::from("X"),
                Color::from_srgb_hex_rgb(0xf2_59_59),
                model,
                Arc::new(|v| Arc::from(format!("{v:.2}"))),
                Arc::new(|text| text.parse::<f64>().ok()),
            )
            .into_element(cx)
        },
    );

    let ElementKind::Stack(shell) = &element.kind else {
        panic!("axis drag value should mount scrub/typing branches in a stack shell");
    };
    let expected_min_height = {
        let style = EditorStyle::resolve(Theme::global(&app));
        style
            .frame_chrome_small()
            .control_outer_height(style.density.row_height)
    };
    assert_eq!(shell.layout.size.width, Length::Fill);
    assert_eq!(shell.layout.size.height, Length::Px(expected_min_height));
    assert_eq!(
        shell.layout.size.min_height,
        Some(Length::Px(expected_min_height))
    );
    assert_eq!(shell.layout.flex.grow, 1.0);
    assert_eq!(shell.layout.flex.basis, Length::Px(Px(0.0)));

    assert_eq!(element.children.len(), 2);
    assert_branch_is_fill(&element.children[0], "scrub branch");
    assert_branch_is_hidden(&element.children[1], "typing branch");
    let hidden_typing_child = hidden_branch_child(&element.children[1], "typing branch");
    assert!(
        matches!(hidden_typing_child.kind, ElementKind::TextInput(_)),
        "inactive typing branch should keep only the hidden TextInput root, got {:?}",
        hidden_typing_child.kind
    );
    assert!(
        !branch_contains_kind(hidden_typing_child, |kind| matches!(
            kind,
            ElementKind::Container(_)
                | ElementKind::Flex(_)
                | ElementKind::HoverRegion(_)
                | ElementKind::PointerRegion(_)
                | ElementKind::Pressable(_)
        )),
        "inactive typing branch should not keep full input-group chrome wrappers"
    );
}

#[test]
fn axis_drag_value_scrub_branch_keeps_identity_when_temporarily_disabled() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let model = app.models_mut().insert(1.25f64);

    let active_id = render_axis_scrub_branch_id(&mut app, window, model.clone(), true);
    let disabled_id = render_axis_scrub_branch_id(&mut app, window, model.clone(), false);
    let restored_id = render_axis_scrub_branch_id(&mut app, window, model, true);

    assert_eq!(
        active_id, disabled_id,
        "hiding typing-mode axis scrub should not replace the scrub root"
    );
    assert_eq!(
        active_id, restored_id,
        "returning to axis scrub should reuse the scrub root"
    );
}

fn render_axis_scrub_branch_id(
    app: &mut App,
    window: AppWindowId,
    model: fret_runtime::Model<f64>,
    enabled: bool,
) -> fret_ui::GlobalElementId {
    fret_ui::elements::with_element_cx(
        app,
        window,
        Rect::default(),
        "axis-drag-value-scrub-identity",
        |cx| {
            let mut options = super::model::AxisDragValueOptions::default();
            options.enabled = enabled;
            AxisDragValue::new(
                Arc::from("X"),
                Color::from_srgb_hex_rgb(0xf2_59_59),
                model,
                Arc::new(|v| Arc::from(format!("{v:.2}"))),
                Arc::new(|text| text.parse::<f64>().ok()),
            )
            .options(options)
            .into_element(cx)
            .children[0]
                .id
        },
    )
}

fn assert_branch_is_fill(element: &AnyElement, label: &str) {
    let layout = element_layout(element, label);
    assert_eq!(layout.size.width, Length::Fill, "{label} width");
    assert_eq!(layout.size.height, Length::Fill, "{label} height");
    assert_eq!(
        layout.flex.grow, 0.0,
        "{label} should not reuse caller flex grow inside the session shell"
    );
}

fn assert_branch_is_hidden(element: &AnyElement, label: &str) {
    assert!(
        branch_is_hidden_gate(element),
        "{label} should be gated absent"
    );
}

fn element_layout<'a>(element: &'a AnyElement, label: &str) -> &'a fret_ui::element::LayoutStyle {
    match &element.kind {
        ElementKind::Pressable(props) => &props.layout,
        ElementKind::Flex(props) => &props.layout,
        ElementKind::PointerRegion(_) => {
            let Some(child) = element.children.first() else {
                panic!("{label} pointer region should contain a child");
            };
            element_layout(child, label)
        }
        ElementKind::HoverRegion(_) => {
            let Some(child) = element.children.first() else {
                panic!("{label} hover region should contain a child");
            };
            element_layout(child, label)
        }
        ElementKind::Container(props) => &props.layout,
        ElementKind::Stack(props) => &props.layout,
        ElementKind::TextInput(props) => &props.layout,
        ElementKind::InteractivityGate(props) => {
            assert!(
                props.present,
                "{label} active branch should not be absent-gated"
            );
            let Some(child) = element.children.first() else {
                panic!("{label} interactivity gate should contain a child");
            };
            element_layout(child, label)
        }
        other => panic!("{label} should expose layout props, got {other:?}"),
    }
}

fn branch_is_hidden_gate(element: &AnyElement) -> bool {
    matches!(
        &element.kind,
        ElementKind::InteractivityGate(props) if !props.present && !props.interactive
    )
}

fn branch_contains_kind(element: &AnyElement, pred: impl Fn(&ElementKind) -> bool + Copy) -> bool {
    pred(&element.kind)
        || element
            .children
            .iter()
            .any(|child| branch_contains_kind(child, pred))
}

fn hidden_branch_child<'a>(element: &'a AnyElement, label: &str) -> &'a AnyElement {
    assert!(
        branch_is_hidden_gate(element),
        "{label} should be gated absent"
    );
    element
        .children
        .first()
        .unwrap_or_else(|| panic!("{label} hidden gate should contain the retained branch"))
}
