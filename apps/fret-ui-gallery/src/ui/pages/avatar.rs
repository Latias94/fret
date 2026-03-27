use super::super::*;
use fret::{UiChild, UiCx};

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::avatar as snippets;

fn avatar_api_table<I>(cx: &mut UiCx<'_>, title: &'static str, rows: I) -> impl UiChild + use<I>
where
    I: IntoIterator<Item = [&'static str; 3]>,
{
    let title = ui::text(title).font_semibold().into_element(cx);
    let table =
        doc_layout::text_table(cx, ["Surface", "Type", "Default"], rows, true).into_element(cx);

    ui::v_flex(move |_cx| vec![title, table])
        .gap(Space::N3)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
}

fn avatar_api_reference(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let avatar = avatar_api_table(
        cx,
        "Avatar",
        [
            ["`size(...)`", "`AvatarSize`", "`Default`"],
            [
                "`children([...])`",
                "`IntoIterator<Item = AnyElement>`",
                "`[]`",
            ],
            ["`refine_style(...)`", "`ChromeRefinement`", "`default`"],
            ["`refine_layout(...)`", "`LayoutRefinement`", "`default`"],
        ],
    )
    .into_element(cx);
    let image = avatar_api_table(
        cx,
        "AvatarImage",
        [
            ["`new(image)`", "`ImageId`", "required"],
            ["`maybe(image)`", "`Option<ImageId>`", "`None`"],
            ["`model(image)`", "`Model<Option<ImageId>>`", "-"],
            ["`opacity(...)`", "`f32`", "`1.0`"],
        ],
    )
    .into_element(cx);
    let fallback = avatar_api_table(
        cx,
        "AvatarFallback",
        [
            ["`new(text)`", "`impl Into<Arc<str>>`", "required"],
            [
                "`when_image_missing(...)`",
                "`Option<ImageId>`",
                "render always",
            ],
            [
                "`when_image_missing_model(...)`",
                "`Model<Option<ImageId>>`",
                "-",
            ],
            ["`delay_ms(...)`", "`u64`", "immediate"],
            ["`delay_frames(...)`", "`u64`", "immediate"],
        ],
    )
    .into_element(cx);
    let badge = avatar_api_table(
        cx,
        "AvatarBadge",
        [
            [
                "`children([...])`",
                "`IntoIterator<Item = AnyElement>`",
                "empty status dot",
            ],
            ["`size(...)`", "`AvatarSize`", "inherit avatar size"],
            [
                "`refine_style(...)`",
                "`ChromeRefinement`",
                "recipe defaults",
            ],
            ["`refine_layout(...)`", "`LayoutRefinement`", "`default`"],
        ],
    )
    .into_element(cx);
    let group = avatar_api_table(
        cx,
        "AvatarGroup",
        [
            [
                "`new(children)`",
                "`IntoIterator<Item = AnyElement>`",
                "required",
            ],
            [
                "`children([...])`",
                "`IntoIterator<Item = AnyElement>`",
                "`[]` via `empty()`",
            ],
            ["`size(...)`", "`AvatarSize`", "optional shared size scope"],
            ["`refine_style(...)`", "`ChromeRefinement`", "`default`"],
            ["`refine_layout(...)`", "`LayoutRefinement`", "`default`"],
        ],
    )
    .into_element(cx);
    let group_count = avatar_api_table(
        cx,
        "AvatarGroupCount",
        [
            [
                "`new(children)`",
                "`IntoIterator<Item = AnyElement>`",
                "required",
            ],
            [
                "`children([...])`",
                "`IntoIterator<Item = AnyElement>`",
                "`+3` when empty",
            ],
            ["`size(...)`", "`AvatarSize`", "inherit group/avatar size"],
            [
                "`refine_style(...)`",
                "`ChromeRefinement`",
                "recipe defaults",
            ],
            ["`refine_layout(...)`", "`LayoutRefinement`", "`default`"],
        ],
    )
    .into_element(cx);
    let notes = doc_layout::notes_block([
        "Upstream docs path: `repo-ref/ui/apps/v4/content/docs/components/base/avatar.mdx`.",
        "`className`-style upstream customization maps to `refine_style(...)` / `refine_layout(...)` in Fret; page/container width negotiation remains caller-owned.",
        "`Avatar::new([..])` and `Avatar::children([..])` are already sufficient for composable avatar content; no extra generic children or slot-merge API is needed here.",
        "`Avatar::empty().children([..])`, `AvatarGroup::empty().children([..])`, and `AvatarGroupCount::empty().children([..])` now cover the docs-shaped composable children lane without widening the family beyond its typed parts.",
        "`AvatarBadge` stays the icon/content child lane, and `avatar_sized(...)` remains the preferred helper when size-dependent parts should inherit scope before landing.",
    ])
    .into_element(cx);

    ui::v_flex(move |_cx| vec![avatar, image, fallback, badge, group, group_count, notes])
        .gap(Space::N6)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
}

pub(super) fn preview_avatar(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let basic = snippets::basic::render(cx);
    let with_badge = snippets::with_badge::render(cx);
    let badge_icon = snippets::badge_icon::render(cx);
    let avatar_group = snippets::group::render(cx);
    let group_count = snippets::group_count::render(cx);
    let group_count_icon = snippets::group_count_icon::render(cx);
    let sizes = snippets::sizes::render(cx);
    let dropdown = snippets::dropdown::render(cx);
    let rtl = snippets::rtl::render(cx);
    let fallback = snippets::fallback_only::render(cx);

    let api_reference = avatar_api_reference(cx);

    let notes = doc_layout::notes_block([
        "Core parity is already in a good place: default size, circular clipping, fallback timing, overlap geometry, and dropdown trigger attribution all match the audited upstream outcomes.",
        "Gallery sections now mirror shadcn Avatar docs first: Demo, Usage, Basic, Badge, Badge with Icon, Avatar Group, Avatar Group Count, Avatar Group with Icon, Sizes, Dropdown, RTL, API Reference.",
        "The current follow-up work is docs/API surface alignment rather than a mechanism-layer fix: the page now mirrors the upstream part breakdown more directly, and group/count snippets also teach the docs-shaped builder lane.",
        "Gallery snippets now resolve their demo image through the shared gallery demo asset bundle, keeping avatar demos aligned with the rest of the first-party media surface.",
        "`Fallback only` remains a Fret-specific follow-up section for compact regression coverage across sizes.",
    ]);

    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .description("Upstream-shaped part breakdown plus Fret surface/default mapping.")
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-avatar-api-reference");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .description("Usage notes.")
        .test_id_prefix("ui-gallery-avatar-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .description(
            "Docs-aligned demo: basic avatar, avatar with badge, and avatar group with count.",
        )
        .test_id_prefix("ui-gallery-avatar-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Minimal usage mirroring the upstream docs shape, adapted to Fret's `ImageId` asset model.")
        .test_id_prefix("ui-gallery-avatar-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .description("Single avatar with an image + fallback using a bundle-backed demo image.")
        .test_id_prefix("ui-gallery-avatar-basic")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let with_badge = DocSection::build(cx, "Badge", with_badge)
        .description("`AvatarBadge` overlays a status dot at the avatar's bottom-right.")
        .test_id_prefix("ui-gallery-avatar-badge")
        .code_rust_from_file_region(snippets::with_badge::SOURCE, "example");
    let badge_icon = DocSection::build(cx, "Badge with Icon", badge_icon)
        .description("Use `AvatarBadge` children to render an icon badge.")
        .test_id_prefix("ui-gallery-avatar-badge-icon")
        .code_rust_from_file_region(snippets::badge_icon::SOURCE, "example");
    let avatar_group = DocSection::build(cx, "Avatar Group", avatar_group)
        .description("Overlapping avatar group (`-space-x-2`).")
        .test_id_prefix("ui-gallery-avatar-group")
        .code_rust_from_file_region(snippets::group::SOURCE, "example");
    let group_count = DocSection::build(cx, "Avatar Group Count", group_count)
        .description("Trailing count bubble that matches the group's size.")
        .test_id_prefix("ui-gallery-avatar-group-count")
        .code_rust_from_file_region(snippets::group_count::SOURCE, "example");
    let group_count_icon = DocSection::build(cx, "Avatar Group with Icon", group_count_icon)
        .description("Use an icon inside `AvatarGroupCount` for add/invite affordances.")
        .test_id_prefix("ui-gallery-avatar-group-count-icon")
        .code_rust_from_file_region(snippets::group_count_icon::SOURCE, "example");
    let sizes = DocSection::build(cx, "Sizes", sizes)
        .description("Upstream: `size=\"sm\" | \"default\" | \"lg\"`; first-party examples teach `avatar_sized(...)` for that lane.")
        .test_id_prefix("ui-gallery-avatar-sizes")
        .code_rust_from_file_region(snippets::sizes::SOURCE, "example");
    let dropdown = DocSection::build(cx, "Dropdown", dropdown)
        .description("Use Avatar as a DropdownMenu trigger (shadcn `render` / `asChild` composition outcome).")
        .test_id_prefix("ui-gallery-avatar-dropdown")
        .code_rust_from_file_region(snippets::dropdown::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Avatar should behave under an RTL direction provider.")
        .test_id_prefix("ui-gallery-avatar-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let fallback = DocSection::build(cx, "Fallback only (Fret)", fallback)
        .description("Fallback-only avatars at each size.")
        .test_id_prefix("ui-gallery-avatar-fallback")
        .code_rust_from_file_region(snippets::fallback_only::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview now mirrors the shadcn Avatar docs order first, and API Reference now tracks the upstream part breakdown more directly before appending a small Fret-specific fallback-only check.",
        ),
        vec![
            demo,
            usage,
            basic,
            with_badge,
            badge_icon,
            avatar_group,
            group_count,
            group_count_icon,
            sizes,
            dropdown,
            rtl,
            api_reference,
            fallback,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-avatar").into_element(cx)]
}
