const WORKFLOW_CONTROLS_RS: &str = include_str!("elements/workflow/controls.rs");
const MESSAGE_ACTIONS_RS: &str = include_str!("elements/message_actions.rs");
const ARTIFACT_RS: &str = include_str!("elements/artifact.rs");
const CONFIRMATION_RS: &str = include_str!("elements/confirmation.rs");
const PROMPT_INPUT_RS: &str = include_str!("elements/prompt_input.rs");
const CHECKPOINT_RS: &str = include_str!("elements/checkpoint.rs");
const CONVERSATION_DOWNLOAD_RS: &str = include_str!("elements/conversation_download.rs");
const WEB_PREVIEW_RS: &str = include_str!("elements/web_preview.rs");
const SOURCES_RS: &str = include_str!("elements/sources.rs");
const CODE_BLOCK_RS: &str = include_str!("elements/code_block.rs");
const AGENT_RS: &str = include_str!("elements/agent.rs");
const MESSAGE_RESPONSE_RS: &str = include_str!("elements/message_response.rs");
const MESSAGE_PARTS_RS: &str = include_str!("elements/message_parts.rs");
const MESSAGE_BRANCH_RS: &str = include_str!("elements/message_branch.rs");
const AUDIO_PLAYER_RS: &str = include_str!("elements/audio_player.rs");
const ATTACHMENTS_RS: &str = include_str!("elements/attachments.rs");
const ARTIFACT_RS_AI: &str = include_str!("elements/artifact.rs");
const PROMPT_INPUT_RS_AI: &str = include_str!("elements/prompt_input.rs");
const REASONING_RS: &str = include_str!("elements/reasoning.rs");
const STACK_TRACE_RS: &str = include_str!("elements/stack_trace.rs");
const CHAIN_OF_THOUGHT_RS: &str = include_str!("elements/chain_of_thought.rs");
const OPEN_IN_CHAT_RS: &str = include_str!("elements/open_in_chat.rs");
const MESSAGE_ACTIONS_RS_AI: &str = include_str!("elements/message_actions.rs");
const SCHEMA_DISPLAY_RS: &str = include_str!("elements/schema_display.rs");
const SPEECH_INPUT_RS: &str = include_str!("elements/speech_input.rs");
const MIC_SELECTOR_RS: &str = include_str!("elements/mic_selector.rs");
const MODEL_SELECTOR_RS: &str = include_str!("elements/model_selector.rs");
const VOICE_SELECTOR_RS: &str = include_str!("elements/voice_selector.rs");
const CONTEXT_RS: &str = include_str!("elements/context.rs");
const FILE_TREE_RS: &str = include_str!("elements/file_tree.rs");
const CONVERSATION_RS: &str = include_str!("elements/conversation.rs");
const AI_CONVERSATION_RS: &str = include_str!("elements/ai_conversation.rs");
const AI_CHAT_RS: &str = include_str!("elements/ai_chat.rs");
const WEB_PREVIEW_RS_AI: &str = include_str!("elements/web_preview.rs");
const INLINE_CITATION_RS: &str = include_str!("elements/inline_citation.rs");

#[test]
fn default_facing_ai_action_wrappers_keep_native_action_first_aliases() {
    for (label, source, markers) in [
        (
            "workflow/controls.rs",
            WORKFLOW_CONTROLS_RS,
            &["Bind a stable action ID to this workflow controls button (action-first authoring)."]
                [..],
        ),
        (
            "message_actions.rs",
            MESSAGE_ACTIONS_RS,
            &["Bind a stable action ID to this message action (action-first authoring)."][..],
        ),
        (
            "artifact.rs",
            ARTIFACT_RS,
            &[
                "Bind a stable action ID to this artifact action (action-first authoring).",
                "Bind a stable action ID to this artifact close affordance (action-first authoring).",
            ][..],
        ),
        (
            "confirmation.rs",
            CONFIRMATION_RS,
            &["Bind a stable action ID to this confirmation action (action-first authoring)."][..],
        ),
        (
            "prompt_input.rs",
            PROMPT_INPUT_RS,
            &["Bind a stable action ID to this prompt-input button (action-first authoring)."][..],
        ),
        (
            "checkpoint.rs",
            CHECKPOINT_RS,
            &["Bind a stable action ID to this checkpoint trigger (action-first authoring)."][..],
        ),
        (
            "conversation_download.rs",
            CONVERSATION_DOWNLOAD_RS,
            &[
                "Bind a stable action ID to this conversation download control (action-first authoring).",
            ][..],
        ),
        (
            "web_preview.rs",
            WEB_PREVIEW_RS,
            &[
                "Bind a stable action ID to this web-preview navigation button (action-first authoring).",
            ][..],
        ),
    ] {
        for marker in markers {
            assert!(
                source.contains(marker),
                "{label} should document the native action-first builder alias"
            );
        }
        assert!(
            source.contains(
                "pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {"
            ),
            "{label} should expose a native action-first builder alias"
        );
    }
}

#[test]
fn prompt_input_public_surface_keeps_docs_shaped_children_and_select_lane() {
    for marker in [
        "pub struct PromptInputMessage {",
        "pub struct PromptInputBody {",
        "pub struct PromptInputChildren {",
        "pub enum PromptInputPart {",
        "pub enum PromptInputHeaderChild {",
        "pub enum PromptInputFooterChild {",
        "pub enum PromptInputToolsChild {",
        "pub fn children<I, P>(self, parts: I) -> PromptInputChildren",
        "pub fn empty() -> Self {",
        "pub fn child<T>(mut self, child: T) -> Self",
        "pub fn on_submit(mut self, on_submit: OnPromptInputSubmit) -> Self {",
        "pub fn item(mut self, item: PromptInputActionMenuItem) -> Self {",
        "pub fn add_attachments(mut self, item: PromptInputActionAddAttachments) -> Self {",
        "pub fn add_screenshot(mut self, item: PromptInputActionAddScreenshot) -> Self {",
        "pub struct PromptInputActionAddScreenshot {",
        "pub type PromptInputSelect = ShadcnSelect;",
        "pub struct PromptInputSelectTrigger {",
        "const DEFAULT_PROMPT_INPUT_PLACEHOLDER: &str = \"What would you like to know?\";",
    ] {
        assert!(
            PROMPT_INPUT_RS.contains(marker),
            "prompt_input.rs should keep docs-shaped public surface marker `{marker}`"
        );
    }
}

#[test]
fn sources_public_surface_keeps_docs_shaped_compound_and_custom_children() {
    for marker in [
        "pub struct Sources {",
        "pub fn into_element_parts<H: UiHost>(",
        "pub struct SourcesTrigger {",
        "pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {",
        "pub struct SourcesContent {",
        "pub struct Source {",
        "pub fn with_open_url(mut self) -> Self {",
    ] {
        assert!(
            SOURCES_RS.contains(marker),
            "sources.rs should keep docs-shaped public surface marker `{marker}`"
        );
    }
}

#[test]
fn code_block_public_surface_keeps_docs_shaped_header_and_copy_parts() {
    for marker in [
        "pub struct CodeBlock {",
        "pub fn windowed(mut self, windowed: fret_code_view::CodeBlockWindowedOptions) -> Self {",
        "pub fn into_element_with_children<H: UiHost>(",
        "pub fn into_element_with_children_windowed<H: UiHost + 'static>(",
        "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {",
        "pub fn into_element_windowed<H: UiHost + 'static>(",
        "pub fn into_element_with_children_non_windowed<H: UiHost>(",
        "pub fn into_element_non_windowed<H: UiHost>(",
        "pub struct CodeBlockHeader {",
        "pub struct CodeBlockTitle {",
        "pub struct CodeBlockActions {",
        "pub struct CodeBlockCopyButton {",
        "pub type CodeBlockLanguageSelector = ShadcnSelect;",
        "pub struct CodeBlockLanguageSelectorContent {",
        "pub struct CodeBlockLanguageSelectorTrigger {",
        "pub type CodeBlockLanguageSelectorItem = ShadcnSelectItem;",
        "pub type CodeBlockLanguageSelectorValue = ShadcnSelectValue;",
    ] {
        assert!(
            CODE_BLOCK_RS.contains(marker),
            "code_block.rs should keep docs-shaped public surface marker `{marker}`"
        );
    }
}

#[test]
fn inline_citation_public_surface_keeps_docs_shaped_compound_and_custom_children() {
    for marker in [
        "pub struct InlineCitationRoot {",
        "pub fn into_element_parts<H, TText, TCard>(",
        "pub fn into_element_with_children<H: UiHost>(",
        "pub struct InlineCitationText {",
        "pub struct InlineCitationCard {",
        "pub struct InlineCitationWithChildren {",
        "pub fn with_children<I>(children: I) -> InlineCitationWithChildren",
        "pub fn children<I>(self, children: I) -> InlineCitationWithChildren",
    ] {
        assert!(
            INLINE_CITATION_RS.contains(marker),
            "inline_citation.rs should keep docs-shaped public surface marker `{marker}`"
        );
    }
}

#[test]
fn file_tree_public_surface_keeps_docs_shaped_item_and_action_types() {
    for marker in [
        "pub struct FileTree {",
        "pub enum FileTreeItem {",
        "pub struct FileTreeFolder {",
        "pub struct FileTreeFile {",
        "pub struct FileTreeAction {",
        "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {",
    ] {
        assert!(
            FILE_TREE_RS.contains(marker),
            "file_tree.rs should keep docs-shaped public surface marker `{marker}`"
        );
    }
}

#[test]
fn agent_public_surface_keeps_docs_shaped_children_lane() {
    for marker in [
        "pub struct Agent {",
        "pub fn empty() -> Self {",
        "pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {",
        "pub struct AgentContent {",
        "pub struct AgentTools {",
        "pub fn children(mut self, children: impl IntoIterator<Item = AgentTool>) -> Self {",
        "pub fn multiple_uncontrolled(items: impl IntoIterator<Item = AccordionItem>) -> Self {",
    ] {
        assert!(
            AGENT_RS.contains(marker),
            "agent.rs should keep docs-shaped public surface marker `{marker}`"
        );
    }
}

#[test]
fn message_branch_surface_does_not_require_static_host() {
    assert!(
        !MESSAGE_BRANCH_RS.contains("UiHost + 'static"),
        "message_branch.rs should not require `H: UiHost + 'static` on non-retained surfaces"
    );
    assert!(
        MESSAGE_BRANCH_RS.contains(
            "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {"
        ),
        "message_branch.rs should keep non-static host entry points"
    );
}

#[test]
fn audio_player_surface_does_not_require_static_host() {
    assert!(
        !AUDIO_PLAYER_RS.contains("UiHost + 'static"),
        "audio_player.rs should not require `H: UiHost + 'static` on provider, button, or slider surfaces"
    );
    assert!(
        AUDIO_PLAYER_RS.contains("pub fn into_element_with_children<H: UiHost>("),
        "audio_player.rs should keep the root children lane available to non-static hosts"
    );
}

#[test]
fn attachments_surface_does_not_require_static_host() {
    assert!(
        !ATTACHMENTS_RS.contains("UiHost + 'static"),
        "attachments.rs should not require `H: UiHost + 'static` on hover-region item or remove-button surfaces"
    );
    assert!(
        ATTACHMENTS_RS.contains("pub fn into_element_with_children<H: UiHost>("),
        "attachments.rs should keep the root children lane available to non-static hosts"
    );
}

#[test]
fn open_in_chat_surface_does_not_require_static_host() {
    assert!(
        !OPEN_IN_CHAT_RS.contains("UiHost + 'static"),
        "open_in_chat.rs should not require `H: UiHost + 'static` on dropdown-menu trigger or item-entry surfaces"
    );
    assert!(
        OPEN_IN_CHAT_RS.contains("pub fn into_element_with_entries<H: UiHost>("),
        "open_in_chat.rs should keep the menu entries lane available to non-static hosts"
    );
}

#[test]
fn mic_selector_surface_does_not_require_static_host() {
    assert!(
        !MIC_SELECTOR_RS.contains("UiHost + 'static"),
        "mic_selector.rs should not require `H: UiHost + 'static` on popover, trigger, or command-list surfaces"
    );
    assert!(
        MIC_SELECTOR_RS.contains("pub fn into_element_with_children<H, F>("),
        "mic_selector.rs should keep the compound children lane"
    );
}

#[test]
fn artifact_surface_does_not_require_static_host() {
    assert!(
        !ARTIFACT_RS_AI.contains("UiHost + 'static"),
        "artifact.rs should not require `H: UiHost + 'static` on close-button surfaces"
    );
    assert!(
        ARTIFACT_RS_AI.contains(
            "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {"
        ),
        "artifact.rs should keep non-static host entry points"
    );
}

#[test]
fn message_actions_surface_does_not_require_static_host() {
    assert!(
        !MESSAGE_ACTIONS_RS_AI.contains("UiHost + 'static"),
        "message_actions.rs should not require `H: UiHost + 'static` on tooltip action surfaces"
    );
    assert!(
        MESSAGE_ACTIONS_RS_AI.contains(
            "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {"
        ),
        "message_actions.rs should keep non-static host entry points"
    );
}

#[test]
fn prompt_input_surface_does_not_require_static_host() {
    assert!(
        !PROMPT_INPUT_RS_AI.contains("UiHost + 'static"),
        "prompt_input.rs should not require `H: UiHost + 'static` on provider, dropdown, tooltip, or prompt-button surfaces"
    );
    assert!(
        PROMPT_INPUT_RS_AI.contains("pub fn into_element_with_slots<H: UiHost>("),
        "prompt_input.rs should keep the slot-based root lane available to non-static hosts"
    );
}

#[test]
fn message_response_surface_does_not_require_static_host() {
    assert!(
        !MESSAGE_RESPONSE_RS.contains("UiHost + 'static"),
        "message_response.rs should use the non-windowed markdown lane and avoid static-host bounds"
    );
    for marker in [
        "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {",
        "fret_markdown::MarkdownComponents::<H>::default()",
        "fret_markdown::markdown_streaming_pulldown_with_non_windowed(",
        "into_element_with_non_windowed(cx, &components)",
    ] {
        assert!(
            MESSAGE_RESPONSE_RS.contains(marker),
            "message_response.rs should keep non-windowed markdown evidence marker `{marker}`"
        );
    }
}

#[test]
fn message_parts_surface_does_not_require_static_host() {
    assert!(
        !MESSAGE_PARTS_RS.contains("UiHost + 'static"),
        "message_parts.rs should keep rich-message composition available to non-static hosts"
    );
    for marker in [
        "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {",
        "MessagePart::Markdown(md) => {",
        "let mut response =",
        "MessageResponse::new(md.text.clone()).finalized(md.finalized);",
    ] {
        assert!(
            MESSAGE_PARTS_RS.contains(marker),
            "message_parts.rs should keep markdown delegation marker `{marker}`"
        );
    }
}

#[test]
fn reasoning_surface_does_not_require_static_host_for_markdown_content_lane() {
    assert!(
        REASONING_RS.contains(
            "pub fn into_element<H: UiHost>(\n        self,\n        cx: &mut ElementContext<'_, H>,\n        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,\n        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,\n    ) -> AnyElement {"
        ),
        "reasoning.rs should keep the lower-level root lane available to non-static hosts"
    );
    assert!(
        !REASONING_RS.contains("UiHost + 'static"),
        "reasoning.rs should avoid static-host bounds once reasoning content uses non-windowed markdown"
    );
    assert!(
        REASONING_RS.contains(".into_element_with_non_windowed(cx, &components)"),
        "reasoning.rs should keep non-windowed markdown evidence in ReasoningContent"
    );
}

#[test]
fn code_block_surface_only_keeps_static_host_where_windowed_code_view_still_requires_it() {
    assert!(
        CODE_BLOCK_RS.matches("UiHost + 'static").count() == 2,
        "code_block.rs should only keep two static-host occurrences on the explicit windowed lanes"
    );
    for marker in [
        "pub fn into_element_with_children<H: UiHost>(",
        "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {",
        "pub fn into_element_with_children_non_windowed<H: UiHost>(",
        "pub fn into_element_non_windowed<H: UiHost>(",
        "pub fn into_element_with_children_windowed<H: UiHost + 'static>(",
        "pub fn into_element_windowed<H: UiHost + 'static>(",
    ] {
        assert!(
            CODE_BLOCK_RS.contains(marker),
            "code_block.rs should keep explicit marker `{marker}` on the correct lane"
        );
    }
}

#[test]
fn stack_trace_surface_does_not_require_static_host() {
    assert!(
        !STACK_TRACE_RS.contains("UiHost + 'static"),
        "stack_trace.rs should not require `H: UiHost + 'static` on copy/header/content/frame surfaces"
    );
    assert!(
        STACK_TRACE_RS.contains("pub fn into_element_with_children<H: UiHost>("),
        "stack_trace.rs should keep the root children lane available to non-static hosts"
    );
}

#[test]
fn chain_of_thought_surface_does_not_require_static_host() {
    assert!(
        !CHAIN_OF_THOUGHT_RS.contains("UiHost + 'static"),
        "chain_of_thought.rs should not require `H: UiHost + 'static` on collapsible surfaces"
    );
    assert!(
        CHAIN_OF_THOUGHT_RS.contains("pub fn into_element_with_children<H: UiHost>("),
        "chain_of_thought.rs should keep the root children lane available to non-static hosts"
    );
}

#[test]
fn schema_display_surface_does_not_require_static_host() {
    assert!(
        !SCHEMA_DISPLAY_RS.contains("UiHost + 'static"),
        "schema_display.rs should not require `H: UiHost + 'static` on collapsible section surfaces"
    );
    assert!(
        SCHEMA_DISPLAY_RS.contains(
            "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {"
        ),
        "schema_display.rs should keep non-static host entry points"
    );
}

#[test]
fn speech_input_surface_does_not_require_static_host() {
    assert!(
        !SPEECH_INPUT_RS.contains("UiHost + 'static"),
        "speech_input.rs should not require `H: UiHost + 'static` on button or pulse-ring surfaces"
    );
    assert!(
        SPEECH_INPUT_RS.contains(
            "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {"
        ),
        "speech_input.rs should keep a non-static host entry point"
    );
}

#[test]
fn model_selector_surface_does_not_require_static_host() {
    assert!(
        !MODEL_SELECTOR_RS.contains("UiHost + 'static"),
        "model_selector.rs should not require `H: UiHost + 'static` on dialog composition surfaces"
    );
    assert!(
        MODEL_SELECTOR_RS.contains("pub fn into_element_with_children<H, F>("),
        "model_selector.rs should keep the compound children lane"
    );
}

#[test]
fn voice_selector_surface_does_not_require_static_host() {
    assert!(
        !VOICE_SELECTOR_RS.contains("UiHost + 'static"),
        "voice_selector.rs should not require `H: UiHost + 'static` on dialog, command-list, or preview button surfaces"
    );
    assert!(
        VOICE_SELECTOR_RS.contains("pub fn into_element_with_children<H, F>("),
        "voice_selector.rs should keep the compound children lane"
    );
}

#[test]
fn context_surface_does_not_require_static_host() {
    assert!(
        !CONTEXT_RS.contains("UiHost + 'static"),
        "context.rs should not require `H: UiHost + 'static` on hover-card root or slot-resolution surfaces"
    );
    assert!(
        CONTEXT_RS.contains("pub fn into_element_with_children<H: UiHost>("),
        "context.rs should keep the root children lane available to non-static hosts"
    );
}

#[test]
fn conversation_surface_does_not_require_static_host() {
    assert!(
        CONVERSATION_RS.contains("pub fn into_element_with_children<H: UiHost>("),
        "conversation.rs should keep the arbitrary-children root lane available to non-static hosts"
    );
    assert!(
        CONVERSATION_RS.contains(
            "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {"
        ),
        "conversation.rs should keep the transcript entry point available to non-static hosts"
    );
    assert!(
        !CONVERSATION_RS.contains("UiHost + 'static"),
        "conversation.rs should not require a static host after switching to non-retained virtualization"
    );
    assert!(
        CONVERSATION_RS.contains("cx.virtual_list_keyed_with_layout("),
        "conversation.rs should keep explicit non-retained virtualization evidence"
    );
}

#[test]
fn ai_conversation_surface_does_not_require_static_host() {
    assert!(
        !AI_CONVERSATION_RS.contains("UiHost + 'static"),
        "ai_conversation.rs should keep transcript composition available to non-static hosts"
    );
    for marker in [
        "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {",
        "cx.virtual_list_keyed_with_layout(",
        "let mut bubble = MessageParts::new(msg.role, msg.parts.clone());",
    ] {
        assert!(
            AI_CONVERSATION_RS.contains(marker),
            "ai_conversation.rs should keep transcript evidence marker `{marker}`"
        );
    }
}

#[test]
fn ai_chat_surface_does_not_require_static_host() {
    assert!(
        !AI_CHAT_RS.contains("UiHost + 'static"),
        "ai_chat.rs should keep chat composition available to non-static hosts"
    );
    for marker in [
        "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {",
        "AiConversationTranscript::from_arc(messages_value.clone())",
        "Conversation::new([])",
    ] {
        assert!(
            AI_CHAT_RS.contains(marker),
            "ai_chat.rs should keep transcript delegation evidence marker `{marker}`"
        );
    }
}

#[test]
fn file_tree_surface_does_not_require_static_host() {
    assert!(
        FILE_TREE_RS.contains(
            "pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {"
        ),
        "file_tree.rs should keep the tree entry point available to non-static hosts"
    );
    assert!(
        !FILE_TREE_RS.contains("UiHost + 'static"),
        "file_tree.rs should not require a static host after switching to non-retained virtualization"
    );
    for marker in [
        "cx.virtual_list_keyed_with_layout(",
        "fn render_actions<H: UiHost>(",
        "fn file_tree_indent_el<H: UiHost>(",
        "fn render_folder_row<H: UiHost>(",
        "fn render_file_row<H: UiHost>(",
    ] {
        assert!(
            FILE_TREE_RS.contains(marker),
            "file_tree.rs should keep helper surface `{marker}` non-static"
        );
    }
}

#[test]
fn inline_citation_surface_does_not_require_static_host() {
    assert!(
        !INLINE_CITATION_RS.contains("UiHost + 'static"),
        "inline_citation.rs should not require `H: UiHost + 'static` on hover-card, compound-parts, or children surfaces"
    );
    assert!(
        INLINE_CITATION_RS.contains("pub fn into_element_with_children<H: UiHost>("),
        "inline_citation.rs should keep the root children lane available to non-static hosts"
    );
}

#[test]
fn web_preview_surface_does_not_require_static_host() {
    assert!(
        !WEB_PREVIEW_RS_AI.contains("UiHost + 'static"),
        "web_preview.rs should not require `H: UiHost + 'static` on provider, url, or console surfaces"
    );
    assert!(
        WEB_PREVIEW_RS_AI.contains("pub fn into_element_with_children<H: UiHost>("),
        "web_preview.rs should keep the root children lane available to non-static hosts"
    );
}
