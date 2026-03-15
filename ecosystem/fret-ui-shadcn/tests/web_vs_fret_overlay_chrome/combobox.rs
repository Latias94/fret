use super::*;
use fret_ui_shadcn::facade as shadcn;

#[path = "combobox/fixtures.rs"]
mod fixtures;

fn build_shadcn_combobox_demo_page(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use shadcn::{Combobox, ComboboxItem};

    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    let items = vec![
        ComboboxItem::new("apple", "Apple"),
        ComboboxItem::new("banana", "Banana"),
        ComboboxItem::new("blueberry", "Blueberry"),
        ComboboxItem::new("grapes", "Grapes"),
        ComboboxItem::new("pineapple", "Pineapple"),
    ];

    Combobox::new(value, open.clone())
        .a11y_label("Select a fruit")
        .items(items)
        .into_element_parts(cx, |_cx| {
            vec![shadcn::ComboboxPart::from(
                shadcn::ComboboxTrigger::new().width_px(Px(200.0)),
            )]
        })
}
