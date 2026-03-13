pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_core::scene::DashPatternV1;
use fret_runtime::CommandId;
use fret_ui::Theme;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct BrowserMenuState {
    show_bookmarks: bool,
    show_full_urls: bool,
    person: Option<Arc<str>>,
}

fn trigger_surface<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let theme = Theme::global(&*cx.app);
    let border = theme.color_token("border");
    let bg = theme.color_token("background");
    let fg = theme.color_token("muted-foreground");

    let label = ui::text("Right click here")
        .text_sm()
        .text_color(ColorRef::Color(fg))
        .into_element(cx);

    let content = ui::v_flex(move |_cx| vec![label])
        .layout(LayoutRefinement::default().w_full().h_full())
        .items_center()
        .justify_center()
        .into_element(cx);

    shadcn::AspectRatio::with_child(content)
        .ratio(2.0)
        .refine_style(
            ChromeRefinement::default()
                .rounded(Radius::Md)
                .border_1()
                .border_dash(DashPatternV1::new(Px(4.0), Px(4.0), Px(0.0)))
                .border_color(ColorRef::Color(border))
                .bg(ColorRef::Color(bg)),
        )
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(300.0)))
        .test_id(test_id)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let menu_state = cx.local_model(|| BrowserMenuState {
        show_bookmarks: true,
        show_full_urls: false,
        person: Some(Arc::<str>::from("pedro")),
    });
    let menu_state_now = cx
        .watch_model(&menu_state)
        .layout()
        .cloned()
        .unwrap_or_default();
    let trigger = shadcn::ContextMenuTrigger::build(trigger_surface(
        cx,
        "ui-gallery-context-menu-demo-trigger",
    ));

    shadcn::ContextMenu::uncontrolled(cx)
        .content_test_id("ui-gallery-context-menu-demo-content")
        .build_parts(
            cx,
            trigger,
            shadcn::ContextMenuContent::new()
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0)),
            |cx| {
                vec![
                    shadcn::ContextMenuItem::new("Back")
                        .action(CommandId::new("ui_gallery.context_menu.demo.back"))
                        .inset(true)
                        .trailing(shadcn::ContextMenuShortcut::new("⌘[").into_element(cx))
                        .test_id("ui-gallery-context-menu-demo-back")
                        .into(),
                    shadcn::ContextMenuItem::new("Forward")
                        .action(CommandId::new("ui_gallery.context_menu.demo.forward"))
                        .inset(true)
                        .disabled(true)
                        .trailing(shadcn::ContextMenuShortcut::new("⌘]").into_element(cx))
                        .test_id("ui-gallery-context-menu-demo-forward")
                        .into(),
                    shadcn::ContextMenuItem::new("Reload")
                        .action(CommandId::new("ui_gallery.context_menu.demo.reload"))
                        .inset(true)
                        .trailing(shadcn::ContextMenuShortcut::new("⌘R").into_element(cx))
                        .test_id("ui-gallery-context-menu-demo-reload")
                        .into(),
                    shadcn::ContextMenuSub::new(
                        shadcn::ContextMenuSubTrigger::new("More Tools").refine(|item| {
                            item.inset(true)
                                .test_id("ui-gallery-context-menu-demo-more-tools")
                        }),
                        shadcn::ContextMenuSubContent::new(vec![
                            shadcn::ContextMenuItem::new("Save Page...")
                                .action(CommandId::new("ui_gallery.context_menu.demo.save_page"))
                                .test_id("ui-gallery-context-menu-demo-save-page")
                                .into(),
                            shadcn::ContextMenuItem::new("Create Shortcut...")
                                .action(CommandId::new(
                                    "ui_gallery.context_menu.demo.create_shortcut",
                                ))
                                .test_id("ui-gallery-context-menu-demo-create-shortcut")
                                .into(),
                            shadcn::ContextMenuItem::new("Name Window...")
                                .action(CommandId::new("ui_gallery.context_menu.demo.name_window"))
                                .test_id("ui-gallery-context-menu-demo-name-window")
                                .into(),
                            shadcn::ContextMenuSeparator::new().into(),
                            shadcn::ContextMenuItem::new("Developer Tools")
                                .action(CommandId::new(
                                    "ui_gallery.context_menu.demo.developer_tools",
                                ))
                                .test_id("ui-gallery-context-menu-demo-developer-tools")
                                .into(),
                            shadcn::ContextMenuSeparator::new().into(),
                            shadcn::ContextMenuItem::new("Delete")
                                .action(CommandId::new("ui_gallery.context_menu.demo.delete"))
                                .variant(fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive)
                                .test_id("ui-gallery-context-menu-demo-delete")
                                .into(),
                        ]),
                    )
                    .into_entry(),
                    shadcn::ContextMenuSeparator::new().into(),
                    shadcn::ContextMenuCheckboxItem::from_checked(
                        menu_state_now.show_bookmarks,
                        "Show Bookmarks",
                    )
                        .on_checked_change({
                            let menu_state = menu_state.clone();
                            move |host, _action_cx, checked| {
                                let _ = host.models_mut().update(&menu_state, |state| {
                                    state.show_bookmarks = checked;
                                });
                            }
                        })
                        .action(CommandId::new(
                            "ui_gallery.context_menu.demo.show_bookmarks",
                        ))
                        .test_id("ui-gallery-context-menu-demo-show-bookmarks")
                        .into(),
                    shadcn::ContextMenuCheckboxItem::from_checked(
                        menu_state_now.show_full_urls,
                        "Show Full URLs",
                    )
                        .on_checked_change({
                            let menu_state = menu_state.clone();
                            move |host, _action_cx, checked| {
                                let _ = host.models_mut().update(&menu_state, |state| {
                                    state.show_full_urls = checked;
                                });
                            }
                        })
                        .action(CommandId::new(
                            "ui_gallery.context_menu.demo.show_full_urls",
                        ))
                        .test_id("ui-gallery-context-menu-demo-show-full-urls")
                        .into(),
                    shadcn::ContextMenuSeparator::new().into(),
                    shadcn::ContextMenuLabel::new("People").inset(true).into(),
                    shadcn::ContextMenuRadioGroup::from_value(menu_state_now.person.clone())
                        .on_value_change({
                            let menu_state = menu_state.clone();
                            move |host, _action_cx, value| {
                                let _ = host
                                    .models_mut()
                                    .update(&menu_state, |state| state.person = Some(value));
                            }
                        })
                        .item(
                            shadcn::ContextMenuRadioItemSpec::new("pedro", "Pedro Duarte")
                                .action(CommandId::new("ui_gallery.context_menu.demo.person.pedro"))
                                .test_id("ui-gallery-context-menu-demo-person-pedro"),
                        )
                        .item(
                            shadcn::ContextMenuRadioItemSpec::new("colm", "Colm Tuite")
                                .action(CommandId::new("ui_gallery.context_menu.demo.person.colm"))
                                .test_id("ui-gallery-context-menu-demo-person-colm"),
                        )
                        .into(),
                ]
            },
        )
        .test_id("ui-gallery-context-menu-demo")
}
// endregion: example
