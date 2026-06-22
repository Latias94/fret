use std::sync::Arc;

use super::Slider;
use super::chrome::{resolve_slider_geometry, resolve_slider_paint};
use super::frame::{SliderFrameArgs, slider_frame};
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
    assert!(
        matches!(element.children[1].kind, ElementKind::TextInput(_)),
        "inactive typing branch should keep only the hidden TextInput root, got {:?}",
        element.children[1].kind
    );
    assert!(
        !branch_contains_kind(&element.children[1], |kind| matches!(
            kind,
            ElementKind::HoverRegion(_) | ElementKind::PointerRegion(_)
        )),
        "inactive typing branch should not keep hover/pointer frame wrappers"
    );
}

#[test]
fn slider_frame_tracks_are_direct_flex_children_without_segment_wrappers() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let element = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        Rect::default(),
        "slider-frame-track-shell",
        |cx| {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            let density = style.density;
            let frame_chrome = style.frame_chrome_small();
            let geometry = resolve_slider_geometry(theme);
            let paint = resolve_slider_paint(theme, true, true, false, false);

            slider_frame(
                cx,
                SliderFrameArgs {
                    density,
                    frame_chrome,
                    geometry,
                    paint,
                    t: 0.5,
                    interactive_enabled: true,
                    hovered: false,
                    pressed: false,
                    focused: false,
                    show_value: false,
                    value_width: Px(64.0),
                    value_display_text: Arc::from("50%"),
                    value_display_test_id: None,
                },
            )
        },
    );

    let ElementKind::Container(_) = &element.kind else {
        panic!("slider frame root should stay the shared input-group frame container");
    };
    let outer = element
        .children
        .first()
        .expect("slider frame should contain an outer content flex root");
    let ElementKind::Flex(_) = &outer.kind else {
        panic!(
            "slider frame outer content should be a flex root, got {:?}",
            outer.kind
        );
    };
    let track = outer
        .children
        .first()
        .expect("slider frame should keep a track child");
    let ElementKind::Flex(_) = &track.kind else {
        panic!(
            "slider track should land directly as a flex root, got {:?}",
            track.kind
        );
    };
    assert!(track.children.len() >= 3);
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

fn branch_contains_kind(element: &AnyElement, pred: impl Fn(&ElementKind) -> bool + Copy) -> bool {
    pred(&element.kind)
        || element
            .children
            .iter()
            .any(|child| branch_contains_kind(child, pred))
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
