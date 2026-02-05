use std::sync::Arc;

use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{LayoutRefinement, Space};

use fret_markdown::BlockId;
use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

/// Assistant response renderer (Markdown-first).
///
/// This is the Fret ecosystem equivalent of ai-elements' `MessageResponse` (Streamdown). It uses
/// `fret-markdown` to render markdown content and delegates code fences to `fret-code-view`.
#[derive(Clone)]
pub struct MessageResponse {
    source: Arc<str>,
    layout: LayoutRefinement,
    padding: Space,
    streaming: bool,
    finalized: bool,
    on_link_activate: Option<fret_markdown::OnLinkActivate>,
    code_block_actions: bool,
    test_id_prefix: Option<Arc<str>>,
}

impl std::fmt::Debug for MessageResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageResponse")
            .field("source_len", &self.source.len())
            .field("layout", &self.layout)
            .field("padding", &self.padding)
            .field("streaming", &self.streaming)
            .field("finalized", &self.finalized)
            .field("has_on_link_activate", &self.on_link_activate.is_some())
            .field("code_block_actions", &self.code_block_actions)
            .field("test_id_prefix", &self.test_id_prefix.as_deref())
            .finish()
    }
}

impl MessageResponse {
    pub fn new(source: impl Into<Arc<str>>) -> Self {
        Self {
            source: source.into(),
            layout: LayoutRefinement::default(),
            padding: Space::N0,
            streaming: true,
            finalized: true,
            on_link_activate: None,
            code_block_actions: true,
            test_id_prefix: None,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn padding(mut self, padding: Space) -> Self {
        self.padding = padding;
        self
    }

    /// Enables streaming-friendly parsing and stable block identity.
    ///
    /// When enabled, `MessageResponse` maintains an internal `fret-markdown::MarkdownStreamState`
    /// and applies append-only updates when the new `source` extends the previous `source`.
    ///
    /// If the `source` is not append-only (e.g. it was replaced or truncated), the internal state
    /// is reset and the new `source` is re-applied from scratch.
    pub fn streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }

    /// Marks the stream as finalized (flushes pending blocks like unterminated code fences).
    ///
    /// Only relevant when `streaming(true)` is enabled.
    pub fn finalized(mut self, finalized: bool) -> Self {
        self.finalized = finalized;
        self
    }

    pub fn on_link_activate(mut self, on_link_activate: fret_markdown::OnLinkActivate) -> Self {
        self.on_link_activate = Some(on_link_activate);
        self
    }

    /// Enables a small per-code-block actions row (expand/collapse toggles).
    pub fn code_block_actions(mut self, enabled: bool) -> Self {
        self.code_block_actions = enabled;
        self
    }

    /// Prefix used to stamp stable `test_id`s on interactive affordances (code block actions).
    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = fret_ui::Theme::global(&*cx.app).clone();

        let mut components = fret_markdown::MarkdownComponents::<H>::default();
        components.on_link_activate = self.on_link_activate;

        #[derive(Debug, Default)]
        struct CodeActionOrdinalState {
            map: std::collections::HashMap<BlockId, usize>,
            next: usize,
        }

        #[derive(Debug)]
        struct StreamingState {
            stream: fret_markdown::MarkdownStreamState,
            last_source: Arc<str>,
            finalized: bool,
        }

        impl Default for StreamingState {
            fn default() -> Self {
                Self {
                    stream: fret_markdown::MarkdownStreamState::default(),
                    last_source: Arc::<str>::from(""),
                    finalized: false,
                }
            }
        }

        #[derive(Default)]
        struct ExpandedModelState {
            model: Option<fret_runtime::Model<std::collections::HashSet<BlockId>>>,
        }

        let expanded = cx.with_state(ExpandedModelState::default, |st| st.model.clone());
        let expanded = match expanded {
            Some(model) => model,
            None => {
                let model = cx
                    .app
                    .models_mut()
                    .insert(std::collections::HashSet::<BlockId>::new());
                cx.with_state(ExpandedModelState::default, |st| st.model = Some(model.clone()));
                model
            }
        };

        let expanded_snapshot = cx
            .get_model_cloned(&expanded, Invalidation::Paint)
            .unwrap_or_default();

        let test_id_prefix = self.test_id_prefix.clone();
        let expanded_for_resolver = expanded.clone();
        components.code_block_ui_resolver = Some(Arc::new(move |cx, info, options| {
            let expanded = cx
                .get_model_cloned(&expanded_for_resolver, Invalidation::Paint)
                .unwrap_or_default();
            if expanded.contains(&info.id) {
                options.max_height = None;
            }
        }));

        if self.code_block_actions {
            let expanded = expanded.clone();
            let expanded_snapshot = expanded_snapshot.clone();
            let test_id_prefix = test_id_prefix.clone();
            components.code_block_actions = Some(Arc::new(move |cx, info| {
                let is_expanded = expanded_snapshot.contains(&info.id);

                let ordinal = cx.with_state(CodeActionOrdinalState::default, |st| {
                    if let Some(v) = st.map.get(&info.id) {
                        return *v;
                    }
                    let v = st.next;
                    st.next = st.next.saturating_add(1);
                    st.map.insert(info.id, v);
                    v
                });

                let label = if is_expanded { "Collapse" } else { "Expand" };

                let on_activate: OnActivate = Arc::new({
                    let expanded = expanded.clone();
                    let id = info.id;
                    move |host, _cx, _reason| {
                        let _ = host.models_mut().update(&expanded, |set| {
                            if set.contains(&id) {
                                set.remove(&id);
                            } else {
                                set.insert(id);
                            }
                        });
                    }
                });

                let mut btn = Button::new(label)
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .on_activate(on_activate);

                if let Some(prefix) = test_id_prefix.clone() {
                    btn = btn.test_id(Arc::<str>::from(format!("{prefix}code-expand-{ordinal}")));
                }

                btn.into_element(cx)
            }));
        }

        let content = if self.streaming {
            use std::cell::RefCell;
            use std::rc::Rc;

            let stream = cx.with_state(
                || Rc::<RefCell<StreamingState>>::new(RefCell::new(StreamingState::default())),
                |st| st.clone(),
            );

            {
                let mut st = stream.borrow_mut();
                let source = &self.source;

                if !source.starts_with(st.last_source.as_ref()) {
                    st.stream.clear();
                    let _ = st.stream.append(source);
                } else if source.len() > st.last_source.len() {
                    let suffix = &source[st.last_source.len()..];
                    let _ = st.stream.append(suffix);
                }

                st.last_source = self.source.clone();

                if self.finalized && !st.finalized {
                    let _ = st.stream.finalize();
                    st.finalized = true;
                } else if !self.finalized && st.finalized {
                    st.finalized = false;
                }
            }

            let st = stream.borrow();
            fret_markdown::markdown_streaming_pulldown_with(cx, st.stream.state(), &components)
        } else {
            fret_markdown::Markdown::new(self.source).into_element_with(cx, &components)
        };

        let root_layout = decl_style::layout_style(&theme, self.layout);
        let padding_px = decl_style::space(&theme, self.padding);

        cx.container(
            ContainerProps {
                layout: root_layout,
                padding: fret_core::Edges::all(padding_px),
                ..Default::default()
            },
            move |_cx| vec![content],
        )
    }
}
