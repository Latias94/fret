pub const SOURCE: &str = include_str!("plan_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_icons::ids;
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let streaming = cx.local_model_keyed("streaming", || false);

    let is_streaming = cx
        .get_model_copied(&streaming, Invalidation::Layout)
        .unwrap_or(false);

    let body = ui_ai::Plan::new()
        .default_open(false)
        .is_streaming(is_streaming)
        .test_id_root("ui-ai-plan-root")
        .into_element_with_children(cx, move |cx, controller| {
            let open = cx
                .get_model_copied(&controller.open, Invalidation::Layout)
                .unwrap_or(false);
            let marker = open.then(|| cx.text("").test_id("ui-ai-plan-open-true"));

            let file_icon = icon::icon_with(cx, ids::ui::FILE, Some(Px(16.0)), None);
            let title_row = ui::h_flex(move |cx| {
                    vec![
                        file_icon,
                        ui_ai::PlanTitle::new("Rewrite AI Elements to SolidJS").into_element(cx),
                    ]
                })
                    .layout(LayoutRefinement::default().w_full().min_w_0().mb(Space::N4))
                    .items_center()
                    .gap(Space::N2).into_element(cx);

            let left = ui::v_flex(move |cx| {
                    vec![
                        title_row,
                        ui_ai::PlanDescription::new(
                            "Rewrite the AI Elements component library from React to SolidJS while \
maintaining compatibility with existing React-based shadcn/ui components using solid-js/compat, \
updating all 29 components and their test suite.",
                        )
                        .into_element(cx),
                    ]
                })
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N2).into_element(cx);

            vec![
                ui_ai::PlanHeader::new([
                    left,
                    ui_ai::PlanTrigger::default()
                        .test_id("ui-ai-plan-trigger")
                        .into_element(cx),
                ])
                .into_element(cx),
                marker.unwrap_or_else(|| cx.text("")),
                ui_ai::PlanContent::new([
                    ui::v_flex(move |cx| {
                            vec![
                                ui::v_flex(move |cx| {
                                        vec![
                                            ui::text("Overview")
                                                .font_semibold()
                                                .mb(Space::N2)
                                                .into_element(cx),
                                            ui::text(
                                                "This plan outlines the migration strategy for converting the AI Elements \
library from React to SolidJS, ensuring compatibility and maintaining existing functionality.",
                                            )
                                            .text_sm()
                                            .wrap(TextWrap::Word)
                                            .into_element(cx),
                                        ]
                                    })
                                        .layout(LayoutRefinement::default().w_full().min_w_0())
                                        .gap(Space::N2).into_element(cx),
                                ui::v_flex(move |cx| {
                                        vec![
                                            ui::text("Key Steps")
                                                .font_semibold()
                                                .mb(Space::N2)
                                                .into_element(cx),
                                            ui::v_flex(move |cx| {
                                                    let bullets = [
                                                        "Set up SolidJS project structure",
                                                        "Install solid-js/compat for React compatibility",
                                                        "Migrate components one by one",
                                                        "Update test suite for each component",
                                                        "Verify compatibility with shadcn/ui",
                                                    ];
                                                    bullets
                                                        .into_iter()
                                                        .map(|item| {
                                                            ui::text(format!("• {item}"))
                                                                .text_sm()
                                                                .wrap(TextWrap::Word)
                                                                .into_element(cx)
                                                        })
                                                        .collect::<Vec<_>>()
                                                })
                                                    .layout(
                                                        LayoutRefinement::default()
                                                            .w_full()
                                                            .min_w_0(),
                                                    )
                                                    .gap(Space::N1).into_element(cx),
                                        ]
                                    })
                                        .layout(LayoutRefinement::default().w_full().min_w_0())
                                        .gap(Space::N2).into_element(cx),
                            ]
                        })
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .gap(Space::N4).into_element(cx),
                ])
                .test_id("ui-ai-plan-content-marker")
                .into_element(cx),
                ui_ai::PlanFooter::new([
                    ui::h_flex(move |cx| {
                        vec![ui_ai::PlanAction::new([
                            shadcn::Button::new("Build")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .size(shadcn::ButtonSize::Sm)
                                .children([
                                    ui::text("Build").into_element(cx),
                                    shadcn::Kbd::new("⌘↩").into_element(cx),
                                ])
                                .a11y_label("Build")
                                .into_element(cx),
                        ])
                        .into_element(cx)]
                    })
                    .w_full()
                    .justify_end()
                    .into_element(cx),
                ])
                .into_element(cx),
            ]
        });

    ui::v_flex(move |cx| {
        vec![
            cx.text("Plan (AI Elements)"),
            cx.text("Toggle the chevron button to expand/collapse."),
            shadcn::Button::new("Toggle streaming")
                .variant(shadcn::ButtonVariant::Secondary)
                .size(shadcn::ButtonSize::Sm)
                .toggle_model(streaming.clone())
                .into_element(cx),
            body,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
