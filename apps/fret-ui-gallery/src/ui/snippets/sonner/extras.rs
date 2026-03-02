// region: example
use fret_core::{Axis, Edges};
use fret_ui::element::{FlexProps, LayoutStyle, Length, SemanticsDecoration};
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

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

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>, last_action: Model<Arc<str>>) -> AnyElement {
    let sonner = shadcn::Sonner::global(&mut *cx.app);
    let last_action_model = last_action.clone();

    let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
        sonner.toast(
            host,
            action_cx.window,
            shadcn::ToastRequest::new("Swipe to dismiss")
                .description("Drag up to dismiss (pinned)")
                .duration(None)
                .dismissible(true)
                .test_id("ui-gallery-sonner-demo-toast-swipe"),
        );
        let _ = host.models_mut().update(&last_action_model, |v| {
            *v = Arc::<str>::from("sonner.extras.swipe_dismiss");
        });
        host.request_redraw(action_cx.window);
    });

    let swipe = shadcn::Button::new("Swipe Dismiss Toast")
        .variant(shadcn::ButtonVariant::Outline)
        .on_activate(on_activate)
        .test_id("ui-gallery-sonner-demo-show-swipe")
        .into_element(cx);

    wrap_controls_row(cx, Space::N2, vec![swipe]).attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Group)
            .test_id("ui-gallery-sonner-extras"),
    )
}
// endregion: example
