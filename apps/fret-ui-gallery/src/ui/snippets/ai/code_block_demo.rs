pub const SOURCE: &str = include_str!("code_block_demo.rs");

// region: example
use fret_icons_lucide::generated_ids::lucide::FILE_CODE;
use fret_runtime::Model;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use fret_ui_shadcn::{
    Select, SelectContent, SelectItem, SelectTrigger, SelectTriggerSize, SelectValue,
};
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    language: Option<Model<Option<Arc<str>>>>,
    language_open: Option<Model<bool>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let muted_fg = theme.color_token("muted-foreground");

    let language = cx.with_state(DemoModels::default, |st| st.language.clone());
    let language = match language {
        Some(model) => model,
        None => {
            let model = cx
                .app
                .models_mut()
                .insert(Some(Arc::<str>::from("typescript")));
            cx.with_state(DemoModels::default, |st| {
                st.language = Some(model.clone());
            });
            model
        }
    };

    let language_open = cx.with_state(DemoModels::default, |st| st.language_open.clone());
    let language_open = match language_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(DemoModels::default, |st| {
                st.language_open = Some(model.clone());
            });
            model
        }
    };

    let language_value = cx
        .watch_model(&language)
        .paint()
        .value_or_else(|| Some(Arc::<str>::from("typescript")))
        .unwrap_or_else(|| Arc::<str>::from("typescript"));

    let (code, filename) = match language_value.as_ref() {
        "python" => (
            Arc::<str>::from(
                "def greet(name: str) -> str:\n    return f\"Hello, {name}!\"\n\nprint(greet(\"World\"))\n",
            ),
            Arc::<str>::from("greet.py"),
        ),
        "rust" => (
            Arc::<str>::from(
                "fn greet(name: &str) -> String {\n    format!(\"Hello, {}!\", name)\n}\n\nfn main() {\n    println!(\"{}\", greet(\"World\"));\n}\n",
            ),
            Arc::<str>::from("greet.rs"),
        ),
        "go" => (
            Arc::<str>::from(
                "package main\n\nimport \"fmt\"\n\nfunc greet(name string) string {\n    return fmt.Sprintf(\"Hello, %s!\", name)\n}\n\nfunc main() {\n    fmt.Println(greet(\"World\"))\n}\n",
            ),
            Arc::<str>::from("greet.go"),
        ),
        _ => (
            Arc::<str>::from(
                "function greet(name: string): string {\n  return `Hello, ${name}!`;\n}\n\nconsole.log(greet(\"World\"));\n",
            ),
            Arc::<str>::from("greet.ts"),
        ),
    };

    let language_select = Select::new(language.clone(), language_open.clone())
        .trigger_test_id("ui-ai-code-block-language-trigger")
        .on_value_change({
            let language = language.clone();
            move |host, action_cx, value| {
                let _ = host.models_mut().update(&language, |v| *v = Some(value));
                host.notify(action_cx);
            }
        })
        .into_element_parts(
            cx,
            |_cx| {
                SelectTrigger::new()
                    .size(SelectTriggerSize::Sm)
                    .refine_style(
                        ChromeRefinement::default()
                            .border_width(Px(0.0))
                            .bg(ColorRef::Color(fret_core::Color::TRANSPARENT))
                            .px(Space::N2)
                            .py(Space::N0p5),
                    )
            },
            |_cx| SelectValue::new().placeholder("Language"),
            |_cx| {
                SelectContent::new().with_entries([
                    SelectItem::new("typescript", "TypeScript")
                        .test_id("ui-ai-code-block-language-item-typescript")
                        .into(),
                    SelectItem::new("python", "Python")
                        .test_id("ui-ai-code-block-language-item-python")
                        .into(),
                    SelectItem::new("rust", "Rust")
                        .test_id("ui-ai-code-block-language-item-rust")
                        .into(),
                    SelectItem::new("go", "Go")
                        .test_id("ui-ai-code-block-language-item-go")
                        .into(),
                ])
            },
        );

    let code_block = ui_ai::CodeBlock::new(code.clone())
        .language(language_value.clone())
        .show_line_numbers(false)
        .test_id("ui-ai-code-block-root")
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
                        ui_ai::CodeBlockFilename::new(filename.clone())
                            .into_element(cx)
                            .test_id("ui-ai-code-block-filename"),
                    ])
                    .into_element(cx),
                    ui_ai::CodeBlockActions::new([
                        language_select,
                        ui_ai::CodeBlockCopyButton::from_context()
                            .test_id("ui-ai-code-block-copy")
                            .copied_marker_test_id("ui-ai-code-block-copied-marker")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx),
            ]
        });

    ui::v_flex(|cx| {
        vec![
            cx.text("CodeBlock (AI Elements)"),
            cx.text("Composable header/title/actions composition aligned with the official AI Elements example."),
            code_block,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
