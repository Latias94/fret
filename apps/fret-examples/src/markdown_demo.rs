use anyhow::Context as _;
use fret_app::{App, CommandId, Effect};
use fret_app_kit::{image_asset_state, svg_asset_state};
use fret_core::{AppWindowId, Event, ImageColorSpace, Px, Rect, SvgFit, UiServices};
use fret_launch::{
    WinitAppBuilder, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitWindowContext,
};
use fret_markdown as markdown;
use fret_ui::declarative;
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, ImageProps, LayoutStyle, Length, MainAlign,
    PressableProps, SvgIconProps, TextProps,
};
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

## Images

Raster (procedural, cached via `ImageAssetCache`):

![Checkerboard](fret-demo://checkerboard)

SVG (cached via `SvgAssetCache`):

![Demo SVG](fret-demo://demo.svg)

External (not fetched by markdown; click to open):

![Remote](https://example.com/logo.png)
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
        let demo_svg_bytes = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect x="0" y="0" width="64" height="64" rx="12" fill="#111827"/><path d="M20 44 L32 20 L44 44 Z" fill="#60A5FA"/></svg>"##;
        let (_demo_svg_key, demo_svg) =
            svg_asset_state::use_svg_bytes_cached(app, services, demo_svg_bytes);

        let checker_rgba = Arc::new(checkerboard_rgba8(96, 96));

        let mut components = markdown::MarkdownComponents::<App>::default().with_open_url();
        let on_link_activate = components.on_link_activate.clone();

        components.image = Some(Arc::new(move |cx, info| {
            let theme = Theme::global(&*cx.app).clone();

            let mut size = LayoutStyle::default();
            size.size.width = Length::Px(Px(96.0));
            size.size.height = Length::Px(Px(96.0));

            match info.src.as_ref() {
                "fret-demo://checkerboard" => {
                    let (key, image, status) = image_asset_state::use_rgba8_image_state(
                        cx.app,
                        cx.window,
                        96,
                        96,
                        checker_rgba.as_slice(),
                        ImageColorSpace::Srgb,
                    );
                    let _ = key;
                    let _ = status;

                    if let Some(image) = image {
                        let mut props = ImageProps::new(image);
                        props.layout = size;
                        cx.image_props(props)
                    } else {
                        cx.container(
                            ContainerProps {
                                layout: size,
                                ..Default::default()
                            },
                            |cx| vec![cx.spinner()],
                        )
                    }
                }
                "fret-demo://demo.svg" => {
                    let mut props = SvgIconProps::new(fret_ui::SvgSource::Id(demo_svg));
                    props.layout = size;
                    props.fit = SvgFit::Contain;
                    props.color = theme.colors.text_primary;
                    cx.svg_icon_props(props)
                }
                _ => render_image_placeholder(
                    cx,
                    &theme,
                    on_link_activate.clone(),
                    markdown::LinkInfo {
                        href: info.src.clone(),
                        text: if info.alt.trim().is_empty() {
                            Arc::<str>::from("[image]".to_string())
                        } else {
                            Arc::<str>::from(format!("[image: {}]", info.alt.trim()))
                        },
                    },
                ),
            }
        }));

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

fn checkerboard_rgba8(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let i = ((y * width + x) * 4) as usize;
            let on = ((x / 8) + (y / 8)) % 2 == 0;
            let (r, g, b) = if on { (240, 240, 240) } else { (24, 24, 24) };
            out[i] = r;
            out[i + 1] = g;
            out[i + 2] = b;
            out[i + 3] = 255;
        }
    }
    out
}

fn render_image_placeholder<H: fret_ui::UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    theme: &Theme,
    on_link_activate: Option<markdown::OnLinkActivate>,
    link: markdown::LinkInfo,
) -> AnyElement {
    let label = link.text.clone();

    let text = Arc::<str>::from(format!("{} ({})", label, link.href));
    if let Some(on_link_activate) = on_link_activate {
        let mut props = PressableProps::default();
        props.a11y.role = Some(fret_core::SemanticsRole::Button);
        props.a11y.label = Some(label);

        return cx.pressable(props, |cx, _state| {
            let on_link_activate = on_link_activate.clone();
            let link = link.clone();
            cx.pressable_on_activate(Arc::new(move |host, cx, reason| {
                on_link_activate(host, cx, reason, link.clone());
            }));

            vec![cx.text_props(TextProps {
                layout: Default::default(),
                text: text.clone(),
                style: None,
                color: Some(theme.colors.text_muted),
                wrap: fret_core::TextWrap::Word,
                overflow: fret_core::TextOverflow::Clip,
            })]
        });
    }

    cx.text_props(TextProps {
        layout: Default::default(),
        text,
        style: None,
        color: Some(theme.colors.text_muted),
        wrap: fret_core::TextWrap::Word,
        overflow: fret_core::TextOverflow::Clip,
    })
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
