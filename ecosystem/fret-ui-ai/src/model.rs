use std::sync::Arc;

/// Stable identifier for a message within a transcript.
///
/// This is aligned with `crates/fret-ui::ItemKey` so virtualized transcripts can use it directly
/// as a keyed identity.
pub type MessageId = fret_ui::ItemKey;

/// Derive a deterministic `MessageId` from a stable external identifier.
///
/// This uses a small, deterministic 64-bit hash (FNV-1a) so apps can interop with upstream SDKs
/// that provide string IDs (UUID/nanoid/etc) while keeping the UI keyed identity as a `u64`.
///
/// Notes:
///
/// - IDs only need to be unique within a single transcript.
/// - Hash collisions are possible (as with any hash). If you cannot tolerate collisions, keep a
///   per-conversation `HashMap<ExternalId, MessageId>` and assign monotonic IDs.
pub fn message_id_from_external_id(external_id: &str) -> MessageId {
    message_id_from_salted_external_id(0, external_id)
}

/// Derive a deterministic `MessageId` from a stable external ID with an extra salt.
///
/// The salt can be a conversation/session ID hash to further reduce collision risk across merged
/// transcripts.
pub fn message_id_from_salted_external_id(salt: u64, external_id: &str) -> MessageId {
    fnv1a64_u64_and_bytes(salt, external_id.as_bytes())
}

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
    /// Awaiting user approval/confirmation before running.
    ApprovalRequested,
    /// Approval was responded to (accepted/denied), but a concrete output is not yet available.
    ApprovalResponded,
    /// Tool input is available and the tool is considered running.
    InputAvailable,
    /// Tool input is still streaming / not fully available yet.
    InputStreaming,
    /// Tool output is available and completed successfully.
    OutputAvailable,
    /// Tool output was denied (e.g. user rejected) or the call was cancelled.
    OutputDenied,
    /// Tool output ended in error.
    OutputError,
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

/// An inline citation reference (typically rendered near assistant output).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationItem {
    /// Source identifiers referenced by this citation.
    ///
    /// Upstream AI Elements supports multiple sources per inline citation.
    pub source_ids: Arc<[Arc<str>]>,
    pub label: Arc<str>,
}

impl CitationItem {
    pub fn new(source_id: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            source_ids: vec![source_id.into()].into(),
            label: label.into(),
        }
    }

    pub fn from_arc(source_ids: Arc<[Arc<str>]>, label: impl Into<Arc<str>>) -> Self {
        Self {
            source_ids,
            label: label.into(),
        }
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
            state: ToolCallState::InputStreaming,
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

/// Markdown content part (used for assistant output).
///
/// The `finalized` flag models streaming behavior:
///
/// - `finalized: false` means the part may still grow via append-only updates.
/// - `finalized: true` means the stream has ended and any deferred parsing/flush can run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownPart {
    pub text: Arc<str>,
    pub finalized: bool,
}

impl MarkdownPart {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            finalized: true,
        }
    }

    pub fn streaming(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            finalized: false,
        }
    }

    pub fn finalized(mut self, finalized: bool) -> Self {
        self.finalized = finalized;
        self
    }
}

/// A message content part (portable, UI-oriented).
#[derive(Debug, Clone, PartialEq)]
pub enum MessagePart {
    /// Plain text (typically used for user messages).
    Text(Arc<str>),
    /// Markdown content (typically used for assistant messages).
    Markdown(MarkdownPart),
    /// A structured tool call (input/output + lifecycle).
    ToolCall(ToolCall),
    /// A list of sources (citations, references).
    Sources(Arc<[SourceItem]>),
    /// A list of inline citations that reference a `Sources` part.
    Citations(Arc<[CitationItem]>),
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

fn fnv1a64_u64_and_bytes(prefix: u64, bytes: &[u8]) -> u64 {
    // Keep in sync with `crates/fret-ui/src/elements/hash.rs` (FNV-1a 64).
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    let mut hash = OFFSET_BASIS;
    for b in prefix.to_le_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_id_from_external_id_is_deterministic() {
        let a = message_id_from_external_id("msg_0001");
        let b = message_id_from_external_id("msg_0001");
        assert_eq!(a, b);
    }

    #[test]
    fn salted_and_unsalted_differ() {
        let unsalted = message_id_from_external_id("msg_0001");
        let salted = message_id_from_salted_external_id(42, "msg_0001");
        assert_ne!(unsalted, salted);
    }
}
