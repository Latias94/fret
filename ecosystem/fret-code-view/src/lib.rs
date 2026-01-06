//! Code view component(s) for Fret.

use std::ops::Range;
use std::sync::Arc;

use fret_core::{Color, FontId, FontWeight, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, ScrollAxis, ScrollProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

#[derive(Debug, Clone)]
pub struct CodeBlock {
    code: Arc<str>,
    language: Option<Arc<str>>,
    show_line_numbers: bool,
}

impl CodeBlock {
    pub fn new(code: impl Into<Arc<str>>) -> Self {
        Self {
            code: code.into(),
            language: None,
            show_line_numbers: false,
        }
    }

    pub fn language(mut self, language: impl Into<Arc<str>>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        code_block(
            cx,
            &self.code,
            self.language.as_deref(),
            self.show_line_numbers,
        )
    }
}

pub fn code_block<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    code: &str,
    language: Option<&str>,
    show_line_numbers: bool,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let props = decl_style::container_props(
        &theme,
        ChromeRefinement::default()
            .p(Space::N2)
            .rounded(Radius::Md)
            .border_1()
            .bg(ColorRef::Color(theme.colors.panel_background))
            .border_color(ColorRef::Color(theme.colors.panel_border)),
        LayoutRefinement::default().w_full(),
    );

    let spans = match language.map(|s| s.trim()).filter(|s| !s.is_empty()) {
        Some(language) => fret_syntax::highlight(code, language).unwrap_or_default(),
        None => Vec::new(),
    };

    cx.container(props, |cx| {
        let mut scroll_props = ScrollProps::default();
        scroll_props.axis = ScrollAxis::X;

        vec![cx.scroll(scroll_props, |cx| {
            let lines = split_lines(code);
            let line_number_width = line_number_width(lines.len());

            lines
                .into_iter()
                .enumerate()
                .map(|(i, line)| {
                    if show_line_numbers {
                        render_code_line_with_number(
                            cx,
                            &theme,
                            i + 1,
                            line_number_width,
                            line,
                            &spans,
                        )
                    } else {
                        render_code_line(cx, &theme, line, &spans)
                    }
                })
                .collect()
        })]
    })
}

fn render_code_line_with_number<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    line_no: usize,
    width: usize,
    line: LineSlice<'_>,
    spans: &[fret_syntax::HighlightSpan],
) -> AnyElement {
    let number_style = TextStyle {
        font: FontId::monospace(),
        size: theme.metrics.mono_font_size,
        weight: FontWeight::NORMAL,
        line_height: Some(theme.metrics.mono_font_line_height),
        letter_spacing_em: None,
    };

    stack::hstack(cx, stack::HStackProps::default().gap(Space::N2), |cx| {
        let number = Arc::<str>::from(format!("{line_no:>width$}", width = width));
        let number_el = cx.text_props(TextProps {
            layout: Default::default(),
            text: number,
            style: Some(number_style),
            color: Some(theme.colors.text_muted),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        vec![number_el, render_code_line(cx, theme, line, spans)]
    })
}

fn render_code_line<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    line: LineSlice<'_>,
    spans: &[fret_syntax::HighlightSpan],
) -> AnyElement {
    let text_style = TextStyle {
        font: FontId::monospace(),
        size: theme.metrics.mono_font_size,
        weight: FontWeight::NORMAL,
        line_height: Some(theme.metrics.mono_font_line_height),
        letter_spacing_em: None,
    };

    let segments = segments_for_range(line.range.clone(), spans, line.text);

    stack::hstack(cx, stack::HStackProps::default().gap(Space::N0), |cx| {
        segments
            .into_iter()
            .map(|(text, highlight)| {
                let color = highlight
                    .and_then(|h| syntax_color(theme, h))
                    .unwrap_or(theme.colors.text_primary);
                cx.text_props(TextProps {
                    layout: Default::default(),
                    text: Arc::<str>::from(text),
                    style: Some(text_style.clone()),
                    color: Some(color),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                })
            })
            .collect()
    })
}

fn syntax_color(theme: &Theme, highlight: &str) -> Option<Color> {
    let key = format!("color.syntax.{highlight}");
    if let Some(c) = theme.color_by_key(&key) {
        return Some(c);
    }

    let fallback = highlight.split('.').next().unwrap_or(highlight);
    match fallback {
        "comment" => Some(theme.colors.text_muted),
        "string" => Some(theme.colors.viewport_gizmo_y),
        "number" | "boolean" | "constant" => Some(theme.colors.viewport_rotate_gizmo),
        "keyword" | "operator" => Some(theme.colors.accent),
        "type" | "constructor" => Some(theme.colors.viewport_marker),
        "function" => Some(theme.colors.viewport_drag_line_orbit),
        "property" | "variable" => Some(theme.colors.text_primary),
        "punctuation" => Some(theme.colors.text_muted),
        _ => None,
    }
}

#[derive(Debug, Clone)]
struct LineSlice<'a> {
    range: Range<usize>,
    text: &'a str,
}

fn split_lines(text: &str) -> Vec<LineSlice<'_>> {
    let mut out = Vec::new();
    let mut start = 0usize;
    for (i, b) in text.as_bytes().iter().enumerate() {
        if *b == b'\n' {
            out.push(LineSlice {
                range: start..i,
                text: &text[start..i],
            });
            start = i + 1;
        }
    }
    out.push(LineSlice {
        range: start..text.len(),
        text: &text[start..],
    });
    out
}

fn line_number_width(lines: usize) -> usize {
    let mut n = lines.max(1);
    let mut digits = 0usize;
    while n > 0 {
        digits += 1;
        n /= 10;
    }
    digits
}

fn segments_for_range(
    global_range: Range<usize>,
    spans: &[fret_syntax::HighlightSpan],
    line_text: &str,
) -> Vec<(String, Option<&'static str>)> {
    let mut segments = Vec::new();
    let mut cursor = global_range.start;

    for span in spans {
        if span.range.end <= global_range.start || span.range.start >= global_range.end {
            continue;
        }
        let start = span.range.start.max(global_range.start);
        let end = span.range.end.min(global_range.end);
        if cursor < start {
            let rel = cursor - global_range.start;
            let rel_end = start - global_range.start;
            segments.push((line_text[rel..rel_end].to_string(), None));
        }
        let rel = start - global_range.start;
        let rel_end = end - global_range.start;
        segments.push((line_text[rel..rel_end].to_string(), span.highlight));
        cursor = end;
    }

    if cursor < global_range.end {
        let rel = cursor - global_range.start;
        let rel_end = global_range.end - global_range.start;
        segments.push((line_text[rel..rel_end].to_string(), None));
    }

    if segments.is_empty() {
        segments.push((line_text.to_string(), None));
    }

    segments
}
