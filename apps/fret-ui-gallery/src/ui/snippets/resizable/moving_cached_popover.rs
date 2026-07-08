pub const SOURCE: &str = include_str!("moving_cached_popover.rs");

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

const TEST_ID_PREFIX: &str = "ui-gallery-resizable-view-cache-moving-popover";

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

    ui::h_flex(|_cx| vec![status, toggle])
        .gap(Space::N2)
        .items_center()
        .wrap()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id(format!("{TEST_ID_PREFIX}-controls"))
}

fn cached_popover_source<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
) -> AnyElement {
    let cache_layout = decl_style::layout_style(
        cx.theme(),
        LayoutRefinement::default().w_full().h_full().min_w_0(),
    );

    cx.cached_subtree_with(
        CachedSubtreeProps::default().layout(cache_layout),
        move |cx| {
            let header = ui::h_flex(|cx| {
                vec![
                    shadcn::Badge::new("Cached popover source")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                    shadcn::typography::muted(
                        "Open in one root, close, move, then reopen before outside press.",
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

            let popover = shadcn::Popover::from_open(open.clone())
                .side(shadcn::PopoverSide::Bottom)
                .align(shadcn::PopoverAlign::Start)
                .side_offset(Px(6.0))
                .into_element_with(
                    cx,
                    |cx| {
                        shadcn::PopoverTrigger::build(
                            shadcn::Button::new("Open cached popover")
                                .variant(shadcn::ButtonVariant::Outline)
                                .test_id(format!("{TEST_ID_PREFIX}-trigger")),
                        )
                        .into_element(cx)
                    },
                    |cx| {
                        let title = shadcn::PopoverTitle::new("Cached Popover").into_element(cx);
                        let description = shadcn::PopoverDescription::new(
                            "Outside press should dismiss after the cached source moves roots.",
                        )
                        .into_element(cx);
                        let header = shadcn::PopoverHeader::new([title, description])
                            .into_element(cx)
                            .test_id(format!("{TEST_ID_PREFIX}-content-header"));
                        let action = shadcn::Button::new("Inner action")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .size(shadcn::ButtonSize::Sm)
                            .test_id(format!("{TEST_ID_PREFIX}-inner-action"))
                            .into_element(cx);

                        shadcn::PopoverContent::new([header, action])
                            .refine_layout(LayoutRefinement::default().w_px(Px(280.0)).min_w_0())
                            .test_id(format!("{TEST_ID_PREFIX}-content"))
                            .into_element(cx)
                    },
                )
                .test_id(format!("{TEST_ID_PREFIX}-popover"));

            let bottom_probe = ui::container(|_cx| Vec::<AnyElement>::new())
                .layout(LayoutRefinement::default().w_full().h_px(Px(20.0)))
                .into_element(cx)
                .test_id(format!("{TEST_ID_PREFIX}-bottom-probe"));

            [
                ui::v_flex(|_cx| vec![header, top_gap, popover, bottom_probe])
                    .gap(Space::N2)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().h_full().min_w_0())
                    .into_element(cx),
            ]
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

fn parking_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    side: &'static str,
    underlay_activated: Model<bool>,
    underlay_activated_now: bool,
) -> AnyElement {
    let underlay_state = if underlay_activated_now {
        "activated"
    } else {
        "idle"
    };
    let status = cx
        .text(format!("underlay: {underlay_state}"))
        .attach_semantics(state_value(
            underlay_state,
            format!("{TEST_ID_PREFIX}-underlay-state"),
        ));
    let underlay = shadcn::Button::new("Activate underlay")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .toggle_model(underlay_activated)
        .test_id(format!("{TEST_ID_PREFIX}-underlay"))
        .into_element(cx);

    let body = ui::v_flex(move |cx| {
        vec![
            shadcn::typography::small(if side == "left" {
                "Left viewport root"
            } else {
                "Right viewport root"
            })
            .into_element(cx),
            shadcn::typography::muted(
                "This underlay button must still receive click-through outside press.",
            )
            .into_element(cx),
            status,
            underlay,
        ]
    })
    .gap(Space::N2)
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
        "ui-gallery-resizable-view-cache-moving-popover-fractions",
        || vec![0.5, 0.5],
    );
    let place_right = cx.local_model_keyed(
        "ui-gallery-resizable-view-cache-moving-popover-place-right",
        || false,
    );
    let open = cx.local_model_keyed(
        "ui-gallery-resizable-view-cache-moving-popover-open",
        || false,
    );
    let underlay_activated = cx.local_model_keyed(
        "ui-gallery-resizable-view-cache-moving-popover-underlay-activated",
        || false,
    );

    let place_right_now = cx
        .get_model_copied(&place_right, Invalidation::Layout)
        .unwrap_or(false);
    let underlay_activated_now = cx
        .get_model_copied(&underlay_activated, Invalidation::Layout)
        .unwrap_or(false);
    let controls = controls(cx, place_right.clone(), place_right_now);

    let group = shadcn::resizable_panel_group(cx, fractions, move |cx| {
        let source = cached_popover_source(cx, open.clone());
        let (left, right) = if place_right_now {
            (
                parking_panel(
                    cx,
                    "left",
                    underlay_activated.clone(),
                    underlay_activated_now,
                ),
                source_panel(cx, "right", source),
            )
        } else {
            (
                source_panel(cx, "left", source),
                parking_panel(
                    cx,
                    "right",
                    underlay_activated.clone(),
                    underlay_activated_now,
                ),
            )
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
