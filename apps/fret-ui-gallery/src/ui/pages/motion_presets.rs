use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::motion_presets as snippets;

pub(super) fn preview_motion_presets(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    motion_preset: Model<Option<Arc<str>>>,
    motion_preset_open: Model<bool>,
    dialog_open: Model<bool>,
) -> Vec<AnyElement> {
    cx.keyed("ui_gallery.motion_presets_page", |cx| {
        let preset_selector = snippets::preset_selector::render(
            cx,
            motion_preset.clone(),
            motion_preset_open.clone(),
        );
        let token_snapshot = snippets::token_snapshot::render(cx, theme);
        let overlay_demo = snippets::overlay_demo::render(cx, dialog_open.clone());
        let fluid_tabs_demo = snippets::fluid_tabs_demo::render(cx);
        let stagger_demo = snippets::stagger_demo::render(cx, theme);
        let stack_shift_list_demo = snippets::stack_shift_list_demo::render(cx, theme);

        let body = doc_layout::render_doc_page(
            cx,
            Some(
                "Preview is a contract surface for semantic motion tokens; the goal is stable feel across refresh rates and platforms.",
            ),
            vec![
                DocSection::new("Preset selector", preset_selector)
                    .no_shell()

                    .code_rust_from_file_region(snippets::preset_selector::SOURCE, "example"),
                DocSection::new("Token snapshot", token_snapshot)
                    .no_shell()

                    .code_rust_from_file_region(snippets::token_snapshot::SOURCE, "example"),
                DocSection::new("Overlay demo", overlay_demo)
                    .no_shell()

                    .code_rust_from_file_region(snippets::overlay_demo::SOURCE, "example"),
                DocSection::new("Fluid tabs demo", fluid_tabs_demo)
                    .no_shell()

                    .code_rust_from_file_region(snippets::fluid_tabs_demo::SOURCE, "example"),
                DocSection::new("Stagger / sequence demo", stagger_demo)
                    .no_shell()

                    .code_rust_from_file_region(snippets::stagger_demo::SOURCE, "example"),
                DocSection::new("Stack shift list demo", stack_shift_list_demo)
                    .no_shell()

                    .code_rust_from_file_region(
                        snippets::stack_shift_list_demo::SOURCE,
                        "example",
                    ),
            ],
        );

        vec![body.test_id("ui-gallery-motion-presets-component")]
    })
}
