# UI Gallery Fearless Refactor: AI Elements Tracker

This tracker extends `docs/workstreams/ui-gallery-fearless-refactor/todo.md` for the AI Elements
gallery surfaces.

Goal: migrate AI demos to **snippet-backed pages** so the UI preview and the copyable code are the
same by construction (Preview ≡ Code).

Status labels:

- `Legacy preview`: still implemented under `apps/fret-ui-gallery/src/ui/previews/gallery/ai/**`.
- `Snippet-backed`: implemented as `apps/fret-ui-gallery/src/ui/pages/**` + `apps/fret-ui-gallery/src/ui/snippets/ai/**`.

| Demo (module) | Gallery route | Legacy preview | Snippet | Page | Status |
|---|---|---|---|---|---|
| `agent_demo` | `PAGE_AI_AGENT_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/agent_demo.rs` | — | — | Legacy preview |
| `artifact_demo` | `PAGE_AI_ARTIFACT_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/artifact_demo.rs` | — | — | Legacy preview |
| `attachments_demo` | `PAGE_AI_ATTACHMENTS_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/attachments_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_attachments_demo.rs` | Snippet-backed |
| `audio_player_demo` | `PAGE_AI_AUDIO_PLAYER_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/audio_player_demo.rs` | — | — | Legacy preview |
| `canvas_world_layer_spike` | `PAGE_AI_CANVAS_WORLD_LAYER_SPIKE` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/canvas_world_layer_spike.rs` | — | — | Legacy preview |
| `chain_of_thought_demo` | `PAGE_AI_CHAIN_OF_THOUGHT_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/chain_of_thought_demo.rs` | — | — | Legacy preview |
| `chat_demo` | `PAGE_AI_CHAT_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/chat_demo.rs` | — | — | Legacy preview |
| `checkpoint_demo` | `PAGE_AI_CHECKPOINT_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/checkpoint_demo.rs` | — | — | Legacy preview |
| `code_block_demo` | `PAGE_AI_CODE_BLOCK_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/code_block_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_code_block_demo.rs` | Snippet-backed |
| `commit_demo` | `PAGE_AI_COMMIT_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/commit_demo.rs` | — | — | Legacy preview |
| `commit_large_demo` | `PAGE_AI_COMMIT_LARGE_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/commit_large_demo.rs` | — | — | Legacy preview |
| `confirmation_demo` | `PAGE_AI_CONFIRMATION_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/confirmation_demo.rs` | — | — | Legacy preview |
| `context_demo` | `PAGE_AI_CONTEXT_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/context_demo.rs` | — | — | Legacy preview |
| `conversation_demo` | `PAGE_AI_CONVERSATION_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/conversation_demo.rs` | — | — | Legacy preview |
| `environment_variables_demo` | `PAGE_AI_ENVIRONMENT_VARIABLES_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/environment_variables_demo.rs` | — | — | Legacy preview |
| `file_tree_demo` | `PAGE_AI_FILE_TREE_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/file_tree_demo.rs` | — | — | Legacy preview |
| `image_demo` | `PAGE_AI_IMAGE_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/image_demo.rs` | — | — | Legacy preview |
| `inline_citation_demo` | `PAGE_AI_INLINE_CITATION_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/inline_citation_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_inline_citation_demo.rs` | Snippet-backed |
| `message_branch_demo` | `PAGE_AI_MESSAGE_BRANCH_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/message_branch_demo.rs` | — | — | Legacy preview |
| `message_demo` | `PAGE_AI_MESSAGE_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/message_demo.rs` | — | — | Legacy preview |
| `mic_selector_demo` | `PAGE_AI_MIC_SELECTOR_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/mic_selector_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_mic_selector_demo.rs` | Snippet-backed |
| `model_selector_demo` | `PAGE_AI_MODEL_SELECTOR_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/model_selector_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_model_selector_demo.rs` | Snippet-backed |
| `open_in_chat_demo` | `PAGE_AI_OPEN_IN_CHAT_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/open_in_chat_demo.rs` | — | — | Legacy preview |
| `package_info_demo` | `PAGE_AI_PACKAGE_INFO_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/package_info_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_package_info_demo.rs` | Snippet-backed |
| `persona_demo` | `PAGE_AI_PERSONA_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/persona_demo.rs` | — | — | Legacy preview |
| `plan_demo` | `PAGE_AI_PLAN_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/plan_demo.rs` | — | — | Legacy preview |
| `prompt_input_action_menu_demo` | `PAGE_AI_PROMPT_INPUT_ACTION_MENU_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/prompt_input_action_menu_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_prompt_input_action_menu_demo.rs` | Snippet-backed |
| `prompt_input_provider_demo` | `PAGE_AI_PROMPT_INPUT_PROVIDER_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/prompt_input_provider_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_prompt_input_provider_demo.rs` | Snippet-backed |
| `prompt_input_referenced_sources_demo` | `PAGE_AI_PROMPT_INPUT_REFERENCED_SOURCES_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/prompt_input_referenced_sources_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_prompt_input_referenced_sources_demo.rs` | Snippet-backed |
| `queue_demo` | `PAGE_AI_QUEUE_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/queue_demo.rs` | — | — | Legacy preview |
| `reasoning_demo` | `PAGE_AI_REASONING_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/reasoning_demo.rs` | — | — | Legacy preview |
| `sandbox_demo` | `PAGE_AI_SANDBOX_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/sandbox_demo.rs` | — | — | Legacy preview |
| `schema_display_demo` | `PAGE_AI_SCHEMA_DISPLAY_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/schema_display_demo.rs` | — | — | Legacy preview |
| `shimmer_demo` | `PAGE_AI_SHIMMER_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/shimmer_demo.rs` | — | — | Legacy preview |
| `snippet_demo` | `PAGE_AI_SNIPPET_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/snippet_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_snippet_demo.rs` | Snippet-backed |
| `sources_demo` | `PAGE_AI_SOURCES_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/sources_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_sources_demo.rs` | Snippet-backed |
| `speech_input_demo` | `PAGE_AI_SPEECH_INPUT_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/speech_input_demo.rs` | — | — | Legacy preview |
| `stack_trace_demo` | `PAGE_AI_STACK_TRACE_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/stack_trace_demo.rs` | — | — | Legacy preview |
| `stack_trace_large_demo` | `PAGE_AI_STACK_TRACE_LARGE_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/stack_trace_large_demo.rs` | — | — | Legacy preview |
| `suggestions_demo` | `PAGE_AI_SUGGESTIONS_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/suggestions_demo.rs` | — | — | Legacy preview |
| `task_demo` | `PAGE_AI_TASK_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/task_demo.rs` | — | — | Legacy preview |
| `terminal_demo` | `PAGE_AI_TERMINAL_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/terminal_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_terminal_demo.rs` | Snippet-backed |
| `test_results_demo` | `PAGE_AI_TEST_RESULTS_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/test_results_demo.rs` | — | — | Legacy preview |
| `test_results_large_demo` | `PAGE_AI_TEST_RESULTS_LARGE_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/test_results_large_demo.rs` | — | — | Legacy preview |
| `tool_demo` | `PAGE_AI_TOOL_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/tool_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_tool_demo.rs` | Snippet-backed |
| `transcript_torture` | `PAGE_AI_TRANSCRIPT_TORTURE` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/transcript_torture.rs` | — | — | Legacy preview |
| `transcription_demo` | `PAGE_AI_TRANSCRIPTION_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/transcription_demo.rs` | — | — | Legacy preview |
| `voice_selector_demo` | `PAGE_AI_VOICE_SELECTOR_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/voice_selector_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_voice_selector_demo.rs` | Snippet-backed |
| `web_preview_demo` | `PAGE_AI_WEB_PREVIEW_DEMO` | (removed) | `apps/fret-ui-gallery/src/ui/snippets/ai/web_preview_demo.rs` | `apps/fret-ui-gallery/src/ui/pages/ai_web_preview_demo.rs` | Snippet-backed |
| `workflow_canvas_demo` | `PAGE_AI_WORKFLOW_CANVAS_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/workflow_canvas_demo.rs` | — | — | Legacy preview |
| `workflow_chrome_demo` | `PAGE_AI_WORKFLOW_CHROME_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/workflow_chrome_demo.rs` | — | — | Legacy preview |
| `workflow_connection_demo` | `PAGE_AI_WORKFLOW_CONNECTION_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/workflow_connection_demo.rs` | — | — | Legacy preview |
| `workflow_controls_demo` | `PAGE_AI_WORKFLOW_CONTROLS_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/workflow_controls_demo.rs` | — | — | Legacy preview |
| `workflow_edge_demo` | `PAGE_AI_WORKFLOW_EDGE_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/workflow_edge_demo.rs` | — | — | Legacy preview |
| `workflow_node_demo` | `PAGE_AI_WORKFLOW_NODE_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/workflow_node_demo.rs` | — | — | Legacy preview |
| `workflow_node_graph_demo` | `PAGE_AI_WORKFLOW_NODE_GRAPH_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/workflow_node_graph_demo.rs` | — | — | Legacy preview |
| `workflow_panel_demo` | `PAGE_AI_WORKFLOW_PANEL_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/workflow_panel_demo.rs` | — | — | Legacy preview |
| `workflow_toolbar_demo` | `PAGE_AI_WORKFLOW_TOOLBAR_DEMO` | `apps/fret-ui-gallery/src/ui/previews/gallery/ai/workflow_toolbar_demo.rs` | — | — | Legacy preview |
