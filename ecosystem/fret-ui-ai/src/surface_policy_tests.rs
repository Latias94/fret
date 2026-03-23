const WORKFLOW_CONTROLS_RS: &str = include_str!("elements/workflow/controls.rs");
const MESSAGE_ACTIONS_RS: &str = include_str!("elements/message_actions.rs");
const ARTIFACT_RS: &str = include_str!("elements/artifact.rs");
const CONFIRMATION_RS: &str = include_str!("elements/confirmation.rs");
const PROMPT_INPUT_RS: &str = include_str!("elements/prompt_input.rs");
const CHECKPOINT_RS: &str = include_str!("elements/checkpoint.rs");
const CONVERSATION_DOWNLOAD_RS: &str = include_str!("elements/conversation_download.rs");
const WEB_PREVIEW_RS: &str = include_str!("elements/web_preview.rs");

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
        "pub fn children<I, P>(self, parts: I) -> PromptInputChildren",
        "pub fn on_submit(mut self, on_submit: OnPromptInputSubmit) -> Self {",
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
