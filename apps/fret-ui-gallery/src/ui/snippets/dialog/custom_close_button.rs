pub const SOURCE: &str = include_str!("custom_close_button.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
    share_link: Option<Model<String>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());
    let open = match state.open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
            model
        }
    };
    let share_link = match state.share_link {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(String::from("https://ui.shadcn.com/docs/components/dialog"));
            cx.with_state(Models::default, |st| st.share_link = Some(model.clone()));
            model
        }
    };

    let open_for_trigger = open.clone();
    let open_for_footer = open.clone();
    let link_model = share_link.clone();

    shadcn::Dialog::new(open.clone()).into_element(
        cx,
        move |cx| {
            shadcn::Button::new("Share")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-dialog-custom-close-trigger")
                .toggle_model(open_for_trigger.clone())
                .into_element(cx)
        },
        move |cx| {
            shadcn::DialogContent::new([
                shadcn::DialogHeader::new([
                    shadcn::DialogTitle::new("Share link").into_element(cx),
                    shadcn::DialogDescription::new(
                        "Replace the close affordance with a custom footer action.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::Input::new(link_model.clone())
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
                shadcn::DialogFooter::new([shadcn::Button::new("Close")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .test_id("ui-gallery-dialog-custom-close-footer")
                    .toggle_model(open_for_footer.clone())
                    .into_element(cx)])
                .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
            .into_element(cx)
            .test_id("ui-gallery-dialog-custom-close-content")
        },
    )
}
// endregion: example
