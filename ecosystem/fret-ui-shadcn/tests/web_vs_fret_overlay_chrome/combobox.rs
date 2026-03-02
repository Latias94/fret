use super::*;

#[path = "combobox/fixtures.rs"]
mod fixtures;

fn build_shadcn_combobox_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{Combobox, combobox_option};

    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let items = vec![
        combobox_option("apple", "Apple"),
        combobox_option("banana", "Banana"),
        combobox_option("blueberry", "Blueberry"),
        combobox_option("grapes", "Grapes"),
        combobox_option("pineapple", "Pineapple"),
    ];

    Combobox::new(value, open.clone())
        .a11y_label("Select a fruit")
        .width(Px(200.0))
        .options(items)
        .into_element(cx)
}
