pub const SOURCE: &str = include_str!("file.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

const CMD_INPUT_PICTURE_BROWSE: &str = "ui_gallery.input.picture.browse";

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let file_value = cx.local_model(String::new);
    let picture_id = "ui-gallery-input-file-picture";

    let max_w_xs = LayoutRefinement::default().w_full().max_w(Px(320.0));

    let selected = cx
        .app
        .models()
        .read(&file_value, |v: &String| v.clone())
        .ok()
        .unwrap_or_default();
    let selected = selected.trim();

    let mut children = Vec::new();
    children.push(
        shadcn::FieldLabel::new("Picture")
            .for_control(picture_id)
            .into_element(cx),
    );
    children.push(
        shadcn::ButtonGroup::new([
            shadcn::Input::new(file_value)
                .control_id(picture_id)
                .placeholder("Choose a file")
                .into_element(cx)
                .into(),
            shadcn::Button::new("Browse")
                .variant(shadcn::ButtonVariant::Outline)
                .action(CMD_INPUT_PICTURE_BROWSE)
                .test_id("ui-gallery-input-file-browse")
                .into_element(cx)
                .into(),
        ])
        .into_element(cx),
    );
    children.push(
        shadcn::FieldDescription::new("Select a picture to upload.")
            .for_control(picture_id)
            .into_element(cx),
    );
    if !selected.is_empty() {
        children.push(
            shadcn::raw::typography::muted(format!("Selected file: {selected}"))
                .into_element(cx)
                .test_id("ui-gallery-input-file-selected"),
        );
    }

    shadcn::Field::new(children)
        .refine_layout(max_w_xs)
        .into_element(cx)
        .test_id("ui-gallery-input-file")
}
// endregion: example
