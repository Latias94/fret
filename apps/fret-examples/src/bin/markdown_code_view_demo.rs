#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use anyhow::Context as _;
use fret_app::{App, Effect};
use fret_core::{AppWindowId, Event, Px, Rect, UiServices};
use fret_launch::{WinitAppDriver, WinitEventContext, WinitRenderContext, WinitRunnerConfig};
use fret_markdown as markdown;
use fret_ui::declarative;
use fret_ui::element::{ContainerProps, FlexProps, LayoutStyle, Length, MainAlign};
use fret_ui::{Invalidation, Theme, UiTree};
use fret_ui_kit::declarative::scroll as decl_scroll;
use fret_ui_kit::LayoutRefinement;

struct MarkdownCodeViewWindowState {
    ui: UiTree<App>,
    markdown: Arc<str>,
}

#[derive(Default)]
struct MarkdownCodeViewDriver;

impl MarkdownCodeViewDriver {
    fn render(
        app: &mut App,
        ui: &mut UiTree<App>,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        markdown_source: Arc<str>,
    ) {
        let root = declarative::RenderRootContext::new(ui, app, services, window, bounds)
            .render_root("markdown-code-view-demo", |cx| {
                cx.observe_global::<Theme>(Invalidation::Layout);

                let theme = Theme::global(&*cx.app).clone();

                let mut root_layout = LayoutStyle::default();
                root_layout.size.width = Length::Fill;
                root_layout.size.height = Length::Fill;

                let mut components = markdown::MarkdownComponents::<App>::default();
                components.code_block_ui.max_height = Some(Px(360.0));

                vec![cx.container(
                    ContainerProps {
                        layout: root_layout,
                        background: Some(theme.color_required("background")),
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.flex(
                            FlexProps {
                                layout: root_layout,
                                direction: fret_core::Axis::Vertical,
                                gap: Px(12.0),
                                padding: fret_core::Edges::all(theme.metric_required("metric.padding.md")),
                                justify: MainAlign::Start,
                                align: fret_ui::element::CrossAlign::Stretch,
                                wrap: false,
                            },
                            |cx| {
                                vec![
                                    cx.text("markdown_code_view_demo"),
                                    cx.text("Default markdown code blocks (scrollbar-x + max_height)."),
                                    decl_scroll::overflow_scroll_content(
                                        cx,
                                        LayoutRefinement::default().w_full().flex_1(),
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
                                                        theme.metric_required("metric.padding.md"),
                                                    ),
                                                    ..Default::default()
                                                },
                                                |cx| {
                                                    vec![markdown::Markdown::new(markdown_source.clone())
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
            });

        ui.set_root(root);
    }
}

impl WinitAppDriver for MarkdownCodeViewDriver {
    type WindowState = MarkdownCodeViewWindowState;

    fn create_window_state(&mut self, _app: &mut App, window: AppWindowId) -> Self::WindowState {
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let markdown = Arc::<str>::from(
            r##"
# Code View

This demo uses the default `fret-markdown` fenced code block renderer.

## Long line (horizontal scroll)

```rust
let long = "0123456789012345678901234567890123456789012345678901234567890123456789";
```

## Tall block (max_height + vertical scroll inside code block)

```rust
fn main() {
    for i in 0..200 {
        println!("line {}", i);
    }
}
```
"##
            .to_string(),
        );

        MarkdownCodeViewWindowState { ui, markdown }
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

        MarkdownCodeViewDriver::render(
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
        let mut frame = fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();
        frame.paint_all(scene);
    }
}

fn main() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    fret_bootstrap::BootstrapBuilder::new(App::new(), MarkdownCodeViewDriver::default())
        .configure(|c| {
            *c = WinitRunnerConfig {
                main_window_title: "markdown_code_view_demo".to_string(),
                main_window_size: winit::dpi::LogicalSize::new(920.0, 720.0),
                ..Default::default()
            };
        })
        .with_default_settings_json()
        .context("load .fret/settings.json")?
        .register_icon_pack(fret_icons_lucide::register_icons)
        .run()
        .map_err(anyhow::Error::from)
}
