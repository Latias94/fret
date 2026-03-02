pub const SOURCE: &str = include_str!("stagger_demo.rs");

// region: example
use fret_app::App;
use fret_ui::Theme;
use fret_ui::element::LayoutStyle;
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::time::Duration;

#[derive(Default, Clone)]
struct Models {
    stagger_open: Option<Model<bool>>,
}

fn stagger_open_model(cx: &mut ElementContext<'_, App>) -> Model<bool> {
    cx.with_state(Models::default, |st| st.stagger_open.clone())
        .unwrap_or_else(|| {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.stagger_open = Some(model.clone()));
            model
        })
}

pub fn render(cx: &mut ElementContext<'_, App>, theme: &Theme) -> AnyElement {
    let shell_layout = LayoutRefinement::default()
        .w_full()
        .max_w(Px(760.0))
        .min_w_0();

    let stagger_open = stagger_open_model(cx);
    let is_open = cx
        .watch_model(&stagger_open)
        .paint()
        .copied()
        .unwrap_or(false);

    let duration_ms = theme
        .duration_ms_by_key("duration.motion.stack.shift")
        .unwrap_or_else(|| theme.duration_ms_token("duration.motion.presence.enter"));
    let duration = Duration::from_millis(duration_ms as u64);
    let each_delay_ms = theme
        .duration_ms_by_key("duration.motion.stack.shift.stagger")
        .unwrap_or(24);
    let each_delay = Duration::from_millis(each_delay_ms as u64);
    let easing = theme
        .easing_by_key("easing.motion.stack.shift")
        .unwrap_or_else(|| theme.easing_token("easing.motion.standard"));
    let easing_headless =
        fret_ui_headless::easing::CubicBezier::new(easing.x1, easing.y1, easing.x2, easing.y2);

    let global = fret_ui_kit::primitives::transition::drive_transition_with_durations_and_cubic_bezier_duration_with_mount_behavior(
        cx,
        is_open,
        duration,
        duration,
        easing,
        false,
    );

    let count = 6usize;
    let from = if is_open {
        fret_ui_headless::motion::stagger::StaggerFrom::First
    } else {
        fret_ui_headless::motion::stagger::StaggerFrom::Last
    };

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2)
            .items_center(),
        move |cx| {
            (0..count)
                .map(|i| {
                    let local_linear =
                        fret_ui_headless::motion::stagger::staggered_progress_for_duration(
                            global.linear,
                            i,
                            count,
                            each_delay,
                            duration,
                            from,
                        );
                    let local = easing_headless.sample(local_linear);
                    let dy_px = (1.0 - local) * 10.0;
                    let transform = fret_core::Transform2D::translation(fret_core::Point::new(
                        Px(0.0),
                        Px(dy_px),
                    ));

                    cx.opacity_props(
                        fret_ui::element::OpacityProps {
                            layout: LayoutStyle::default(),
                            opacity: local,
                        },
                        move |cx| {
                            let badge = shadcn::Badge::new(format!("Item {}", i + 1))
                                .variant(shadcn::BadgeVariant::Secondary)
                                .into_element(cx)
                                .test_id(format!("ui-gallery-motion-presets-stagger-item-{i}"));

                            vec![cx.visual_transform_props(
                                fret_ui::element::VisualTransformProps {
                                    layout: LayoutStyle::default(),
                                    transform,
                                },
                                |_cx| vec![badge],
                            )]
                        },
                    )
                })
                .collect::<Vec<_>>()
        },
    );

    let toggle = shadcn::Button::new("Toggle sequence")
        .variant(shadcn::ButtonVariant::Secondary)
        .toggle_model(stagger_open.clone())
        .test_id("ui-gallery-motion-presets-stagger-toggle")
        .into_element(cx);

    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3)
            .items_start(),
        move |_cx| vec![toggle, row],
    );

    shadcn::Card::new([
        shadcn::CardHeader::new([
            shadcn::CardTitle::new("Stagger / sequence demo").into_element(cx),
            shadcn::CardDescription::new(
                "One shared timeline mapped into per-item progress via a small headless stagger helper.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new([content]).into_element(cx),
    ])
    .refine_layout(shell_layout)
    .into_element(cx)
    .test_id("ui-gallery-motion-presets-stagger-demo")
}
// endregion: example
