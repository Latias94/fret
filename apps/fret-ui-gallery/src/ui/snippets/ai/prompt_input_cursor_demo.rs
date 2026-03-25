pub const SOURCE: &str = include_str!("prompt_input_cursor_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, LengthRefinement, Space};
use fret_ui_shadcn::facade as shadcn;
use std::sync::Arc;

const ACTIVE_TABS: &[&str] = &["packages/elements/src/task-queue-panel.tsx"];
const RECENT_TABS: &[&str] = &[
    "apps/test/app/examples/task-queue-panel.tsx",
    "apps/test/app/page.tsx",
    "packages/elements/src/task.tsx",
];

fn source_item(
    cx: &mut UiCx<'_>,
    title: &'static str,
    filename: &'static str,
    test_id: &'static str,
) -> ui_ai::PromptInputCommandItem {
    let text = ui::v_flex(move |cx| {
        vec![
            ui::text(title)
                .font_weight(fret_core::FontWeight::MEDIUM)
                .text_size_px(Px(14.0))
                .into_element(cx),
            ui::text(filename)
                .text_size_px(Px(12.0))
                .text_color(fret_ui_kit::ColorRef::Token {
                    key: "muted-foreground",
                    fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
                })
                .into_element(cx),
        ]
    })
    .gap(Space::N0p5)
    .layout(LayoutRefinement::default().min_w_0())
    .into_element(cx);

    ui_ai::PromptInputCommandItem::new(title)
        .value(title)
        .test_id(test_id)
        .children([decl_icon::icon(cx, IconId::new("lucide.globe")), text])
}

fn path_item(
    cx: &mut UiCx<'_>,
    path: &'static str,
    test_id: &'static str,
) -> ui_ai::PromptInputTabItem {
    ui_ai::PromptInputTabItem::new([
        decl_icon::icon(cx, IconId::new("lucide.globe")),
        ui::text(path)
            .truncate()
            .layout(LayoutRefinement::default().min_w_0().flex_1())
            .into_element(cx),
    ])
    .test_id(test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let on_submit: ui_ai::OnPromptInputSubmit = Arc::new(|_host, _action_cx, _message, _reason| {});

    let files_menu = ui_ai::PromptInputCommand::new()
        .input(
            ui_ai::PromptInputCommandInput::new()
                .placeholder("Add files, folders, docs...")
                .input_test_id("ui-gallery-ai-prompt-input-cursor-command-input"),
        )
        .list(
            ui_ai::PromptInputCommandList::new()
                .list_test_id("ui-gallery-ai-prompt-input-cursor-command-list")
                .empty(ui_ai::PromptInputCommandEmpty::new("No results found."))
                .group(
                    ui_ai::PromptInputCommandGroup::new().heading("Added").item(
                        ui_ai::PromptInputCommandItem::new("Active Tabs")
                            .test_id("ui-gallery-ai-prompt-input-cursor-active-tabs-item")
                            .children([
                                decl_icon::icon(cx, IconId::new("lucide.globe")),
                                ui::text("Active Tabs").into_element(cx),
                                ui::text("✓")
                                    .layout(LayoutRefinement::default().ml_auto())
                                    .text_color(fret_ui_kit::ColorRef::Token {
                                        key: "muted-foreground",
                                        fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
                                    })
                                    .into_element(cx),
                            ]),
                    ),
                )
                .separator(ui_ai::PromptInputCommandSeparator::new())
                .group(
                    ui_ai::PromptInputCommandGroup::new()
                        .heading("Other Files")
                        .children([
                            source_item(
                                cx,
                                "prompt-input.tsx",
                                "packages/elements/src",
                                "ui-gallery-ai-prompt-input-cursor-source-prompt-input",
                            ),
                            source_item(
                                cx,
                                "queue.tsx",
                                "apps/test/app/examples",
                                "ui-gallery-ai-prompt-input-cursor-source-queue",
                            ),
                        ]),
                ),
        )
        .into_element(cx);

    let rules_content = ui::v_flex(move |cx| {
        vec![
            ui::v_flex(move |cx| {
                vec![
                    ui::text("Attached Project Rules")
                        .font_weight(fret_core::FontWeight::MEDIUM)
                        .text_size_px(Px(14.0))
                        .text_color(fret_ui_kit::ColorRef::Token {
                            key: "muted-foreground",
                            fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
                        })
                        .into_element(cx),
                    ui::text("Always Apply:")
                        .text_size_px(Px(14.0))
                        .text_color(fret_ui_kit::ColorRef::Token {
                            key: "muted-foreground",
                            fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
                        })
                        .layout(LayoutRefinement::default().ml(Space::N4))
                        .into_element(cx),
                    ui::text("ultracite.mdc")
                        .text_size_px(Px(14.0))
                        .layout(LayoutRefinement::default().ml(Space::N8))
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .p(Space::N3)
            .into_element(cx),
            shadcn::Separator::new().into_element(cx),
            ui::h_flex(move |cx| {
                vec![
                    ui::text("Click to manage")
                        .text_size_px(Px(14.0))
                        .text_color(fret_ui_kit::ColorRef::Token {
                            key: "muted-foreground",
                            fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
                        })
                        .into_element(cx),
                ]
            })
            .px(Space::N4)
            .py(Space::N3)
            .into_element(cx),
        ]
    })
    .gap(Space::N0)
    .into_element(cx);

    let tabs_list = ui_ai::PromptInputTabsList::new().children([
        ui_ai::PromptInputTab::new()
            .label(ui_ai::PromptInputTabLabel::new("Active Tabs"))
            .body(
                ui_ai::PromptInputTabBody::new().children(
                    ACTIVE_TABS
                        .iter()
                        .enumerate()
                        .map(|(index, path)| {
                            path_item(
                                cx,
                                path,
                                if index == 0 {
                                    "ui-gallery-ai-prompt-input-cursor-active-tab"
                                } else {
                                    "ui-gallery-ai-prompt-input-cursor-active-tab-extra"
                                },
                            )
                        })
                        .collect::<Vec<_>>(),
                ),
            ),
        ui_ai::PromptInputTab::new()
            .label(ui_ai::PromptInputTabLabel::new("Recents"))
            .body(
                ui_ai::PromptInputTabBody::new().children(
                    RECENT_TABS
                        .iter()
                        .enumerate()
                        .map(|(index, path)| {
                            path_item(
                                cx,
                                path,
                                match index {
                                    0 => "ui-gallery-ai-prompt-input-cursor-recent-tab-0",
                                    1 => "ui-gallery-ai-prompt-input-cursor-recent-tab-1",
                                    _ => "ui-gallery-ai-prompt-input-cursor-recent-tab-2",
                                },
                            )
                        })
                        .collect::<Vec<_>>(),
                ),
            ),
    ]);

    let tools = ui_ai::PromptInputTools::empty().child(
        ui_ai::PromptInputButton::new("Search")
            .icon(IconId::new("lucide.globe"))
            .test_id("ui-gallery-ai-prompt-input-cursor-search")
            .into_element(cx),
    );

    ui_ai::PromptInput::new_uncontrolled()
        .on_submit(on_submit)
        .test_id_root("ui-gallery-ai-prompt-input-cursor")
        .test_id_send("ui-gallery-ai-prompt-input-cursor-submit")
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(760.0)))
        .children([
            ui_ai::PromptInputPart::from(ui_ai::PromptInputHeader::new([
                ui_ai::PromptInputHoverCard::new()
                    .trigger(ui_ai::PromptInputHoverCardTrigger::new(
                        ui_ai::PromptInputButton::new("Add files")
                            .icon(IconId::new("lucide.at-sign"))
                            .size(shadcn::ButtonSize::IconSm)
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-ai-prompt-input-cursor-files-trigger")
                            .into_element(cx),
                    ))
                    .content(
                        ui_ai::PromptInputHoverCardContent::new([files_menu])
                            .refine_layout(
                                LayoutRefinement::default()
                                    .w(LengthRefinement::Px(Px(400.0).into())),
                            )
                            .refine_style(ChromeRefinement::default().p(Space::N0))
                            .test_id("ui-gallery-ai-prompt-input-cursor-files-content"),
                    )
                    .into_element(cx),
                ui_ai::PromptInputHoverCard::new()
                    .trigger(ui_ai::PromptInputHoverCardTrigger::new(
                        ui_ai::PromptInputButton::new("Rules")
                            .children([
                                decl_icon::icon(cx, IconId::new("lucide.ruler")),
                                ui::text("1").into_element(cx),
                            ])
                            .size(shadcn::ButtonSize::Sm)
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-ai-prompt-input-cursor-rules-trigger")
                            .into_element(cx),
                    ))
                    .content(
                        ui_ai::PromptInputHoverCardContent::new([rules_content])
                            .refine_style(ChromeRefinement::default().p(Space::N0))
                            .test_id("ui-gallery-ai-prompt-input-cursor-rules-content"),
                    )
                    .into_element(cx),
                ui_ai::PromptInputHoverCard::new()
                    .trigger(ui_ai::PromptInputHoverCardTrigger::new(
                        ui_ai::PromptInputButton::new("Tabs")
                            .children([
                                decl_icon::icon(cx, IconId::new("lucide.files")),
                                ui::text("1 Tab").into_element(cx),
                            ])
                            .size(shadcn::ButtonSize::Sm)
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-ai-prompt-input-cursor-tabs-trigger")
                            .into_element(cx),
                    ))
                    .content(
                        ui_ai::PromptInputHoverCardContent::new([
                            tabs_list.into_element(cx),
                            shadcn::Separator::new().into_element(cx),
                            ui::h_flex(move |cx| {
                                vec![
                                    ui::text("Only file paths are included")
                                        .text_size_px(Px(12.0))
                                        .text_color(fret_ui_kit::ColorRef::Token {
                                            key: "muted-foreground",
                                            fallback: fret_ui_kit::ColorFallback::ThemeTextMuted,
                                        })
                                        .into_element(cx),
                                ]
                            })
                            .px(Space::N3)
                            .pt(Space::N2)
                            .into_element(cx),
                        ])
                        .refine_layout(
                            LayoutRefinement::default().w(LengthRefinement::Px(Px(320.0).into())),
                        )
                        .refine_style(ChromeRefinement::default().px(Space::N0).py(Space::N3))
                        .test_id("ui-gallery-ai-prompt-input-cursor-tabs-content"),
                    )
                    .into_element(cx),
            ])),
            ui_ai::PromptInputPart::from(ui_ai::PromptInputBody::new([
                ui_ai::PromptInputTextarea::new()
                    .placeholder("Plan, search, build anything")
                    .test_id("ui-gallery-ai-prompt-input-cursor-textarea"),
            ])),
            ui_ai::PromptInputPart::from(ui_ai::PromptInputFooter::new(
                [tools],
                [ui_ai::PromptInputSubmit::new()],
            )),
        ])
        .into_element(cx)
}
// endregion: example
