// region: example
use fret_core::{Axis, Edges};
use fret_ui::element::{FlexProps, LayoutStyle, Length, SemanticsDecoration};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

fn toast_position_key(position: shadcn::ToastPosition) -> &'static str {
    match position {
        shadcn::ToastPosition::TopLeft => "top-left",
        shadcn::ToastPosition::TopCenter => "top-center",
        shadcn::ToastPosition::TopRight => "top-right",
        shadcn::ToastPosition::BottomLeft => "bottom-left",
        shadcn::ToastPosition::BottomCenter => "bottom-center",
        shadcn::ToastPosition::BottomRight => "bottom-right",
    }
}

fn wrap_controls_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    gap: Space,
    children: Vec<AnyElement>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let gap = fret_ui_kit::MetricRef::space(gap).resolve(theme);

    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;

    cx.flex(
        FlexProps {
            layout,
            direction: Axis::Horizontal,
            gap: gap.into(),
            padding: Edges::all(Px(0.0)).into(),
            justify: fret_ui::element::MainAlign::Start,
            align: fret_ui::element::CrossAlign::Center,
            wrap: true,
        },
        |_cx| children,
    )
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    last_action: Model<Arc<str>>,
    sonner_position: Model<shadcn::ToastPosition>,
) -> AnyElement {
    let sonner = shadcn::Sonner::global(&mut *cx.app);

    let current = cx
        .get_model_copied(&sonner_position, Invalidation::Layout)
        .unwrap_or(shadcn::ToastPosition::TopCenter);

    let action_button = |cx: &mut ElementContext<'_, H>,
                         label: &'static str,
                         test_id: &'static str,
                         target: shadcn::ToastPosition| {
        let sonner = sonner.clone();
        let position_model = sonner_position.clone();
        let last_action_model = last_action.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&position_model, |v| *v = target);
            sonner.toast(
                host,
                action_cx.window,
                shadcn::ToastRequest::new("Event has been created")
                    .position(target)
                    .description(format!("position: {}", toast_position_key(target))),
            );
            let _ = host.models_mut().update(&last_action_model, |v| {
                *v = Arc::<str>::from(format!("sonner.position.{}", toast_position_key(target)));
            });
            host.request_redraw(action_cx.window);
        });

        shadcn::Button::new(label)
            .variant(shadcn::ButtonVariant::Outline)
            .on_activate(on_activate)
            .test_id(test_id)
            .into_element(cx)
    };

    let top_children = vec![
        action_button(
            cx,
            "Top Left",
            "ui-gallery-sonner-position-top-left",
            shadcn::ToastPosition::TopLeft,
        ),
        action_button(
            cx,
            "Top Center",
            "ui-gallery-sonner-position-top-center",
            shadcn::ToastPosition::TopCenter,
        ),
        action_button(
            cx,
            "Top Right",
            "ui-gallery-sonner-position-top-right",
            shadcn::ToastPosition::TopRight,
        ),
    ];
    let top_row = wrap_controls_row(cx, Space::N2, top_children);

    let bottom_children = vec![
        action_button(
            cx,
            "Bottom Left",
            "ui-gallery-sonner-position-bottom-left",
            shadcn::ToastPosition::BottomLeft,
        ),
        action_button(
            cx,
            "Bottom Center",
            "ui-gallery-sonner-position-bottom-center",
            shadcn::ToastPosition::BottomCenter,
        ),
        action_button(
            cx,
            "Bottom Right",
            "ui-gallery-sonner-position-bottom-right",
            shadcn::ToastPosition::BottomRight,
        ),
    ];
    let bottom_row = wrap_controls_row(cx, Space::N2, bottom_children);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            vec![
                top_row,
                bottom_row,
                shadcn::typography::muted(
                    cx,
                    format!("Current toaster position: {}", toast_position_key(current)),
                ),
            ]
        },
    )
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-sonner-position"),
    )
    .test_id("ui-gallery-sonner-position-root")
}
// endregion: example
