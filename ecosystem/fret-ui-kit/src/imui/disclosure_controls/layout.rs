use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::spec::DisclosureSpec;
use crate::imui::ImUiFacade;

mod props;

pub(super) fn disclosure_content_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    spec: &DisclosureSpec,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let mut build = Some(f);
    let mut content = cx.named("content", |cx| {
        cx.container(props::disclosure_content_container_props(spec), move |cx| {
            vec![
                cx.column(props::disclosure_content_column_props(), move |cx| {
                    let mut out = Vec::new();
                    let mut body_ui = ImUiFacade {
                        cx,
                        out: &mut out,
                        build_focus: None,
                    };
                    if let Some(build) = build.take() {
                        build(&mut body_ui);
                    }
                    out
                }),
            ]
        })
    });
    if let Some(test_id) = spec.content_test_id.as_ref() {
        content = content.test_id(test_id.clone());
    }
    content
}

pub(super) fn disclosure_root_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    spec: &DisclosureSpec,
    root_children: Vec<AnyElement>,
) -> AnyElement {
    let mut root = cx.column(props::disclosure_root_column_props(), move |_cx| {
        root_children
    });
    if let Some(test_id) = spec.root_test_id.as_ref() {
        root = root.test_id(test_id.clone());
    }
    root
}
