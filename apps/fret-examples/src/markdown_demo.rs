use anyhow::Context as _;
use fret_app::{App, CommandId, Effect};
use fret_core::{AppWindowId, Event, ImageColorSpace, Px, Rect, SvgFit, SvgId, UiServices};
use fret_launch::{
    WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext, WinitRunnerConfig,
    WinitWindowContext,
};
use fret_markdown as markdown;
use fret_runtime::Model;
use fret_ui::declarative;
use fret_ui::element::{
    AnyElement, ImageProps, LayoutStyle, Length, PressableProps, SvgIconProps, TextProps,
};
use fret_ui::{Invalidation, Theme, ThemeConfig, UiTree};
use fret_ui_assets::{image_asset_state, svg_asset_state};
use fret_ui_kit::declarative::GlobalWatchExt as _;
use fret_ui_kit::declarative::scroll as decl_scroll;
use fret_ui_kit::{ColorRef, Edges4, LayoutRefinement, MetricRef, Space, ui};
use fret_ui_shadcn as shadcn;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::mpsc;

const CMD_TOGGLE_CODE_BLOCK_EXPAND_PREFIX: &str = "markdown_demo.code_block.toggle_expand:";

struct MarkdownDemoWindowState {
    ui: UiTree<App>,
    markdown: Arc<str>,
    wrap_code: Model<bool>,
    cap_code_height: Model<bool>,
    expanded_code_blocks: Model<HashSet<markdown::BlockId>>,
}

#[derive(Debug, Clone)]
enum RemoteImageState {
    Loading,
    ReadyRaster {
        width: u32,
        height: u32,
        rgba: Arc<[u8]>,
    },
    ReadySvg {
        bytes: Arc<[u8]>,
        svg: Option<SvgId>,
    },
    Error(Arc<str>),
}

#[derive(Debug)]
struct RemoteImageCache {
    states: HashMap<Arc<str>, RemoteImageState>,
    tx: mpsc::Sender<(Arc<str>, Result<RemoteImageState, Arc<str>>)>,
    rx: mpsc::Receiver<(Arc<str>, Result<RemoteImageState, Arc<str>>)>,
}

impl Default for RemoteImageCache {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            states: HashMap::new(),
            tx,
            rx,
        }
    }
}

impl RemoteImageCache {
    fn poll_completed(&mut self) -> bool {
        let mut changed = false;
        while let Ok((url, result)) = self.rx.try_recv() {
            changed = true;
            match result {
                Ok(state) => {
                    self.states.insert(url, state);
                }
                Err(err) => {
                    self.states.insert(url, RemoteImageState::Error(err));
                }
            }
        }
        changed
    }

    fn ensure_svgs_registered(&mut self, host: &mut App, services: &mut dyn UiServices) -> bool {
        let mut changed = false;
        for state in self.states.values_mut() {
            let RemoteImageState::ReadySvg { bytes, svg } = state else {
                continue;
            };
            if svg.is_some() {
                continue;
            }
            let (_key, id) = svg_asset_state::use_svg_bytes_cached(host, services, bytes);
            *svg = Some(id);
            changed = true;
        }
        changed
    }

    fn ensure_fetch_started(&mut self, url: Arc<str>) {
        if self.states.contains_key(&url) {
            return;
        }
        self.states.insert(url.clone(), RemoteImageState::Loading);

        let tx = self.tx.clone();
        std::thread::spawn(move || {
            let result = download_remote_image(&url);
            let _ = tx.send((url, result));
        });
    }

    fn get(&self, url: &str) -> Option<&RemoteImageState> {
        self.states.get(url)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn download_remote_image(url: &str) -> Result<RemoteImageState, Arc<str>> {
    use std::io::Read as _;

    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(Arc::<str>::from(
            "only http/https are supported in this demo",
        ));
    }

    let response = ureq::get(url)
        .set("User-Agent", "fret-markdown-demo")
        .set("Accept", "image/*")
        .call()
        .map_err(|e| Arc::<str>::from(format!("request failed: {e}")))?;

    let status = response.status();
    if !(200..=299).contains(&status) {
        return Err(Arc::<str>::from(format!("http status {status}")));
    }

    let content_type = response
        .header("content-type")
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();

    let is_svg = content_type == "image/svg+xml" || url.to_ascii_lowercase().ends_with(".svg");

    let mut reader = response.into_reader();
    let mut bytes = Vec::new();
    let mut buf = [0u8; 16 * 1024];
    let max_bytes = 4 * 1024 * 1024;
    while let Ok(n) = reader.read(&mut buf) {
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&buf[..n]);
        if bytes.len() > max_bytes {
            return Err(Arc::<str>::from("image too large (>4MiB)"));
        }
    }

    if is_svg {
        return Ok(RemoteImageState::ReadySvg {
            bytes: Arc::<[u8]>::from(bytes),
            svg: None,
        });
    }

    let image = image::load_from_memory(&bytes)
        .map_err(|e| Arc::<str>::from(format!("decode failed: {e}")))?;
    let rgba = image.to_rgba8();
    let (w, h) = rgba.dimensions();

    let pixel_budget = 8_000_000u64;
    let px = (w as u64) * (h as u64);
    if px > pixel_budget {
        return Err(Arc::<str>::from("decoded image too large"));
    }

    Ok(RemoteImageState::ReadyRaster {
        width: w,
        height: h,
        rgba: Arc::<[u8]>::from(rgba.into_raw()),
    })
}

#[cfg(target_arch = "wasm32")]
fn download_remote_image(_url: &str) -> Result<RemoteImageState, Arc<str>> {
    Err(Arc::<str>::from(
        "remote images are not supported on wasm demo",
    ))
}

#[derive(Default)]
struct MarkdownDemoDriver;

impl MarkdownDemoDriver {
    fn build_ui(app: &mut App, window: AppWindowId) -> MarkdownDemoWindowState {
        apply_markdown_demo_theme_tokens(app);

        let mut ui = UiTree::new();
        ui.set_window(window);

        let wrap_code = app.models_mut().insert(false);
        let cap_code_height = app.models_mut().insert(true);
        let expanded_code_blocks = app.models_mut().insert(HashSet::new());

        let markdown: Arc<str> = Arc::from(
            r##"# Markdown Demo

This is a focused demo for `fret-markdown` + `fret-code-view`.

## Text

Paragraphs should wrap and respect the viewport width. Inline code looks like `let x = 1;`.
Emphasis looks like *italic* and **bold**, and both ***together***.
Strikethrough looks like ~~deleted~~.

- Unordered list item A
- Unordered list item B

1. Ordered list item 1
2. Ordered list item 2

- [ ] Task unchecked
- [x] Task checked

Footnotes are supported.[^note]

[^note]: This is a footnote definition.

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
// Word-wrap test (only wraps at whitespace today):
println!("word wrap test: a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a");
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

Network image (demo-only; fetched by the host's image hook):

![HTTPBin PNG](https://httpbin.org/image/png)
![HTTPBin JPEG](https://httpbin.org/image/jpeg)
![Rust SVG](https://raw.githubusercontent.com/simple-icons/simple-icons/develop/icons/rust.svg)

## Math

Inline math: $E = mc^2$.

Display math:

$$
\int_0^1 x^2\,dx = \frac{1}{3}
$$
 "##,
        );

        MarkdownDemoWindowState {
            ui,
            markdown,
            wrap_code,
            cap_code_height,
            expanded_code_blocks,
        }
    }

    fn render(
        app: &mut App,
        ui: &mut UiTree<App>,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        markdown_source: Arc<str>,
        wrap_code: Model<bool>,
        cap_code_height: Model<bool>,
        expanded_code_blocks: Model<HashSet<markdown::BlockId>>,
    ) {
        let cache_changed = app.with_global_mut(RemoteImageCache::default, |cache, app| {
            let mut changed = cache.poll_completed();
            changed |= cache.ensure_svgs_registered(app, services);
            changed
        });
        if cache_changed {
            app.request_redraw(window);
        }

        let demo_svg_bytes = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect x="0" y="0" width="64" height="64" rx="12" fill="#111827"/><path d="M20 44 L32 20 L44 44 Z" fill="#60A5FA"/></svg>"##;
        let (_demo_svg_key, demo_svg) =
            svg_asset_state::use_svg_bytes_cached(app, services, demo_svg_bytes);

        let checker_rgba = Arc::new(checkerboard_rgba8(96, 96));

        let wrap_enabled = app.models().get_copied(&wrap_code).unwrap_or(false);
        let cap_enabled = app.models().get_copied(&cap_code_height).unwrap_or(true);

        let mut components = markdown::MarkdownComponents::<App>::default().with_open_url();
        components.code_block_ui.wrap = if wrap_enabled {
            fret_code_view::CodeBlockWrap::Word
        } else {
            fret_code_view::CodeBlockWrap::ScrollX
        };
        components.code_block_max_height_from_theme = cap_enabled;
        components.code_block_ui.show_scrollbar_y = cap_enabled;
        components.code_block_ui.scrollbar_y_on_hover = true;

        if cap_enabled {
            let expanded_for_resolver = expanded_code_blocks.clone();
            components.code_block_ui_resolver = Some(Arc::new(move |cx, info, options| {
                cx.observe_model(&expanded_for_resolver, Invalidation::Layout);
                let expanded = cx
                    .app
                    .models()
                    .read(&expanded_for_resolver, |set| set.contains(&info.id))
                    .ok()
                    .unwrap_or(false);
                if expanded {
                    options.max_height = None;
                    options.show_scrollbar_y = false;
                }
            }));

            let expanded_for_actions = expanded_code_blocks.clone();
            components.code_block_actions = Some(Arc::new(move |cx, info| {
                cx.observe_model(&expanded_for_actions, Invalidation::Layout);
                let expanded = cx
                    .app
                    .models()
                    .read(&expanded_for_actions, |set| set.contains(&info.id))
                    .ok()
                    .unwrap_or(false);

                let label = if expanded { "Collapse" } else { "Expand" };
                let cmd = CommandId::new(format!(
                    "{CMD_TOGGLE_CODE_BLOCK_EXPAND_PREFIX}{}",
                    info.id.0
                ));

                shadcn::Button::new(label)
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .on_click(cmd)
                    .into_element(cx)
            }));
        }

        let on_link_activate = components.on_link_activate.clone();

        components.image = Some(Arc::new(move |cx, info| {
            let theme = Theme::global(&*cx.app).clone();

            let size_px = Px(96.0);
            let mut size = LayoutStyle::default();
            size.size.width = Length::Px(size_px);
            size.size.height = Length::Px(size_px);

            let spinner_box = |cx: &mut fret_ui::ElementContext<'_, App>| {
                ui::container(cx, |cx| [cx.spinner()])
                    .w_px(MetricRef::Px(size_px))
                    .h_px(MetricRef::Px(size_px))
                    .into_element(cx)
            };

            if info.src.starts_with("http://") || info.src.starts_with("https://") {
                let state = cx
                    .app
                    .with_global_mut(RemoteImageCache::default, |cache, _app| {
                        cache.ensure_fetch_started(info.src.clone());
                        cache.get(&info.src).cloned()
                    });

                let Some(state) = state else {
                    return spinner_box(cx);
                };

                match state {
                    RemoteImageState::Loading => {
                        return spinner_box(cx);
                    }
                    RemoteImageState::Error(msg) => {
                        return render_image_placeholder(
                            cx,
                            &theme,
                            on_link_activate.clone(),
                            markdown::LinkInfo {
                                href: info.src.clone(),
                                text: Arc::<str>::from(format!("[image error: {msg}]")),
                            },
                        );
                    }
                    RemoteImageState::ReadySvg { svg: Some(svg), .. } => {
                        let mut props = SvgIconProps::new(fret_ui::SvgSource::Id(svg));
                        props.layout = size;
                        props.fit = SvgFit::Contain;
                        props.color = theme.color_required("foreground");
                        return cx.svg_icon_props(props);
                    }
                    RemoteImageState::ReadySvg { svg: None, .. } => {
                        return spinner_box(cx);
                    }
                    RemoteImageState::ReadyRaster {
                        width,
                        height,
                        rgba,
                    } => {
                        let (key, image, status) = image_asset_state::use_rgba8_image_state(
                            cx.app,
                            cx.window,
                            width,
                            height,
                            rgba.as_ref(),
                            ImageColorSpace::Srgb,
                        );
                        let _ = key;
                        let _ = status;

                        if let Some(image) = image {
                            let mut props = ImageProps::new(image);
                            props.layout = size;
                            return cx.image_props(props);
                        }
                        return spinner_box(cx);
                    }
                }
            }

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
                        spinner_box(cx)
                    }
                }
                "fret-demo://demo.svg" => {
                    let mut props = SvgIconProps::new(fret_ui::SvgSource::Id(demo_svg));
                    props.layout = size;
                    props.fit = SvgFit::Contain;
                    props.color = theme.color_required("foreground");
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

        let root = declarative::RenderRootContext::new(ui, app, services, window, bounds)
            .render_root("markdown-demo", |cx| {
                cx.watch_global::<Theme>().layout().observe();

                let theme = Theme::global(&*cx.app).clone();
                let padding_md = MetricRef::Px(theme.metric_required("metric.padding.md"));

                let content = ui::v_flex(cx, |cx| {
                    cx.observe_model(&wrap_code, Invalidation::Layout);
                    let enabled = cx.app.models().get_copied(&wrap_code).unwrap_or(false);

                    cx.observe_model(&cap_code_height, Invalidation::Layout);
                    let cap_enabled = cx.app.models().get_copied(&cap_code_height).unwrap_or(true);

                    let toggles = ui::h_flex(cx, |cx| {
                        [
                            cx.text(format!("wrap code: {}", if enabled { "on" } else { "off" })),
                            shadcn::Switch::new(wrap_code.clone())
                                .a11y_label("Wrap code blocks")
                                .into_element(cx),
                            cx.text(format!(
                                "cap code height: {}",
                                if cap_enabled { "on" } else { "off" }
                            )),
                            shadcn::Switch::new(cap_code_height.clone())
                                .a11y_label("Cap code block height")
                                .into_element(cx),
                        ]
                    })
                    .gap(Space::N3)
                    .wrap()
                    .items_center()
                    .into_element(cx);

                    let scroll = decl_scroll::overflow_scroll_content(
                        cx,
                        LayoutRefinement::default()
                            .w_full()
                            .flex_1()
                            .min_h(MetricRef::Px(Px(0.0))),
                        true,
                        |cx| {
                                 ui::container(cx, |cx| {
                                     [markdown::Markdown::new(markdown_source.clone())
                                         .into_element_with(cx, &components)]
                                 })
                                 .w_full()
                                 .paddings(Edges4::all(padding_md.clone()))
                                 .into_element(cx)
                             },
                         );

                    [
                        cx.text("markdown_demo"),
                        cx.text("Scrollable markdown preview (links open via platform shell)."),
                        toggles,
                        scroll,
                    ]
                     })
                     .w_full()
                     .h_full()
                     .gap(Space::N3)
                     .paddings(Edges4::all(padding_md.clone()))
                     .into_element(cx);

                vec![
                    ui::container(cx, |_cx| [content])
                        .bg(ColorRef::Color(theme.color_required("background")))
                        .w_full()
                        .h_full()
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        if cache_changed {
            ui.invalidate(root, Invalidation::Layout);
            ui.invalidate(root, Invalidation::Paint);
        }
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
                color: Some(theme.color_required("muted-foreground")),
                wrap: fret_core::TextWrap::Word,
                overflow: fret_core::TextOverflow::Clip,
            })]
        });
    }

    cx.text_props(TextProps {
        layout: Default::default(),
        text,
        style: None,
        color: Some(theme.color_required("muted-foreground")),
        wrap: fret_core::TextWrap::Word,
        overflow: fret_core::TextOverflow::Clip,
    })
}

impl WinitAppDriver for MarkdownDemoDriver {
    type WindowState = MarkdownDemoWindowState;

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        MarkdownDemoDriver::build_ui(app, window)
    }

    fn hot_reload_window(
        &mut self,
        app: &mut App,
        _services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
    ) {
        crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
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
        if let Some(id) = command
            .as_str()
            .strip_prefix(CMD_TOGGLE_CODE_BLOCK_EXPAND_PREFIX)
        {
            if let Ok(id) = id.parse::<u64>() {
                let id = markdown::BlockId(id);
                let _ = app.models_mut().update(&state.expanded_code_blocks, |set| {
                    if set.contains(&id) {
                        set.remove(&id);
                    } else {
                        set.insert(id);
                    }
                });
                app.push_effect(Effect::Redraw(window));
            }
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
            state.wrap_code.clone(),
            state.cap_code_height.clone(),
            state.expanded_code_blocks.clone(),
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

fn apply_markdown_demo_theme_tokens(app: &mut App) {
    Theme::with_global_mut(app, |theme| {
        // Demo-only: inject explicit markdown math tokens so theme tuning is discoverable.
        let font_size = theme.metric_required("metric.font.size").0;
        let line_height = theme.metric_required("metric.font.line_height").0;
        let block_height = (line_height * 3.25).max(font_size * 4.0);

        let mut cfg = ThemeConfig {
            name: theme.name.clone(),
            author: theme.author.clone(),
            url: theme.url.clone(),
            colors: HashMap::new(),
            metrics: HashMap::new(),
        };

        cfg.metrics
            .insert("fret.markdown.math.inline.height".to_string(), line_height);
        cfg.metrics.insert(
            "fret.markdown.math.block.padding".to_string(),
            theme.metric_required("metric.padding.md").0,
        );
        cfg.metrics
            .insert("fret.markdown.math.block.height".to_string(), block_height);

        theme.apply_config(&cfg);
    });
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap())
                .add_directive("fret_markdown::mdstream=info".parse().unwrap())
                .add_directive("fret_markdown::math=info".parse().unwrap()),
        )
        .try_init();

    crate::run_native_demo(
        WinitRunnerConfig {
            main_window_title: "markdown_demo".to_string(),
            main_window_size: winit::dpi::LogicalSize::new(920.0, 720.0),
            ..Default::default()
        },
        App::new(),
        MarkdownDemoDriver::default(),
    )
    .context("run markdown_demo")
}
