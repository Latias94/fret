pub const SOURCE: &str = include_str!("file.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

const CMD_INPUT_PICTURE_BROWSE: &str = "ui_gallery.input.picture.browse";

#[derive(Default)]
struct Models {
    file_value: Option<Model<String>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let file_value = cx.with_state(Models::default, |st| st.file_value.clone());
    let file_value = file_value.unwrap_or_else(|| {
        let model = cx.app.models_mut().insert(String::new());
        cx.with_state(Models::default, |st| st.file_value = Some(model.clone()));
        model
    });

    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let selected = cx
        .app
        .models()
        .read(&file_value, |v: &String| v.clone())
        .ok()
        .unwrap_or_default();
    let selected = selected.trim();

    let mut children = Vec::new();
    children.push(shadcn::FieldLabel::new("Picture").into_element(cx));
    children.push(
        shadcn::ButtonGroup::new([
            shadcn::Input::new(file_value)
                .a11y_label("Picture path")
                .placeholder("Choose a file")
                .into_element(cx)
                .into(),
            shadcn::Button::new("Browse")
                .variant(shadcn::ButtonVariant::Outline)
                .on_click(CMD_INPUT_PICTURE_BROWSE)
                .test_id("ui-gallery-input-file-browse")
                .into_element(cx)
                .into(),
        ])
        .into_element(cx),
    );
    children.push(
        shadcn::FieldDescription::new("Native file picking uses a file dialog.").into_element(cx),
    );
    if !selected.is_empty() {
        children.push(
            shadcn::raw::typography::muted(cx, format!("Selected file: {selected}"))
                .test_id("ui-gallery-input-file-selected"),
        );
    }

    shadcn::Field::new(children)
        .refine_layout(max_w_xs)
        .into_element(cx)
        .test_id("ui-gallery-input-file")
}
// endregion: example
