pub const SOURCE: &str = include_str!("moving_cached_combobox.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::{Axis, Px};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::{AnyElement, SemanticsDecoration};
use fret_ui_kit::declarative::{CachedSubtreeExt as _, CachedSubtreeProps, style as decl_style};
use fret_ui_kit::{IntoUiElement, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

const TEST_ID_PREFIX: &str = "ui-gallery-resizable-view-cache-moving-combobox";

fn deployment_items(review_disabled: bool) -> Vec<shadcn::ComboboxItem> {
    vec![
        shadcn::ComboboxItem::new("draft", "Draft"),
        shadcn::ComboboxItem::new("review", "In Review").disabled(review_disabled),
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

fn state_value(value: &str, test_id: impl Into<Arc<str>>) -> SemanticsDecoration {
    SemanticsDecoration::default()
        .test_id(test_id)
        .value(Arc::<str>::from(value))
}

fn controls<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    place_right: Model<bool>,
    place_right_now: bool,
    review_disabled: Model<bool>,
    review_disabled_now: bool,
) -> AnyElement {
    let target_label = if place_right_now {
        "Move source left"
    } else {
        "Move source right"
    };
    let source_root = if place_right_now { "right" } else { "left" };
    let status = cx
        .text(format!("source root: {source_root}"))
        .attach_semantics(state_value(
            source_root,
            format!("{TEST_ID_PREFIX}-source-root-state"),
        ));

    let toggle = shadcn::Button::new(target_label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .toggle_model(place_right)
        .test_id(format!("{TEST_ID_PREFIX}-move-source"))
        .into_element(cx);

    let review_state = if review_disabled_now {
        "disabled"
    } else {
        "enabled"
    };
    let review_status = cx
        .text(format!("review item: {review_state}"))
        .attach_semantics(state_value(
            review_state,
            format!("{TEST_ID_PREFIX}-review-disabled-state"),
        ));
    let review_toggle_label = if review_disabled_now {
        "Enable Review item"
    } else {
        "Disable Review item"
    };
    let review_toggle = shadcn::Button::new(review_toggle_label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .toggle_model(review_disabled)
        .test_id(format!("{TEST_ID_PREFIX}-toggle-review-disabled"))
        .into_element(cx);

    ui::h_flex(|_cx| vec![status, toggle, review_status, review_toggle])
        .gap(Space::N2)
        .items_center()
        .wrap()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id(format!("{TEST_ID_PREFIX}-controls"))
}

fn cached_combobox_source<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
    review_disabled_now: bool,
) -> AnyElement {
    let cache_layout = decl_style::layout_style(
        cx.theme(),
        LayoutRefinement::default().w_full().h_full().min_w_0(),
    );

    cx.cached_subtree_with(
        CachedSubtreeProps::default()
            .layout(cache_layout)
            .cache_key(0x7a59_d111_6a1d_c0de)
            .cache_key_bool(review_disabled_now),
        move |cx| {
            let header = ui::h_flex(|cx| {
                vec![
                    shadcn::Badge::new("Cached source")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                    shadcn::typography::muted(
                        "The source element keeps one callsite identity while the parent panel changes.",
                    )
                    .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .wrap()
            .items_center()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)
            .test_id(format!("{TEST_ID_PREFIX}-cache-header"));

            let top_gap = ui::container(|_cx| Vec::<AnyElement>::new())
                .layout(LayoutRefinement::default().w_full().h_px(Px(220.0)))
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
                            shadcn::ComboboxList::new().items(deployment_items(
                                review_disabled_now,
                            )),
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

            [ui::v_flex(|_cx| vec![header, top_gap, combo, bottom_probe])
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full().h_full().min_w_0())
                .into_element(cx)]
        },
    )
    .test_id(format!("{TEST_ID_PREFIX}-cache-root"))
}

fn source_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    side: &'static str,
    source: AnyElement,
) -> AnyElement {
    frame(
        cx,
        LayoutRefinement::default().w_full().h_full().min_w_0(),
        source,
    )
    .into_element(cx)
    .test_id(format!("{TEST_ID_PREFIX}-{side}-panel"))
}

fn parking_panel<H: UiHost>(cx: &mut ElementContext<'_, H>, side: &'static str) -> AnyElement {
    let body = ui::v_flex(move |cx| {
        vec![
            shadcn::typography::small(if side == "left" {
                "Left viewport root"
            } else {
                "Right viewport root"
            })
            .into_element(cx),
            shadcn::typography::muted("The cached Combobox source is parked in the other panel.")
                .into_element(cx),
        ]
    })
    .gap(Space::N1)
    .items_center()
    .justify_center()
    .layout(LayoutRefinement::default().w_full().h_full().min_w_0())
    .into_element(cx)
    .test_id(format!("{TEST_ID_PREFIX}-{side}-parking"));

    frame(
        cx,
        LayoutRefinement::default().w_full().h_full().min_w_0(),
        body,
    )
    .into_element(cx)
    .test_id(format!("{TEST_ID_PREFIX}-{side}-panel"))
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let fractions = cx.local_model_keyed(
        "ui-gallery-resizable-view-cache-moving-combobox-fractions",
        || vec![0.5, 0.5],
    );
    let place_right = cx.local_model_keyed(
        "ui-gallery-resizable-view-cache-moving-combobox-place-right",
        || false,
    );
    let value = cx.local_model_keyed(
        "ui-gallery-resizable-view-cache-moving-combobox-value",
        || None::<Arc<str>>,
    );
    let open = cx.local_model_keyed(
        "ui-gallery-resizable-view-cache-moving-combobox-open",
        || false,
    );
    let query = cx.local_model_keyed(
        "ui-gallery-resizable-view-cache-moving-combobox-query",
        String::new,
    );
    let review_disabled = cx.local_model_keyed(
        "ui-gallery-resizable-view-cache-moving-combobox-review-disabled",
        || false,
    );

    let place_right_now = cx
        .get_model_copied(&place_right, Invalidation::Layout)
        .unwrap_or(false);
    let review_disabled_now = cx
        .get_model_copied(&review_disabled, Invalidation::Layout)
        .unwrap_or(false);
    let controls = controls(
        cx,
        place_right.clone(),
        place_right_now,
        review_disabled.clone(),
        review_disabled_now,
    );

    let group = shadcn::resizable_panel_group(cx, fractions, move |cx| {
        let source = cached_combobox_source(
            cx,
            value.clone(),
            open.clone(),
            query.clone(),
            review_disabled_now,
        );
        let (left, right) = if place_right_now {
            (parking_panel(cx, "left"), source_panel(cx, "right", source))
        } else {
            (source_panel(cx, "left", source), parking_panel(cx, "right"))
        };

        [
            shadcn::ResizablePanel::new([left]).min_px(Px(320.0)).into(),
            shadcn::ResizableHandle::new().with_handle(true).into(),
            shadcn::ResizablePanel::new([right])
                .min_px(Px(320.0))
                .into(),
        ]
    })
    .axis(Axis::Horizontal)
    .test_id_prefix(TEST_ID_PREFIX)
    .refine_layout(
        LayoutRefinement::default()
            .w_full()
            .h_px(Px(390.0))
            .min_w_0(),
    )
    .into_element(cx);

    let body = ui::v_flex(move |_cx| vec![controls, group])
        .gap(Space::N3)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

    frame(
        cx,
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(860.0))
            .min_w_0(),
        body,
    )
    .into_element(cx)
    .test_id(TEST_ID_PREFIX)
}
// endregion: example
