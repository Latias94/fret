//! AI/chat UI components built on `fret-ui-shadcn`.
//!
//! Reference implementation: `repo-ref/ai-elements/packages/elements`.

pub mod elements;
pub mod model;

pub use elements::{
    Conversation, ConversationMessage, ConversationTranscript, Message, MessageResponse,
};
pub use model::{
    AiMessage, ExternalId, MessageId, MessagePart, MessageRole, SourceItem, ToolCall,
    ToolCallPayload, ToolCallState,
};
