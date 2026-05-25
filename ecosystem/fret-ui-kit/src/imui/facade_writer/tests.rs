use super::*;

use fret_app::App;
use fret_authoring::UiWriter;
use fret_core::{AppWindowId, Px, Rect, TextOverflow, TextWrap};
use fret_ui::element::{AnyElement, ElementKind, Length};
use fret_ui::{ElementContext, UiHost, elements};

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

#[test]
fn imui_text_item_is_single_line_and_shrinkable() {
    let mut app = App::new();

    elements::with_element_cx(
        &mut app,
        AppWindowId::default(),
        Rect::default(),
        "imui-text-item",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };

            ui.text("Long editor status text that should not wrap inside a dense row");

            assert_eq!(out.len(), 1);
            let ElementKind::Text(props) = &out[0].kind else {
                panic!("expected imui text item to produce a Text element");
            };

            assert_eq!(props.layout.flex.shrink, 1.0);
            assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
            assert_eq!(props.wrap, TextWrap::None);
            assert_eq!(props.overflow, TextOverflow::Ellipsis);
            assert!(out[0].inherited_text_style.is_some());
        },
    );
}

#[test]
fn imui_text_wrapped_is_explicit_wrapping_text() {
    let mut app = App::new();

    elements::with_element_cx(
        &mut app,
        AppWindowId::default(),
        Rect::default(),
        "imui-text-wrapped",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };

            ui.text_wrapped("Long explanatory text can opt into wrapping explicitly");

            assert_eq!(out.len(), 1);
            let ElementKind::Text(props) = &out[0].kind else {
                panic!("expected imui wrapped text item to produce a Text element");
            };

            assert_eq!(props.layout.size.width, Length::Fill);
            assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
            assert_eq!(props.layout.flex.grow, 1.0);
            assert_eq!(props.layout.flex.shrink, 1.0);
            assert_eq!(props.layout.flex.basis, Length::Px(Px(0.0)));
            assert_eq!(props.wrap, TextWrap::Word);
            assert_eq!(props.overflow, TextOverflow::Clip);
            assert!(out[0].inherited_text_style.is_some());
        },
    );
}
