use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::OnceLock;
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

fn merman_renderer() -> &'static merman::render::HeadlessRenderer {
    static RENDERER: OnceLock<merman::render::HeadlessRenderer> = OnceLock::new();
    RENDERER.get_or_init(|| {
        // Use vendored metrics for "Mermaid-like" label sizing. Rendering runs in a background
        // query and is cached, so the extra setup cost is amortized.
        merman::render::HeadlessRenderer::default().with_vendored_text_measurer()
    })
}

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

    // Mermaid uses the root `<svg id="...">` as a prefix for internal ids (<defs> markers,
    // accessibility title ids, ...). Make it stable and per-block to avoid collisions when
    // multiple Markdown diagrams are inlined into the same UI tree.
    let diagram_id = merman::render::sanitize_svg_id(&format!("md-{:?}", info.id));

    let handle = cx.use_query(key, policy, move |_token| {
        let code: &str = code_owned.as_ref();

        tracing::debug!(
            target: "fret_markdown::mermaid",
            code_len = code.len(),
            "mermaid svg: render queued"
        );

        let result = catch_unwind(AssertUnwindSafe(|| {
            let renderer = merman_renderer();
            let svg = renderer
                .render_svg_readable_sync_with_diagram_id(code, &diagram_id)
                .map_err(|e| QueryError::permanent(format!("mermaid render error: {e}")))?
                .ok_or_else(|| {
                    QueryError::permanent("mermaid render error: no diagram detected")
                })?;

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
