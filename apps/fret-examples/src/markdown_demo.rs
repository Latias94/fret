//! Markdown demo (view runtime + typed actions).

#![cfg(not(target_arch = "wasm32"))]

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;
use fret::query::{QueryError, QueryKey, QueryPolicy, QueryStatus, with_query_client};
use fret::{FretApp, advanced::prelude::*, component::prelude::*};
use fret_core::{ImageColorSpace, Point, Px, SvgFit};
use fret_markdown as markdown;
use fret_ui::element::{
    ImageProps, LayoutQueryRegionProps, LayoutStyle, Length, PressableProps, SvgIconProps,
    TextProps,
};
use fret_ui::{Invalidation, Theme, ThemeConfig};
use fret_ui_assets::image_asset_state;
use fret_ui_kit::declarative::QueryHandleWatchExt as _;
use fret_ui_kit::{ColorRef, IntoUiElement, Space, ui};
use fret_ui_shadcn::facade as shadcn;

mod act {
    fret::actions!([RefreshRemoteImages = "markdown_demo.refresh_remote_images.v1"]);
    fret::payload_actions!([
        ToggleCodeBlockExpand(fret_markdown::BlockId) = "markdown_demo.toggle_code_block_expand.v2"
    ]);
}

const REMOTE_IMAGE_NAMESPACE: &str = "fret-examples.markdown_demo.remote_image.v1";
const TRANSIENT_REFRESH_REMOTE_IMAGES: u64 = 0xAFA0_0103;

#[derive(Debug)]
enum RemoteImageData {
    Raster {
        width: u32,
        height: u32,
        rgba: Arc<[u8]>,
    },
    Svg {
        bytes: Arc<[u8]>,
    },
}

fn remote_image_key(url: &Arc<str>) -> QueryKey<RemoteImageData> {
    QueryKey::new(REMOTE_IMAGE_NAMESPACE, url)
}

fn remote_image_policy() -> QueryPolicy {
    // stale_time only controls freshness (no implicit polling). Keep entries fresh within the
    // cache window and use explicit invalidate to force refresh.
    QueryPolicy {
        stale_time: Duration::from_secs(5 * 60),
        cache_time: Duration::from_secs(5 * 60),
        keep_previous_data_while_loading: true,
        ..Default::default()
    }
}

fn download_remote_image(url: &str) -> Result<RemoteImageData, QueryError> {
    use std::io::Read as _;

    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(QueryError::permanent(
            "only http/https are supported in this demo",
        ));
    }

    let response = ureq::get(url)
        .set("User-Agent", "fret-markdown-demo")
        .set("Accept", "image/*")
        .call()
        .map_err(|e| QueryError::transient(format!("request failed: {e}")))?;

    let status = response.status();
    if !(200..=299).contains(&status) {
        return Err(QueryError::permanent(format!("http status {status}")));
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
            return Err(QueryError::permanent("image too large (>4MiB)"));
        }
    }

    if is_svg {
        return Ok(RemoteImageData::Svg {
            bytes: Arc::<[u8]>::from(bytes),
        });
    }

    let image = image::load_from_memory(&bytes)
        .map_err(|e| QueryError::permanent(format!("decode failed: {e}")))?;
    let rgba = image.to_rgba8();
    let (w, h) = rgba.dimensions();

    let pixel_budget = 8_000_000u64;
    let px = (w as u64) * (h as u64);
    if px > pixel_budget {
        return Err(QueryError::permanent("decoded image too large"));
    }

    Ok(RemoteImageData::Raster {
        width: w,
        height: h,
        rgba: Arc::<[u8]>::from(rgba.into_raw()),
    })
}

struct MarkdownDemoState {
    markdown: Arc<str>,
    scroll: fret_ui::scroll::ScrollHandle,
    anchor_regions: Rc<RefCell<HashMap<Arc<str>, fret_ui::GlobalElementId>>>,
    demo_svg_bytes: Arc<[u8]>,
    checker_rgba: Arc<[u8]>,
}

struct MarkdownDemoView {
    st: MarkdownDemoState,
}

impl MarkdownDemoView {
    fn on_link_activate(pending_anchor: LocalState<Option<Arc<str>>>) -> markdown::OnLinkActivate {
        Arc::new(move |host, cx, _reason, link| {
            let href = link.href.trim();
            if let Some(fragment) = href.strip_prefix('#') {
                let fragment = fragment.trim();
                if fragment.is_empty() {
                    return;
                }
                let fragment: Arc<str> = Arc::from(fragment.to_string());
                let _ = pending_anchor.update_in(host.models_mut(), |v| {
                    *v = Some(fragment.clone());
                });
                host.request_redraw(cx.window);
                return;
            }

            if !markdown::is_safe_open_url(href) {
                return;
            }

            host.push_effect(Effect::OpenUrl {
                url: href.to_string(),
                target: None,
                rel: None,
            });
        })
    }

    fn maybe_scroll_pending_anchor(
        &mut self,
        cx: &mut AppUi<'_, '_>,
        pending_anchor: &LocalState<Option<Arc<str>>>,
        viewport_region: Option<fret_ui::GlobalElementId>,
        padding_top: Px,
    ) {
        let Some(viewport_region) = viewport_region else {
            return;
        };

        let pending = pending_anchor.layout_value(cx);
        let Some(fragment) = pending.as_deref() else {
            return;
        };

        let test_id = markdown::anchor_test_id_from_fragment(fragment);
        let anchor_id = self
            .st
            .anchor_regions
            .borrow()
            .get(test_id.as_ref())
            .copied();
        let Some(anchor_id) = anchor_id else {
            return;
        };

        let Some(anchor_bounds) = cx.layout_query_bounds(anchor_id, Invalidation::Layout) else {
            return;
        };
        let Some(viewport_bounds) = cx.layout_query_bounds(viewport_region, Invalidation::Layout)
        else {
            return;
        };

        let desired_top_y = viewport_bounds.origin.y.0 + padding_top.0;
        let delta_y = anchor_bounds.origin.y.0 - desired_top_y;
        if delta_y.abs() > 0.5 {
            let prev = self.st.scroll.offset();
            self.st
                .scroll
                .set_offset(Point::new(prev.x, Px(prev.y.0 + delta_y)));
        }

        let _ = pending_anchor.set_in(cx.app.models_mut(), None);
    }
}

impl View for MarkdownDemoView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
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
- [Jump to Math](#math)
- [Jump to Footnote](#fn-note)
- [Back to top](#markdown-demo)

## Images

Raster (procedural, cached via `ImageAssetCache`):

![Checkerboard](fret-demo://checkerboard)

SVG (inline bytes in this demo; can also be cached via `SvgAssetCache`):

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

        let demo_svg_bytes = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect x="0" y="0" width="64" height="64" rx="12" fill="#111827"/><path d="M20 44 L32 20 L44 44 Z" fill="#60A5FA"/></svg>"##;
        let demo_svg_bytes: Arc<[u8]> = Arc::from(demo_svg_bytes.to_vec());

        let checker_rgba: Arc<[u8]> = Arc::from(checkerboard_rgba8(96, 96));

        Self {
            st: MarkdownDemoState {
                markdown,
                scroll: fret_ui::scroll::ScrollHandle::default(),
                anchor_regions: Rc::new(RefCell::new(HashMap::new())),
                demo_svg_bytes,
                checker_rgba,
            },
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let wrap_code_state = cx.state().local_init(|| false);
        let cap_code_height_state = cx.state().local_init(|| true);
        let expanded_code_blocks_state = cx.state().local_init(HashSet::new);
        let pending_anchor_state = cx.state().local_init(|| None::<Arc<str>>);

        if cx.effects().take_transient(TRANSIENT_REFRESH_REMOTE_IMAGES) {
            let _ = with_query_client(cx.app, |client, _app| {
                client.invalidate_namespace(REMOTE_IMAGE_NAMESPACE);
            });
        }

        cx.actions()
            .transient::<act::RefreshRemoteImages>(TRANSIENT_REFRESH_REMOTE_IMAGES);
        cx.actions()
            .local(&expanded_code_blocks_state)
            .payload_update_if::<act::ToggleCodeBlockExpand>(|set, id| {
                if set.contains(&id) {
                    set.remove(&id);
                } else {
                    set.insert(id);
                }
                true
            });

        self.st.anchor_regions.borrow_mut().clear();

        let theme = cx.theme_snapshot();
        let padding_md = theme.metric_token("metric.padding.md");

        let wrap_enabled = wrap_code_state.layout_value(cx);
        let cap_enabled = cap_code_height_state.layout_value(cx);

        let mut components = markdown::MarkdownComponents::<KernelApp>::default();
        components.on_link_activate = Some(Self::on_link_activate(pending_anchor_state.clone()));
        components.code_block_ui.wrap = if wrap_enabled {
            fret_code_view::CodeBlockWrap::Word
        } else {
            fret_code_view::CodeBlockWrap::ScrollX
        };
        components.code_block_windowed =
            cap_enabled.then_some(fret_code_view::CodeBlockWindowedOptions::default());
        components.code_block_max_height_from_theme = cap_enabled;
        components.code_block_ui.show_scrollbar_y = cap_enabled;
        components.code_block_ui.scrollbar_y_on_hover = true;

        if cap_enabled {
            let expanded_for_resolver = expanded_code_blocks_state.clone();
            components.code_block_ui_resolver = Some(Arc::new(move |cx, info, options| {
                let expanded = expanded_for_resolver
                    .layout_in(cx)
                    .read_ref(|set| set.contains(&info.id))
                    .ok()
                    .unwrap_or(false);
                if expanded {
                    options.max_height = None;
                    options.show_scrollbar_y = false;
                }
            }));

            let expanded_for_actions = expanded_code_blocks_state.clone();
            components.code_block_actions = Some(Arc::new(move |cx, info| {
                let expanded = expanded_for_actions
                    .layout_in(cx)
                    .read_ref(|set| set.contains(&info.id))
                    .ok()
                    .unwrap_or(false);

                let label = if expanded { "Collapse" } else { "Expand" };
                shadcn::Button::new(label)
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .action(act::ToggleCodeBlockExpand)
                    .action_payload(info.id)
                    .into_element(cx)
            }));
        }

        let anchor_regions = self.st.anchor_regions.clone();
        components.anchor_decorate = Some(Arc::new(move |cx, test_id, el| {
            let regions = anchor_regions.clone();
            let key = test_id.clone();
            let mut props = LayoutQueryRegionProps::default();
            props.name = Some(test_id);
            cx.layout_query_region_with_id(props, move |_cx, id| {
                regions.borrow_mut().insert(key, id);
                vec![el]
            })
        }));

        let on_link_activate = components.on_link_activate.clone();
        let checker_rgba = self.st.checker_rgba.clone();
        let demo_svg_bytes = self.st.demo_svg_bytes.clone();

        components.image = Some(Arc::new(move |cx, info| {
            let theme = Theme::global(&*cx.app).snapshot();
            let padding_md = theme.metric_token("metric.padding.md");

            let mut size = LayoutStyle::default();
            size.size.width = Length::Fill;
            size.size.height = Length::Px(Px(240.0));

            let spinner_box = |cx: &mut UiCx<'_>| {
                cx.container(
                    fret_ui::element::ContainerProps {
                        layout: size,
                        padding: Default::default(),
                        background: Some(theme.color_token("card")),
                        shadow: None,
                        border: Default::default(),
                        border_color: Some(theme.color_token("border")),
                        corner_radii: Default::default(),
                        ..Default::default()
                    },
                    |cx| {
                        let mut spinner_layout = LayoutStyle::default();
                        spinner_layout.size.width = Length::Px(Px(28.0));
                        spinner_layout.size.height = Length::Px(Px(28.0));
                        vec![cx.spinner_props(fret_ui::element::SpinnerProps {
                            layout: spinner_layout,
                            color: Some(theme.color_token("muted-foreground")),
                            ..Default::default()
                        })]
                    },
                )
            };

            if info.src.starts_with("http://") || info.src.starts_with("https://") {
                let src_for_fetch = info.src.clone();
                let key = remote_image_key(&info.src);
                let policy = remote_image_policy();

                let handle = cx.data().query(key, policy, move |_token| {
                    download_remote_image(src_for_fetch.as_ref())
                });

                let state = handle.layout_query(cx).value_or_default();

                match state.status {
                    QueryStatus::Idle | QueryStatus::Loading => return spinner_box(cx),
                    QueryStatus::Error => {
                        let msg = state
                            .error
                            .as_ref()
                            .map(|e| e.to_string())
                            .unwrap_or_else(|| "unknown error".to_string());
                        let placeholder = render_image_placeholder(
                            cx,
                            theme,
                            on_link_activate.clone(),
                            markdown::LinkInfo {
                                href: info.src.clone(),
                                text: Arc::<str>::from(format!("[image error: {msg}]")),
                            },
                        );
                        return placeholder.into_element(cx);
                    }
                    QueryStatus::Success => {
                        let Some(data) = state.data.as_ref() else {
                            return spinner_box(cx);
                        };

                        match data.as_ref() {
                            RemoteImageData::Svg { bytes } => {
                                let mut props =
                                    SvgIconProps::new(fret_ui::SvgSource::Bytes(bytes.clone()));
                                props.layout = size;
                                props.fit = SvgFit::Contain;
                                props.color = theme.color_token("foreground");
                                return cx.svg_icon_props(props);
                            }
                            RemoteImageData::Raster {
                                width,
                                height,
                                rgba,
                            } => {
                                let (_key, image, _status) =
                                    image_asset_state::use_rgba8_image_state(
                                        cx.app,
                                        cx.window,
                                        *width,
                                        *height,
                                        rgba.as_ref(),
                                        ImageColorSpace::Srgb,
                                    );

                                if let Some(image) = image {
                                    let mut props = ImageProps::new(image);
                                    props.layout = size;
                                    return cx.image_props(props);
                                }
                                return spinner_box(cx);
                            }
                        }
                    }
                }
            }

            let inner = match info.src.as_ref() {
                "fret-demo://checkerboard" => {
                    let (_key, image, _status) = image_asset_state::use_rgba8_image_state(
                        cx.app,
                        cx.window,
                        96,
                        96,
                        checker_rgba.as_ref(),
                        ImageColorSpace::Srgb,
                    );

                    if let Some(image) = image {
                        let mut props = ImageProps::new(image);
                        props.layout = size;
                        cx.image_props(props)
                    } else {
                        spinner_box(cx)
                    }
                }
                "fret-demo://demo.svg" => {
                    let mut props =
                        SvgIconProps::new(fret_ui::SvgSource::Bytes(demo_svg_bytes.clone()));
                    props.layout = size;
                    props.fit = SvgFit::Contain;
                    props.color = theme.color_token("foreground");
                    cx.svg_icon_props(props)
                }
                _ => {
                    let placeholder = render_image_placeholder(
                        cx,
                        theme,
                        on_link_activate.clone(),
                        markdown::LinkInfo {
                            href: info.src.clone(),
                            text: if info.alt.trim().is_empty() {
                                Arc::<str>::from("[image]".to_string())
                            } else {
                                Arc::<str>::from(format!("[image: {}]", info.alt.trim()))
                            },
                        },
                    );
                    placeholder.into_element(cx)
                }
            };

            ui::container(|_cx| [inner])
                .w_full()
                .padding_px(padding_md)
                .into_element(cx)
        }));

        let expanded_count = expanded_code_blocks_state.layout_read_ref(cx, |set| set.len());

        let toggles = ui::h_flex(|cx| {
            [
                cx.text(format!(
                    "wrap code: {}",
                    if wrap_enabled { "on" } else { "off" }
                )),
                shadcn::Switch::new(&wrap_code_state)
                    .a11y_label("Wrap code blocks")
                    .into_element(cx),
                cx.text(format!(
                    "cap code height: {}",
                    if cap_enabled { "on" } else { "off" }
                )),
                shadcn::Switch::new(&cap_code_height_state)
                    .a11y_label("Cap code block height")
                    .into_element(cx),
                shadcn::Button::new("Refresh remote images")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .action(act::RefreshRemoteImages)
                    .into_element(cx),
                cx.text(format!("expanded code blocks: {expanded_count}")),
            ]
        })
        .gap(Space::N3)
        .wrap()
        .items_center()
        .into_element(cx);

        let mut scroll_viewport: Option<fret_ui::GlobalElementId> = None;
        let scroll = cx.layout_query_region_with_id(
            LayoutQueryRegionProps {
                name: Some(Arc::<str>::from("markdown_demo.scroll.viewport")),
                ..Default::default()
            },
            |cx, id| {
                scroll_viewport = Some(id);
                let scroll = shadcn::ScrollArea::new([ui::container(|cx| {
                    [markdown::Markdown::new(self.st.markdown.clone())
                        .into_element_with(cx, &components)]
                })
                .w_full()
                .padding_px(padding_md)
                .into_element(cx)])
                .scroll_handle(self.st.scroll.clone())
                .refine_layout(LayoutRefinement::default().w_full().flex_1())
                .into_element(cx);
                vec![scroll]
            },
        );

        let content = ui::v_flex(|cx| {
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
        .padding_px(padding_md)
        .into_element(cx);

        self.maybe_scroll_pending_anchor(cx, &pending_anchor_state, scroll_viewport, padding_md);

        ui::container(|_cx| [content])
            .bg(ColorRef::Color(theme.color_token("background")))
            .w_full()
            .h_full()
            .into_element(cx)
            .into()
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
    theme: fret_ui::ThemeSnapshot,
    on_link_activate: Option<markdown::OnLinkActivate>,
    link: markdown::LinkInfo,
) -> impl IntoUiElement<H> + use<H> {
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
                color: Some(theme.color_token("muted-foreground")),
                align: fret_core::TextAlign::Start,
                wrap: fret_core::TextWrap::Word,
                overflow: fret_core::TextOverflow::Clip,
                ink_overflow: Default::default(),
            })]
        });
    }

    cx.text_props(TextProps {
        layout: Default::default(),
        text,
        style: None,
        color: Some(theme.color_token("muted-foreground")),
        align: fret_core::TextAlign::Start,
        wrap: fret_core::TextWrap::Word,
        overflow: fret_core::TextOverflow::Clip,
        ink_overflow: Default::default(),
    })
}

fn apply_markdown_demo_theme_tokens(app: &mut KernelApp) {
    Theme::with_global_mut(app, |theme| {
        // Demo-only: inject explicit markdown math tokens so theme tuning is discoverable.
        let font_size = theme.metric_token("metric.font.size").0;
        let line_height = theme.metric_token("metric.font.line_height").0;
        let block_height = (line_height * 3.25).max(font_size * 4.0);

        let mut cfg = ThemeConfig {
            name: theme.name.clone(),
            author: theme.author.clone(),
            url: theme.url.clone(),
            colors: HashMap::new(),
            metrics: HashMap::new(),
            ..ThemeConfig::default()
        };

        cfg.metrics
            .insert("fret.markdown.math.inline.height".to_string(), line_height);
        cfg.metrics.insert(
            "fret.markdown.math.block.padding".to_string(),
            theme.metric_token("metric.padding.md").0,
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

    FretApp::new("markdown-demo")
        .window("markdown-demo", (920.0, 720.0))
        .setup(apply_markdown_demo_theme_tokens)
        .config_files(false)
        .view::<MarkdownDemoView>()?
        .run()
        .with_context(|| "failed to run markdown demo")
}
