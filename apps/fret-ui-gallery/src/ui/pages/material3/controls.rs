use super::*;
use fret::UiCx;

use crate::ui::snippets;

use super::shared::{MATERIAL3_INTRO, render_material3_demo_page};

pub(in crate::ui) fn preview_material3_touch_targets(
    cx: &mut UiCx<'_>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::touch_targets::render(
        cx,
        material3_checkbox,
        material3_switch,
        material3_radio_value,
    );

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::touch_targets::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_button(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
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

pub(in crate::ui) fn preview_material3_icon_button(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::material3::icon_button::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::icon_button::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_checkbox(
    cx: &mut UiCx<'_>,
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
    cx: &mut UiCx<'_>,
    selected: Model<bool>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::switch::render(cx, selected);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::switch::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_slider(
    cx: &mut UiCx<'_>,
    value: Model<f32>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::slider::render(cx, value);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::slider::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_radio(
    cx: &mut UiCx<'_>,
    group_value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let demo = snippets::material3::radio::render(cx, group_value);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::radio::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_badge(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::material3::badge::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::badge::SOURCE,
    )
}

pub(in crate::ui) fn preview_material3_segmented_button(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::material3::segmented_button::render(cx);

    render_material3_demo_page(
        cx,
        Some(MATERIAL3_INTRO),
        demo,
        snippets::material3::segmented_button::SOURCE,
    )
}
