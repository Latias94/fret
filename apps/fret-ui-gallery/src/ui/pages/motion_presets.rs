use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::motion_presets as snippets;

pub(super) fn preview_motion_presets(
    cx: &mut AppComponentCx<'_>,
    motion_preset: Model<Option<Arc<str>>>,
    motion_preset_open: Model<bool>,
) -> Vec<AnyElement> {
    cx.keyed("ui_gallery.motion_presets_page", |cx| {
        let preset_selector = snippets::preset_selector::render(
            cx,
            motion_preset.clone(),
            motion_preset_open.clone(),
        );
        let token_snapshot = snippets::token_snapshot::render(cx);
        let overlay_demo = snippets::overlay_demo::render(cx);
        let fluid_tabs_demo = snippets::fluid_tabs_demo::render(cx);
        let stagger_demo = snippets::stagger_demo::render(cx);
        let stack_shift_list_demo = snippets::stack_shift_list_demo::render(cx);
        let preset_selector = DocSection::build(cx, "Preset selector", preset_selector)
            .no_shell()
            .code_rust_from_file_region(snippets::preset_selector::SOURCE, "example");
        let token_snapshot = DocSection::build(cx, "Token snapshot", token_snapshot)
            .no_shell()
            .code_rust_from_file_region(snippets::token_snapshot::SOURCE, "example");
        let overlay_demo = DocSection::build(cx, "Overlay demo", overlay_demo)
            .no_shell()
            .code_rust_from_file_region(snippets::overlay_demo::SOURCE, "example");
        let fluid_tabs_demo = DocSection::build(cx, "Fluid tabs demo", fluid_tabs_demo)
            .no_shell()
            .code_rust_from_file_region(snippets::fluid_tabs_demo::SOURCE, "example");
        let stagger_demo = DocSection::build(cx, "Stagger / sequence demo", stagger_demo)
            .no_shell()
            .code_rust_from_file_region(snippets::stagger_demo::SOURCE, "example");
        let stack_shift_list_demo =
            DocSection::build(cx, "Stack shift list demo", stack_shift_list_demo)
                .no_shell()
                .code_rust_from_file_region(snippets::stack_shift_list_demo::SOURCE, "example");

        let body = doc_layout::render_doc_page(
            cx,
            Some(
                "Preview is a contract surface for semantic motion tokens; the goal is stable feel across refresh rates and platforms.",
            ),
            vec![
                preset_selector,
                token_snapshot,
                overlay_demo,
                fluid_tabs_demo,
                stagger_demo,
                stack_shift_list_demo,
            ],
        );

        vec![body.test_id("ui-gallery-motion-presets-component").into_element(cx)]
    })
}
