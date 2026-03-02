use super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets;

const MATERIAL3_INTRO: &str =
    "Material 3 surfaces are still migrating to snippet-backed pages (Preview ≡ Code).";

fn render_material3_demo_page(
    cx: &mut ElementContext<'_, App>,
    intro: Option<&'static str>,
    demo: AnyElement,
    source: &'static str,
) -> Vec<AnyElement> {
    let page = doc_layout::render_doc_page(
        cx,
        intro,
        vec![DocSection::new("Demo", demo).code_rust_from_file_region(source, "example")],
    );

    vec![page]
}

pub(in crate::ui) fn material3_scoped_page<I, F>(
    cx: &mut ElementContext<'_, App>,
    material3_expressive: Model<bool>,
    content: F,
) -> Vec<AnyElement>
where
    F: FnOnce(&mut ElementContext<'_, App>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    let enabled = cx
        .get_model_copied(&material3_expressive, Invalidation::Layout)
        .unwrap_or(false);

    let mut out: Vec<AnyElement> = Vec::new();
    out.push(material3_variant_toggle_row(cx, material3_expressive));

    let body = if enabled {
        crate::ui::material3::context::with_material_design_variant(
            cx,
            crate::ui::material3::MaterialDesignVariant::Expressive,
            content,
        )
    } else {
        content(cx)
    };

    out.extend(body);
    out
}

pub(in crate::ui) fn material3_variant_toggle_row(
    cx: &mut ElementContext<'_, App>,
    material3_expressive: Model<bool>,
) -> AnyElement {
    let enabled = cx
        .get_model_copied(&material3_expressive, Invalidation::Layout)
        .unwrap_or(false);

    stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            vec![
                shadcn::Switch::new(material3_expressive.clone())
                    .a11y_label("Enable Material 3 Expressive variant")
                    .test_id("ui-gallery-material3-design-variant-toggle")
                    .into_element(cx),
                ui::label(
                    cx,
                    if enabled {
                        "Variant: Expressive"
                    } else {
                        "Variant: Standard"
                    },
                )
                .into_element(cx),
            ]
        },
    )
}

pub(in crate::ui) fn preview_material3_gallery(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
    material3_list_value: Model<Arc<str>>,
    material3_navigation_bar_value: Model<Arc<str>>,
    material3_text_field_value: Model<String>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let _ = last_action;

    let demo = snippets::material3::gallery::render(
        cx,
        material3_checkbox,
        material3_switch,
        material3_radio_value,
        material3_tabs_value,
        material3_list_value,
        material3_navigation_bar_value,
        material3_text_field_value,
        material3_text_field_disabled,
        material3_text_field_error,
    );

    render_material3_demo_page(cx, Some(MATERIAL3_INTRO), demo, snippets::material3::gallery::SOURCE)
}

pub(in crate::ui) fn preview_material3_state_matrix(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
    material3_navigation_bar_value: Model<Arc<str>>,
    material3_text_field_value: Model<String>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    material3_menu_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::state_matrix::render(
        cx,
        material3_checkbox,
        material3_switch,
        material3_radio_value,
        material3_tabs_value,
        material3_navigation_bar_value,
        material3_text_field_value,
        material3_text_field_disabled,
        material3_text_field_error,
        material3_menu_open,
        last_action,
    );

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::state_matrix::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_touch_targets(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::touch_targets::render(
        cx,
        material3_checkbox,
        material3_switch,
        material3_radio_value,
        material3_tabs_value,
    );

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::touch_targets::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::material3::button::render(cx);

    render_material3_demo_page(
        cx,
        Some(
            "Material 3 surfaces are still migrating to snippet-backed pages. This page is the first scaffolded example.",
        ),
        demo,
        snippets::material3::button::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_icon_button(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::icon_button::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::icon_button::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_checkbox(
    cx: &mut ElementContext<'_, App>,
    checked: Model<bool>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::checkbox::render(cx, checked);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::checkbox::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_switch(
    cx: &mut ElementContext<'_, App>,
    selected: Model<bool>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::switch::render(cx, selected);

    render_material3_demo_page(cx, Some(MATERIAL3_INTRO), demo, snippets::material3::switch::SOURCE)
}

pub(in crate::ui) fn preview_material3_slider(
    cx: &mut ElementContext<'_, App>,
    value: Model<f32>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::slider::render(cx, value);

    render_material3_demo_page(cx, Some(MATERIAL3_INTRO), demo, snippets::material3::slider::SOURCE)
}

pub(in crate::ui) fn preview_material3_radio(
    cx: &mut ElementContext<'_, App>,
    group_value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::radio::render(cx, group_value);

    render_material3_demo_page(cx, Some(MATERIAL3_INTRO), demo, snippets::material3::radio::SOURCE)
}

pub(in crate::ui) fn preview_material3_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::material3::badge::render(cx);

    render_material3_demo_page(cx, Some(MATERIAL3_INTRO), demo, snippets::material3::badge::SOURCE)
}

pub(in crate::ui) fn preview_material3_top_app_bar(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::top_app_bar::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::top_app_bar::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_bottom_sheet(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::bottom_sheet::render(cx, open);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::bottom_sheet::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<time::Date>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::date_picker::render(cx, open, month, selected);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::date_picker::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_time_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    selected: Model<time::Time>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::time_picker::render(cx, open, selected);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::time_picker::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_segmented_button(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::segmented_button::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::segmented_button::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_select(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::material3::select::render(cx);

    render_material3_demo_page(cx, Some(MATERIAL3_INTRO), demo, snippets::material3::select::SOURCE)
}

pub(in crate::ui) fn preview_material3_autocomplete(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
    disabled: Model<bool>,
    error: Model<bool>,
    dialog_open: Model<bool>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::autocomplete::render(cx, value, disabled, error, dialog_open);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::autocomplete::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_text_field(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
    disabled: Model<bool>,
    error: Model<bool>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::text_field::render(cx, value, disabled, error);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::text_field::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_tabs(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::tabs::render(cx, value);

    render_material3_demo_page(cx, Some(MATERIAL3_INTRO), demo, snippets::material3::tabs::SOURCE)
}

pub(in crate::ui) fn preview_material3_list(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::list::render(cx, value);

    render_material3_demo_page(cx, Some(MATERIAL3_INTRO), demo, snippets::material3::list::SOURCE)
}

pub(in crate::ui) fn preview_material3_navigation_bar(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::navigation_bar::render(cx, value);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::navigation_bar::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_navigation_rail(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::navigation_rail::render(cx, value);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::navigation_rail::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_navigation_drawer(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::navigation_drawer::render(cx, value);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::navigation_drawer::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_modal_navigation_drawer(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::modal_navigation_drawer::render(cx, open, value);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::modal_navigation_drawer::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::dialog::render(cx, open, last_action);

    render_material3_demo_page(cx, Some(MATERIAL3_INTRO), demo, snippets::material3::dialog::SOURCE)
}

pub(in crate::ui) fn preview_material3_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::menu::render(cx, open, last_action);

    render_material3_demo_page(cx, Some(MATERIAL3_INTRO), demo, snippets::material3::menu::SOURCE)
}

pub(in crate::ui) fn preview_material3_snackbar(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::snackbar::render(cx, last_action);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::snackbar::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_tooltip(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::tooltip::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::tooltip::SOURCE,
    )
}
