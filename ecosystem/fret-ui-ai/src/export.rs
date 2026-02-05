use crate::model::{AiMessage, MessagePart, MessageRole, ToolCallPayload};

/// Converts an AI transcript into a Markdown string (portable; no host effects).
///
/// This is intended for “download transcript” / “copy as markdown” features. Apps decide where to
/// store the output (file, clipboard, share sheet).
pub fn messages_to_markdown(messages: &[AiMessage]) -> String {
    let mut out = String::new();

    for message in messages {
        if !out.is_empty() {
            out.push('\n');
            out.push('\n');
        }

        out.push_str(match message.role {
            MessageRole::User => "## User\n\n",
            MessageRole::Assistant => "## Assistant\n\n",
            MessageRole::System => "## System\n\n",
            MessageRole::Tool => "## Tool\n\n",
        });

        for part in message.parts.iter() {
            match part {
                MessagePart::Text(text) => {
                    out.push_str(text);
                    out.push('\n');
                    out.push('\n');
                }
                MessagePart::Markdown(md) => {
                    out.push_str(&md.text);
                    out.push('\n');
                    out.push('\n');
                }
                MessagePart::ToolCall(call) => {
                    out.push_str("### Tool call\n\n");
                    out.push_str("- name: `");
                    out.push_str(call.name.as_ref());
                    out.push_str("`\n");
                    out.push_str("- id: `");
                    out.push_str(call.id.as_ref());
                    out.push_str("`\n");
                    out.push_str("- state: `");
                    out.push_str(format!("{:?}", call.state).as_str());
                    out.push_str("`\n\n");

                    if let Some(input) = call.input.as_ref() {
                        out.push_str("#### Input\n\n");
                        out.push_str(&payload_to_fenced_markdown(input));
                        out.push('\n');
                        out.push('\n');
                    }
                    if let Some(output) = call.output.as_ref() {
                        out.push_str("#### Output\n\n");
                        out.push_str(&payload_to_fenced_markdown(output));
                        out.push('\n');
                        out.push('\n');
                    }
                    if let Some(error) = call.error.as_ref() {
                        out.push_str("#### Error\n\n");
                        out.push_str("```text\n");
                        out.push_str(error.as_ref());
                        out.push_str("\n```\n\n");
                    }
                }
                MessagePart::Sources(items) => {
                    out.push_str("### Sources\n\n");
                    for (idx, item) in items.iter().enumerate() {
                        out.push_str("- ");
                        out.push_str(&format!("[{}] ", idx + 1));
                        out.push_str(item.title.as_ref());
                        if let Some(url) = item.url.as_ref() {
                            out.push_str(" (");
                            out.push_str(url.as_ref());
                            out.push(')');
                        }
                        out.push('\n');
                    }
                    out.push('\n');
                }
                MessagePart::Citations(items) => {
                    out.push_str("### Citations\n\n");
                    for item in items.iter() {
                        out.push_str("- ");
                        out.push_str(item.label.as_ref());
                        out.push_str(" → `");
                        out.push_str(item.source_id.as_ref());
                        out.push_str("`\n");
                    }
                    out.push('\n');
                }
            }
        }
    }

    out
}

fn payload_to_fenced_markdown(payload: &ToolCallPayload) -> String {
    match payload {
        ToolCallPayload::Text(text) => format!("```text\n{text}\n```"),
        ToolCallPayload::Json(value) => {
            let pretty = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
            format!("```json\n{pretty}\n```")
        }
    }
}
