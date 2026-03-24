use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

pub(super) fn preview_ai_conversation_demo(cx: &mut UiCx<'_>, _theme: &Theme) -> Vec<AnyElement> {
    let demo = snippets::conversation_demo::render(cx);
    let features = doc_layout::notes_block([
        "Automatic stick-to-bottom behavior when new messages arrive and the reader is already near the tail.",
        "Dedicated `ConversationContent` slot keeps transcript spacing/padding aligned with the official AI Elements defaults (`gap-8`, `p-4`).",
        "Overlay parts stay composable: `ConversationDownload` anchors to the top-right and `ConversationScrollButton` appears only when the transcript is away from the tail.",
        "Transcript export remains app-owned in Fret: pair `ConversationDownload` with an action/effect and `messages_to_markdown` for the pure formatter.",
        "The UI Gallery example leaves stable `test_id` hooks behind so the existing diag scripts can keep the conversation surface reproducible.",
    ]);
    let composable_children = doc_layout::notes_block([
        "`Conversation`, `ConversationContent`, and `ConversationEmptyState` already support direct compound children composition.",
        "`ConversationDownload` and `ConversationScrollButton` now both accept composed child content, which keeps custom icon/label affordances closer to the official React children surface.",
        "Use `into_element_with_children(...)` when move-only Fret trees are easier to assemble lazily inside a live element scope.",
    ]);
    let export_notes = doc_layout::notes_block([
        "Official AI Elements downloads a Markdown file directly from the component.",
        "Fret intentionally stops at the pure formatting boundary: `messages_to_markdown` returns the transcript text, while the host app decides whether to save, copy, or share it.",
        "The gallery demo wires `ConversationDownload` through an action to keep the component policy layer portable across native and web hosts.",
    ]);
    let parity_notes = doc_layout::notes_block([
        "Behavior parity is good on the main surface: existing gallery diagnostics cover the empty state screenshot and prompt-send flow.",
        "The conversation root now maps to `SemanticsRole::Log`, so the upstream `role=\"log\"` outcome is represented in both diagnostics and AccessKit.",
        "Because Fret is GPU-first and cross-platform, DOM-style `...props` passthrough is translated into explicit builder methods (`refine_layout`, `test_id`, action hooks) instead of raw HTML attribute forwarding.",
    ]);
    let props = conversation_props_table(cx).test_id("ui-gallery-ai-conversation-props");
    let usage_section = DocSection::build(cx, "Usage with AI SDK", demo)
        .description("Docs-aligned Fret translation of the official AI Elements conversation example.")
        .description(
            "The live preview keeps transcript export app-owned: `ConversationDownload` emits an action, and the gallery harness formats the markdown with `messages_to_markdown`.",
        )
        .test_id_prefix("ui-gallery-ai-conversation-demo")
        .max_w(Px(980.0))
        .code_rust_from_file_region(snippets::conversation_demo::SOURCE, "example");
    let features_section = DocSection::build(cx, "Features", features)
        .description("Behavior and layering outcomes to preserve while aligning against the upstream docs surface.")
        .max_w(Px(980.0))
        .no_shell();
    let composable_section = DocSection::build(cx, "Composable Children", composable_children)
        .description("Compound children composition is supported on the core conversation slots and overlay affordances.")
        .max_w(Px(980.0))
        .code_rust_from_file_region(snippets::conversation_demo::SOURCE, "custom_scroll_button")
        .no_shell();
    let export_section = DocSection::build(cx, "messages_to_markdown", export_notes)
        .description("Fret keeps transcript export pure and portable instead of embedding file IO directly inside the component.")
        .max_w(Px(980.0))
        .code_rust_from_file_region(
            snippets::conversation_demo::SOURCE,
            "messages_to_markdown_export",
        )
        .no_shell();
    let props_section = DocSection::build(cx, "Builder Surface", props)
        .description(
            "Fret builder methods corresponding to the official `Conversation*` component family.",
        )
        .max_w(Px(980.0));
    let parity_section = DocSection::build(cx, "Parity Notes", parity_notes)
        .description(
            "What is aligned today versus what still belongs to the semantics/runtime layer.",
        )
        .max_w(Px(980.0))
        .no_shell();

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "The `Conversation` component wraps messages and automatically scrolls to the bottom. It also exposes overlay slots for transcript export and a scroll-to-latest affordance when the reader is away from the tail.",
        ),
        vec![
            usage_section,
            features_section,
            composable_section,
            export_section,
            props_section,
            parity_section,
        ],
    );

    vec![body.into_element(cx)]
}

fn conversation_props_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    doc_layout::text_table(
        cx,
        ["Part", "Method", "Type", "Default", "Description"],
        [
            [
                "Conversation",
                "new(children) / children",
                "IntoIterator<Item = AnyElement>",
                "-",
                "Composes direct transcript or overlay children on the root conversation surface.",
            ],
            [
                "Conversation",
                "into_element_with_children",
                "FnOnce(&mut ElementContext) -> Vec<AnyElement>",
                "-",
                "Builds move-only children lazily inside a live element scope.",
            ],
            [
                "Conversation",
                "content_revision",
                "u64",
                "0",
                "Marks when new transcript content arrived so stick-to-bottom only advances when the reader was already near the tail.",
            ],
            [
                "Conversation",
                "content_padding / content_gap",
                "Space",
                "N4 / N8",
                "Matches the upstream `p-4` and `gap-8` defaults on the managed content lane.",
            ],
            [
                "Conversation",
                "stick_to_bottom / stick_threshold",
                "bool / Px",
                "true / 8px",
                "Controls tail-follow behavior and the threshold used to decide whether the reader is effectively at the bottom.",
            ],
            [
                "Conversation",
                "show_scroll_to_bottom_button / scroll_handle / test_id",
                "builder methods",
                "true / None / None",
                "Configures fallback scroll affordance visibility, a custom scroll handle, and diagnostics hooks.",
            ],
            [
                "ConversationContent",
                "new(children)",
                "IntoIterator<Item = AnyElement>",
                "-",
                "Primary transcript slot aligned with AI Elements `ConversationContent`.",
            ],
            [
                "ConversationContent",
                "refine_layout / content_padding / content_gap / test_id",
                "builder methods",
                "w_full + min_w_0 / N4 / N8 / None",
                "Adjusts layout and diagnostics hooks without re-implementing the content slot recipe.",
            ],
            [
                "ConversationEmptyState",
                "title / description / icon",
                "builder methods",
                "\"No messages yet\" / docs copy / None",
                "Mirrors the official empty-state API while keeping the centered conversation recipe.",
            ],
            [
                "ConversationEmptyState",
                "children",
                "IntoIterator<Item = AnyElement>",
                "None",
                "Replaces the default empty-state copy block with fully composed custom content.",
            ],
            [
                "ConversationDownload",
                "action / action_payload / on_activate",
                "builder methods",
                "None",
                "Keeps export effects app-owned while the component stays a portable conversation affordance.",
            ],
            [
                "ConversationDownload",
                "children / show_label / disabled / test_id",
                "builder methods",
                "None / false / false / None",
                "Supports custom visible content, a labeled button mode, disabled state, and diagnostics ids.",
            ],
            [
                "ConversationScrollButton",
                "children / into_element_with_children",
                "builder methods",
                "None",
                "Lets apps swap the default chevron for composed icon/label content without re-implementing the scroll-to-latest behavior.",
            ],
            [
                "ConversationScrollButton",
                "label / threshold / test_id / refine_layout",
                "builder methods",
                "\"Scroll to bottom\" / 8px / None / default",
                "Controls assistive label copy, visibility threshold, diagnostics ids, and overlay positioning refinements.",
            ],
            [
                "Export Utility",
                "messages_to_markdown",
                "&[AiMessage] -> String",
                "-",
                "Pure transcript formatter that replaces the upstream DOM download side effect in Fret's app-owned export flow.",
            ],
        ],
        true,
    )
}
