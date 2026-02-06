use super::super::*;

pub(super) fn preview_toggle(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct ToggleModels {
        pressed: Option<Model<bool>>,
    }

    let pressed = cx.with_state(ToggleModels::default, |st| st.pressed.clone());
    let pressed = match pressed {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ToggleModels::default, |st| st.pressed = Some(model.clone()));
            model
        }
    };

    vec![stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            vec![
                shadcn::Toggle::new(pressed.clone())
                    .label("Bold")
                    .a11y_label("Bold")
                    .into_element(cx),
                shadcn::Toggle::new(pressed.clone())
                    .label("Outline")
                    .variant(shadcn::ToggleVariant::Outline)
                    .a11y_label("Bold (outline)")
                    .into_element(cx),
            ]
        },
    )]
}
