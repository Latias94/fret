use std::sync::Arc;

use fret::imui::kit;
use fret_core::{Color, Px, Rect};
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;
use fret_ui::scroll::ScrollHandle;

use super::super::KernelApp;

pub(super) fn collection_browser_child_region_id() -> &'static str {
    "imui-editor-proof.authoring.imui.collection.browser"
}

fn collection_browser_viewport_id() -> &'static str {
    "imui-editor-proof.authoring.imui.collection.browser.viewport"
}

fn collection_browser_content_id() -> &'static str {
    "imui-editor-proof.authoring.imui.collection.browser.content"
}

pub(super) fn collection_browser_box_select_scope_id() -> &'static str {
    "imui-editor-proof.authoring.imui.collection.box-select.scope"
}

fn collection_browser_box_select_marquee_id() -> &'static str {
    "imui-editor-proof.authoring.imui.collection.box-select.marquee"
}

pub(super) fn collection_browser_child_region_options(
    scroll: ScrollHandle,
) -> kit::ChildRegionOptions {
    kit::ChildRegionOptions {
        layout: fret_ui_kit::LayoutRefinement::default()
            .w_full()
            .h_px(Px(220.0)),
        scroll: kit::ScrollOptions {
            handle: Some(scroll),
            viewport_test_id: Some(Arc::from(collection_browser_viewport_id())),
            ..Default::default()
        },
        test_id: Some(Arc::from(collection_browser_child_region_id())),
        content_test_id: Some(Arc::from(collection_browser_content_id())),
        ..Default::default()
    }
}

pub(super) fn collection_browser_box_select_marquee(
    cx: &mut ElementContext<'_, KernelApp>,
    drag_rect: Rect,
) -> AnyElement {
    let theme = fret_ui::Theme::global(&*cx.app);
    let ring = theme.color_token("ring");
    let fill = Color { a: 0.14, ..ring };
    let border = Color { a: 0.88, ..ring };

    fret_ui_kit::ui::container(|_cx| Vec::<AnyElement>::new())
        .absolute()
        .left_px(drag_rect.origin.x)
        .top_px(drag_rect.origin.y)
        .w_px(drag_rect.size.width)
        .h_px(drag_rect.size.height)
        .bg(fret_ui_kit::ColorRef::Color(fill))
        .border_1()
        .border_color(fret_ui_kit::ColorRef::Color(border))
        .test_id(collection_browser_box_select_marquee_id())
        .into_element(cx)
}
