use std::sync::Arc;

use fret::prelude::*;
use fret_code_view::CodeBlockWrap;
use fret_markdown as markdown;

const TEST_ID_ROOT: &str = "cookbook.markdown_and_code_basics.root";
const TEST_ID_SOURCE: &str = "cookbook.markdown_and_code_basics.source";
const TEST_ID_PREVIEW: &str = "cookbook.markdown_and_code_basics.preview";
const TEST_ID_PREVIEW_SCROLL: &str = "cookbook.markdown_and_code_basics.preview_scroll";
const TEST_ID_WRAP: &str = "cookbook.markdown_and_code_basics.wrap";
const TEST_ID_WRAP_SCROLL_X: &str = "cookbook.markdown_and_code_basics.wrap.scroll_x";
const TEST_ID_WRAP_WORD: &str = "cookbook.markdown_and_code_basics.wrap.word";
const TEST_ID_CAP_HEIGHT: &str = "cookbook.markdown_and_code_basics.cap_height";
const TEST_ID_RESET: &str = "cookbook.markdown_and_code_basics.reset";

const WRAP_SCROLL_X: &str = "scroll_x";
const WRAP_WORD: &str = "word";

const SAMPLE_MARKDOWN: &str = r#"# Markdown and code basics

This example renders Markdown via `fret-markdown` and fenced code blocks via `fret-code-view`.

- Inline code looks like `let x = 1;`
- Links are safe-opened via the host policy: https://example.com

## Fenced code block (Rust)

```rust
fn main() {
    let answer = 42;
    println!("answer={answer}");

    // Long line to verify horizontal scrolling / wrapping behavior:
    println!("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");
}
```
"#;

#[derive(Debug, Clone, Copy)]
enum Msg {
    Reset,
}

struct MarkdownAndCodeBasicsState {
    source: Model<String>,
    wrap: Model<Option<Arc<str>>>,
    cap_height: Model<bool>,
}

struct MarkdownAndCodeBasicsProgram;

impl MvuProgram for MarkdownAndCodeBasicsProgram {
    type State = MarkdownAndCodeBasicsState;
    type Message = Msg;

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        Self::State {
            source: app.models_mut().insert(SAMPLE_MARKDOWN.to_string()),
            wrap: app.models_mut().insert(Some(Arc::from(WRAP_SCROLL_X))),
            cap_height: app.models_mut().insert(true),
        }
    }

    fn update(app: &mut App, state: &mut Self::State, message: Self::Message) {
        match message {
            Msg::Reset => {
                let _ = app
                    .models_mut()
                    .update(&state.source, |v| *v = SAMPLE_MARKDOWN.to_string());
            }
        }
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        let source = cx.watch_model(&state.source).layout().cloned_or_default();
        let wrap = state
            .wrap
            .read(&mut *cx.app, |_host, v| v.clone())
            .ok()
            .flatten()
            .unwrap_or_else(|| Arc::from(WRAP_SCROLL_X));
        let cap_height = cx.watch_model(&state.cap_height).layout().copied_or(true);

        let wrap_mode = match wrap.as_ref() {
            WRAP_WORD => CodeBlockWrap::Word,
            _ => CodeBlockWrap::ScrollX,
        };

        let max_height = if cap_height { Some(Px(220.0)) } else { None };

        let mut components = markdown::MarkdownComponents::<App>::default()
            .with_open_url()
            .with_code_block_wrap(wrap_mode)
            .with_code_block_max_height(max_height);
        // Keep the "Copy" affordance visible in scripts/screenshots without requiring hover.
        components.code_block_ui.copy_button_on_hover = false;

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("Markdown and code basics").into_element(cx),
            shadcn::CardDescription::new(
                "Markdown rendering + fenced code blocks (copy button, wrap mode, max height).",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let wrap_toggle = shadcn::ToggleGroup::single(state.wrap.clone())
            .items([
                shadcn::ToggleGroupItem::new(WRAP_SCROLL_X, [cx.text("Scroll X")])
                    .a11y_label("Scroll horizontally")
                    .test_id(TEST_ID_WRAP_SCROLL_X),
                shadcn::ToggleGroupItem::new(WRAP_WORD, [cx.text("Word wrap")])
                    .a11y_label("Word wrap")
                    .test_id(TEST_ID_WRAP_WORD),
            ])
            .refine_layout(LayoutRefinement::default().flex_none())
            .into_element(cx)
            .test_id(TEST_ID_WRAP);

        let cap_switch = shadcn::Switch::new(state.cap_height.clone())
            .test_id(TEST_ID_CAP_HEIGHT)
            .into_element(cx);

        let reset = shadcn::Button::new("Reset sample")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Sm)
            .icon(IconId::new_static("ui.reset"))
            .on_click(msg.cmd(Msg::Reset))
            .into_element(cx)
            .test_id(TEST_ID_RESET);

        let controls = ui::v_flex(cx, |cx| {
            [
                ui::h_flex(cx, |cx| {
                    [
                        shadcn::Label::new("Code wrap:").into_element(cx),
                        stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full())
                                .justify_center(),
                            |_cx| [wrap_toggle],
                        ),
                    ]
                })
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
                ui::h_flex(cx, |cx| {
                    [
                        shadcn::Label::new("Cap code block height:").into_element(cx),
                        cap_switch,
                        reset,
                    ]
                })
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .into_element(cx);

        let editor = shadcn::Textarea::new(state.source.clone())
            .a11y_label("Markdown source")
            .placeholder("Markdown…")
            .min_height(Px(420.0))
            .into_element(cx)
            .test_id(TEST_ID_SOURCE);

        let preview_content =
            markdown::markdown_with(cx, &source, &components).test_id(TEST_ID_PREVIEW);
        let preview_scroll = shadcn::ScrollArea::new([preview_content])
            .refine_layout(
                LayoutRefinement::default()
                    .w_full()
                    .h_px(MetricRef::Px(Px(420.0))),
            )
            .into_element(cx)
            .test_id(TEST_ID_PREVIEW_SCROLL);

        let left = ui::v_flex(cx, |_cx| [editor])
            .gap(Space::N2)
            .flex_1()
            .min_w_0()
            .into_element(cx);
        let right = ui::v_flex(cx, |_cx| [preview_scroll])
            .gap(Space::N2)
            .flex_1()
            .min_w_0()
            .into_element(cx);

        let panels = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap_x(Space::N4)
                .items_stretch()
                .layout(LayoutRefinement::default().w_full()),
            |_cx| [left, right],
        );

        let body = ui::v_flex(cx, |cx| {
            [controls, shadcn::Separator::new().into_element(cx), panels]
        })
        .gap(Space::N3)
        .into_element(cx);

        let card = shadcn::Card::new([header, shadcn::CardContent::new([body]).into_element(cx)])
            .ui()
            .w_full()
            .max_w(Px(980.0))
            .into_element(cx);

        fret_cookbook::scaffold::centered_page_background(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-markdown-and-code-basics")
        .window("cookbook-markdown-and-code-basics", (1080.0, 820.0))
        .config_files(false)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_mvu::<MarkdownAndCodeBasicsProgram>()
        .map_err(anyhow::Error::from)
}
