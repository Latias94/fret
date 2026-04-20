pub const SOURCE: &str = include_str!("adaptive_panel.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::{Axis, Px};
use fret_ui::element::{AnyElement, LayoutQueryRegionProps};
use fret_ui::Invalidation;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{IntoUiElement, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn shell_frame<H: UiHost, B>(
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

fn proof_step<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    step: &'static str,
    copy: &'static str,
    test_id: &'static str,
) -> AnyElement {
    ui::h_flex(|cx| {
        vec![
            shadcn::Badge::new(step)
                .variant(shadcn::BadgeVariant::Outline)
                .into_element(cx),
            shadcn::raw::typography::muted(copy).into_element(cx),
        ]
    })
    .gap(Space::N2)
    .wrap()
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id(test_id)
}

fn instruction_panel<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    ui::h_flex(|cx| {
                        vec![
                            shadcn::Badge::new("Fixed window")
                                .variant(shadcn::BadgeVariant::Outline)
                                .into_element(cx),
                            shadcn::Badge::new("Postman-style shell")
                                .variant(shadcn::BadgeVariant::Secondary)
                                .into_element(cx),
                        ]
                    })
                    .gap(Space::N2)
                    .wrap()
                    .items_center()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
                    shadcn::card_title("Resize the splitter, not the viewport"),
                    shadcn::card_description(
                        "This proof keeps the docs shell width stable. Only the right panel crosses the container-query breakpoint, so the compact branch cannot be blamed on viewport width.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| {
                ui::children![
                    cx;
                    ui::v_flex(|cx| {
                        vec![
                            proof_step(
                                cx,
                                "1",
                                "Keep the window wide enough for the fixed shell to stay fully expanded.",
                                "ui-gallery-resizable-adaptive-panel-step-1",
                            ),
                            proof_step(
                                cx,
                                "2",
                                "Move the splitter right to shrink the request pane until the badge flips from `Wide panel` to `Compact panel`.",
                                "ui-gallery-resizable-adaptive-panel-step-2",
                            ),
                            proof_step(
                                cx,
                                "3",
                                "The form below reuses `FieldOrientation::ContainerAdaptive`, so the field rows stack only because the panel width changed.",
                                "ui-gallery-resizable-adaptive-panel-step-3",
                            ),
                        ]
                    })
                    .gap(Space::N4)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                ]
            }),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().h_full())
    .into_element(cx)
    .test_id("ui-gallery-resizable-adaptive-panel-instructions")
}

fn adaptive_state_row<H: UiHost>(cx: &mut ElementContext<'_, H>, wide: bool) -> AnyElement {
    let (label, variant, badge_id, copy) = if wide {
        (
            "Wide panel",
            shadcn::BadgeVariant::Default,
            "ui-gallery-resizable-adaptive-panel-state-wide",
            "Container width is still at or above `md`; the request form stays in its wide row layout.",
        )
    } else {
        (
            "Compact panel",
            shadcn::BadgeVariant::Secondary,
            "ui-gallery-resizable-adaptive-panel-state-compact",
            "Container width fell below `md`; the request form switches to the compact stacked branch.",
        )
    };

    ui::h_flex(|cx| {
        vec![
            shadcn::Badge::new(label)
                .variant(variant)
                .test_id(badge_id)
                .into_element(cx),
            shadcn::raw::typography::muted(copy)
                .into_element(cx)
                .test_id("ui-gallery-resizable-adaptive-panel-state-copy"),
        ]
    })
    .gap(Space::N2)
    .wrap()
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-resizable-adaptive-panel-state-row")
}

fn request_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    endpoint: Model<String>,
    token: Model<String>,
) -> AnyElement {
    let layout = decl_style::layout_style(
        cx.theme(),
        LayoutRefinement::default().w_full().h_full().min_w_0(),
    );

    fret_ui_kit::declarative::container_query_region_with_id(
        cx,
        "ui-gallery.resizable.adaptive_panel.target",
        LayoutQueryRegionProps { layout, name: None },
        move |cx, region_id| {
            let default_when_unknown = cx.environment_viewport_width(Invalidation::Layout).0
                >= fret_ui_kit::declarative::container_queries::tailwind::MD.0;
            let wide = fret_ui_kit::declarative::container_width_at_least(
                cx,
                region_id,
                Invalidation::Layout,
                default_when_unknown,
                fret_ui_kit::declarative::container_queries::tailwind::MD,
                fret_ui_kit::declarative::ContainerQueryHysteresis::default(),
            );

            let endpoint_id = "ui-gallery-resizable-adaptive-panel-endpoint";
            let token_id = "ui-gallery-resizable-adaptive-panel-token";

            let request_form = shadcn::FieldSet::new([
                shadcn::FieldLegend::new("Request pane").into_element(cx),
                shadcn::FieldDescription::new(
                    "This side models an API client detail panel. The outer proof badge watches the panel width directly, and the fields below use the shipped container-adaptive recipe.",
                )
                .into_element(cx),
                shadcn::FieldGroup::new([
                    shadcn::Field::new([
                        shadcn::FieldContent::new([
                            shadcn::FieldLabel::new("Endpoint")
                                .for_control(endpoint_id)
                                .into_element(cx),
                            shadcn::FieldDescription::new(
                                "Keep the label/content pair together until the panel gets tight.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::Input::new(endpoint.clone())
                            .control_id(endpoint_id)
                            .a11y_label("Request endpoint")
                            .placeholder("https://api.example.dev/v1/messages")
                            .into_element(cx)
                            .test_id("ui-gallery-resizable-adaptive-panel-endpoint-input"),
                    ])
                    .orientation(shadcn::FieldOrientation::ContainerAdaptive)
                    .into_element(cx),
                    shadcn::Field::new([
                        shadcn::FieldContent::new([
                            shadcn::FieldLabel::new("Auth token")
                                .for_control(token_id)
                                .into_element(cx),
                            shadcn::FieldDescription::new(
                                "A second adaptive field keeps the row-vs-stack change obvious.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::Input::new(token.clone())
                            .control_id(token_id)
                            .a11y_label("Auth token")
                            .placeholder("Bearer sk_demo_123")
                            .into_element(cx)
                            .test_id("ui-gallery-resizable-adaptive-panel-token-input"),
                    ])
                    .orientation(shadcn::FieldOrientation::ContainerAdaptive)
                    .into_element(cx),
                    shadcn::Field::new([
                        shadcn::Button::new("Send request")
                            .into_element(cx)
                            .test_id("ui-gallery-resizable-adaptive-panel-send"),
                        shadcn::Button::new("Save example")
                            .variant(shadcn::ButtonVariant::Outline)
                            .into_element(cx)
                            .test_id("ui-gallery-resizable-adaptive-panel-save"),
                    ])
                    .orientation(shadcn::FieldOrientation::ContainerAdaptive)
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
            .test_id("ui-gallery-resizable-adaptive-panel-form");

            vec![
                shadcn::card(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_header(|cx| {
                            ui::children![
                                cx;
                                shadcn::card_title("Adaptive request panel"),
                                shadcn::card_description(
                                    "This branch follows container width. The viewport stays unchanged while the splitter promotes the compact layout.",
                                ),
                            ]
                        }),
                        shadcn::card_content(|cx| {
                            ui::children![
                                cx;
                                ui::v_flex(|cx| {
                                    vec![
                                        adaptive_state_row(cx, wide),
                                        shadcn::Separator::new()
                                            .refine_layout(
                                                LayoutRefinement::default()
                                                    .w_full()
                                                    .my(Space::N1),
                                            )
                                            .into_element(cx),
                                        request_form,
                                    ]
                                })
                                .gap(Space::N4)
                                .items_start()
                                .layout(LayoutRefinement::default().w_full().min_w_0()),
                            ]
                        }),
                    ]
                })
                .refine_layout(LayoutRefinement::default().w_full().h_full())
                .into_element(cx)
                .test_id("ui-gallery-resizable-adaptive-panel-target-card"),
            ]
        },
    )
    .test_id("ui-gallery-resizable-adaptive-panel-target")
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let fractions = cx.local_model_keyed("ui-gallery-resizable-adaptive-panel-fractions", || {
        vec![0.18, 0.82]
    });
    let endpoint = cx
        .local_model_keyed("ui-gallery-resizable-adaptive-panel-endpoint-value", || {
            "https://api.acme.test/v1/messages".to_owned()
        });
    let token = cx.local_model_keyed("ui-gallery-resizable-adaptive-panel-token-value", || {
        "Bearer sk_demo_panel".to_owned()
    });

    let group = shadcn::resizable_panel_group(cx, fractions, move |cx| {
        [
            shadcn::ResizablePanel::new([instruction_panel(cx)])
                .min_px(Px(196.0))
                .into(),
            shadcn::ResizableHandle::new().with_handle(true).into(),
            shadcn::ResizablePanel::new([request_panel(cx, endpoint.clone(), token.clone())])
                .min_px(Px(320.0))
                .into(),
        ]
    })
    .axis(Axis::Horizontal)
    .test_id_prefix("ui-gallery-resizable-adaptive-panel")
    .refine_layout(LayoutRefinement::default().w_full().h_full())
    .into_element(cx);

    shell_frame(
        cx,
        LayoutRefinement::default()
            .w_full()
            .max_w(Px(1040.0))
            .h_px(Px(360.0))
            .min_w_0(),
        group,
    )
    .into_element(cx)
    .test_id("ui-gallery-resizable-adaptive-panel-proof")
}
// endregion: example
