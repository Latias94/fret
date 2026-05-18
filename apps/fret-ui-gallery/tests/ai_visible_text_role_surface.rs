fn canonicalize_rust_fragment(fragment: &str) -> String {
    fragment.split_whitespace().collect()
}

#[test]
fn ai_message_demo_visible_text_uses_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/message_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_control_readout(cx, format!(\"last_action={last_action}\"))",
        "decl_text::text_paragraph",
        "\"User messages render as a bubble aligned to the right.\"",
        "decl_text::text_paragraph(cx, \"Bubble chrome is controlled by theme tokens.\")",
        "decl_text::text_section_chrome_label",
        "\"Message (AI Elements): alignment + bubble + actions + markdown response.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "message_demo should route fixed visible text through shared text roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(format!(\"last_action={last_action}\"))",
        "cx.text(\"User messages render as a bubble aligned to the right.\")",
        "cx.text(\"Bubble chrome is controlled by theme tokens.\")",
        "cx.text(\"Message (AI Elements): alignment + bubble + actions + markdown response.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "message_demo reintroduced visible bare text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_terminal_demo_visible_text_and_state_marker_use_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/terminal_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement",
        "role: fret_core::SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "decl_text::text_section_chrome_label(cx, \"Terminal (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Chrome-only viewer: apps own streaming + clear behavior.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "terminal_demo should route fixed visible text and state markers through shared non-bare roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "role: fret_core::SemanticsRole::Text",
        "cx.text(\"Terminal (AI Elements)\")",
        "cx.text(\"Chrome-only viewer: apps own streaming + clear behavior.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "terminal_demo reintroduced bare visible text/state marker semantics: `{forbidden}`"
        );
    }
}

#[test]
fn ai_artifact_demo_visible_text_uses_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/artifact_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_paragraph",
        "\"Artifacts are chrome-only: apps own rendering, export, and lifecycle.\"",
        "decl_text::text_control_readout(cx, \"Artifact closed.\")",
        "decl_text::text_section_chrome_label(cx, \"Artifact (AI Elements)\")",
        "decl_text::text_paragraph(cx, \"Close hides the artifact; reset re-mounts it.\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "artifact_demo should route visible text through shared roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(\"Artifacts are chrome-only: apps own rendering, export, and lifecycle.\")",
        "cx.text(\"Artifact closed.\")",
        "cx.text(\"Artifact (AI Elements)\")",
        "cx.text(\"Close hides the artifact; reset re-mounts it.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "artifact_demo reintroduced visible bare text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_code_block_demo_visible_text_and_state_marker_use_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/code_block_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: String) -> AnyElement",
        "role: fret_core::SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "decl_text::text_section_chrome_label(cx, \"CodeBlock (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Composable header/title/actions composition aligned with the official AI Elements language-selector example.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "code_block_demo should route visible text and state markers through shared non-bare roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(\"\")",
        "cx.text(\"CodeBlock (AI Elements)\")",
        "cx.text(\"Composable header/title/actions composition aligned with the official AI Elements language-selector example.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "code_block_demo reintroduced bare visible text/state marker text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_sandbox_demo_visible_text_uses_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/sandbox_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_section_chrome_label(cx, \"Sandbox console output (demo).\")",
        "decl_text::text_paragraph(cx, \"Apps own execution backends; this is UI-only.\")",
        "decl_text::text_section_chrome_label(cx, \"Sandbox files view (demo).\")",
        "decl_text::text_paragraph(cx, \"Tabs are composable; provide your own panels.\")",
        "decl_text::text_section_chrome_label(cx, \"Sandbox (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Collapsible + tabs chrome. Apps own the sandbox backend.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "sandbox_demo should route visible text through shared text roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(\"Sandbox console output (demo).\")",
        "cx.text(\"Apps own execution backends; this is UI-only.\")",
        "cx.text(\"Sandbox files view (demo).\")",
        "cx.text(\"Tabs are composable; provide your own panels.\")",
        "cx.text(\"Sandbox (AI Elements)\")",
        "cx.text(\"Collapsible + tabs chrome. Apps own the sandbox backend.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "sandbox_demo reintroduced visible bare text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_queue_demo_visible_text_and_state_marker_use_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/queue_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement",
        "role: fret_core::SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "decl_text::text_section_chrome_label(cx, \"Queue (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Hover an item to reveal actions; actions increment a demo marker.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "queue_demo should route visible text and state markers through shared non-bare roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "unwrap_or_else(|| cx.text(\"\"))",
        "cx.text(\"Queue (AI Elements)\")",
        "cx.text(\"Hover an item to reveal actions; actions increment a demo marker.\")",
        "cx.container(fret_ui::element::ContainerProps::default()",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "queue_demo reintroduced bare visible text/state marker text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_checkpoint_demo_visible_text_uses_shared_roles() {
    let source = include_str!("../src/ui/snippets/ai/checkpoint_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_control_readout(cx, \"Preview restored to the checkpoint.\")",
        "decl_text::text_control_readout",
        "\"The preview currently shows the latest conversation state.\"",
        "decl_text::text_paragraph(cx, message.content)",
        "decl_text::text_button_label",
        "checkpoint.trigger_label",
        "decl_text::text_paragraph",
        "\"The `Checkpoint` component provides a way to mark specific points in a conversation history and restore the chat to that state.\"",
        "\"Docs-aligned composition: `Conversation` + `Message` + `Checkpoint`. Hover the trigger to preview the tooltip, then activate it to restore the conversation.\"",
        "decl_text::text_chrome_glyph(cx, \"⟲\")",
        "decl_text::text_chrome_glyph(cx, \"•\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "checkpoint_demo should route visible text through shared text roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(\"Preview restored to the checkpoint.\")",
        "cx.text(\"The preview currently shows the latest conversation state.\")",
        "cx.text(message.content)",
        "cx.text(checkpoint.trigger_label)",
        "cx.text(\"The `Checkpoint` component provides a way to mark specific points in a conversation history and restore the chat to that state.\")",
        "cx.text(\"Docs-aligned composition: `Conversation` + `Message` + `Checkpoint`. Hover the trigger to preview the tooltip, then activate it to restore the conversation.\")",
        "cx.text(\"⟲\")",
        "cx.text(\"•\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "checkpoint_demo reintroduced visible bare text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_simple_chrome_snippets_use_shared_title_and_paragraph_roles() {
    for (name, source, title, body) in [
        (
            "agent_demo",
            include_str!("../src/ui/snippets/ai/agent_demo.rs"),
            "Agent",
            "Composable agent chrome with model, instructions, expandable tool schemas, and structured output.",
        ),
        (
            "code_block_usage",
            include_str!("../src/ui/snippets/ai/code_block_usage.rs"),
            "CodeBlock usage",
            "Minimal compound-parts composition aligned with the official AI Elements usage block.",
        ),
        (
            "environment_variables_demo",
            include_str!("../src/ui/snippets/ai/environment_variables_demo.rs"),
            "Environment Variables (AI Elements)",
            "Toggle to reveal values; copy uses a clipboard effect.",
        ),
        (
            "open_in_chat_demo",
            include_str!("../src/ui/snippets/ai/open_in_chat_demo.rs"),
            "OpenIn (AI Elements)",
            "Selecting a provider emits Effect::OpenUrl (URLs match upstream).",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed visible title/body text through shared roles; missing `{marker}`"
            );
        }

        for forbidden in [
            format!("cx.text(\"{title}\")"),
            format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(&forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare fixed visible text: `{forbidden}`"
            );
        }
    }
}

#[test]
fn ai_selector_branch_snippets_use_shared_text_roles_and_non_text_markers() {
    for (name, source, title, body) in [
        (
            "message_branch_demo",
            include_str!("../src/ui/snippets/ai/message_branch_demo.rs"),
            "MessageBranch (AI Elements)",
            "Prev/Next cycles through branches; only active branch is mounted.",
        ),
        (
            "mic_selector_demo",
            include_str!("../src/ui/snippets/ai/mic_selector_demo.rs"),
            "MicSelector (AI Elements)",
            "Docs-shaped compound example with typed item rows. Device inventory + permissions stay app-owned.",
        ),
        (
            "model_selector_demo",
            include_str!("../src/ui/snippets/ai/model_selector_demo.rs"),
            "ModelSelector (AI Elements)",
            "Dialog + Command surfaces; selection is app-owned.",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "fn state_marker",
            "SemanticsRole::Generic",
            "cx.spacer(SpacerProps",
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed visible text and state markers through shared non-bare roles; missing `{marker}`"
            );
        }

        for forbidden in [
            "cx.text(\"\")",
            "role: fret_core::SemanticsRole::Text",
            &format!("cx.text(\"{title}\")"),
            &format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare visible text/state marker text: `{forbidden}`"
            );
        }
    }
}

#[test]
fn ai_prompt_and_plan_snippets_use_shared_outer_text_roles() {
    for (name, source, title, body) in [
        (
            "plan_demo",
            include_str!("../src/ui/snippets/ai/plan_demo.rs"),
            "Plan (AI Elements)",
            "Toggle the chevron button to expand/collapse.",
        ),
        (
            "prompt_input_action_menu_demo",
            include_str!("../src/ui/snippets/ai/prompt_input_action_menu_demo.rs"),
            "Prompt Input Action Menu (AI Elements)",
            "Use the + menu to add attachments.",
        ),
        (
            "prompt_input_tooltip_demo",
            include_str!("../src/ui/snippets/ai/prompt_input_tooltip_demo.rs"),
            "Prompt Input Button Tooltips (AI Elements)",
            "Hover the toolbar actions to preview a simple tooltip, a shortcut hint, and a bottom-positioned tooltip.",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed outer title/body text through shared roles; missing `{marker}`"
            );
        }

        for forbidden in [
            format!("cx.text(\"{title}\")"),
            format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(&forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare outer visible text: `{forbidden}`"
            );
        }
    }
}

#[test]
fn ai_commit_large_uses_shared_outer_text_roles_and_non_text_marker() {
    let source = include_str!("../src/ui/snippets/ai/commit_large_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement",
        "SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "opened_now.is_some().then(|| state_marker(cx, \"ui-ai-commit-large-opened-marker\"))",
        "decl_text::text_section_chrome_label(cx, \"Commit (Large)\")",
        "decl_text::text_paragraph",
        "\"Scroll-heavy surface for hit testing + viewport scrolling.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "commit_large_demo should route fixed visible text and state marker through shared non-bare roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "role: fret_core::SemanticsRole::Text",
        "cx.text(\"\")",
        "cx.text(\"Commit (Large)\")",
        "cx.text(\"Scroll-heavy surface for hit testing + viewport scrolling.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "commit_large_demo reintroduced bare visible text/state marker text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_large_stack_and_test_results_use_shared_text_roles_and_non_text_markers() {
    for (name, source, title, body, marker_id) in [
        (
            "stack_trace_large_demo",
            include_str!("../src/ui/snippets/ai/stack_trace_large_demo.rs"),
            "StackTrace (Large)",
            "Scroll in the frames viewport and click a file path.",
            "ui-ai-stack-trace-large-opened-marker",
        ),
        (
            "test_results_large_demo",
            include_str!("../src/ui/snippets/ai/test_results_large_demo.rs"),
            "Test Results Large (AI Elements)",
            "Scroll the page and click a deep row to set a marker.",
            "ui-ai-test-results-large-activated-marker",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "fn state_marker",
            "SemanticsRole::Generic",
            "cx.spacer(SpacerProps",
            marker_id,
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed visible text and state markers through shared non-bare roles; missing `{marker}`"
            );
        }

        for forbidden in [
            "role: fret_core::SemanticsRole::Text",
            "cx.text(\"\")",
            &format!("cx.text(\"{title}\")"),
            &format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare visible text/state marker text: `{forbidden}`"
            );
        }
    }
}

#[test]
fn ai_tool_and_suggestions_use_shared_text_roles_and_non_text_markers() {
    for (name, source, title, body, marker_id) in [
        (
            "tool_demo",
            include_str!("../src/ui/snippets/ai/tool_demo.rs"),
            "Tool (AI Elements)",
            "Docs-shaped compound composition with the four official Tool states.",
            "ui-ai-tool-demo-content-marker",
        ),
        (
            "suggestions_demo",
            include_str!("../src/ui/snippets/ai/suggestions_demo.rs"),
            "Suggestions (AI Elements)",
            "Suggestion pills emit intents; apps own prompt insertion.",
            "ui-ai-suggestions-clicked-marker",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative",
            "text as decl_text",
            "fn state_marker",
            "SemanticsRole::Generic",
            "cx.spacer(SpacerProps",
            marker_id,
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed visible text and state markers through shared non-bare roles; missing `{marker}`"
            );
        }

        for forbidden in [
            "role: SemanticsRole::Text",
            "role: fret_core::SemanticsRole::Text",
            "cx.text(\"\")",
            &format!("cx.text(\"{title}\")"),
            &format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare visible text/state marker text: `{forbidden}`"
            );
        }
    }

    let tool = canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/tool_demo.rs"));
    for section in [
        "Input Streaming (Pending)",
        "Input Available (Running)",
        "Output Available (Completed)",
        "Output Error",
    ] {
        let role_marker = canonicalize_rust_fragment(&format!(
            "decl_text::text_section_chrome_label(cx, \"{section}\")"
        ));
        assert!(
            tool.contains(&role_marker),
            "tool_demo should route `{section}` section labels through shared section chrome roles"
        );
        let bare_marker = canonicalize_rust_fragment(&format!("cx.text(\"{section}\")"));
        assert!(
            !tool.contains(&bare_marker),
            "tool_demo reintroduced bare section label text: `{bare_marker}`"
        );
    }
}

#[test]
fn ai_queue_prompt_input_and_transcription_use_shared_text_roles_and_non_text_markers() {
    for (name, source, title, body, marker_id) in [
        (
            "queue_prompt_input_demo",
            include_str!("../src/ui/snippets/ai/queue_prompt_input_demo.rs"),
            "Queue + PromptInput (AI Elements)",
            "Docs-aligned composition: content-only QueueSection above PromptInput tools.",
            "ui-ai-queue-prompt-input-sent-count-1",
        ),
        (
            "transcription_demo",
            include_str!("../src/ui/snippets/ai/transcription_demo.rs"),
            "App-owned timeline + interactive transcript",
            "Drag the scrubber or click a segment to seek. The transcript consumes app-owned current_time just like the official AI Elements example.",
            "ui-ai-transcription-demo-time-zero",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "fn state_marker",
            "SemanticsRole::Generic",
            "cx.spacer(SpacerProps",
            marker_id,
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed visible text and state markers through shared non-bare roles; missing `{marker}`"
            );
        }

        for forbidden in [
            "role: fret_core::SemanticsRole::Text",
            "cx.text(\"\")",
            &format!("cx.text(\"{title}\")"),
            &format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare visible text/state marker text: `{forbidden}`"
            );
        }
    }

    let queue_prompt = canonicalize_rust_fragment(include_str!(
        "../src/ui/snippets/ai/queue_prompt_input_demo.rs"
    ));
    for marker in [
        "decl_text::text_button_label(cx, \"Search\")",
        "state_marker(cx, \"ui-ai-queue-prompt-input-sent-count-1\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            queue_prompt.contains(&marker),
            "queue_prompt_input_demo should route custom prompt-button text and sent marker through shared roles; missing `{marker}`"
        );
    }
    for forbidden in ["ui::text(\"Search\")", "cx.text(\"Search\")"] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !queue_prompt.contains(&forbidden),
            "queue_prompt_input_demo reintroduced a bare custom prompt-button label: `{forbidden}`"
        );
    }

    let transcription =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/transcription_demo.rs"));
    for marker in [
        "ui-ai-transcription-demo-time-nonzero",
        "ui-ai-transcription-demo-active-{active_index}",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            transcription.contains(&marker),
            "transcription_demo should preserve diagnostics marker ids while keeping them non-text; missing `{marker}`"
        );
    }
}

#[test]
fn ai_web_preview_uses_shared_text_roles_and_non_text_markers() {
    let source = include_str!("../src/ui/snippets/ai/web_preview_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement",
        "role: fret_core::SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "state_marker(cx, \"ui-ai-web-preview-demo-loading-false\")",
        "state_marker(cx, \"ui-ai-web-preview-demo-committed-true\")",
        "state_marker(cx, \"ui-ai-web-preview-demo-can-back-true\")",
        "state_marker(cx, \"ui-ai-web-preview-demo-can-forward-true\")",
        "state_marker(cx, \"ui-ai-web-preview-demo-can-forward-false\")",
        "decl_text::text_chrome_glyph(cx, \"←\")",
        "decl_text::text_chrome_glyph(cx, \"→\")",
        "decl_text::text_chrome_glyph(cx, \"↺\")",
        "decl_text::text_section_chrome_label(cx, \"Custom body content\")",
        "decl_text::text_paragraph",
        "\"Use this lane when preview chrome is enough for the current build.\"",
        "decl_text::text_section_chrome_label(cx, \"Custom console footer\")",
        "decl_text::text_paragraph(cx, \"Backend navigation is app-owned and optional in Fret.\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "web_preview_demo should route fixed visible text and state markers through shared non-bare roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "role: fret_core::SemanticsRole::Text",
        "cx.text(\"\")",
        "cx.text(\"←\")",
        "cx.text(\"→\")",
        "cx.text(\"↺\")",
        "cx.text(\"Custom body content\")",
        "cx.text(\"Use this lane when preview chrome is enough for the current build.\")",
        "cx.text(\"Custom console footer\")",
        "cx.text(\"Backend navigation is app-owned and optional in Fret.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "web_preview_demo reintroduced bare visible text/state marker text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_chat_demo_uses_shared_outer_text_roles_and_non_text_markers() {
    let source = include_str!("../src/ui/snippets/ai/chat_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn empty_spacer(cx: &mut AppComponentCx<'_>) -> AnyElement",
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement",
        "role: fret_core::SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "state_marker(cx, \"ui-gallery-ai-chat-prompt-nonempty\")",
        "decl_text::text_paragraph",
        "\"Goal: interactive demo for PromptInput + transcript append.\"",
        "\"Send triggers a short \\\"loading\\\" window where Stop is available.\"",
        "decl_text::text_control_readout",
        "format!(\"Exported markdown: {len} chars\")",
        "prompt_non_empty_marker.unwrap_or_else(|| empty_spacer(cx))",
        "exported.unwrap_or_else(|| empty_spacer(cx))",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "chat_demo should route fixed outer text, readouts, and state markers through shared roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "role: fret_core::SemanticsRole::Text, test_id: Some(Arc::<str>::from(\"ui-gallery-ai-chat-prompt-nonempty\"))",
        "cx.text(\"Goal: interactive demo for PromptInput + transcript append.\")",
        "cx.text(\"Send triggers a short \\\"loading\\\" window where Stop is available.\")",
        "vec![cx.text(format!(\"Exported markdown: {len} chars\"))]",
        "prompt_non_empty_marker.unwrap_or_else(|| cx.text(\"\"))",
        "exported.unwrap_or_else(|| cx.text(\"\"))",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "chat_demo reintroduced bare outer text/readout/marker text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_prompt_input_provider_and_docs_use_shared_text_roles_and_non_text_markers() {
    for (name, source, title, body) in [
        (
            "prompt_input_provider_demo",
            include_str!("../src/ui/snippets/ai/prompt_input_provider_demo.rs"),
            "Prompt Input Provider (AI Elements)",
            "External add mutates the provider attachments; send clears attachments.",
        ),
        (
            "prompt_input_docs_demo",
            include_str!("../src/ui/snippets/ai/prompt_input_docs_demo.rs"),
            "Prompt Input (AI Elements)",
            "Docs-aligned chat example: transcript + prompt composer, add attachments/screenshot actions, model picker, and upstream-like onSubmit(message).",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed title/body text through shared text roles; missing `{marker}`"
            );
        }

        for forbidden in [
            format!("cx.text(\"{title}\")"),
            format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(&forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare fixed visible text: `{forbidden}`"
            );
        }
    }

    let provider = canonicalize_rust_fragment(include_str!(
        "../src/ui/snippets/ai/prompt_input_provider_demo.rs"
    ));
    for marker in [
        "fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement",
        "role: fret_core::SemanticsRole::Generic",
        "cx.spacer(SpacerProps",
        "state_marker(cx, \"ui-gallery-ai-prompt-input-provider-sent-count-1\")",
        "decl_text::text_button_label(cx, add_external_label.clone())",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            provider.contains(&marker),
            "prompt_input_provider_demo should route custom label and state marker through shared non-bare roles; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(\"\")",
        "ui::text(add_external_label.clone())",
        "role: fret_core::SemanticsRole::Text",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !provider.contains(&forbidden),
            "prompt_input_provider_demo reintroduced bare custom label or marker text: `{forbidden}`"
        );
    }

    let docs = canonicalize_rust_fragment(include_str!(
        "../src/ui/snippets/ai/prompt_input_docs_demo.rs"
    ));
    for marker in ["decl_text::text_button_label(cx, \"Search\")"] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            docs.contains(&marker),
            "prompt_input_docs_demo should route custom Search button text through shared button-label role"
        );
    }
    for forbidden in ["ui::text(\"Search\")", "cx.text(\"Search\")"] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !docs.contains(&forbidden),
            "prompt_input_docs_demo reintroduced bare custom Search label: `{forbidden}`"
        );
    }
}

#[test]
fn ai_reasoning_stack_trace_and_voice_selector_use_shared_chrome_text_roles() {
    for (name, source, title, body) in [
        (
            "reasoning_demo",
            include_str!("../src/ui/snippets/ai/reasoning_demo.rs"),
            "Reasoning (AI Elements)",
            "Start streaming to auto-open; stop to auto-close.",
        ),
        (
            "stack_trace_demo",
            include_str!("../src/ui/snippets/ai/stack_trace_demo.rs"),
            "StackTrace (AI Elements)",
            "Docs-aligned compound parts API with copy + file-open seams.",
        ),
        (
            "voice_selector_demo",
            include_str!("../src/ui/snippets/ai/voice_selector_demo.rs"),
            "VoiceSelector (AI Elements)",
            "Composable dialog + command recipe. Apps still own inventory and preview playback state.",
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "decl_text::text_section_chrome_label",
            title,
            "decl_text::text_paragraph",
            body,
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route fixed title/body text through shared chrome/prose roles; missing `{marker}`"
            );
        }

        for forbidden in [
            format!("cx.text(\"{title}\")"),
            format!("cx.text(\"{body}\")"),
        ] {
            let forbidden = canonicalize_rust_fragment(&forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare fixed visible text: `{forbidden}`"
            );
        }
    }

    let stack_trace =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/stack_trace_demo.rs"));
    for marker in [
        "decl_text::text_control_readout(cx, format!(\"Status: {status_text}\"))",
        "test_id(\"ui-ai-stack-trace-status\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            stack_trace.contains(&marker),
            "stack_trace_demo should keep status text on the shared control-readout role; missing `{marker}`"
        );
    }
    let forbidden = canonicalize_rust_fragment("cx.text(format!(\"Status: {status_text}\"))");
    assert!(
        !stack_trace.contains(&forbidden),
        "stack_trace_demo reintroduced bare status readout text: `{forbidden}`"
    );

    let voice_selector =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/voice_selector_demo.rs"));
    for marker in [
        "decl_text::text_control_readout",
        "selected.as_deref().unwrap_or(\"<none>\")",
        "decl_text::text_control_readout(cx, format!(\"open={open_now}\"))",
        "ui-ai-voice-selector-demo-selected",
        "ui-ai-voice-selector-demo-open-true",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            voice_selector.contains(&marker),
            "voice_selector_demo should keep diagnostics readouts on the shared control-readout role; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(format!(\"selected={}\", selected.as_deref().unwrap_or(\"<none>\")))",
        "cx.text(format!(\"open={open_now}\"))",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !voice_selector.contains(&forbidden),
            "voice_selector_demo reintroduced bare diagnostics readout text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_confirmation_snippets_use_shared_content_text_roles() {
    for (name, source) in [
        (
            "confirmation_demo",
            include_str!("../src/ui/snippets/ai/confirmation_demo.rs"),
        ),
        (
            "confirmation_accepted",
            include_str!("../src/ui/snippets/ai/confirmation_accepted.rs"),
        ),
        (
            "confirmation_rejected",
            include_str!("../src/ui/snippets/ai/confirmation_rejected.rs"),
        ),
    ] {
        let canonical = canonicalize_rust_fragment(source);

        for marker in [
            "use fret_ui_kit::declarative::text as decl_text;",
            "decl_text::text_paragraph(cx, \"This tool wants to delete the file\")",
            "decl_text::text_code_wrap(cx, \"/tmp/example.txt\")",
            "decl_text::text_paragraph(cx, \". Do you approve this action?\")",
            "decl_text::text_control_readout(cx, \"You approved this tool execution\")",
            "decl_text::text_control_readout(cx, \"You rejected this tool execution\")",
        ] {
            let marker = canonicalize_rust_fragment(marker);
            assert!(
                canonical.contains(&marker),
                "{name} should route confirmation request/result content through shared roles; missing `{marker}`"
            );
        }

        for forbidden in [
            "cx.text(\"This tool wants to delete the file\")",
            "shadcn::raw::typography::inline_code(\"/tmp/example.txt\").into_element(cx)",
            "cx.text(\". Do you approve this action?\")",
            "cx.text(\"You approved this tool execution\")",
            "cx.text(\"You rejected this tool execution\")",
        ] {
            let forbidden = canonicalize_rust_fragment(forbidden);
            assert!(
                !canonical.contains(&forbidden),
                "{name} reintroduced bare confirmation content text: `{forbidden}`"
            );
        }
    }

    let demo =
        canonicalize_rust_fragment(include_str!("../src/ui/snippets/ai/confirmation_demo.rs"));
    for marker in [
        "decl_text::text_section_chrome_label(cx, \"Confirmation (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Docs-aligned tool approval workflow: request a destructive action, confirm or reject it, then inspect the final output state.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            demo.contains(&marker),
            "confirmation_demo should route fixed outer title/body through shared roles; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(\"Confirmation (AI Elements)\")",
        "cx.text(\"Docs-aligned tool approval workflow: request a destructive action, confirm or reject it, then inspect the final output state.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !demo.contains(&forbidden),
            "confirmation_demo reintroduced bare outer text: `{forbidden}`"
        );
    }

    let request = canonicalize_rust_fragment(include_str!(
        "../src/ui/snippets/ai/confirmation_request.rs"
    ));
    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::text_paragraph",
        "\"This tool wants to execute a query on the production database:\"",
        "decl_text::text_code_wrap",
        "SELECT * FROM users WHERE role = 'admin'",
        "decl_text::text_control_readout(cx, \"You approved this tool execution\")",
        "decl_text::text_control_readout(cx, \"You rejected this tool execution\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            request.contains(&marker),
            "confirmation_request should route query request/result content through shared roles; missing `{marker}`"
        );
    }
    for forbidden in [
        "cx.text(\"This tool wants to execute a query on the production database:\")",
        "cx.text(\"You approved this tool execution\")",
        "cx.text(\"You rejected this tool execution\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !request.contains(&forbidden),
            "confirmation_request reintroduced bare confirmation content text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_task_demo_uses_shared_content_text_roles() {
    let source = include_str!("../src/ui/snippets/ai/task_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::{icon, style as decl_style, text as decl_text};",
        "decl_text::text_list_row_label(cx, text)",
        "decl_text::text_code_wrap(cx, file_name)",
        "decl_text::text_section_chrome_label(cx, \"Task (AI Elements)\")",
        "decl_text::text_paragraph",
        "\"Collapsible task list demo aligned with the official AI Elements Task structure.\"",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "task_demo should route task rows, file labels, and fixed outer text through shared roles; missing `{marker}`"
        );
    }

    for forbidden in [
        "cx.text(text)",
        "cx.text(file_name)",
        "cx.text(\"Task (AI Elements)\")",
        "cx.text(\"Collapsible task list demo aligned with the official AI Elements Task structure.\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "task_demo reintroduced bare task content text: `{forbidden}`"
        );
    }
}

#[test]
fn ai_conversation_demo_uses_non_text_instrumentation_and_button_label() {
    let source = include_str!("../src/ui/snippets/ai/conversation_demo.rs");
    let canonical = canonicalize_rust_fragment(source);

    for marker in [
        "use fret_ui_kit::declarative::text as decl_text;",
        "role: SemanticsRole::Generic",
        "test_id: Some(Arc::<str>::from(\"ui-ai-conversation-demo-exported-md-len\"))",
        "test_id: Some(Arc::<str>::from(\"ui-ai-conversation-demo-messages-len\"))",
        "numeric_value: Some(exported_md_len as f64)",
        "numeric_value: Some(messages.len() as f64)",
        "decl_text::text_button_label(cx, \"Latest\")",
    ] {
        let marker = canonicalize_rust_fragment(marker);
        assert!(
            canonical.contains(&marker),
            "conversation_demo should keep diagnostics out of text layout and custom button text on the shared role; missing `{marker}`"
        );
    }

    for forbidden in [
        "role: SemanticsRole::Text",
        "role: fret_core::SemanticsRole::Text",
        "cx.text(\"Latest\")",
    ] {
        let forbidden = canonicalize_rust_fragment(forbidden);
        assert!(
            !canonical.contains(&forbidden),
            "conversation_demo reintroduced text-role diagnostics or bare button text: `{forbidden}`"
        );
    }
}
