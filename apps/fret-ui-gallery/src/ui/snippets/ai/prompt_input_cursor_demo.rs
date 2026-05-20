pub const SOURCE: &str = include_str!("prompt_input_cursor_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_icons::IconId;
use fret_ui::Theme;
use fret_ui::element::{AnyElement, ElementKind};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, LengthRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

const ACTIVE_TABS: &[&str] = &["packages/elements/src/task-queue-panel.tsx"];
const RECENT_TABS: &[&str] = &[
    "apps/test/app/examples/task-queue-panel.tsx",
    "apps/test/app/page.tsx",
    "packages/elements/src/task.tsx",
];

fn muted_foreground(cx: &AppComponentCx<'_>) -> fret_core::Color {
    Theme::global(&*cx.app).color_token("muted-foreground")
}

fn apply_text_layout(
    cx: &AppComponentCx<'_>,
    mut element: AnyElement,
    refinement: LayoutRefinement,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    match &mut element.kind {
        ElementKind::Text(props) => {
            decl_style::apply_layout_refinement(theme, refinement, &mut props.layout);
        }
        ElementKind::StyledText(props) => {
            decl_style::apply_layout_refinement(theme, refinement, &mut props.layout);
        }
        ElementKind::SelectableText(props) => {
            decl_style::apply_layout_refinement(theme, refinement, &mut props.layout);
        }
        _ => {}
    }
    element
}

fn source_item(
    cx: &mut AppComponentCx<'_>,
    title: &'static str,
    filename: &'static str,
    test_id: &'static str,
) -> ui_ai::PromptInputCommandItem {
    let muted = muted_foreground(cx);
    let text = ui::v_flex(move |cx| {
        vec![
            decl_text::text_list_row_label(cx, title),
            decl_text::text_code_label(cx, filename).inherit_foreground(muted),
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
    cx: &mut AppComponentCx<'_>,
    path: &'static str,
    test_id: &'static str,
) -> ui_ai::PromptInputTabItem {
    ui_ai::PromptInputTabItem::new([decl_icon::icon(cx, IconId::new("lucide.globe")), {
        let label = decl_text::text_code_label(cx, path);
        apply_text_layout(cx, label, LayoutRefinement::default().min_w_0().flex_1())
    }])
    .test_id(test_id)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let on_submit: ui_ai::OnPromptInputSubmit = Arc::new(|_host, _action_cx, _message, _reason| {});
    let muted = muted_foreground(cx);

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
                                decl_text::text_list_row_label(cx, "Active Tabs"),
                                {
                                    let check = decl_text::text_control_readout(cx, "✓");
                                    apply_text_layout(
                                        cx,
                                        check,
                                        LayoutRefinement::default().ml_auto(),
                                    )
                                    .inherit_foreground(muted)
                                },
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
                    decl_text::text_section_chrome_label(cx, "Attached Project Rules")
                        .inherit_foreground(muted),
                    {
                        let readout = decl_text::text_control_readout(cx, "Always Apply:")
                            .inherit_foreground(muted);
                        apply_text_layout(cx, readout, LayoutRefinement::default().ml(Space::N4))
                    },
                    {
                        let label = decl_text::text_code_label(cx, "ultracite.mdc");
                        apply_text_layout(cx, label, LayoutRefinement::default().ml(Space::N8))
                    },
                ]
            })
            .gap(Space::N2)
            .p(Space::N3)
            .into_element(cx),
            shadcn::Separator::new().into_element(cx),
            ui::h_flex(move |cx| {
                vec![
                    decl_text::text_control_readout(cx, "Click to manage")
                        .inherit_foreground(muted),
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
                                decl_text::text_button_label(cx, "1"),
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
                                decl_text::text_button_label(cx, "1 Tab"),
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
                                    decl_text::text_control_readout(
                                        cx,
                                        "Only file paths are included",
                                    )
                                    .inherit_foreground(muted),
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
