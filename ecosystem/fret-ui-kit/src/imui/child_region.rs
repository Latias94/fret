//! Immediate child-region helpers.

use std::cell::Cell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{ChildRegionOptions, ImUiFacade, containers::build_imui_children_with_focus};

pub(super) fn child_region_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: ChildRegionOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    cx.keyed(id, |cx| {
        let layout = options.layout.clone();
        let scroll_options = options.scroll.clone();
        let test_id = options.test_id.clone();
        let content_test_id = options.content_test_id.clone();

        let mut builder = crate::ui::scroll_area_build(move |cx, out| {
            let mut content = crate::ui::v_flex_build(move |cx, out| {
                build_imui_children_with_focus(cx, out, build_focus, f);
            })
            .no_wrap();

            if let Some(test_id) = content_test_id.clone() {
                content = content.test_id(test_id);
            }

            out.push(content.into_element(cx));
        })
        .axis(scroll_options.axis)
        .show_scrollbars(
            scroll_options.show_scrollbar_x,
            scroll_options.show_scrollbar_y,
        )
        .layout(layout)
        .p_2()
        .rounded_md()
        .border_1()
        .bg(crate::ColorRef::Token {
            key: "card",
            fallback: crate::ColorFallback::ThemePanelBackground,
        })
        .border_color(crate::ColorRef::Token {
            key: "border",
            fallback: crate::ColorFallback::ThemePanelBorder,
        });

        if let Some(handle) = scroll_options.handle {
            builder = builder.handle(handle);
        }

        if let Some(test_id) = test_id {
            builder = builder.test_id(test_id);
        }

        builder.into_element(cx)
    })
}
