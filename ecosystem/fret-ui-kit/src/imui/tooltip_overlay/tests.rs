use std::sync::Arc;

use fret_app::App;
use fret_authoring::UiWriter;
use fret_core::{AppWindowId, Px, Rect, TextOverflow, TextWrap};
use fret_ui::element::{AnyElement, ElementKind, Length};
use fret_ui::{ElementContext, UiHost, elements};

use super::text::{tooltip_body_text, tooltip_text_with_options};
use crate::imui::{ResponseExt, TooltipOptions};

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
fn tooltip_returns_false_without_trigger_id() {
    let mut app = App::new();
    fret_ui::elements::with_element_cx(
        &mut app,
        Default::default(),
        Default::default(),
        "test",
        |cx| {
            let mut out = Vec::new();
            let mut ui = TestWriter { cx, out: &mut out };
            assert!(!tooltip_text_with_options(
                &mut ui,
                "tooltip",
                ResponseExt::default(),
                Arc::from("tip"),
                TooltipOptions::default(),
            ));
            assert!(out.is_empty());
        },
    );
}

#[test]
fn tooltip_body_text_uses_compact_paragraph_role() {
    let mut app = App::new();
    elements::with_element_cx(
        &mut app,
        AppWindowId::default(),
        Rect::default(),
        "imui-tooltip-text-role",
        |cx| {
            let mut out = Vec::new();
            {
                let mut ui = TestWriter { cx, out: &mut out };

                let mounted = tooltip_text_with_options(
                    &mut ui,
                    "tooltip",
                    ResponseExt::default(),
                    Arc::from("Tooltip body copy may wrap when it needs to explain an action"),
                    TooltipOptions::default(),
                );

                assert!(!mounted);
            }
            assert!(out.is_empty());

            let element = tooltip_body_text(
                cx,
                "Tooltip body copy may wrap when it needs to explain an action",
            );
            let ElementKind::Text(props) = &element.kind else {
                panic!("expected tooltip text role to produce a Text element");
            };
            assert_eq!(props.layout.size.width, Length::Fill);
            assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
            assert_eq!(props.layout.flex.shrink, 1.0);
            assert_eq!(props.wrap, TextWrap::Word);
            assert_eq!(props.overflow, TextOverflow::Clip);
        },
    );
}

#[test]
fn tooltip_default_options_use_top_center_placement() {
    let options = TooltipOptions::default();
    assert_eq!(options.placement.side, crate::primitives::popper::Side::Top);
    assert_eq!(
        options.placement.align,
        crate::primitives::popper::Align::Center
    );
    assert_eq!(options.window_margin, Px(8.0));
    assert_eq!(options.open_delay_frames_override, None);
    assert_eq!(options.close_delay_frames_override, None);
    assert!(options.test_id.is_none());
}
