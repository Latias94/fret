//! AI/chat UI components built on `fret-ui-shadcn`.
//!
//! Reference implementation: `repo-ref/ai-elements/packages/elements`.

pub mod elements;
pub mod model;

pub use elements::{
    AiConversationTranscript, Conversation, ConversationMessage, ConversationTranscript,
    InlineCitation, Message, MessageParts, MessageResponse, PromptInput, SourcesBlock,
    ToolCallBlock,
};
pub use model::{
    AiMessage, ExternalId, MessageId, MessagePart, MessageRole, SourceItem, ToolCall,
    ToolCallPayload, ToolCallState,
};
