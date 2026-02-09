use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::time::Duration;

use fret_core::{
    Edges, FontId, FontWeight, Px, SvgFit, TextOverflow, TextSlant, TextStyle, TextWrap,
};
use fret_query::ui::QueryElementContextExt as _;
use fret_query::{QueryError, QueryKey, QueryPolicy, QueryStatus};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, ScrollAxis, ScrollProps,
    SvgIconProps, TextProps,
};
use fret_ui::{ElementContext, SvgSource, Theme, UiHost};
use fret_ui_kit::declarative::ModelWatchExt as _;

use super::CodeBlockInfo;

const MERMAID_SVG_NAMESPACE: &str = "fret-markdown.mermaid_svg.v1";
const MERMAID_SVG_CACHE_WINDOW: Duration = Duration::from_secs(60 * 10);

#[derive(Debug, Clone)]
struct MermaidSvgReady {
    svg_bytes: Arc<[u8]>,
    viewbox_w: Option<f32>,
    viewbox_h: Option<f32>,
}

#[derive(Debug, Clone)]
enum MermaidSvgEntry {
    Loading,
    Ready(MermaidSvgReady),
    Error(Arc<str>),
}

fn mermaid_svg_query_policy() -> QueryPolicy {
    // Mermaid rendering is deterministic and purely local.
    QueryPolicy {
        stale_time: MERMAID_SVG_CACHE_WINDOW,
        cache_time: MERMAID_SVG_CACHE_WINDOW,
        keep_previous_data_while_loading: true,
        ..Default::default()
    }
}

pub(super) fn render_mermaid_code_fence<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    info: CodeBlockInfo,
    options: fret_code_view::CodeBlockUiOptions,
    header: fret_code_view::CodeBlockHeaderSlots,
) -> AnyElement {
    let entry = mermaid_svg_entry(cx, &info);
    match entry {
        MermaidSvgEntry::Ready(ready) => render_mermaid_svg(cx, theme, options, header, ready),
        MermaidSvgEntry::Loading => fret_code_view::code_block_with_header_slots(
            cx,
            &info.code,
            info.language.as_deref(),
            false,
            options,
            header,
        ),
        MermaidSvgEntry::Error(err) => fret_code_view::code_block_with_header_slots(
            cx,
            &Arc::<str>::from(format!(
                "{}\n\n(fret-markdown: mermaid render failed: {err})",
                info.code
            )),
            info.language.as_deref(),
            false,
            options,
            header,
        ),
    }
}

fn render_mermaid_svg<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    options: fret_code_view::CodeBlockUiOptions,
    header: fret_code_view::CodeBlockHeaderSlots,
    ready: MermaidSvgReady,
) -> AnyElement {
    let bg = theme.color_required("card");
    let border = theme.color_required("border");

    let mut container = ContainerProps::default();
    container.layout.size.width = Length::Fill;
    container.layout.position = fret_ui::element::PositionStyle::Relative;
    container.background = Some(bg);
    container.border = if options.border {
        Edges::all(Px(1.0))
    } else {
        Edges::all(Px(0.0))
    };
    container.border_color = Some(border);
    container.corner_radii = fret_core::Corners::all(theme.metric_required("metric.radius.md"));

    let header_row = render_mermaid_header_row(cx, theme, header);

    let mut scroll = ScrollProps::default();
    scroll.axis = ScrollAxis::Both;
    scroll.layout.size.width = Length::Fill;
    scroll.layout.size.height = Length::Px(options.max_height.unwrap_or(Px(320.0)));

    cx.container(container, |cx| {
        vec![
            header_row,
            cx.scroll(scroll, |cx| {
                let mut icon = SvgIconProps::new(SvgSource::Bytes(ready.svg_bytes));
                icon.fit = SvgFit::Contain;
                icon.color = fret_core::Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                };
                // Use the SVG's own viewBox as a "natural size" hint. This keeps diagrams readable
                // without forcing them to shrink to fit the Markdown column.
                if let (Some(w), Some(h)) = (ready.viewbox_w, ready.viewbox_h) {
                    icon.layout.size.width = Length::Px(Px(w));
                    icon.layout.size.height = Length::Px(Px(h));
                } else {
                    icon.layout.size.width = Length::Fill;
                    icon.layout.size.height = Length::Px(Px(320.0));
                    icon.layout.aspect_ratio = None;
                }
                vec![cx.svg_icon_props(icon)]
            }),
        ]
    })
}

fn render_mermaid_header_row<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    header: fret_code_view::CodeBlockHeaderSlots,
) -> AnyElement {
    let border = theme.color_required("border");

    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.border = Edges {
        top: Px(0.0),
        right: Px(0.0),
        bottom: Px(1.0),
        left: Px(0.0),
    };
    props.border_color = Some(border);
    props.padding = Edges::all(theme.metric_required("metric.padding.sm"));

    cx.container(props, |cx| {
        let mut row = FlexProps::default();
        row.layout.size.width = Length::Fill;
        row.justify = MainAlign::SpaceBetween;
        row.align = CrossAlign::Center;

        let mut left_props = FlexProps::default();
        left_props.gap = theme.metric_required("metric.gap.sm");
        left_props.align = CrossAlign::Center;

        let mut right_props = FlexProps::default();
        right_props.gap = theme.metric_required("metric.gap.sm");
        right_props.align = CrossAlign::Center;

        vec![cx.flex(row, |cx| {
            let left_children = if header.left.is_empty() {
                vec![cx.text_props(TextProps {
                    layout: Default::default(),
                    text: Arc::<str>::from("Mermaid"),
                    style: Some(TextStyle {
                        font: FontId::monospace(),
                        size: theme.metric_required("metric.font.mono_size"),
                        weight: FontWeight::SEMIBOLD,
                        slant: TextSlant::Normal,
                        line_height: Some(theme.metric_required("metric.font.mono_line_height")),
                        letter_spacing_em: None,
                    }),
                    color: Some(theme.color_required("muted-foreground")),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                })]
            } else {
                header.left
            };
            let right_children = header.right;

            vec![
                cx.flex(left_props, |_cx| left_children),
                cx.flex(right_props, |_cx| right_children),
            ]
        })]
    })
}

fn mermaid_svg_entry<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    info: &CodeBlockInfo,
) -> MermaidSvgEntry {
    let code_trimmed = info.code.trim();
    if code_trimmed.is_empty() {
        return MermaidSvgEntry::Error(Arc::<str>::from("empty diagram"));
    }

    let key = QueryKey::<MermaidSvgReady>::new_named(
        MERMAID_SVG_NAMESPACE,
        &(info.id, code_trimmed),
        "mermaid_svg",
    );
    let policy = mermaid_svg_query_policy();

    let code_owned: Arc<str> = if code_trimmed.len() == info.code.len() {
        info.code.clone()
    } else {
        Arc::<str>::from(code_trimmed.to_string())
    };

    let diagram_id = sanitize_svg_id(&format!("md-{:?}", info.id));

    let handle = cx.use_query(key, policy, move |_token| {
        let code: &str = code_owned.as_ref();

        tracing::debug!(
            target: "fret_markdown::mermaid",
            code_len = code.len(),
            "mermaid svg: render queued"
        );

        let result = catch_unwind(AssertUnwindSafe(|| {
            let svg = mermaid_rs_renderer::render(code)
                .map_err(|e| QueryError::permanent(format!("mermaid render error: {e}")))?;
            let svg = set_svg_root_id(svg, &diagram_id);

            let svg = foreign_object_label_fallback_svg_text(&svg);
            let (viewbox_w, viewbox_h) = svg_viewbox_wh(&svg);

            Ok(MermaidSvgReady {
                svg_bytes: Arc::<[u8]>::from(svg.into_bytes()),
                viewbox_w,
                viewbox_h,
            })
        }));

        match result {
            Ok(v) => {
                tracing::debug!(target: "fret_markdown::mermaid", "mermaid svg: rendered");
                v
            }
            Err(_) => Err(QueryError::permanent("mermaid svg: panic")),
        }
    });

    let state = cx
        .watch_model(handle.model())
        .layout()
        .cloned()
        .unwrap_or_default();

    let Some(data) = state.data else {
        return match state.status {
            QueryStatus::Error => MermaidSvgEntry::Error(
                state
                    .error
                    .as_ref()
                    .map(|e| e.message().clone())
                    .unwrap_or_else(|| Arc::<str>::from("mermaid svg: error")),
            ),
            QueryStatus::Idle | QueryStatus::Loading | QueryStatus::Success => {
                MermaidSvgEntry::Loading
            }
        };
    };

    MermaidSvgEntry::Ready((*data).clone())
}

fn sanitize_svg_id(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        let ok = matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | ':' | '.');
        out.push(if ok { ch } else { '-' });
    }
    if out.is_empty() {
        out.push_str("mermaid");
    }
    out
}

fn set_svg_root_id(mut svg: String, id: &str) -> String {
    let Some(svg_tag_start) = svg.find("<svg") else {
        return svg;
    };
    let Some(tag_end) = svg[svg_tag_start..].find('>') else {
        return svg;
    };
    let tag_end = svg_tag_start + tag_end;
    if svg[svg_tag_start..tag_end].contains(" id=\"") {
        return svg;
    }

    svg.insert_str(tag_end, &format!(" id=\"{id}\""));
    svg
}

fn svg_viewbox_wh(svg: &str) -> (Option<f32>, Option<f32>) {
    let needle = "viewBox=\"";
    let Some(i) = svg.find(needle) else {
        return (None, None);
    };
    let after = &svg[i + needle.len()..];
    let Some(end) = after.find('"') else {
        return (None, None);
    };
    let vb = &after[..end];
    let mut it = vb.split_whitespace();
    let _min_x = it.next();
    let _min_y = it.next();
    let w = it.next().and_then(|s| s.parse::<f32>().ok());
    let h = it.next().and_then(|s| s.parse::<f32>().ok());
    (w, h)
}

// --- foreignObject overlay fallback ---
//
// Mermaid SVG output uses `<foreignObject>` for many labels. Most headless SVG stacks (including
// `resvg`) do not fully support HTML inside SVG. For Markdown rendering we keep the upstream-like
// SVG as-is, but add a best-effort `<text>` overlay so diagrams remain readable.

#[derive(Clone, Copy, Debug, Default)]
struct Translate {
    x: f64,
    y: f64,
}

fn parse_attr_str<'a>(tag: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!(r#"{key}=""#);
    let i = tag.find(&needle)?;
    let rest = &tag[i + needle.len()..];
    let end = rest.find('"')?;
    Some(rest[..end].trim())
}

fn parse_attr_f64(tag: &str, key: &str) -> Option<f64> {
    parse_attr_str(tag, key)?.parse::<f64>().ok()
}

fn is_self_closing(tag: &str) -> bool {
    tag.trim_end().ends_with("/>")
}

fn parse_translate(transform: &str) -> Translate {
    let lower = transform.to_ascii_lowercase();
    let Some(i) = lower.find("translate(") else {
        return Translate::default();
    };
    let after = &transform[i + "translate(".len()..];
    let Some(end) = after.find(')') else {
        return Translate::default();
    };
    let args = &after[..end];

    let mut nums = Vec::<f64>::with_capacity(2);
    let mut cur = String::new();
    for ch in args.chars() {
        if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' || ch == 'e' || ch == 'E' {
            cur.push(ch);
        } else if !cur.is_empty() {
            if let Ok(v) = cur.parse::<f64>() {
                nums.push(v);
            }
            cur.clear();
        }
    }
    if !cur.is_empty() {
        if let Ok(v) = cur.parse::<f64>() {
            nums.push(v);
        }
    }

    Translate {
        x: *nums.get(0).unwrap_or(&0.0),
        y: *nums.get(1).unwrap_or(&0.0),
    }
}

fn sum_translate(stack: &[Translate]) -> Translate {
    let mut acc = Translate::default();
    for t in stack {
        acc.x += t.x;
        acc.y += t.y;
    }
    acc
}

fn escape_xml_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

fn strip_html_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn htmlish_to_text_lines(html: &str) -> Vec<String> {
    // Mermaid foreignObject labels often look like:
    //   <div class="label">Line 1<br/>Line 2</div>
    // We treat `<br>` as line breaks and strip remaining tags.
    let mut normalized = html.replace("<br/>", "\n");
    normalized = normalized.replace("<br />", "\n");
    normalized = normalized.replace("<br>", "\n");
    normalized = normalized.replace("</br>", "\n");
    let text = strip_html_tags(&normalized);

    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

fn foreign_object_label_fallback_svg_text(svg: &str) -> String {
    let close_tag = "</foreignObject>";
    let mut out = String::with_capacity(svg.len() + 2048);
    let mut overlays = String::new();
    let mut g_translate_stack: Vec<Translate> = Vec::new();

    let mut i = 0usize;
    while let Some(lt_rel) = svg[i..].find('<') {
        let lt = i + lt_rel;
        out.push_str(&svg[i..lt]);

        let Some(gt_rel) = svg[lt..].find('>') else {
            out.push_str(&svg[lt..]);
            i = svg.len();
            break;
        };
        let gt = lt + gt_rel + 1;
        let tag = &svg[lt..gt];

        // Comments / declarations: passthrough.
        if tag.starts_with("<!--") || tag.starts_with("<!") || tag.starts_with("<?") {
            out.push_str(tag);
            i = gt;
            continue;
        }

        if tag.starts_with("</g") {
            if !g_translate_stack.is_empty() {
                g_translate_stack.pop();
            }
            out.push_str(tag);
            i = gt;
            continue;
        }

        if tag.starts_with("<g") {
            let t = parse_attr_str(tag, "transform")
                .map(parse_translate)
                .unwrap_or_default();
            if !is_self_closing(tag) {
                g_translate_stack.push(t);
            }
            out.push_str(tag);
            i = gt;
            continue;
        }

        if tag.starts_with("<foreignObject") {
            let x = parse_attr_f64(tag, "x").unwrap_or(0.0);
            let y = parse_attr_f64(tag, "y").unwrap_or(0.0);
            let w = parse_attr_f64(tag, "width").unwrap_or(0.0);
            let h = parse_attr_f64(tag, "height").unwrap_or(0.0);
            let t = sum_translate(&g_translate_stack);

            let body_start = gt;
            let Some(close_rel) = svg[body_start..].find(close_tag) else {
                out.push_str(tag);
                i = gt;
                continue;
            };
            let body_end = body_start + close_rel;
            let body = &svg[body_start..body_end];

            let lines = htmlish_to_text_lines(body);
            if !lines.is_empty() {
                // Approximate line height. Mermaid defaults look like 14–16px.
                let font_size = 14.0f64;
                let line_h = (font_size * 1.2).max(12.0);
                let text_x = x + t.x + w * 0.5;
                let text_y0 = y + t.y + h * 0.5 - (lines.len() as f64 - 1.0) * line_h * 0.5;

                for (idx, line) in lines.iter().enumerate() {
                    let yy = text_y0 + idx as f64 * line_h;
                    let line = escape_xml_text(line);
                    overlays.push_str(&format!(
                        "<text x=\"{text_x}\" y=\"{yy}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"{font_size}\">{line}</text>"
                    ));
                }
            }

            // Skip the original foreignObject block.
            let after_close = body_end + close_tag.len();
            i = after_close;
            continue;
        }

        out.push_str(tag);
        i = gt;
    }
    if i < svg.len() {
        out.push_str(&svg[i..]);
    }

    if overlays.is_empty() {
        return out;
    }

    // Insert overlays near the end of the SVG so they draw above shapes.
    if let Some(pos) = out.rfind("</svg>") {
        out.insert_str(pos, &overlays);
        out
    } else {
        out.push_str(&overlays);
        out
    }
}
