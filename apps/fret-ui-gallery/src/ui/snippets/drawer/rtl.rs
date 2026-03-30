pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{UiChild, UiCx};

use fret_core::{Corners, Edges, Px};
use fret_ui::Theme;
use fret_ui::element::{ContainerProps, LayoutStyle, Length, SizeStyle};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

const GOAL_MIN: i32 = 200;
const GOAL_MAX: i32 = 400;
const GOAL_STEP: i32 = 10;
const GOAL_SERIES: [i32; 13] = [
    400, 300, 200, 300, 200, 278, 189, 239, 300, 200, 278, 189, 349,
];

fn goal_adjust_button(
    cx: &mut UiCx<'_>,
    goal: Model<i32>,
    adjustment: i32,
    icon: &'static str,
    a11y_label: &'static str,
    disabled: bool,
) -> shadcn::Button {
    shadcn::Button::new("")
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::IconSm)
        .icon(IconId::new_static(icon))
        .a11y_label(a11y_label)
        .disabled(disabled)
        .on_activate(cx.actions().listen(move |host, action_cx| {
            let _ = host.models_mut().update(&goal, |value| {
                *value = (*value + adjustment).clamp(GOAL_MIN, GOAL_MAX);
            });
            host.request_redraw(action_cx.window);
        }))
}

fn goal_chart<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    goal: i32,
) -> impl IntoUiElement<H> + use<H> {
    let theme = Theme::global(&*cx.app).snapshot();
    let active = theme.color_token("foreground");
    let mut inactive = active;
    inactive.a *= 0.35;

    ui::h_flex(move |cx| {
        GOAL_SERIES
            .iter()
            .map(|value| {
                let height = Px(((*value as f32 / GOAL_MAX as f32) * 96.0).clamp(24.0, 96.0));
                let fill = if *value <= goal { active } else { inactive };
                cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Px(Px(14.0)),
                                height: Length::Px(height),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        padding: Edges::all(Px(0.0)).into(),
                        background: Some(fill),
                        shadow: None,
                        border: Edges::all(Px(0.0)),
                        border_color: None,
                        corner_radii: Corners::all(Px(999.0)),
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                )
            })
            .collect::<Vec<_>>()
    })
    .gap(Space::N1p5)
    .items_end()
    .layout(
        LayoutRefinement::default()
            .w_full()
            .h_px(Px(120.0))
            .min_w_0(),
    )
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let goal_model = cx.local_model(|| 350);
    let current_goal = cx.watch_model(&goal_model).copied().unwrap_or(350);
    let theme = Theme::global(&*cx.app).snapshot();
    let muted_fg = theme.color_token("muted-foreground");

    with_direction_provider(cx, LayoutDirection::Rtl, move |cx| {
        shadcn::Drawer::new_controllable(cx, None, false)
            .children([
                shadcn::DrawerPart::trigger(shadcn::DrawerTrigger::build(
                    shadcn::Button::new("فتح الدرج")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-drawer-rtl-trigger"),
                )),
                shadcn::DrawerPart::content_with(move |cx| {
                    let goal_for_decrease = goal_model.clone();
                    let goal_for_increase = goal_model.clone();

                    let content = ui::v_stack(|cx| {
                        vec![
                            ui::h_flex(|cx| {
                                vec![
                                    goal_adjust_button(
                                        cx,
                                        goal_for_decrease.clone(),
                                        -GOAL_STEP,
                                        "lucide.minus",
                                        "تقليل",
                                        current_goal <= GOAL_MIN,
                                    )
                                    .into_element(cx),
                                    ui::v_stack(|cx| {
                                        vec![
                                            ui::text(current_goal.to_string())
                                                .text_size_px(Px(56.0))
                                                .font_bold()
                                                .tabular_nums()
                                                .into_element(cx),
                                            ui::text("سعرات حرارية/يوم")
                                                .text_sm()
                                                .font_medium()
                                                .text_color(ColorRef::Color(muted_fg))
                                                .into_element(cx),
                                        ]
                                    })
                                    .gap(Space::N1)
                                    .items_center()
                                    .layout(LayoutRefinement::default().flex_1().min_w_0())
                                    .into_element(cx),
                                    goal_adjust_button(
                                        cx,
                                        goal_for_increase.clone(),
                                        GOAL_STEP,
                                        "lucide.plus",
                                        "زيادة",
                                        current_goal >= GOAL_MAX,
                                    )
                                    .into_element(cx),
                                ]
                            })
                            .gap(Space::N3)
                            .items_center()
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .into_element(cx),
                            goal_chart(cx, current_goal).into_element(cx),
                        ]
                    })
                    .gap(Space::N3)
                    .px_4()
                    .pb(Space::N0)
                    .items_stretch()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx);

                    shadcn::DrawerContent::new([])
                        .children(|cx| {
                            vec![
                                ui::v_stack(move |cx| {
                                    ui::children![
                                        cx;
                                        shadcn::DrawerHeader::new([]).children(|cx| {
                                            ui::children![
                                                cx;
                                                shadcn::DrawerTitle::new("حرّك الهدف"),
                                                shadcn::DrawerDescription::new("اضبط هدف نشاطك اليومي.")
                                            ]
                                        }),
                                        content,
                                        shadcn::DrawerFooter::new([]).children(|cx| {
                                            ui::children![
                                                cx;
                                                shadcn::Button::new("إرسال"),
                                                shadcn::DrawerClose::from_scope().child(
                                                    shadcn::Button::new("إلغاء")
                                                        .variant(shadcn::ButtonVariant::Outline),
                                                )
                                            ]
                                        })
                                    ]
                                })
                                .gap(Space::N0)
                                .items_stretch()
                                .layout(
                                    LayoutRefinement::default()
                                        .w_full()
                                        .max_w(Px(384.0))
                                        .min_w_0()
                                        .mx_auto(),
                                )
                                .into_element(cx),
                            ]
                        })
                        .test_id("ui-gallery-drawer-rtl-content")
                        .into_element(cx)
                }),
            ])
            .into_element(cx)
    })
}
// endregion: example
