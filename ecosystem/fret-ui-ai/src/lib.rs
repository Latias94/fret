//! AI/chat UI components built on `fret-ui-shadcn`.
//!
//! Reference implementation: `repo-ref/ai-elements/packages/elements`.

pub mod elements;
pub mod model;

pub use elements::{
    AiConversationTranscript, Conversation, ConversationDownload, ConversationEmptyState,
    ConversationMessage, ConversationScrollButton, ConversationTranscript, InlineCitation, Message,
    MessageParts, MessageResponse, MessageToolbar, PromptInput, SourcesBlock, ToolCallBlock,
};
pub use model::{
    AiMessage, CitationItem, ExternalId, MarkdownPart, MessageId, MessagePart, MessageRole,
    SourceItem, ToolCall, ToolCallPayload, ToolCallState,
};
