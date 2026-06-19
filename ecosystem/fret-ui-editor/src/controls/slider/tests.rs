use std::sync::Arc;

use super::Slider;
use crate::primitives::NumericPresentation;
use crate::primitives::style::EditorStyle;
use fret_app::App;
use fret_core::{AppWindowId, Px, Rect};
use fret_ui::Theme;
use fret_ui::element::{AnyElement, ElementKind, Length, PositionStyle};

#[test]
fn slider_from_presentation_adopts_format_parse_and_chrome_affixes() {
    let mut app = App::new();
    let model = app.models_mut().insert(0.25f64);
    let presentation = NumericPresentation::<f64>::fixed_decimals(1)
        .with_chrome_prefix("$")
        .with_chrome_suffix("ms");

    let slider = Slider::from_presentation(model, 0.0, 1.0, presentation);

    assert_eq!((slider.format)(0.25).as_ref(), "0.2");
    assert_eq!((slider.parse)("0.2"), Some(0.2));
    assert_eq!(slider.options.prefix, Some(Arc::from("$")));
    assert_eq!(slider.options.suffix, Some(Arc::from("ms")));
}

#[test]
fn slider_uses_stable_session_shell_for_slide_and_typing_branches() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let model = app.models_mut().insert(0.25f64);
    let element = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        Rect::default(),
        "slider-session-shell",
        |cx| Slider::new(model, 0.0, 1.0).into_element(cx),
    );

    let ElementKind::Stack(shell) = &element.kind else {
        panic!("slider should mount slide/typing branches in a stack shell");
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
    assert_branch_is_fill(&element.children[0], "slide branch");
    assert_branch_is_hidden(&element.children[1], "typing branch");
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
    assert_eq!(
        branch_has_hidden_layout(element),
        true,
        "{label} should keep a hidden zero-sized branch mounted"
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
        other => panic!("{label} should expose layout props, got {other:?}"),
    }
}

fn branch_has_hidden_layout(element: &AnyElement) -> bool {
    if let Some(layout) = layout_for_hidden_check(element)
        && layout.size.width == Length::Px(Px(0.0))
        && layout.size.height == Length::Px(Px(0.0))
        && layout.position == PositionStyle::Absolute
    {
        return true;
    }

    element.children.iter().any(branch_has_hidden_layout)
}

fn layout_for_hidden_check<'a>(
    element: &'a AnyElement,
) -> Option<&'a fret_ui::element::LayoutStyle> {
    match &element.kind {
        ElementKind::Pressable(props) => Some(&props.layout),
        ElementKind::Flex(props) => Some(&props.layout),
        ElementKind::PointerRegion(props) => Some(&props.layout),
        ElementKind::HoverRegion(props) => Some(&props.layout),
        ElementKind::Container(props) => Some(&props.layout),
        ElementKind::Stack(props) => Some(&props.layout),
        ElementKind::TextInput(props) => Some(&props.layout),
        _ => None,
    }
}
