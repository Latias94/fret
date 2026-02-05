use std::sync::Arc;

/// Stable identifier for a message within a transcript.
///
/// This is aligned with `crates/fret-ui::ItemKey` so virtualized transcripts can use it directly
/// as a keyed identity.
pub type MessageId = fret_ui::ItemKey;

/// Message role taxonomy aligned with typical chat UIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// Optional external identifier for interoperability with upstream SDKs.
///
/// Apps can store their source-of-truth ID here (UUID/nanoid/etc) while still using `MessageId`
/// for stable UI identity.
pub type ExternalId = Arc<str>;

/// Tool call lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolCallState {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

/// A source/citation item associated with assistant output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceItem {
    pub id: Arc<str>,
    pub title: Arc<str>,
    pub url: Option<Arc<str>>,
    pub excerpt: Option<Arc<str>>,
}

impl SourceItem {
    pub fn new(id: impl Into<Arc<str>>, title: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            url: None,
            excerpt: None,
        }
    }

    pub fn url(mut self, url: impl Into<Arc<str>>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn excerpt(mut self, excerpt: impl Into<Arc<str>>) -> Self {
        self.excerpt = Some(excerpt.into());
        self
    }
}

/// Tool call payload representation (portable; apps decide the actual schema).
#[derive(Debug, Clone, PartialEq)]
pub enum ToolCallPayload {
    Text(Arc<str>),
    Json(serde_json::Value),
}

/// A structured tool call event emitted by an assistant.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    pub id: Arc<str>,
    pub name: Arc<str>,
    pub state: ToolCallState,
    pub input: Option<ToolCallPayload>,
    pub output: Option<ToolCallPayload>,
    pub error: Option<Arc<str>>,
}

impl ToolCall {
    pub fn new(id: impl Into<Arc<str>>, name: impl Into<Arc<str>>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            state: ToolCallState::Pending,
            input: None,
            output: None,
            error: None,
        }
    }

    pub fn state(mut self, state: ToolCallState) -> Self {
        self.state = state;
        self
    }

    pub fn input(mut self, payload: ToolCallPayload) -> Self {
        self.input = Some(payload);
        self
    }

    pub fn output(mut self, payload: ToolCallPayload) -> Self {
        self.output = Some(payload);
        self
    }

    pub fn error(mut self, error: impl Into<Arc<str>>) -> Self {
        self.error = Some(error.into());
        self
    }
}

/// A message content part (portable, UI-oriented).
#[derive(Debug, Clone, PartialEq)]
pub enum MessagePart {
    /// Plain text (typically used for user messages).
    Text(Arc<str>),
    /// Markdown content (typically used for assistant messages).
    Markdown(Arc<str>),
    /// A structured tool call (input/output + lifecycle).
    ToolCall(ToolCall),
    /// A list of sources (citations, references).
    Sources(Arc<[SourceItem]>),
}

/// A message record suitable for conversation/transcript UIs.
#[derive(Debug, Clone, PartialEq)]
pub struct AiMessage {
    pub id: MessageId,
    pub external_id: Option<ExternalId>,
    pub role: MessageRole,
    pub parts: Arc<[MessagePart]>,
}

impl AiMessage {
    pub fn new(
        id: MessageId,
        role: MessageRole,
        parts: impl IntoIterator<Item = MessagePart>,
    ) -> Self {
        Self {
            id,
            external_id: None,
            role,
            parts: parts.into_iter().collect::<Vec<_>>().into(),
        }
    }

    pub fn external_id(mut self, external_id: ExternalId) -> Self {
        self.external_id = Some(external_id);
        self
    }
}
