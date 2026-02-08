use std::sync::Arc;

use fret_core::{FontWeight, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsDecoration, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};

use crate::elements::{
    InlineCitation, Message, MessageContent, MessageResponse, SourcesBlock, ToolCallBlock,
};
use crate::model::{MessagePart, MessageRole};

#[derive(Clone)]
/// Renders a list of `MessagePart` items into a role-colored message bubble.
///
/// This is the “rich message” path: markdown, tool calls, sources, and inline citations are all
/// expressed as parts. Prefer this over `Message` when you are aligning to AI Elements behavior.
pub struct MessageParts {
    role: MessageRole,
    parts: Arc<[MessagePart]>,
    on_link_activate: Option<fret_markdown::OnLinkActivate>,
    test_id_prefix: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for MessageParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageParts")
            .field("role", &self.role)
            .field("parts_len", &self.parts.len())
            .field("has_on_link_activate", &self.on_link_activate.is_some())
            .field("test_id_prefix", &self.test_id_prefix.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl MessageParts {
    pub fn new(role: MessageRole, parts: impl Into<Arc<[MessagePart]>>) -> Self {
        Self {
            role,
            parts: parts.into(),
            on_link_activate: None,
            test_id_prefix: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn on_link_activate(mut self, on_link_activate: fret_markdown::OnLinkActivate) -> Self {
        self.on_link_activate = Some(on_link_activate);
        self
    }

    /// Optional `test_id` prefix used to stamp stable selectors on rendered parts.
    ///
    /// This is intended for automation/debug gates. Callers should provide a stable prefix (e.g.
    /// derived from `MessageId`) so part selectors remain stable across inserts/removals.
    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        #[derive(Default)]
        struct CitationSelectionState {
            selected_source_id: Option<Model<Option<Arc<str>>>>,
        }

        let selected_source_id = cx.with_state(CitationSelectionState::default, |st| {
            st.selected_source_id.clone()
        });
        let selected_source_id = match selected_source_id {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(None::<Arc<str>>);
                cx.with_state(CitationSelectionState::default, |st| {
                    st.selected_source_id = Some(model.clone());
                });
                model
            }
        };

        let sources_for_citations = self.parts.iter().find_map(|part| match part {
            MessagePart::Sources(items) => Some(items.clone()),
            _ => None,
        });

        fn role_fg_key(role: MessageRole) -> &'static str {
            match role {
                MessageRole::User => "fret.ai.message.user.fg",
                MessageRole::Assistant => "fret.ai.message.assistant.fg",
                MessageRole::System => "fret.ai.message.system.fg",
                MessageRole::Tool => "fret.ai.message.tool.fg",
            }
        }

        let fg = match self.role {
            MessageRole::User => theme
                .color_by_key(role_fg_key(self.role))
                .unwrap_or_else(|| theme.color_required("secondary-foreground")),
            _ => theme
                .color_by_key(role_fg_key(self.role))
                .unwrap_or_else(|| theme.color_required("foreground")),
        };

        let on_link_activate = self.on_link_activate;
        let test_id_prefix = self.test_id_prefix;
        let parts = self.parts;
        let selected_source_id = selected_source_id.clone();

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().min_w_0())
                .gap(Space::N3),
            move |cx| {
                let mut out = Vec::new();
                for (index, part) in parts.iter().enumerate() {
                    let part_id = test_id_prefix
                        .clone()
                        .map(|p| Arc::<str>::from(format!("{p}part-{index}")));
                    match part {
                        MessagePart::Text(text) => {
                            let text_style = TextStyle {
                                font: Default::default(),
                                size: theme.metric_required("font.size"),
                                weight: FontWeight::NORMAL,
                                slant: Default::default(),
                                line_height: Some(theme.metric_required("font.line_height")),
                                letter_spacing_em: None,
                            };

                            let el = cx.text_props(TextProps {
                                layout: LayoutStyle::default(),
                                text: text.clone(),
                                style: Some(text_style),
                                color: Some(fg),
                                wrap: TextWrap::Word,
                                overflow: TextOverflow::Clip,
                            });

                            let Some(test_id) = part_id else {
                                out.push(el);
                                continue;
                            };

                            out.push(
                                el.attach_semantics(
                                    SemanticsDecoration::default()
                                        .role(SemanticsRole::Group)
                                        .test_id(test_id),
                                ),
                            );
                        }
                        MessagePart::Markdown(md) => {
                            let mut response =
                                MessageResponse::new(md.text.clone()).finalized(md.finalized);
                            if let Some(handler) = on_link_activate.clone() {
                                response = response.on_link_activate(handler);
                            }
                            if let Some(prefix) = test_id_prefix.clone() {
                                response = response.test_id_prefix(prefix);
                            }
                            let el = response.into_element(cx);
                            let Some(test_id) = part_id else {
                                out.push(el);
                                continue;
                            };

                            out.push(
                                el.attach_semantics(
                                    SemanticsDecoration::default()
                                        .role(SemanticsRole::Group)
                                        .test_id(test_id),
                                ),
                            );
                        }
                        MessagePart::ToolCall(call) => {
                            let mut block = ToolCallBlock::new(call.clone());
                            if let Some(prefix) = test_id_prefix.clone() {
                                let root = Arc::<str>::from(format!("{prefix}toolcall-{index}"));
                                let trigger =
                                    Arc::<str>::from(format!("{prefix}toolcall-trigger-{index}"));
                                block = block.test_id_root(root).test_id_trigger(trigger);
                            }
                            out.push(block.into_element(cx));
                        }
                        MessagePart::Sources(items) => {
                            let mut block = SourcesBlock::new(items.clone())
                                .highlighted_source_model(selected_source_id.clone());
                            if let Some(handler) = on_link_activate.clone() {
                                block = block.on_open_url(handler);
                            }
                            if let Some(prefix) = test_id_prefix.clone() {
                                block = block
                                    .test_id_root(Arc::<str>::from(format!(
                                        "{prefix}sources-{index}"
                                    )))
                                    .test_id_row_prefix(Arc::<str>::from(format!(
                                        "{prefix}source-row-{index}-"
                                    )));
                            }
                            out.push(block.into_element(cx));
                        }
                        MessagePart::Citations(items) => {
                            let row = stack::hstack(
                                cx,
                                stack::HStackProps::default()
                                    .layout(LayoutRefinement::default().w_full())
                                    .gap(Space::N2),
                                |cx| {
                                    let mut out = Vec::new();
                                    for (citation_index, item) in items.iter().enumerate() {
                                        out.push(cx.keyed(citation_index, |cx| {
                                            let mut citation =
                                                InlineCitation::new(item.label.clone())
                                                    .source_ids(item.source_ids.clone())
                                                    .select_source_model(
                                                        selected_source_id.clone(),
                                                    );
                                            if let Some(sources) = sources_for_citations.clone() {
                                                citation = citation.sources(sources);
                                            }
                                            if let Some(handler) = on_link_activate.clone() {
                                                citation = citation.on_open_url(handler);
                                            }

                                            if let Some(prefix) = test_id_prefix.clone() {
                                                citation =
                                                    citation.test_id(Arc::<str>::from(format!(
                                                        "{prefix}citation-{index}-{citation_index}"
                                                    )));
                                            }

                                            citation.into_element(cx)
                                        }));
                                    }
                                    out
                                },
                            );

                            let Some(test_id) = part_id else {
                                out.push(row);
                                continue;
                            };

                            out.push(
                                row.attach_semantics(
                                    SemanticsDecoration::default()
                                        .role(SemanticsRole::Group)
                                        .test_id(test_id),
                                ),
                            );
                        }
                    }
                }

                out
            },
        );

        let content = MessageContent::new(self.role, [content])
            .refine_layout(self.layout)
            .into_element(cx);
        Message::new(self.role, [content]).into_element(cx)
    }
}
