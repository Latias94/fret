pub const SOURCE: &str = include_str!("extras.rs");

// region: example
use fret_ui::element::SemanticsDecoration;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

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

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    last_action: Model<Arc<str>>,
) -> AnyElement {
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

    wrap_controls_row::<H>(Space::N2, vec![swipe])
        .into_element(cx)
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Group)
                .test_id("ui-gallery-sonner-extras"),
        )
}
// endregion: example
