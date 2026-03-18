use std::sync::Arc;

use fret::app::prelude::*;
use fret::{
    children::UiElementSinkExt as _,
    icons::IconId,
    style::{LayoutRefinement, MetricRef, Space},
};
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

mod act {
    fret::actions!([Reset = "cookbook.markdown_and_code_basics.reset.v1"]);
}

#[derive(Clone)]
struct PreviewSettings {
    wrap: Arc<str>,
    cap_height: bool,
}

struct MarkdownAndCodeBasicsView;

impl View for MarkdownAndCodeBasicsView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let source_state = cx.state().local_init(|| SAMPLE_MARKDOWN.to_string());
        let wrap_state = cx.state().local_init(|| Some(Arc::from(WRAP_SCROLL_X)));
        let cap_height_state = cx.state().local_init(|| true);

        let source = source_state.layout_value(cx);
        let preview_settings: PreviewSettings =
            cx.data()
                .selector_layout((&wrap_state, &cap_height_state), |(wrap, cap_height)| {
                    PreviewSettings {
                        wrap: wrap.unwrap_or_else(|| Arc::from(WRAP_SCROLL_X)),
                        cap_height,
                    }
                });

        let wrap_mode = match preview_settings.wrap.as_ref() {
            WRAP_WORD => CodeBlockWrap::Word,
            _ => CodeBlockWrap::ScrollX,
        };

        let max_height = if preview_settings.cap_height {
            Some(Px(220.0))
        } else {
            None
        };

        let mut components = markdown::MarkdownComponents::<App>::default()
            .with_open_url()
            .with_code_block_wrap(wrap_mode)
            .with_code_block_max_height(max_height);
        // Keep the "Copy" affordance visible in scripts/screenshots without requiring hover.
        components.code_block_ui.copy_button_on_hover = false;

        let wrap_toggle = shadcn::ToggleGroup::single(wrap_state.clone_model())
            .items([
                shadcn::ToggleGroupItem::new(WRAP_SCROLL_X, [cx.text("Scroll X")])
                    .a11y_label("Scroll horizontally")
                    .test_id(TEST_ID_WRAP_SCROLL_X),
                shadcn::ToggleGroupItem::new(WRAP_WORD, [cx.text("Word wrap")])
                    .a11y_label("Word wrap")
                    .test_id(TEST_ID_WRAP_WORD),
            ])
            .refine_layout(LayoutRefinement::default().flex_none())
            .test_id(TEST_ID_WRAP);

        let cap_switch =
            shadcn::Switch::new(cap_height_state.clone_model()).test_id(TEST_ID_CAP_HEIGHT);

        let reset = shadcn::Button::new("Reset sample")
            .variant(shadcn::ButtonVariant::Secondary)
            .size(shadcn::ButtonSize::Sm)
            .icon(IconId::new_static("ui.reset"))
            .action(act::Reset)
            .test_id(TEST_ID_RESET);

        let controls = ui::v_flex(|cx| {
            ui::children![
                cx;
                ui::h_flex(|cx| {
                    ui::children![
                        cx;
                        shadcn::Label::new("Code wrap:"),
                        ui::h_flex(|_cx| [wrap_toggle]).w_full().justify_center(),
                    ]
                })
                .gap(Space::N2)
                .items_center(),
                ui::h_flex(|cx| {
                    ui::children![
                        cx;
                        shadcn::Label::new("Cap code block height:"),
                        cap_switch,
                        reset,
                    ]
                })
                .gap(Space::N2)
                .items_center(),
            ]
        })
        .gap(Space::N2);

        let editor = shadcn::Textarea::new(&source_state)
            .a11y_label("Markdown source")
            .placeholder("Markdown…")
            .min_height(Px(420.0))
            .test_id(TEST_ID_SOURCE);

        let preview_content =
            markdown::markdown_with(cx, &source, &components).test_id(TEST_ID_PREVIEW);
        let preview_scroll = shadcn::ScrollArea::build(|cx, out| {
            out.push_ui(cx, preview_content);
        })
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(420.0))),
        )
        .into_element(cx)
        .test_id(TEST_ID_PREVIEW_SCROLL);

        let left = ui::v_flex(|cx| ui::children![cx; editor])
            .gap(Space::N2)
            .flex_1()
            .min_w_0();
        let right = ui::v_flex(|cx| ui::children![cx; preview_scroll])
            .gap(Space::N2)
            .flex_1()
            .min_w_0();

        let panels = ui::h_flex(|cx| ui::children![cx; left, right])
            .gap(Space::N4)
            .items_stretch()
            .w_full();

        let body = ui::v_flex(|cx| ui::children![cx; controls, shadcn::Separator::new(), panels])
            .gap(Space::N3);

        let card = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("Markdown and code basics"),
                        shadcn::card_description(
                            "Markdown rendering + fenced code blocks (copy button, wrap mode, max height).",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| ui::children![cx; body]),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(980.0));

        cx.actions()
            .local_set::<act::Reset, String>(&source_state, SAMPLE_MARKDOWN.to_string());

        fret_cookbook::scaffold::centered_page_background(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-markdown-and-code-basics")
        .window("cookbook-markdown-and-code-basics", (1080.0, 820.0))
        .config_files(false)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<MarkdownAndCodeBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
