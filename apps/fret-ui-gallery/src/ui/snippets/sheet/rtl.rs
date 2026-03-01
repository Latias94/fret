// region: example
use fret_core::Px;
use fret_ui_kit::primitives::direction::{LayoutDirection, with_direction_provider};
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
    name: Option<Model<String>>,
    username: Option<Model<String>>,
}

fn profile_fields<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    name: Model<String>,
    username: Model<String>,
) -> AnyElement {
    let field = |cx: &mut ElementContext<'_, H>, label: &'static str, model: Model<String>| {
        shadcn::Field::new([
            shadcn::FieldLabel::new(label).into_element(cx),
            shadcn::Input::new(model)
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
        ])
        .into_element(cx)
    };

    shadcn::FieldSet::new([field(cx, "Name", name), field(cx, "Username", username)])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
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

    let name = match state.name {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("Pedro Duarte"));
            cx.with_state(Models::default, |st| st.name = Some(model.clone()));
            model
        }
    };

    let username = match state.username {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("peduarte"));
            cx.with_state(Models::default, |st| st.username = Some(model.clone()));
            model
        }
    };

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        let trigger_open = open.clone();
        let save_open = open.clone();
        let close_open = open.clone();
        let name_model = name.clone();
        let username_model = username.clone();

        shadcn::Sheet::new(open.clone())
            .side(shadcn::SheetSide::Left)
            .size(Px(420.0))
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Open")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(trigger_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    shadcn::SheetContent::new([
                        shadcn::SheetHeader::new([
                            shadcn::SheetTitle::new("Edit profile").into_element(cx),
                            shadcn::SheetDescription::new(
                                "RTL layout keeps spacing and focus flow aligned.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        profile_fields(cx, name_model.clone(), username_model.clone()),
                        shadcn::SheetFooter::new([
                            shadcn::Button::new("Save changes")
                                .toggle_model(save_open.clone())
                                .into_element(cx),
                            shadcn::Button::new("Close")
                                .variant(shadcn::ButtonVariant::Outline)
                                .toggle_model(close_open.clone())
                                .into_element(cx),
                        ])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                },
            )
    })
    .test_id("ui-gallery-sheet-rtl")
}
// endregion: example
