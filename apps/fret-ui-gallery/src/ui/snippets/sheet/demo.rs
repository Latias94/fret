pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

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
            let model = cx.app.models_mut().insert(String::from("@peduarte"));
            cx.with_state(Models::default, |st| st.username = Some(model.clone()));
            model
        }
    };

    let trigger_open = open.clone();
    let save_open = open.clone();
    let close_open = open.clone();
    let name_model = name.clone();
    let username_model = username.clone();

    shadcn::Sheet::new(open.clone())
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("Open")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-sheet-demo-trigger")
                    .toggle_model(trigger_open.clone())
                    .into_element(cx)
            },
            |cx| {
                let fields = {
                    let fields = profile_fields(cx, name_model.clone(), username_model.clone());
                    let props = decl_style::container_props(
                        Theme::global(&*cx.app),
                        ChromeRefinement::default().px(Space::N4),
                        LayoutRefinement::default()
                            .w_full()
                            .min_w_0()
                            .min_h_0()
                            .flex_1(),
                    );
                    cx.container(props, move |_cx| vec![fields])
                };
                shadcn::SheetContent::new([
                    shadcn::SheetHeader::new([
                        shadcn::SheetTitle::new("Edit profile").into_element(cx),
                        shadcn::SheetDescription::new(
                            "Make changes to your profile here. Click save when you're done.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    fields,
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
                .test_id("ui-gallery-sheet-demo-content")
            },
        )
        .test_id("ui-gallery-sheet-demo")
}
// endregion: example
