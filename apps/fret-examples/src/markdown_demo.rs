use anyhow::Context as _;
use fret_app::{App, CommandId, Effect};
use fret_core::{AppWindowId, Event, Px, Rect, UiServices};
use fret_launch::{
    WinitAppBuilder, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitWindowContext,
};
use fret_markdown as markdown;
use fret_ui::declarative;
use fret_ui::element::{ContainerProps, FlexProps, LayoutStyle, Length, MainAlign};
use fret_ui::{Invalidation, Theme, UiTree};
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::declarative::scroll as decl_scroll;
use std::sync::Arc;

struct MarkdownDemoWindowState {
    ui: UiTree<App>,
    markdown: Arc<str>,
}

#[derive(Default)]
struct MarkdownDemoDriver;

impl MarkdownDemoDriver {
    fn build_ui(_app: &mut App, window: AppWindowId) -> MarkdownDemoWindowState {
        let mut ui = UiTree::new();
        ui.set_window(window);

        let markdown: Arc<str> = Arc::from(
            r##"# Markdown Demo

This is a focused demo for `fret-markdown` + `fret-code-view`.

## Text

Paragraphs should wrap and respect the viewport width. Inline code looks like `let x = 1;`.

- Unordered list item A
- Unordered list item B

1. Ordered list item 1
2. Ordered list item 2

> Blockquotes are supported.
> They can span multiple lines.

---

## Fenced Code Blocks

Rust (highlight enabled when `fret-markdown` is built with `syntax-rust`):

```rust
fn main() {
    let answer = 42;
    println!("answer={answer}");
    // Long line to verify horizontal scrolling inside code blocks:
    println!("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");
}
```

Plain fenced code:

```text
hello
world
```

## Table (raw MVP)

| Name | Value |
| ---- | ----- |
| foo  | 1     |
| bar  | 2     |

## Links

- https://example.com
- [OpenAI](https://openai.com)
 "##,
        );

        MarkdownDemoWindowState { ui, markdown }
    }

    fn render(
        app: &mut App,
        ui: &mut UiTree<App>,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        markdown_source: Arc<str>,
    ) {
        let components = markdown::MarkdownComponents::<App>::default().with_open_url();

        let root =
            declarative::RenderRootContext::new(ui, app, services, window, bounds).render_root(
                "markdown-demo",
                |cx| {
                    cx.observe_global::<Theme>(Invalidation::Layout);

                    let theme = Theme::global(&*cx.app).clone();

                    let mut root_layout = LayoutStyle::default();
                    root_layout.size.width = Length::Fill;
                    root_layout.size.height = Length::Fill;

                    vec![cx.container(
                        ContainerProps {
                            layout: root_layout,
                            background: Some(theme.colors.surface_background),
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.flex(
                                FlexProps {
                                    layout: root_layout,
                                    direction: fret_core::Axis::Vertical,
                                    gap: Px(12.0),
                                    padding: fret_core::Edges::all(theme.metrics.padding_md),
                                    justify: MainAlign::Start,
                                    align: fret_ui::element::CrossAlign::Stretch,
                                    wrap: false,
                                },
                                |cx| {
                                    vec![
                                        cx.text("markdown_demo"),
                                        cx.text(
                                            "Scrollable markdown preview (links open via platform shell).",
                                        ),
                                        decl_scroll::overflow_scroll_content(
                                            cx,
                                            LayoutRefinement::default()
                                                .w_full()
                                                .h_full(),
                                            true,
                                            |cx| {
                                                cx.container(
                                                    ContainerProps {
                                                        layout: {
                                                            let mut layout = LayoutStyle::default();
                                                            layout.size.width = Length::Fill;
                                                            layout
                                                        },
                                                        padding: fret_core::Edges::all(
                                                            theme.metrics.padding_md,
                                                        ),
                                                        ..Default::default()
                                                    },
                                                    |cx| {
                                                        vec![markdown::Markdown::new(
                                                            markdown_source.clone(),
                                                        )
                                                        .into_element_with(cx, &components)]
                                                    },
                                                )
                                            },
                                        ),
                                    ]
                                },
                            )]
                        },
                    )]
                },
            );
        ui.set_root(root);
    }
}

impl WinitAppDriver for MarkdownDemoDriver {
    type WindowState = MarkdownDemoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        MarkdownDemoDriver::build_ui(app, window)
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        context
            .state
            .ui
            .propagate_model_changes(context.app, changed);
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        context
            .state
            .ui
            .propagate_global_changes(context.app, changed);
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: CommandId,
    ) {
        let WinitCommandContext {
            app,
            services,
            window,
            state,
        } = context;
        if state.ui.dispatch_command(app, services, &command) {
            return;
        }
        if command.as_str() == "window.close" {
            app.push_effect(Effect::Window(fret_app::WindowRequest::Close(window)));
        }
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
        } = context;
        if matches!(event, Event::WindowCloseRequested) {
            app.push_effect(Effect::Window(fret_app::WindowRequest::Close(window)));
            return;
        }
        state.ui.dispatch_event(app, services, event);
    }

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        } = context;

        MarkdownDemoDriver::render(
            app,
            &mut state.ui,
            services,
            window,
            bounds,
            state.markdown.clone(),
        );

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);
        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
        frame.paint_all(scene);
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<fret_launch::WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: AppWindowId,
    ) {
    }
}

pub fn run() -> anyhow::Result<()> {
    WinitAppBuilder::new(App::new(), MarkdownDemoDriver::default())
        .configure(|config| {
            config.main_window_title = "markdown_demo".to_string();
            config.main_window_size = winit::dpi::LogicalSize::new(920.0, 720.0);
        })
        .run()
        .map_err(anyhow::Error::from)
        .context("run markdown_demo")
}
