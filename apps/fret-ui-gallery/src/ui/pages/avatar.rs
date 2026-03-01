use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::avatar as snippets;

pub(super) fn preview_avatar(
    cx: &mut ElementContext<'_, App>,
    avatar_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct AvatarDropdownOpenState {
        model: Option<Model<bool>>,
    }

    let dropdown_open = cx.with_state(AvatarDropdownOpenState::default, |st| st.model.clone());
    let dropdown_open = if let Some(model) = dropdown_open {
        model
    } else {
        let model = cx.app.models_mut().insert(false);
        cx.with_state(AvatarDropdownOpenState::default, |st| {
            st.model = Some(model.clone());
        });
        model
    };

    let sizes = snippets::sizes::render(cx, avatar_image.clone());
    let fallback = snippets::fallback_only::render(cx);
    let with_badge = snippets::with_badge::render(cx, avatar_image.clone());
    let avatar_group = snippets::group::render(cx, avatar_image.clone());
    let group_count = snippets::group_count::render(cx, avatar_image.clone());
    let dropdown = snippets::dropdown::render(cx, avatar_image.clone(), dropdown_open.clone());
    let rtl = snippets::rtl::render(cx, avatar_image.clone());
    let basic = snippets::basic::render(cx, avatar_image.clone());

    let notes = doc_layout::notes(
        cx,
        [
            "Use `AvatarImage` when you already have an `ImageId` (cached/decoded).",
            "Use `AvatarFallback` to cover missing images and slow network loads.",
            "If you customize sizes, set both width and height to keep the avatar circular.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview aims to match shadcn Avatar docs: Basic, Badge, Group, Sizes, Dropdown, RTL (plus a small Fallback-only extra).",
        ),
        vec![
            DocSection::new("Basic", basic)
            .description("A basic avatar with an image + fallback.")
            .code_rust_from_file_region(include_str!("../snippets/avatar/basic.rs"), "example"),
            DocSection::new("With Badge", with_badge)
                .description(
                    "`AvatarBadge` overlays a status dot or icon at the avatar's bottom-right.",
                )
                .code_rust_from_file_region(
                    include_str!("../snippets/avatar/with_badge.rs"),
                    "example",
                ),
            DocSection::new("Avatar Group", avatar_group)
                .description("Overlapping avatar group (`-space-x-2`).")
                .code_rust_from_file_region(include_str!("../snippets/avatar/group.rs"), "example"),
            DocSection::new("Avatar Group Count", group_count)
                .description("Trailing count bubble that matches the group's size.")
                .code_rust_from_file_region(
                    include_str!("../snippets/avatar/group_count.rs"),
                    "example",
                ),
            DocSection::new("Sizes", sizes)
                .description("Upstream: `size=\"sm\" | \"default\" | \"lg\"`.")
                .code_rust_from_file_region(include_str!("../snippets/avatar/sizes.rs"), "example"),
            DocSection::new("Dropdown", dropdown)
                .description("Use Avatar as a DropdownMenu trigger (shadcn `asChild`-style composition).")
                .code_rust_from_file_region(
                    include_str!("../snippets/avatar/dropdown.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .description("Avatar should behave under an RTL direction provider.")
                .code_rust_from_file_region(include_str!("../snippets/avatar/rtl.rs"), "example"),
            DocSection::new("Extras: Fallback only", fallback)
                .description("Fallback-only avatars at each size.")
                .code_rust_from_file_region(
                    include_str!("../snippets/avatar/fallback_only.rs"),
                    "example",
                ),
            DocSection::new("Notes", notes).description("Usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-avatar")]
}
