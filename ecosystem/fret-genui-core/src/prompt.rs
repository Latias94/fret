//! Prompt helpers (json-render-inspired).
//!
//! This module provides a small, reusable helper for building user prompts that:
//! - optionally include the current spec (refine mode, patch-only),
//! - optionally include runtime state context,
//! - include stable instructions for JSONL RFC 6902 patches.

use serde_json::Value;

/// Output mode for the prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserPromptModeV1 {
    /// The LLM should output only JSONL patch lines (no prose).
    Generate,
    /// The LLM may include conversational text, but must keep patches inside a ```spec fence.
    Chat,
}

/// Options for building a user prompt.
#[derive(Debug, Clone)]
pub struct UserPromptOptionsV1<'a> {
    /// The user's request (natural language).
    pub prompt: &'a str,
    /// Existing spec JSON (when provided, prompt enters "refine / patch-only" mode).
    pub current_spec: Option<&'a Value>,
    /// Optional state context the app wants the LLM to know about.
    pub state: Option<&'a Value>,
    /// Optional maximum length for the user's prompt.
    pub max_prompt_chars: Option<usize>,
    /// Output mode.
    pub mode: UserPromptModeV1,
}

impl<'a> Default for UserPromptOptionsV1<'a> {
    fn default() -> Self {
        Self {
            prompt: "",
            current_spec: None,
            state: None,
            max_prompt_chars: None,
            mode: UserPromptModeV1::Generate,
        }
    }
}

const PATCH_ONLY_INSTRUCTIONS: &str = r#"IMPORTANT: The current UI is already loaded.
Output ONLY the patches needed to make the requested change.

- To add a new element: {"op":"add","path":"/elements/new-key","value":{...}}
- To modify an existing element: {"op":"replace","path":"/elements/existing-key","value":{...}}
- To remove an element: {"op":"remove","path":"/elements/old-key"}
- To update the root: {"op":"replace","path":"/root","value":"new-root-key"}
- To update state: {"op":"replace","path":"/state/foo","value":123}

DO NOT output patches for elements that don't need to change.
Only output what's necessary for the requested modification."#;

const FRESH_GENERATION_INSTRUCTIONS: &str = r#"Output JSONL RFC 6902 patches, one JSON object per line.

Rules:
- Output /root first.
- Then interleave /elements and /state patches so the UI can fill in progressively.
- Do not wrap patch lines in Markdown code fences (unless mode=chat, see below).
"#;

const CHAT_MODE_INSTRUCTIONS: &str = r#"You may respond with short prose first, but patches MUST be inside a fenced block:

```spec
{"op":"replace","path":"/root","value":"root"}
{"op":"add","path":"/elements/root","value":{...}}
```

Outside the ```spec fence, do not emit JSON patches."#;

fn truncate_chars(s: &str, max_chars: Option<usize>) -> String {
    let Some(max) = max_chars else {
        return s.to_string();
    };
    if max == 0 {
        return String::new();
    }
    s.chars().take(max).collect()
}

fn is_non_empty_spec(v: &Value) -> bool {
    let Some(obj) = v.as_object() else {
        return false;
    };
    let root_ok = obj.get("root").and_then(|v| v.as_str()).is_some();
    let elements_ok = obj
        .get("elements")
        .and_then(|v| v.as_object())
        .is_some_and(|m| !m.is_empty());
    root_ok && elements_ok
}

/// Build a user prompt suitable for generating or refining a GenUI spec.
///
/// This is intentionally pure string formatting: apps decide which LLM provider/tooling to use.
pub fn build_user_prompt_v1(opts: UserPromptOptionsV1<'_>) -> String {
    let user_text = truncate_chars(opts.prompt, opts.max_prompt_chars);

    let mut parts: Vec<String> = Vec::new();

    let refine_mode = opts
        .current_spec
        .is_some_and(|spec| is_non_empty_spec(spec));

    if refine_mode {
        let spec = opts.current_spec.expect("refine_mode implies current_spec");
        parts.push("CURRENT UI STATE (already loaded, DO NOT recreate existing elements):".into());
        parts.push(serde_json::to_string_pretty(spec).unwrap_or_else(|_| "<spec>".to_string()));
        parts.push(String::new());
        parts.push(format!("USER REQUEST: {user_text}"));

        if let Some(state) = opts.state {
            if state != &Value::Null {
                parts.push(String::new());
                parts.push("AVAILABLE STATE:".into());
                parts.push(
                    serde_json::to_string_pretty(state).unwrap_or_else(|_| "<state>".to_string()),
                );
            }
        }

        parts.push(String::new());
        parts.push(PATCH_ONLY_INSTRUCTIONS.into());

        return parts.join("\n");
    }

    // Fresh generation mode.
    parts.push(user_text);

    if let Some(state) = opts.state {
        if state != &Value::Null {
            parts.push(String::new());
            parts.push("AVAILABLE STATE:".into());
            parts.push(
                serde_json::to_string_pretty(state).unwrap_or_else(|_| "<state>".to_string()),
            );
        }
    }

    parts.push(String::new());
    parts.push(FRESH_GENERATION_INSTRUCTIONS.into());
    if opts.mode == UserPromptModeV1::Chat {
        parts.push(String::new());
        parts.push(CHAT_MODE_INSTRUCTIONS.into());
    }

    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn refine_mode_includes_patch_only_instructions_and_current_spec() {
        let current = json!({
            "schema_version": 1,
            "root": "root",
            "elements": { "root": { "type": "VStack", "props": {}, "children": [] } },
            "state": { "count": 1 }
        });

        let out = build_user_prompt_v1(UserPromptOptionsV1 {
            prompt: "add a button",
            current_spec: Some(&current),
            state: None,
            max_prompt_chars: None,
            mode: UserPromptModeV1::Generate,
        });

        assert!(out.contains("CURRENT UI STATE"));
        assert!(out.contains("\"root\": \"root\""));
        assert!(out.contains("USER REQUEST: add a button"));
        assert!(out.contains("Output ONLY the patches"));
    }

    #[test]
    fn fresh_mode_can_include_state_and_chat_instructions() {
        let state = json!({ "todos": [] });
        let out = build_user_prompt_v1(UserPromptOptionsV1 {
            prompt: "make a todo list",
            current_spec: None,
            state: Some(&state),
            max_prompt_chars: Some(5),
            mode: UserPromptModeV1::Chat,
        });

        assert!(out.contains("make "));
        assert!(out.contains("AVAILABLE STATE"));
        assert!(out.contains("\"todos\""));
        assert!(out.contains("Output JSONL RFC 6902 patches"));
        assert!(out.contains("```spec"));
    }
}
