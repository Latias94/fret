pub const SOURCE: &str = include_str!("position.rs");

// region: example
use crate::ui::snippets::sonner::{last_action_model, position_model, request};
use fret::app::AppActivateExt as _;
use fret::{UiChild, UiCx};
use fret_ui::element::SemanticsDecoration;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
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
    gap: Space,
    children: Vec<AnyElement>,
) -> impl IntoUiElement<H> + use<H> {
    ui::h_flex(move |_cx| children)
        .gap(gap)
        .items_center()
        .wrap()
        .layout(LayoutRefinement::default().w_full())
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let sonner = shadcn::Sonner::global(&mut *cx.app);
    let last_action = last_action_model(cx);
    let sonner_position = position_model(cx);

    let current = cx
        .get_model_copied(&sonner_position, Invalidation::Layout)
        .unwrap_or(shadcn::ToastPosition::TopCenter);

    let action_button = |cx: &mut UiCx<'_>,
                         label: &'static str,
                         test_id: &'static str,
                         target: shadcn::ToastPosition| {
        let sonner = sonner.clone();
        let position_model = sonner_position.clone();
        let last_action_model = last_action.clone();

        shadcn::Button::new(label)
            .variant(shadcn::ButtonVariant::Outline)
            .listen(cx, move |host, action_cx| {
                let _ = host.models_mut().update(&position_model, |v| *v = target);
                sonner.toast(
                    host,
                    action_cx.window,
                    request("Event has been created")
                        .position(target)
                        .description(format!("position: {}", toast_position_key(target))),
                );
                let _ = host.models_mut().update(&last_action_model, |v| {
                    *v =
                        Arc::<str>::from(format!("sonner.position.{}", toast_position_key(target)));
                });
                host.request_redraw(action_cx.window);
            })
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
    let top_row = wrap_controls_row::<fret_app::App>(Space::N2, top_children).into_element(cx);

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
    let bottom_row =
        wrap_controls_row::<fret_app::App>(Space::N2, bottom_children).into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            top_row,
            bottom_row,
            shadcn::raw::typography::muted(format!(
                "Current toaster position: {}",
                toast_position_key(current)
            ))
            .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-sonner-position"),
    )
    .test_id("ui-gallery-sonner-position-root")
}
// endregion: example
