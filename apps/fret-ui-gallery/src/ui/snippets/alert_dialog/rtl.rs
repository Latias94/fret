pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
}

fn open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
            model
        }
    }
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = open_model(cx);
    let open_for_trigger = open.clone();
    let open_for_children = open.clone();

    with_direction_provider(cx, LayoutDirection::Rtl, move |cx| {
        shadcn::AlertDialog::new(open.clone()).into_element(
            cx,
            move |cx| {
                shadcn::Button::new("عرض الحوار")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(open_for_trigger.clone())
                    .test_id("ui-gallery-alert-dialog-rtl-trigger")
                    .into_element(cx)
            },
            move |cx| {
                let header = shadcn::AlertDialogHeader::new(vec![
                    shadcn::AlertDialogTitle::new("هل أنت متأكد تمامًا؟").into_element(cx),
                    shadcn::AlertDialogDescription::new(
                        "لا يمكن التراجع عن هذا الإجراء. سيؤدي ذلك إلى حذف حسابك نهائيًا من خوادمنا.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx);
                let footer = shadcn::AlertDialogFooter::new(vec![
                    shadcn::AlertDialogCancel::new("إلغاء", open_for_children.clone())
                        .test_id("ui-gallery-alert-dialog-rtl-cancel")
                        .into_element(cx),
                    shadcn::AlertDialogAction::new("متابعة", open_for_children.clone())
                        .test_id("ui-gallery-alert-dialog-rtl-action")
                        .into_element(cx),
                ])
                .into_element(cx);

                shadcn::AlertDialogContent::new(vec![header, footer])
                    .into_element(cx)
                    .test_id("ui-gallery-alert-dialog-rtl-content")
            },
        )
    })
}
// endregion: example
