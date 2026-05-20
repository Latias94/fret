pub const SOURCE: &str = include_str!("multi_viewport_select.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::{Axis, Px};
use fret_ui::element::AnyElement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{IntoUiElement, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

const TEST_ID_PREFIX: &str = "ui-gallery-resizable-multi-viewport-select";

fn status_items() -> Vec<shadcn::SelectItem> {
    vec![
        shadcn::SelectItem::new("draft", "Draft"),
        shadcn::SelectItem::new("review", "In Review"),
        shadcn::SelectItem::new("staged", "Staged"),
        shadcn::SelectItem::new("release", "Release Ready"),
        shadcn::SelectItem::new("archived", "Archived"),
    ]
}

fn frame<H: UiHost, B>(
    cx: &mut ElementContext<'_, H>,
    layout: LayoutRefinement,
    body: B,
) -> impl IntoUiElement<H> + use<H, B>
where
    B: IntoUiElement<H>,
{
    let props = decl_style::container_props(
        cx.theme(),
        ChromeRefinement::default().border_1().rounded(Radius::Lg),
        layout.merge(LayoutRefinement::default().overflow_hidden()),
    );
    ui::container_props(props, move |cx| [body.into_element(cx)])
}

fn overview_panel<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let body = ui::v_flex(|cx| {
        vec![
            shadcn::raw::typography::small("Primary viewport").into_element(cx),
            shadcn::raw::typography::muted("Panel A").into_element(cx),
        ]
    })
    .gap(Space::N1)
    .items_center()
    .justify_center()
    .layout(LayoutRefinement::default().w_full().h_full().min_w_0())
    .into_element(cx);

    frame(
        cx,
        LayoutRefinement::default().w_full().h_full().min_w_0(),
        body,
    )
    .into_element(cx)
    .test_id(format!("{TEST_ID_PREFIX}-left-panel"))
}

fn select_panel<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let body = ui::v_flex(|cx| {
        let top_gap = ui::container(|_cx| Vec::<AnyElement>::new())
            .layout(LayoutRefinement::default().w_full().h_px(Px(288.0)))
            .into_element(cx)
            .test_id(format!("{TEST_ID_PREFIX}-top-gap"));

        let select = shadcn::Select::new_controllable(cx, None, None::<Arc<str>>, None, false)
            .test_id_prefix(TEST_ID_PREFIX)
            .refine_layout(LayoutRefinement::default().w_px(Px(224.0)))
            .trigger(shadcn::SelectTrigger::new())
            .value(shadcn::SelectValue::new().placeholder("Select status"))
            .content(
                shadcn::SelectContent::new()
                    .test_id(format!("{TEST_ID_PREFIX}-listbox"))
                    .position(shadcn::raw::select::SelectPosition::Popper)
                    .side_offset(Px(6.0)),
            )
            .items(status_items())
            .into_element(cx)
            .test_id(format!("{TEST_ID_PREFIX}-root"));

        let bottom_probe = ui::container(|_cx| Vec::<AnyElement>::new())
            .layout(LayoutRefinement::default().w_full().h_px(Px(20.0)))
            .into_element(cx)
            .test_id(format!("{TEST_ID_PREFIX}-bottom-probe"));

        vec![top_gap, select, bottom_probe]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().h_full().min_w_0())
    .into_element(cx);

    frame(
        cx,
        LayoutRefinement::default().w_full().h_full().min_w_0(),
        body,
    )
    .into_element(cx)
    .test_id(format!("{TEST_ID_PREFIX}-panel"))
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let fractions = cx.local_model_keyed(
        "ui-gallery-resizable-multi-viewport-select-fractions",
        || vec![0.42, 0.58],
    );

    let group = shadcn::resizable_panel_group(cx, fractions, move |cx| {
        [
            shadcn::ResizablePanel::new([overview_panel(cx)])
                .min_px(Px(220.0))
                .into(),
            shadcn::ResizableHandle::new().with_handle(true).into(),
            shadcn::ResizablePanel::new([select_panel(cx)])
                .min_px(Px(300.0))
                .into(),
        ]
    })
    .axis(Axis::Horizontal)
    .test_id_prefix(TEST_ID_PREFIX)
    .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
    .into_element(cx);

    frame(
        cx,
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(780.0))
            .h_px(Px(380.0))
            .min_w_0(),
        group,
    )
    .into_element(cx)
    .test_id(TEST_ID_PREFIX)
}
// endregion: example
