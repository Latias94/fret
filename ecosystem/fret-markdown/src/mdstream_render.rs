use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, SemanticsDecoration};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::Space;
use fret_ui_kit::ui;

use crate::theme::MarkdownTheme;
use crate::{
    BlockQuoteInfo, CodeBlockInfo, HeadingInfo, MarkdownComponents, ParagraphInfo, RawBlockInfo,
    TableInfo, ThematicBreakInfo,
};

#[derive(Debug)]
struct MarkdownSnapshot {
    doc: mdstream::DocumentState,
    adapter: mdstream::adapters::pulldown::PulldownAdapter,
}

#[derive(Debug)]
struct MarkdownCachedState {
    source: Arc<str>,
    snapshot: Option<Arc<MarkdownSnapshot>>,
}

impl MarkdownCachedState {
    fn new() -> Self {
        Self {
            source: Arc::from(""),
            snapshot: None,
        }
    }

    fn snapshot_for_source(&mut self, source: &str) -> Arc<MarkdownSnapshot> {
        if self.source.as_ref() == source {
            if let Some(snapshot) = self.snapshot.as_ref() {
                return snapshot.clone();
            }
        }

        self.source = Arc::from(source);

        let mut stream = mdstream::MdStream::new(crate::mdstream_options_for_markdown());
        let update = stream.append(self.source.as_ref());

        let mut state = MarkdownPulldownState::new();
        state.apply_update(update);
        state.apply_update(stream.finalize());

        let snapshot = Arc::new(MarkdownSnapshot {
            doc: state.doc,
            adapter: state.adapter,
        });
        self.snapshot = Some(snapshot.clone());
        snapshot
    }
}

#[track_caller]
pub fn markdown_with<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    source: &str,
    components: &MarkdownComponents<H>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let markdown_theme = MarkdownTheme::resolve(&theme);

    let snapshot = cx.named("markdown", |cx| {
        cx.with_state(MarkdownCachedState::new, |state| {
            state.snapshot_for_source(source)
        })
    });

    markdown_mdstream_pulldown_with(
        cx,
        &theme,
        markdown_theme,
        &snapshot.doc,
        &snapshot.adapter,
        components,
    )
}

#[derive(Debug)]
pub struct MarkdownPulldownState {
    pub(crate) doc: mdstream::DocumentState,
    pub(crate) adapter: mdstream::adapters::pulldown::PulldownAdapter,
}

impl MarkdownPulldownState {
    pub fn new() -> Self {
        Self {
            doc: mdstream::DocumentState::default(),
            adapter: mdstream::adapters::pulldown::PulldownAdapter::new(
                mdstream::adapters::pulldown::PulldownAdapterOptions {
                    pulldown: crate::pulldown_options_default(),
                    prefer_display_for_pending: true,
                },
            ),
        }
    }

    pub fn doc(&self) -> &mdstream::DocumentState {
        &self.doc
    }

    pub fn clear(&mut self) {
        self.doc.clear();
        self.adapter.clear();
    }

    pub fn apply_update(&mut self, update: mdstream::Update) -> mdstream::AppliedUpdate {
        self.adapter.apply_update(&update);
        self.doc.apply(update)
    }

    pub fn apply_update_ref(
        &mut self,
        update: &mdstream::UpdateRef<'_>,
    ) -> mdstream::AppliedUpdate {
        // Note: `UpdateRef` borrows from `MdStream`. Convert to an owned update to keep this state
        // render- and pipeline-agnostic (safe to store).
        self.apply_update(update.to_owned())
    }
}

impl Default for MarkdownPulldownState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct MarkdownStreamState {
    opts: mdstream::Options,
    stream: mdstream::MdStream,
    state: MarkdownPulldownState,
}

impl MarkdownStreamState {
    pub fn new() -> Self {
        Self::new_with_options(crate::mdstream_options_for_markdown())
    }

    pub fn new_with_options(opts: mdstream::Options) -> Self {
        Self {
            stream: mdstream::MdStream::new(opts.clone()),
            opts,
            state: MarkdownPulldownState::new(),
        }
    }

    pub fn state(&self) -> &MarkdownPulldownState {
        &self.state
    }

    pub fn clear(&mut self) {
        self.stream = mdstream::MdStream::new(self.opts.clone());
        self.state.clear();
    }

    pub fn append(&mut self, chunk: &str) -> mdstream::AppliedUpdate {
        self.state.apply_update(self.stream.append(chunk))
    }

    pub fn finalize(&mut self) -> mdstream::AppliedUpdate {
        self.state.apply_update(self.stream.finalize())
    }
}

impl Default for MarkdownStreamState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn markdown_streaming_pulldown<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    state: &MarkdownPulldownState,
) -> AnyElement {
    markdown_streaming_pulldown_with(cx, state, &MarkdownComponents::default())
}

pub fn markdown_streaming_pulldown_with<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    state: &MarkdownPulldownState,
    components: &MarkdownComponents<H>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let markdown_theme = MarkdownTheme::resolve(&theme);
    markdown_mdstream_pulldown_with(
        cx,
        &theme,
        markdown_theme,
        state.doc(),
        &state.adapter,
        components,
    )
}

fn markdown_mdstream_pulldown_with<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    doc: &mdstream::DocumentState,
    adapter: &mdstream::adapters::pulldown::PulldownAdapter,
    components: &MarkdownComponents<H>,
) -> AnyElement {
    let committed = doc.committed();
    let pending = doc.pending();

    let log_once = cx.with_state(
        || false,
        |logged| {
            if *logged {
                false
            } else {
                *logged = true;
                true
            }
        },
    );
    if log_once {
        let mut lines: Vec<String> = Vec::new();
        for block in committed.iter().take(32) {
            let raw = block.display_or_raw();
            let raw_one_line = raw
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .chars()
                .take(80)
                .collect::<String>();
            let has_dollars = raw.contains("$$");
            let has_adapter_events = adapter.committed_events(block.id).is_some();
            lines.push(format!(
                "{:?} id={:?} adapter_events={} has_$$={} raw0={:?}",
                block.kind, block.id, has_adapter_events, has_dollars, raw_one_line
            ));
        }
        if let Some(p) = pending {
            let raw = p.display_or_raw();
            let raw_one_line = raw
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .chars()
                .take(80)
                .collect::<String>();
            let has_dollars = raw.contains("$$");
            let has_adapter_events = adapter.parse_pending(p).iter().any(|_| true);
            lines.push(format!(
                "PENDING {:?} id={:?} adapter_events={} has_$$={} raw0={:?}",
                p.kind, p.id, has_adapter_events, has_dollars, raw_one_line
            ));
        }

        tracing::debug!(
            target: "fret_markdown::mdstream",
            committed = committed.len(),
            pending = pending.is_some(),
            "mdstream blocks:\n{}",
            lines.join("\n")
        );
    }

    ui::v_flex(|cx| {
        let mut out = Vec::with_capacity(committed.len() + usize::from(pending.is_some()));

        cx.for_each_keyed(
            committed,
            |b| b.id,
            |cx, _i, block| match adapter.committed_events(block.id) {
                Some(events) => out.push(render_mdstream_block_with_events(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    block,
                    events,
                )),
                None => {
                    let tmp = crate::parse_events(block.display_or_raw());
                    out.push(render_mdstream_block_with_events(
                        cx,
                        theme,
                        markdown_theme,
                        components,
                        block,
                        &tmp,
                    ));
                }
            },
        );

        if let Some(pending) = pending {
            cx.keyed(pending.id, |cx| {
                let events = adapter.parse_pending(pending);
                out.push(render_mdstream_block_with_events(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    pending,
                    &events,
                ));
            });
        }

        out
    })
    .gap(Space::N2)
    .w_full()
    .into_element(cx)
}

fn render_mdstream_block_with_events<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    markdown_theme: MarkdownTheme,
    components: &MarkdownComponents<H>,
    block: &mdstream::Block,
    events: &[pulldown_cmark::Event<'static>],
) -> AnyElement {
    match block.kind {
        mdstream::BlockKind::Heading => {
            let (level, text, explicit_id) = crate::parse_heading_text(block.display_or_raw())
                .unwrap_or_else(|| {
                    let raw = block.display_or_raw().trim();
                    let (title, id) = crate::parse::split_trailing_heading_id(raw);
                    (1, title, id)
                });
            let info = HeadingInfo { level, text };
            let semantics_label = info.text.clone();
            let test_id =
                crate::anchors::heading_anchor_test_id_with_id(&info.text, explicit_id.as_deref());

            let el = if let Some(render) = &components.heading {
                render(cx, info)
            } else {
                crate::render_heading_inline(cx, theme, markdown_theme, components, info, events)
            };

            let el = if let Some(decorate) = &components.anchor_decorate {
                decorate(cx, test_id.clone(), el)
            } else {
                el
            };
            el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Heading)
                    .label(semantics_label)
                    .level(u32::from(level))
                    .test_id(test_id),
            )
        }
        mdstream::BlockKind::Paragraph => {
            if crate::is_display_math_block_text(block.display_or_raw()) {
                let latex = crate::parse_math_block_body(block.display_or_raw());
                tracing::debug!(
                    target: "fret_markdown::math",
                    block_id = ?block.id,
                    latex_len = latex.len(),
                    "render paragraph as display math (by raw)"
                );
                return crate::render_math_block(cx, theme, markdown_theme, components, latex);
            }
            if let Some(latex) = crate::display_math_only_events(events) {
                tracing::debug!(
                    target: "fret_markdown::math",
                    block_id = ?block.id,
                    latex_len = latex.len(),
                    "render paragraph as display math"
                );
                return crate::render_math_block(cx, theme, markdown_theme, components, latex);
            }
            let info = ParagraphInfo {
                text: Arc::<str>::from(block.display_or_raw().trim_end().to_string()),
            };
            if let Some(render) = &components.paragraph {
                render(cx, info)
            } else {
                crate::render_paragraph_inline(cx, theme, markdown_theme, components, events)
            }
        }
        mdstream::BlockKind::ThematicBreak => {
            if let Some(render) = &components.thematic_break {
                render(cx, ThematicBreakInfo)
            } else {
                crate::render_thematic_break(cx, theme, markdown_theme)
            }
        }
        mdstream::BlockKind::CodeFence => {
            let (language, code) = crate::parse_code_fence_body(block.display_or_raw());
            let info = CodeBlockInfo {
                id: block.id,
                language,
                code,
            };
            if let Some(render) = &components.code_block {
                render(cx, info)
            } else {
                crate::render_code_block(cx, info, components)
            }
        }
        mdstream::BlockKind::List => {
            let list = crate::parse_list_info(block.display_or_raw());
            if let Some(render) = &components.list {
                render(cx, list)
            } else {
                crate::pulldown_render::render_pulldown_events_root(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    events,
                )
            }
        }
        mdstream::BlockKind::BlockQuote => {
            let info = BlockQuoteInfo {
                text: crate::strip_blockquote_prefix(block.display_or_raw()),
            };
            if let Some(render) = &components.blockquote {
                render(cx, info)
            } else {
                crate::pulldown_render::render_pulldown_events_root(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    events,
                )
            }
        }
        mdstream::BlockKind::Table => {
            let info = TableInfo {
                text: Arc::<str>::from(block.display_or_raw().trim_end().to_string()),
            };
            if let Some(render) = &components.table {
                render(cx, info)
            } else {
                // Intentionally not using fret-ui-kit's TanStack-inspired table:
                // it is a data-grid with fixed-row virtualized layout (sorting/resizing/pinning),
                // while Markdown tables need content-driven, multi-line cell layout.
                crate::pulldown_render::render_pulldown_events_root(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    events,
                )
            }
        }
        mdstream::BlockKind::MathBlock => {
            // mdstream already classifies the block as MathBlock; don't rely on pulldown to
            // re-discover `Event::DisplayMath` because the adapter may have stripped delimiters.
            let mut latex = crate::parse_math_block_body(block.display_or_raw());
            let latex_from_events = crate::latex_from_pulldown_math_events(events);
            if latex.trim().is_empty() {
                if let Some(from_events) = latex_from_events.clone() {
                    latex = from_events;
                }
            }
            let log_once = cx.with_state(
                || false,
                |logged| {
                    if *logged {
                        false
                    } else {
                        *logged = true;
                        true
                    }
                },
            );
            if log_once {
                let has_display_math_event = events.iter().any(|e| {
                    matches!(
                        e,
                        pulldown_cmark::Event::DisplayMath(_)
                            | pulldown_cmark::Event::InlineMath(_)
                    )
                });
                tracing::debug!(
                    target: "fret_markdown::math",
                    block_id = ?block.id,
                    raw = %block.display_or_raw().replace('\n', "\\n"),
                    latex_len = latex.len(),
                    latex_from_events_len = latex_from_events.as_ref().map(|s| s.len()),
                    has_math_event = has_display_math_event,
                    "render mdstream math block"
                );
            }
            crate::render_math_block(cx, theme, markdown_theme, components, latex)
        }
        mdstream::BlockKind::HtmlBlock
        | mdstream::BlockKind::FootnoteDefinition
        | mdstream::BlockKind::Unknown => {
            let info = RawBlockInfo {
                kind: crate::raw_block_kind_from_mdstream(block.kind),
                text: Arc::<str>::from(block.display_or_raw().trim_end().to_string()),
            };
            if let Some(render) = &components.raw_block {
                render(cx, info)
            } else {
                crate::pulldown_render::render_pulldown_events_root(
                    cx,
                    theme,
                    markdown_theme,
                    components,
                    events,
                )
            }
        }
    }
}
