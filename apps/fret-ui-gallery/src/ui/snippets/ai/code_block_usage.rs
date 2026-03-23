pub const SOURCE: &str = include_str!("code_block_usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_icons_lucide::generated_ids::lucide::FILE_CODE;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::{ColorRef, LayoutRefinement, Space, ui};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app).clone();
    let muted_fg = theme.color_token("muted-foreground");
    let code = Arc::<str>::from(
        "function greet(name: string): string {\n  return `Hello, ${name}!`;\n}\n\nconsole.log(greet(\"World\"));\n",
    );

    let code_block = ui_ai::CodeBlock::new(code)
        .language("typescript")
        .test_id("ui-ai-code-block-usage-root")
        .into_element_with_children(cx, |cx| {
            vec![
                ui_ai::CodeBlockHeader::new([
                    ui_ai::CodeBlockTitle::new([
                        decl_icon::icon_with(
                            cx,
                            FILE_CODE,
                            Some(Px(14.0)),
                            Some(ColorRef::Color(muted_fg)),
                        ),
                        ui_ai::CodeBlockFilename::new("example.ts")
                            .into_element(cx)
                            .test_id("ui-ai-code-block-usage-filename"),
                    ])
                    .into_element(cx),
                    ui_ai::CodeBlockActions::new([ui_ai::CodeBlockCopyButton::from_context()
                        .test_id("ui-ai-code-block-usage-copy")
                        .into_element(cx)])
                    .into_element(cx),
                ])
                .into_element(cx),
            ]
        });

    ui::v_flex(|cx| {
        vec![
            cx.text("CodeBlock usage"),
            cx.text("Minimal compound-parts composition aligned with the official AI Elements usage block."),
            code_block,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
