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

    let demo = snippets::demo::render(cx, avatar_image.clone());
    let usage = snippets::usage::render(cx);
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
            "Core parity is already in a good place: default size, circular clipping, fallback timing, and overlap geometry match the upstream references we audit against.",
            "Fret uses `ImageId` / `Model<Option<ImageId>>` instead of DOM `src`, so the minimal usage snippet intentionally shows the asset-ready composition shape.",
            "`Avatar::new([..])` already gives a composable children API; `Avatar::children([..])` is also available now for builder-style composition consistency.",
            "If you customize sizes, set both width and height to keep the avatar circular.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview now follows the shadcn Avatar docs flow more closely: docs-aligned demo first, then a minimal usage example, then Fret-specific extras.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Docs-aligned demo: round avatar, rounded variant, and overlapping group.")
                .test_id_prefix("ui-gallery-avatar-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Minimal usage mirroring the upstream docs shape, adapted to Fret's `ImageId` asset model.")
                .test_id_prefix("ui-gallery-avatar-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .description("Single avatar with an image + fallback, using a gallery-owned image model.")
                .test_id_prefix("ui-gallery-avatar-basic")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("With Badge", with_badge)
                .description(
                    "`AvatarBadge` overlays a status dot or icon at the avatar's bottom-right.",
                )
                .test_id_prefix("ui-gallery-avatar-badge")
                .code_rust_from_file_region(snippets::with_badge::SOURCE, "example"),
            DocSection::new("Avatar Group", avatar_group)
                .description("Overlapping avatar group (`-space-x-2`).")
                .test_id_prefix("ui-gallery-avatar-group")
                .code_rust_from_file_region(snippets::group::SOURCE, "example"),
            DocSection::new("Avatar Group Count", group_count)
                .description("Trailing count bubble that matches the group's size.")
                .test_id_prefix("ui-gallery-avatar-group-count")
                .code_rust_from_file_region(snippets::group_count::SOURCE, "example"),
            DocSection::new("Sizes", sizes)
                .description("Upstream: `size=\"sm\" | \"default\" | \"lg\"`.")
                .test_id_prefix("ui-gallery-avatar-sizes")
                .code_rust_from_file_region(snippets::sizes::SOURCE, "example"),
            DocSection::new("Dropdown", dropdown)
                .description(
                    "Use Avatar as a DropdownMenu trigger (shadcn `asChild`-style composition).",
                )
                .test_id_prefix("ui-gallery-avatar-dropdown")
                .code_rust_from_file_region(snippets::dropdown::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Avatar should behave under an RTL direction provider.")
                .test_id_prefix("ui-gallery-avatar-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Extras: Fallback only", fallback)
                .description("Fallback-only avatars at each size.")
                .test_id_prefix("ui-gallery-avatar-fallback")
                .code_rust_from_file_region(snippets::fallback_only::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .description("Usage notes.")
                .test_id_prefix("ui-gallery-avatar-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-avatar")]
}
