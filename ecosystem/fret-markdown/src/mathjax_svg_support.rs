use std::sync::Arc;

use fret_core::{
    Edges, FontId, FontWeight, Px, SvgFit, TextOverflow, TextSlant, TextStyle, TextWrap,
};
use fret_ui::SvgSource;
use fret_ui::element::{
    AnyElement, ContainerProps, Length, ScrollAxis, ScrollProps, SvgIconProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};

use super::{InlineMathInfo, MarkdownTheme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MathJaxMode {
    Inline,
    Display,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MathJaxKey {
    mode: MathJaxMode,
    latex: String,
}

#[derive(Debug, Clone)]
struct MathJaxSvgReady {
    svg_bytes: Arc<[u8]>,
    aspect_ratio: Option<f32>,
}

#[derive(Debug, Clone)]
enum MathJaxSvgEntry {
    Loading,
    Ready(MathJaxSvgReady),
    Error(Arc<str>),
}

struct MathJaxWorker {
    tx: std::sync::mpsc::Sender<MathJaxWorkItem>,
}

struct MathJaxWorkItem {
    map: Arc<std::sync::Mutex<std::collections::HashMap<MathJaxKey, MathJaxSvgEntry>>>,
    key: MathJaxKey,
}

static MATHJAX_WORKER: std::sync::OnceLock<MathJaxWorker> = std::sync::OnceLock::new();

fn mathjax_worker() -> &'static MathJaxWorker {
    MATHJAX_WORKER.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel::<MathJaxWorkItem>();
        std::thread::spawn(move || {
            for item in rx {
                let key = item.key;
                let latex = key.latex.clone();
                tracing::debug!(
                    target: "fret_markdown::math",
                    mode = ?key.mode,
                    latex_len = latex.len(),
                    "mathjax svg: convert queued"
                );

                let result = std::panic::catch_unwind(|| match key.mode {
                    MathJaxMode::Inline => mathjax_svg::convert_to_svg_inline(&latex),
                    MathJaxMode::Display => mathjax_svg::convert_to_svg(&latex),
                });

                let mut map_guard = item.map.lock().expect("mathjax svg cache lock");
                match result {
                    Ok(Ok(svg)) => {
                        let has_current_color =
                            svg.contains("currentColor") || svg.contains("currentcolor");
                        let svg = if has_current_color {
                            svg.replace("currentColor", "#000000")
                                .replace("currentcolor", "#000000")
                        } else {
                            svg
                        };

                        tracing::debug!(
                            target: "fret_markdown::math",
                            mode = ?key.mode,
                            latex_len = latex.len(),
                            has_current_color,
                            "mathjax svg: converted"
                        );

                        let aspect_ratio = svg_viewbox_aspect_ratio(&svg);
                        map_guard.insert(
                            key,
                            MathJaxSvgEntry::Ready(MathJaxSvgReady {
                                svg_bytes: Arc::<[u8]>::from(svg.into_bytes()),
                                aspect_ratio,
                            }),
                        );
                    }
                    Ok(Err(err)) => {
                        tracing::warn!(
                            target: "fret_markdown::math",
                            mode = ?key.mode,
                            latex_len = latex.len(),
                            error = %err,
                            "mathjax svg: convert failed"
                        );
                        map_guard.insert(
                            key,
                            MathJaxSvgEntry::Error(Arc::<str>::from(err.to_string())),
                        );
                    }
                    Err(_) => {
                        map_guard.insert(
                            key,
                            MathJaxSvgEntry::Error(Arc::<str>::from("mathjax svg: panic")),
                        );
                    }
                }
            }
        });
        MathJaxWorker { tx }
    })
}

#[derive(Default, Clone)]
struct MathJaxSvgCache {
    inner: Arc<std::sync::Mutex<std::collections::HashMap<MathJaxKey, MathJaxSvgEntry>>>,
}

pub(super) fn render_math_block_mathjax_svg<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    latex: Arc<str>,
) -> AnyElement {
    let entry = mathjax_svg_entry(cx, MathJaxMode::Display, latex.as_ref());

    let mut scroll_props = ScrollProps::default();
    scroll_props.axis = ScrollAxis::X;
    scroll_props.layout.size.width = Length::Fill;

    let mut container = ContainerProps::default();
    container.layout.size.width = Length::Fill;
    container.padding = Edges::all(markdown_theme.math_block_padding);
    container.background = Some(markdown_theme.math_block_bg);
    container.border = Edges::all(Px(0.0));
    container.corner_radii = fret_core::Corners::all(theme.metric_required("metric.radius.md"));

    cx.container(container, |cx| {
        vec![cx.scroll(scroll_props, |cx| match entry {
            MathJaxSvgEntry::Ready(ready) => {
                let mut icon = SvgIconProps::new(SvgSource::Bytes(ready.svg_bytes));
                icon.fit = SvgFit::Contain;
                icon.color = markdown_theme.math_block_fg;
                icon.layout.size.height = Length::Px(markdown_theme.math_block_height);
                icon.layout.aspect_ratio = ready.aspect_ratio;
                vec![cx.svg_icon_props(icon)]
            }
            MathJaxSvgEntry::Loading => vec![cx.text_props(TextProps {
                layout: Default::default(),
                text: latex.clone(),
                style: Some(TextStyle {
                    font: FontId::monospace(),
                    size: theme.metric_required("metric.font.mono_size"),
                    weight: FontWeight::NORMAL,
                    slant: TextSlant::Normal,
                    line_height: Some(theme.metric_required("metric.font.mono_line_height")),
                    letter_spacing_em: None,
                }),
                color: Some(markdown_theme.math_block_fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            })],
            MathJaxSvgEntry::Error(err) => vec![cx.text_props(TextProps {
                layout: Default::default(),
                text: Arc::<str>::from(format!("{latex} (mathjax error: {err})")),
                style: Some(TextStyle {
                    font: FontId::monospace(),
                    size: theme.metric_required("metric.font.mono_size"),
                    weight: FontWeight::NORMAL,
                    slant: TextSlant::Normal,
                    line_height: Some(theme.metric_required("metric.font.mono_line_height")),
                    letter_spacing_em: None,
                }),
                color: Some(markdown_theme.math_block_fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            })],
        })]
    })
}

pub(super) fn render_inline_math_mathjax_svg<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    info: InlineMathInfo,
) -> AnyElement {
    let entry = mathjax_svg_entry(cx, MathJaxMode::Inline, info.latex.as_ref());
    match entry {
        MathJaxSvgEntry::Ready(ready) => render_inline_math_svg(
            cx,
            theme,
            markdown_theme,
            ready.svg_bytes,
            ready.aspect_ratio,
        ),
        MathJaxSvgEntry::Loading => render_inline_math_source(cx, theme, markdown_theme, info),
        MathJaxSvgEntry::Error(err) => render_inline_math_source(
            cx,
            theme,
            markdown_theme,
            InlineMathInfo {
                latex: Arc::<str>::from(format!("{} (mathjax error: {err})", info.latex)),
            },
        ),
    }
}

fn render_inline_math_svg<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    svg_bytes: Arc<[u8]>,
    aspect_ratio: Option<f32>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.padding = Edges {
        top: markdown_theme.inline_math_padding_y,
        right: markdown_theme.inline_math_padding_x,
        bottom: markdown_theme.inline_math_padding_y,
        left: markdown_theme.inline_math_padding_x,
    };
    props.background = Some(markdown_theme.inline_math_bg);
    props.border = Edges::all(Px(0.0));
    props.corner_radii = fret_core::Corners::all(theme.metric_required("metric.radius.sm"));

    cx.container(props, |cx| {
        let mut icon = SvgIconProps::new(SvgSource::Bytes(svg_bytes));
        icon.fit = SvgFit::Contain;
        icon.color = markdown_theme.inline_math_fg;
        icon.layout.size.height = Length::Px(markdown_theme.inline_math_height);
        icon.layout.aspect_ratio = aspect_ratio;
        vec![cx.svg_icon_props(icon)]
    })
}

fn render_inline_math_source<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    info: InlineMathInfo,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.padding = Edges {
        top: markdown_theme.inline_math_padding_y,
        right: markdown_theme.inline_math_padding_x,
        bottom: markdown_theme.inline_math_padding_y,
        left: markdown_theme.inline_math_padding_x,
    };
    props.background = Some(markdown_theme.inline_math_bg);
    props.border = Edges::all(Px(0.0));
    props.corner_radii = fret_core::Corners::all(theme.metric_required("metric.radius.sm"));

    cx.container(props, |cx| {
        vec![cx.text_props(TextProps {
            layout: Default::default(),
            text: Arc::<str>::from(info.latex.trim().to_string()),
            style: Some(TextStyle {
                font: FontId::monospace(),
                size: theme.metric_required("metric.font.mono_size"),
                weight: FontWeight::NORMAL,
                slant: TextSlant::Normal,
                line_height: Some(theme.metric_required("metric.font.mono_line_height")),
                letter_spacing_em: None,
            }),
            color: Some(markdown_theme.inline_math_fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })]
    })
}

fn mathjax_svg_entry<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mode: MathJaxMode,
    latex: &str,
) -> MathJaxSvgEntry {
    let latex = latex.trim();
    if latex.is_empty() {
        return MathJaxSvgEntry::Error(Arc::<str>::from("empty latex"));
    }

    let key = MathJaxKey {
        mode,
        latex: latex.to_string(),
    };

    let mut spawn = None::<(
        Arc<std::sync::Mutex<std::collections::HashMap<MathJaxKey, MathJaxSvgEntry>>>,
        MathJaxKey,
    )>;
    let entry = cx
        .app
        .with_global_mut(MathJaxSvgCache::default, |cache, host| {
            let map = cache.inner.clone();
            let mut map_guard = map.lock().expect("mathjax svg cache lock");

            match map_guard.get(&key) {
                Some(existing) => {
                    if matches!(existing, MathJaxSvgEntry::Loading) {
                        host.request_redraw(cx.window);
                    }
                    return existing.clone();
                }
                None => {
                    map_guard.insert(key.clone(), MathJaxSvgEntry::Loading);
                    host.request_redraw(cx.window);
                    spawn = Some((map.clone(), key.clone()));
                    MathJaxSvgEntry::Loading
                }
            }
        });

    if let Some((map, key)) = spawn {
        let work = MathJaxWorkItem {
            map: map.clone(),
            key: key.clone(),
        };
        if let Err(_err) = mathjax_worker().tx.send(work) {
            let mut map_guard = map.lock().expect("mathjax svg cache lock");
            map_guard.insert(
                key,
                MathJaxSvgEntry::Error(Arc::<str>::from("mathjax svg worker unavailable")),
            );
        }
    }

    entry
}

fn svg_viewbox_aspect_ratio(svg: &str) -> Option<f32> {
    let idx = svg.find("viewBox=")?;
    let rest = &svg[idx + "viewBox=".len()..];
    let mut chars = rest.chars();
    let quote = chars.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = chars.as_str();
    let end = rest.find(quote)?;
    let value = &rest[..end];

    let mut nums: [f32; 4] = [0.0; 4];
    let mut i = 0usize;
    for part in value.split(|c: char| c.is_whitespace() || c == ',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if i >= 4 {
            break;
        }
        nums[i] = part.parse::<f32>().ok()?;
        i += 1;
    }
    if i < 4 {
        return None;
    }
    let w = nums[2];
    let h = nums[3];
    if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
        return None;
    }
    Some(w / h)
}
