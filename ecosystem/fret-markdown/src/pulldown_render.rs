use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use fret_ui_kit::Space;
use fret_ui_kit::declarative::stack;

use super::*;

pub(super) fn render_pulldown_events_root<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
) -> AnyElement {
    let mut cursor = 0usize;
    let children = render_pulldown_blocks(
        cx,
        theme,
        markdown_theme,
        components,
        events,
        &mut cursor,
        None,
    );
    if children.len() == 1 {
        return children.into_iter().next().unwrap();
    }

    stack::vstack(cx, stack::VStackProps::default().gap(Space::N2), |_cx| {
        children
    })
}

#[derive(Debug, Clone, Copy)]
enum PulldownStop {
    Item,
    BlockQuote,
    FootnoteDefinition,
}

fn stop_matches(end: &pulldown_cmark::TagEnd, stop: PulldownStop) -> bool {
    use pulldown_cmark::TagEnd;
    match (stop, end) {
        (PulldownStop::Item, TagEnd::Item) => true,
        (PulldownStop::BlockQuote, TagEnd::BlockQuote(_)) => true,
        (PulldownStop::FootnoteDefinition, TagEnd::FootnoteDefinition) => true,
        _ => false,
    }
}

fn render_pulldown_blocks<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
    stop: Option<PulldownStop>,
) -> Vec<AnyElement> {
    use pulldown_cmark::{Event, Tag, TagEnd};

    let mut out = Vec::new();
    while *cursor < events.len() {
        match (&events[*cursor], stop) {
            (Event::End(end), Some(stop)) if stop_matches(end, stop) => {
                *cursor += 1;
                break;
            }
            _ => {}
        }

        match &events[*cursor] {
            Event::Start(Tag::Paragraph) => out.push(render_pulldown_paragraph(
                cx,
                theme,
                markdown_theme,
                components,
                events,
                cursor,
            )),
            Event::Start(Tag::Heading { level, .. }) => out.push(render_pulldown_heading(
                cx,
                theme,
                markdown_theme,
                components,
                events,
                cursor,
                heading_level_to_u8(*level),
            )),
            Event::Start(Tag::CodeBlock(kind)) => out.push(render_pulldown_code_block(
                cx,
                components,
                events,
                cursor,
                kind.clone(),
            )),
            Event::Start(Tag::List(start)) => out.push(render_pulldown_list(
                cx,
                theme,
                markdown_theme,
                components,
                events,
                cursor,
                *start,
            )),
            Event::Start(Tag::BlockQuote(_)) => out.push(render_pulldown_blockquote(
                cx,
                theme,
                markdown_theme,
                components,
                events,
                cursor,
            )),
            Event::Start(Tag::FootnoteDefinition(label)) => {
                out.push(render_pulldown_footnote_definition(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    events,
                    cursor,
                    Arc::<str>::from(label.to_string()),
                ))
            }
            Event::Start(Tag::Table(_)) => out.push(render_pulldown_table(
                cx,
                theme,
                markdown_theme,
                components,
                events,
                cursor,
            )),
            Event::DisplayMath(latex) => {
                out.push(render_math_block(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    Arc::<str>::from(latex.to_string()),
                ));
                *cursor += 1;
            }
            Event::Rule => {
                out.push(render_thematic_break(cx, theme, markdown_theme));
                *cursor += 1;
            }
            Event::End(TagEnd::List(_))
            | Event::End(TagEnd::Item)
            | Event::End(TagEnd::BlockQuote(_)) => {
                *cursor += 1;
            }
            _ => {
                *cursor += 1;
            }
        }
    }

    out
}

fn render_pulldown_table<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
) -> AnyElement {
    use pulldown_cmark::{Alignment, Event, Tag, TagEnd};

    let alignments = match events.get(*cursor) {
        Some(Event::Start(Tag::Table(alignments))) => alignments.clone(),
        _ => Vec::new(),
    };

    *cursor += 1;

    let mut in_head = false;
    let mut header_rows: Vec<Vec<Vec<InlinePiece>>> = Vec::new();
    let mut body_rows: Vec<Vec<Vec<InlinePiece>>> = Vec::new();

    while *cursor < events.len() {
        match &events[*cursor] {
            Event::Start(Tag::TableHead) => {
                in_head = true;
                *cursor += 1;
            }
            Event::End(TagEnd::TableHead) => {
                in_head = false;
                *cursor += 1;
            }
            Event::Start(Tag::TableRow) => {
                let row = parse_pulldown_table_row(events, cursor);
                if in_head {
                    header_rows.push(row);
                } else {
                    body_rows.push(row);
                }
            }
            Event::End(TagEnd::Table) => {
                *cursor += 1;
                break;
            }
            _ => {
                *cursor += 1;
            }
        }
    }

    let mut column_count = alignments.len();
    for row in header_rows.iter().chain(body_rows.iter()) {
        column_count = column_count.max(row.len());
    }

    fn justify_for_alignment(alignment: Alignment) -> MainAlign {
        match alignment {
            Alignment::Center => MainAlign::Center,
            Alignment::Right => MainAlign::End,
            Alignment::None | Alignment::Left => MainAlign::Start,
        }
    }

    let all_rows = header_rows
        .iter()
        .map(|r| (true, r))
        .chain(body_rows.iter().map(|r| (false, r)));

    let mut scroll_props = ScrollProps::default();
    scroll_props.axis = ScrollAxis::X;

    cx.scroll(scroll_props, |cx| {
        let mut table_props = ContainerProps::default();
        table_props.padding = Edges::all(Px(0.0));
        table_props.border = Edges::all(Px(1.0));
        table_props.border_color = Some(markdown_theme.table_border);
        table_props.background = None;

        vec![cx.container(table_props, |cx| {
            let mut column_props = FlexProps::default();
            column_props.direction = Axis::Vertical;
            column_props.wrap = false;
            column_props.gap = Px(0.0);
            column_props.padding = Edges::all(Px(0.0));
            column_props.justify = MainAlign::Start;
            column_props.align = CrossAlign::Start;

            vec![cx.flex(column_props, |cx| {
                let mut row_index = 0usize;
                all_rows
                    .map(|(is_header, row)| {
                        let mut row_props = FlexProps::default();
                        row_props.direction = Axis::Horizontal;
                        row_props.wrap = false;
                        row_props.gap = Px(0.0);
                        row_props.padding = Edges::all(Px(0.0));
                        row_props.justify = MainAlign::Start;
                        row_props.align = CrossAlign::Stretch;

                        let cur_row_index = row_index;
                        row_index += 1;

                        cx.flex(row_props, |cx| {
                            (0..column_count)
                                .map(|col_index| {
                                    let pieces = row.get(col_index).cloned().unwrap_or_default();
                                    let justify = alignments
                                        .get(col_index)
                                        .copied()
                                        .map(justify_for_alignment)
                                        .unwrap_or(MainAlign::Start);
                                    render_table_cell(
                                        cx,
                                        theme,
                                        markdown_theme,
                                        components,
                                        is_header,
                                        cur_row_index,
                                        col_index,
                                        pieces,
                                        justify,
                                    )
                                })
                                .collect::<Vec<_>>()
                        })
                    })
                    .collect::<Vec<_>>()
            })]
        })]
    })
}

fn parse_pulldown_table_row(
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
) -> Vec<Vec<InlinePiece>> {
    use pulldown_cmark::{Event, Tag, TagEnd};

    *cursor += 1;
    let mut cells: Vec<Vec<InlinePiece>> = Vec::new();
    while *cursor < events.len() {
        match &events[*cursor] {
            Event::Start(Tag::TableCell) => cells.push(parse_pulldown_table_cell(events, cursor)),
            Event::End(TagEnd::TableRow) => {
                *cursor += 1;
                break;
            }
            _ => {
                *cursor += 1;
            }
        }
    }
    cells
}

fn parse_pulldown_table_cell(
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
) -> Vec<InlinePiece> {
    use pulldown_cmark::{Event, TagEnd};

    let start = *cursor;
    *cursor += 1;
    while *cursor < events.len() {
        if matches!(&events[*cursor], Event::End(TagEnd::TableCell)) {
            *cursor += 1;
            break;
        }
        *cursor += 1;
    }
    inline_pieces_from_events_unwrapped(&events[start..*cursor])
}

fn render_table_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    is_header: bool,
    row_index: usize,
    col_index: usize,
    pieces: Vec<InlinePiece>,
    justify: MainAlign,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.flex.grow = 1.0;
    props.layout.flex.basis = Length::Px(Px(0.0));
    props.layout.size.min_width = Some(Px(0.0));
    props.padding = Edges {
        top: markdown_theme.table_cell_padding_y,
        right: markdown_theme.table_cell_padding_x,
        bottom: markdown_theme.table_cell_padding_y,
        left: markdown_theme.table_cell_padding_x,
    };
    props.border = Edges {
        top: if row_index > 0 { Px(1.0) } else { Px(0.0) },
        right: Px(0.0),
        bottom: Px(0.0),
        left: if col_index > 0 { Px(1.0) } else { Px(0.0) },
    };
    props.border_color = Some(markdown_theme.table_border);
    props.background = is_header.then_some(markdown_theme.table_header_bg);

    let font_size = theme.metric_required("metric.font.size");
    let line_height = theme.metric_required("metric.font.line_height");
    let fg = theme.color_required("foreground");
    let base = InlineBaseStyle {
        font: FontId::default(),
        size: font_size,
        weight: if is_header {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        },
        line_height: Some(line_height),
        color: fg,
    };

    cx.container(props, |cx| {
        vec![render_inline_flow_with_layout(
            cx,
            theme,
            markdown_theme,
            components,
            base,
            &pieces,
            justify,
        )]
    })
}

fn render_pulldown_paragraph<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
) -> AnyElement {
    use pulldown_cmark::{Event, TagEnd};

    let start = *cursor;
    *cursor += 1;
    while *cursor < events.len() {
        if matches!(&events[*cursor], Event::End(TagEnd::Paragraph)) {
            *cursor += 1;
            break;
        }
        *cursor += 1;
    }
    render_paragraph_inline(
        cx,
        theme,
        markdown_theme,
        components,
        &events[start..*cursor],
    )
}

fn render_pulldown_heading<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
    level: u8,
) -> AnyElement {
    use pulldown_cmark::{Event, TagEnd};

    let start = *cursor;
    *cursor += 1;
    while *cursor < events.len() {
        if matches!(&events[*cursor], Event::End(TagEnd::Heading(_))) {
            *cursor += 1;
            break;
        }
        *cursor += 1;
    }

    let slice = &events[start..*cursor];
    let info = HeadingInfo {
        level,
        text: plain_text_from_events(slice),
    };
    render_heading_inline(cx, theme, markdown_theme, components, info, slice)
}

fn render_pulldown_code_block<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
    kind: pulldown_cmark::CodeBlockKind<'static>,
) -> AnyElement {
    use pulldown_cmark::{CodeBlockKind, Event, TagEnd};

    let language = match &kind {
        CodeBlockKind::Indented => None,
        CodeBlockKind::Fenced(info) => parse_fenced_code_language(info),
    };

    let start = *cursor;
    *cursor += 1;
    let mut buf = String::new();
    while *cursor < events.len() {
        match &events[*cursor] {
            Event::Text(t) => buf.push_str(t.as_ref()),
            Event::SoftBreak | Event::HardBreak => buf.push('\n'),
            Event::End(TagEnd::CodeBlock) => {
                *cursor += 1;
                break;
            }
            _ => {}
        }
        *cursor += 1;
    }

    let mut hasher = DefaultHasher::new();
    start.hash(&mut hasher);
    language.as_deref().hash(&mut hasher);
    buf.hash(&mut hasher);
    let id = BlockId(hasher.finish());

    let info = CodeBlockInfo {
        id,
        language,
        code: Arc::<str>::from(buf),
    };
    if let Some(render) = &components.code_block {
        render(cx, info)
    } else {
        render_code_block(cx, info, components)
    }
}

fn render_pulldown_blockquote<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
) -> AnyElement {
    *cursor += 1;
    let children = render_pulldown_blocks(
        cx,
        theme,
        markdown_theme,
        components,
        events,
        cursor,
        Some(PulldownStop::BlockQuote),
    );
    render_blockquote_container(cx, theme, markdown_theme, children)
}

fn render_blockquote_container<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    _theme: &Theme,
    markdown_theme: MarkdownTheme,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.padding = Edges::all(markdown_theme.blockquote_padding);
    props.border = Edges {
        top: Px(0.0),
        right: Px(0.0),
        bottom: Px(0.0),
        left: markdown_theme.blockquote_border_width,
    };
    props.border_color = Some(markdown_theme.blockquote_border);

    cx.container(props, |cx| {
        if children.len() == 1 {
            children
        } else {
            vec![stack::vstack(
                cx,
                stack::VStackProps::default().gap(Space::N2),
                |_cx| children,
            )]
        }
    })
}

fn render_pulldown_list<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
    start: Option<u64>,
) -> AnyElement {
    use pulldown_cmark::{Event, Tag, TagEnd};

    struct ListItem {
        task: Option<bool>,
        children: Vec<AnyElement>,
    }

    let ordered = start.is_some();
    let start_no = start.unwrap_or(1) as u32;

    *cursor += 1;
    let mut items: Vec<ListItem> = Vec::new();

    while *cursor < events.len() {
        match &events[*cursor] {
            Event::Start(Tag::Item) => {
                *cursor += 1;
                let task = match events.get(*cursor) {
                    Some(Event::TaskListMarker(checked)) => {
                        *cursor += 1;
                        Some(*checked)
                    }
                    _ => None,
                };
                let children = render_pulldown_blocks(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    events,
                    cursor,
                    Some(PulldownStop::Item),
                );
                items.push(ListItem { task, children });
            }
            Event::End(TagEnd::List(_)) => {
                *cursor += 1;
                break;
            }
            _ => {
                *cursor += 1;
            }
        }
    }

    stack::vstack(cx, stack::VStackProps::default().gap(Space::N1), |cx| {
        items
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let body = if item.children.len() == 1 {
                    item.children.into_iter().next().unwrap()
                } else {
                    stack::vstack(cx, stack::VStackProps::default().gap(Space::N1), |_cx| {
                        item.children
                    })
                };

                let marker_el = match item.task {
                    Some(checked) => {
                        let task_el = render_task_list_marker(cx, theme, markdown_theme, checked);
                        if ordered {
                            let no =
                                Arc::<str>::from(format!("{}.", start_no.saturating_add(i as u32)));
                            let no_el = cx.text_props(TextProps {
                                layout: Default::default(),
                                text: no,
                                style: None,
                                color: Some(markdown_theme.muted),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                            });
                            stack::hstack(
                                cx,
                                stack::HStackProps::default().gap(Space::N1).items_start(),
                                |_cx| vec![no_el, task_el],
                            )
                        } else {
                            task_el
                        }
                    }
                    None => {
                        let marker = if ordered {
                            Arc::<str>::from(format!("{}.", start_no.saturating_add(i as u32)))
                        } else {
                            Arc::<str>::from("?".to_string())
                        };

                        cx.text_props(TextProps {
                            layout: Default::default(),
                            text: marker,
                            style: None,
                            color: Some(markdown_theme.muted),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        })
                    }
                };

                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_start(),
                    |_cx| vec![marker_el, body],
                )
            })
            .collect::<Vec<_>>()
    })
}

fn render_task_list_marker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    checked: bool,
) -> AnyElement {
    let (text, color) = if checked {
        ("?", markdown_theme.task_checked)
    } else {
        ("?", markdown_theme.task_unchecked)
    };

    cx.text_props(TextProps {
        layout: Default::default(),
        text: Arc::<str>::from(text.to_string()),
        style: Some(TextStyle {
            font: FontId::default(),
            size: theme.metric_required("metric.font.size"),
            weight: FontWeight::NORMAL,
            slant: TextSlant::Normal,
            line_height: Some(theme.metric_required("metric.font.line_height")),
            letter_spacing_em: None,
        }),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    })
}

fn render_pulldown_footnote_definition<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    events: &[pulldown_cmark::Event<'static>],
    cursor: &mut usize,
    label: Arc<str>,
) -> AnyElement {
    *cursor += 1;
    let children = render_pulldown_blocks(
        cx,
        theme,
        markdown_theme,
        components,
        events,
        cursor,
        Some(PulldownStop::FootnoteDefinition),
    );

    let label_el = cx.text_props(TextProps {
        layout: Default::default(),
        text: Arc::<str>::from(format!("[^{}]", label)),
        style: None,
        color: Some(markdown_theme.muted),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    });

    let body = if children.len() == 1 {
        children.into_iter().next().unwrap()
    } else {
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N1), |_cx| {
            children
        })
    };

    stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_start(),
        |_cx| vec![label_el, body],
    )
}

fn plain_text_from_events(events: &[pulldown_cmark::Event<'static>]) -> Arc<str> {
    use pulldown_cmark::{Event, Tag, TagEnd};

    let mut out = String::new();
    let mut wrapper_depth = 0usize;

    for e in events {
        match e {
            Event::Start(Tag::Paragraph) | Event::Start(Tag::Heading { .. }) => {
                wrapper_depth += 1;
            }
            Event::End(TagEnd::Paragraph) | Event::End(TagEnd::Heading(_)) => {
                wrapper_depth = wrapper_depth.saturating_sub(1);
            }
            _ => {}
        }

        if wrapper_depth == 0 {
            continue;
        }

        match e {
            Event::Text(t) | Event::Code(t) | Event::InlineMath(t) | Event::DisplayMath(t) => {
                out.push_str(t.as_ref())
            }
            Event::SoftBreak => out.push(' '),
            Event::HardBreak => out.push('\n'),
            _ => {}
        }
    }

    Arc::<str>::from(out.trim().to_string())
}
