pub const SOURCE: &str = include_str!("multi_viewport_combobox.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::{Axis, Px};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{IntoUiElement, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

const TEST_ID_PREFIX: &str = "ui-gallery-resizable-multi-viewport-combobox";

fn deployment_items() -> Vec<shadcn::ComboboxItem> {
    vec![
        shadcn::ComboboxItem::new("draft", "Draft"),
        shadcn::ComboboxItem::new("review", "In Review"),
        shadcn::ComboboxItem::new("staged", "Staged"),
        shadcn::ComboboxItem::new("release", "Release Ready"),
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
            shadcn::typography::small("Primary viewport").into_element(cx),
            shadcn::typography::muted("Panel A").into_element(cx),
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

fn combobox_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> AnyElement {
    let body = ui::v_flex(move |cx| {
        let top_gap = ui::container(|_cx| Vec::<AnyElement>::new())
            .layout(LayoutRefinement::default().w_full().h_px(Px(252.0)))
            .into_element(cx)
            .test_id(format!("{TEST_ID_PREFIX}-top-gap"));

        let combo = shadcn::Combobox::new(value.clone(), open.clone())
            .a11y_label("Deployment status")
            .query_model(query.clone())
            .test_id_prefix(TEST_ID_PREFIX)
            .trigger(
                shadcn::ComboboxTrigger::new()
                    .variant(shadcn::ComboboxTriggerVariant::Button)
                    .width_px(Px(224.0)),
            )
            .input(shadcn::ComboboxInput::new().placeholder("Pick status"))
            .content(
                shadcn::ComboboxContent::new([
                    shadcn::ComboboxContentPart::input(
                        shadcn::ComboboxInput::new().placeholder("Filter status..."),
                    ),
                    shadcn::ComboboxContentPart::empty(shadcn::ComboboxEmpty::new(
                        "No status found.",
                    )),
                    shadcn::ComboboxContentPart::list(
                        shadcn::ComboboxList::new().items(deployment_items()),
                    ),
                ])
                .width_px(Px(260.0))
                .side_offset_px(Px(6.0))
                .test_id(format!("{TEST_ID_PREFIX}-content")),
            )
            .into_element(cx);

        let bottom_probe = ui::container(|_cx| Vec::<AnyElement>::new())
            .layout(LayoutRefinement::default().w_full().h_px(Px(20.0)))
            .into_element(cx)
            .test_id(format!("{TEST_ID_PREFIX}-bottom-probe"));

        vec![top_gap, combo, bottom_probe]
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
        "ui-gallery-resizable-multi-viewport-combobox-fractions",
        || vec![0.42, 0.58],
    );
    let value = cx.local_model_keyed("ui-gallery-resizable-multi-viewport-combobox-value", || {
        None::<Arc<str>>
    });
    let open = cx.local_model_keyed("ui-gallery-resizable-multi-viewport-combobox-open", || {
        false
    });
    let query = cx.local_model_keyed(
        "ui-gallery-resizable-multi-viewport-combobox-query",
        String::new,
    );

    let group = shadcn::resizable_panel_group(cx, fractions, move |cx| {
        [
            shadcn::ResizablePanel::new([overview_panel(cx)])
                .min_px(Px(220.0))
                .into(),
            shadcn::ResizableHandle::new().with_handle(true).into(),
            shadcn::ResizablePanel::new([combobox_panel(
                cx,
                value.clone(),
                open.clone(),
                query.clone(),
            )])
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
