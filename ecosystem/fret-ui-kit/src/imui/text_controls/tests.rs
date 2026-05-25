use super::*;

use std::sync::Arc;

use fret_app::App;
use fret_authoring::UiWriter;
use fret_core::{AppWindowId, Corners, Edges, Px, Rect};
use fret_ui::ElementContext;
use fret_ui::element::{AnyElement, ElementKind};

struct TestWriter<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    out: &'cx mut Vec<AnyElement>,
}

impl<'cx, 'a, H: UiHost> UiWriter<H> for TestWriter<'cx, 'a, H> {
    fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {
        f(self.cx)
    }

    fn add(&mut self, element: AnyElement) {
        self.out.push(element);
    }
}

fn first_text_input(root: &AnyElement) -> Option<&fret_ui::element::TextInputProps> {
    match &root.kind {
        ElementKind::TextInput(props) => Some(props),
        _ => root.children.iter().find_map(first_text_input),
    }
}

fn first_text_area(root: &AnyElement) -> Option<&fret_ui::element::TextAreaProps> {
    match &root.kind {
        ElementKind::TextArea(props) => Some(props),
        _ => root.children.iter().find_map(first_text_area),
    }
}

#[test]
fn input_text_model_uses_compact_imui_chrome_without_focus_ring() {
    let mut app = App::new();
    let model = app.models_mut().insert(String::new());

    fret_ui::elements::with_element_cx(
        &mut app,
        AppWindowId::default(),
        Rect::default(),
        "imui-input-text-chrome",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };

            let response = input_text_model_with_options(
                &mut ui,
                &model,
                InputTextOptions {
                    test_id: Some(Arc::from("imui-input-text-chrome")),
                    ..Default::default()
                },
            );

            assert!(response.id().is_some());
            assert_eq!(out.len(), 1);

            let props = first_text_input(&out[0]).expect("expected text input element");
            assert!(props.chrome.focus_ring.is_none());
            assert_eq!(props.chrome.border, Edges::all(Px(1.0)));
            assert_eq!(props.chrome.padding.left, Px(8.0));
            assert_eq!(props.chrome.padding.right, Px(8.0));
            assert_eq!(props.chrome.padding.top, Px(3.0));
            assert_eq!(props.chrome.padding.bottom, Px(3.0));
            assert_eq!(
                props.chrome.corner_radii,
                Corners::all(super::super::control_chrome::CONTROL_RADIUS)
            );
            assert_eq!(
                props.layout.size.height,
                Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT)
            );
            assert_eq!(
                props.layout.size.min_height,
                Some(Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT))
            );
            assert_eq!(
                props.layout.size.max_height,
                Some(Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT))
            );
        },
    );
}

#[test]
fn textarea_model_uses_compact_imui_chrome_without_focus_ring() {
    let mut app = App::new();
    let model = app.models_mut().insert(String::new());

    fret_ui::elements::with_element_cx(
        &mut app,
        AppWindowId::default(),
        Rect::default(),
        "imui-textarea-chrome",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };

            let response = textarea_model_with_options(
                &mut ui,
                &model,
                TextAreaOptions {
                    test_id: Some(Arc::from("imui-textarea-chrome")),
                    ..Default::default()
                },
            );

            assert!(response.id().is_some());
            assert_eq!(out.len(), 1);

            let props = first_text_area(&out[0]).expect("expected text area element");
            assert!(props.chrome.focus_ring.is_none());
            assert_eq!(props.chrome.border, Edges::all(Px(1.0)));
            assert_eq!(props.chrome.padding_x, Px(8.0));
            assert_eq!(props.chrome.padding_y, Px(3.0));
            assert_eq!(
                props.chrome.corner_radii,
                Corners::all(super::super::control_chrome::CONTROL_RADIUS)
            );
            assert_eq!(props.layout.size.width, Length::Fill);
        },
    );
}
