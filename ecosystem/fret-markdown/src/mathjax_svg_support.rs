use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::time::Duration;

use fret_core::{
    Edges, FontId, FontWeight, Px, SvgFit, TextOverflow, TextSlant, TextStyle, TextWrap,
};
use fret_query::ui::QueryElementContextExt as _;
use fret_query::{QueryError, QueryKey, QueryPolicy, QueryStatus};
use fret_ui::SvgSource;
use fret_ui::element::{
    AnyElement, ContainerProps, Length, ScrollAxis, ScrollProps, SvgIconProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::ModelWatchExt as _;

use super::{InlineMathInfo, MarkdownTheme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MathJaxMode {
    Inline,
    Display,
}

const MATHJAX_SVG_NAMESPACE: &str = "fret-markdown.mathjax_svg.v1";

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

fn mathjax_svg_query_policy() -> QueryPolicy {
    // MathJax SVG conversion is deterministic and purely local, so we treat it as "fresh" for a
    // long time and rely on cache eviction to bound memory usage.
    QueryPolicy {
        stale_time: Duration::from_secs(60 * 60 * 24 * 365),
        cache_time: Duration::from_secs(60 * 10),
        keep_previous_data_while_loading: true,
        ..Default::default()
    }
}

pub(super) fn render_math_block_mathjax_svg<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    latex: Arc<str>,
) -> AnyElement {
    let entry = mathjax_svg_entry(cx, MathJaxMode::Display, latex.clone());

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
    let entry = mathjax_svg_entry(cx, MathJaxMode::Inline, info.latex.clone());
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
    latex: Arc<str>,
) -> MathJaxSvgEntry {
    let latex_trimmed = latex.trim();
    if latex_trimmed.is_empty() {
        return MathJaxSvgEntry::Error(Arc::<str>::from("empty latex"));
    }

    let key = QueryKey::<MathJaxSvgReady>::new_named(
        MATHJAX_SVG_NAMESPACE,
        &(mode, latex_trimmed),
        "mathjax_svg",
    );
    let policy = mathjax_svg_query_policy();

    let latex_owned: Arc<str> = if latex_trimmed.len() == latex.len() {
        latex.clone()
    } else {
        Arc::<str>::from(latex_trimmed.to_string())
    };

    let handle = cx.use_query(key, policy, move |_token| {
        let latex: &str = latex_owned.as_ref();

        tracing::debug!(
            target: "fret_markdown::math",
            mode = ?mode,
            latex_len = latex.len(),
            "mathjax svg: convert queued"
        );

        let result = catch_unwind(AssertUnwindSafe(|| match mode {
            MathJaxMode::Inline => mathjax_svg::convert_to_svg_inline(latex),
            MathJaxMode::Display => mathjax_svg::convert_to_svg(latex),
        }));

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
                    mode = ?mode,
                    latex_len = latex.len(),
                    has_current_color,
                    "mathjax svg: converted"
                );

                let aspect_ratio = svg_viewbox_aspect_ratio(&svg);
                Ok(MathJaxSvgReady {
                    svg_bytes: Arc::<[u8]>::from(svg.into_bytes()),
                    aspect_ratio,
                })
            }
            Ok(Err(err)) => {
                tracing::warn!(
                    target: "fret_markdown::math",
                    mode = ?mode,
                    latex_len = latex.len(),
                    error = %err,
                    "mathjax svg: convert failed"
                );
                Err(QueryError::permanent(err.to_string()))
            }
            Err(_) => Err(QueryError::permanent("mathjax svg: panic")),
        }
    });

    let state = cx
        .watch_model(handle.model())
        .layout()
        .cloned()
        .unwrap_or_default();

    let Some(data) = state.data else {
        return match state.status {
            QueryStatus::Error => MathJaxSvgEntry::Error(
                state
                    .error
                    .as_ref()
                    .map(|e| e.message().clone())
                    .unwrap_or_else(|| Arc::<str>::from("mathjax svg: error")),
            ),
            QueryStatus::Idle | QueryStatus::Loading | QueryStatus::Success => {
                MathJaxSvgEntry::Loading
            }
        };
    };

    MathJaxSvgEntry::Ready((*data).clone())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_viewbox_aspect_ratio_parses_basic_dimensions() {
        let svg = "<svg viewBox=\"0 0 4 2\"/>";
        assert_eq!(svg_viewbox_aspect_ratio(svg), Some(2.0));
    }

    #[test]
    fn svg_viewbox_aspect_ratio_handles_single_quotes_and_commas() {
        let svg = "<svg viewBox='0,0, 9, 3'/>";
        assert_eq!(svg_viewbox_aspect_ratio(svg), Some(3.0));
    }

    #[test]
    fn svg_viewbox_aspect_ratio_rejects_missing_or_invalid_values() {
        assert_eq!(svg_viewbox_aspect_ratio("<svg/>"), None);
        assert_eq!(svg_viewbox_aspect_ratio("<svg viewBox=\"0 0 0 1\"/>"), None);
        assert_eq!(svg_viewbox_aspect_ratio("<svg viewBox=\"0 0 1 0\"/>"), None);
        assert_eq!(svg_viewbox_aspect_ratio("<svg viewBox=\"0 0 x 1\"/>"), None);
    }
}
