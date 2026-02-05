//! AI/chat UI components built on `fret-ui-shadcn`.
//!
//! This crate ports the **AI Elements** component taxonomy (Vercel) into the Fret ecosystem:
//!
//! - `crates/fret-ui` stays mechanism-only.
//! - `ecosystem/fret-ui-shadcn` provides the baseline shadcn v4 taxonomy/recipes.
//! - `ecosystem/fret-ui-ai` provides AI-specific policy surfaces (chat transcripts, tool blocks,
//!   streaming markdown, citations).
//!
//! Reference implementation (local snapshot): `repo-ref/ai-elements/packages/elements`.
//!
//! ## Quick start
//!
//! Render a parts-based transcript (`AiConversationTranscript`) and a prompt composer (`PromptInput`)
//! in your app shell:
//!
//! ```rust
//! use std::sync::Arc;
//!
//! use fret_runtime::Model;
//! use fret_ui::scroll::VirtualListScrollHandle;
//! use fret_ui_ai::{AiConversationTranscript, AiMessage, MessagePart, MessageRole, PromptInput};
//!
//! fn build_ui(cx: &mut fret_ui::ElementContext<'_, impl fret_ui::UiHost>) {
//!     let prompt: Model<String> = cx.app.models_mut().insert(String::new());
//!     let messages: Arc<[AiMessage]> = Arc::from(vec![
//!         AiMessage::new(1, MessageRole::User, [MessagePart::Text(Arc::<str>::from("Hello"))]),
//!         AiMessage::new(
//!             2,
//!             MessageRole::Assistant,
//!             [MessagePart::Markdown(fret_ui_ai::MarkdownPart::new(Arc::<str>::from(
//!                 "Hi! ```rust\nfn demo() {}\n```",
//!             )))],
//!         ),
//!     ]);
//!
//!     let scroll_handle = VirtualListScrollHandle::new();
//!     let _transcript = AiConversationTranscript::from_arc(messages)
//!         .scroll_handle(scroll_handle.clone())
//!         .test_id_message_prefix("ui-ai-msg-")
//!         .into_element(cx);
//!
//!     let _input = PromptInput::new(prompt)
//!         .test_id_textarea("my-chat-prompt-textarea")
//!         .into_element(cx);
//! }
//! ```
//!
//! ## Streaming markdown
//!
//! Use `MessagePart::Markdown(MarkdownPart { text, finalized })`:
//!
//! - While streaming: keep `finalized = false` and append to `text`.
//! - When done: set `finalized = true` so markdown can flush pending blocks (e.g. unterminated
//!   code fences) deterministically.
//!
//! ## Export
//!
//! Convert a transcript to Markdown for “download/copy” flows (effects are app-owned):
//!
//! ```rust
//! use fret_ui_ai::messages_to_markdown;
//! # use fret_ui_ai::AiMessage;
//! # let messages: Vec<AiMessage> = Vec::new();
//! let md = messages_to_markdown(&messages);
//! assert!(md.is_empty() || md.contains("##"));
//! ```

pub mod elements;
pub mod export;
pub mod model;

pub use elements::{
    AiChat, AiConversationTranscript, Conversation, ConversationDownload, ConversationEmptyState,
    ConversationMessage, ConversationScrollButton, ConversationTranscript, InlineCitation, Message,
    MessageParts, MessageResponse, MessageToolbar, PromptInput, SourcesBlock, Tool, ToolCallBlock,
    ToolContent, ToolHeader, ToolInput, ToolOutput, ToolSectionTitle, ToolStatus,
};
pub use export::messages_to_markdown;
pub use model::{
    AiMessage, CitationItem, ExternalId, MarkdownPart, MessageId, MessagePart, MessageRole,
    SourceItem, ToolCall, ToolCallPayload, ToolCallState,
};
