use super::super::*;

use crate::ui::doc_layout::{DocSection, notes};
use crate::ui::snippets::ai as snippets;

pub(super) fn preview_ai_checkpoint_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::checkpoint_demo::render(cx);
    let features = notes(
        cx,
        [
            "Simple flex layout with icon, trigger, and separator",
            "Visual separator line for clear conversation breaks",
            "Clickable restore button for reverting to checkpoint",
            "Customizable icon (defaults to BookmarkIcon)",
            "Keyboard accessible with proper ARIA labels",
            "Responsive design that adapts to different screen sizes",
            "Seamless light/dark theme integration",
        ],
    );
    let customizable_icon = notes(
        cx,
        [
            "Swap the default bookmark for a product-specific icon while keeping the surrounding checkpoint recipe unchanged.",
        ],
    );
    let manual_checkpoints = notes(
        cx,
        ["Allow users to manually create checkpoints at important conversation points."],
    );
    let automatic_checkpoints = notes(
        cx,
        ["Create checkpoints automatically after significant conversation milestones."],
    );
    let branching = notes(
        cx,
        [
            "Use checkpoints to enable branching conversations where users can explore different paths.",
        ],
    );

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "The `Checkpoint` component provides a way to mark specific points in a conversation history and restore the chat to that state. Inspired by VSCode's Copilot checkpoint feature, it allows users to revert to an earlier conversation state while maintaining a clear visual separation between different conversation segments.",
        ),
        vec![
            DocSection::new("Usage with AI SDK", demo)
                .description(
                    "Build a chat interface with conversation checkpoints that allow users to restore to previous states.",
                )
                .test_id_prefix("ui-gallery-ai-checkpoint-demo")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::checkpoint_demo::SOURCE, "example"),
            DocSection::new("Features", features)
                .description("Official AI Elements defaults and outcomes to keep in view while porting.")
                .max_w(Px(980.0))
                .no_shell(),
            DocSection::new("Customizable Icon", customizable_icon)
                .description("The icon slot stays composable even though the default visual is a bookmark.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::checkpoint_demo::SOURCE, "custom_icon")
                .no_shell(),
            DocSection::new("Manual Checkpoints", manual_checkpoints)
                .description("Create a checkpoint at an explicit user-visible milestone.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::checkpoint_demo::SOURCE, "manual_checkpoints")
                .no_shell(),
            DocSection::new("Automatic Checkpoints", automatic_checkpoints)
                .description("Use a lightweight milestone rule when you want history snapshots without extra UI chrome.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::checkpoint_demo::SOURCE, "automatic_checkpoints")
                .no_shell(),
            DocSection::new("Branching Conversations", branching)
                .description("Restore earlier context while preserving the truncated tail as a branch.")
                .max_w(Px(980.0))
                .code_rust_from_file_region(snippets::checkpoint_demo::SOURCE, "branching_conversations")
                .no_shell(),
        ],
    );

    vec![body]
}
