use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::{CommandId, Effect, Model};
use fret_ui::element::{AnyElement, ContainerProps, PressableA11y, PressableProps, SpacerProps};
use fret_ui::scroll::{ScrollStrategy, VirtualListScrollHandle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::action_hooks::ActionHooksExt;
use crate::declarative::collection_semantics::CollectionSemanticsExt as _;
use crate::declarative::model_watch::ModelWatchExt as _;
use crate::declarative::stack;
use crate::{Items, Justify, MetricRef, Size, Space};

use std::sync::Arc;

fn resolve_list_colors(theme: &Theme) -> (Color, Color, Color, Color) {
    let list_bg = theme
        .color_by_key("list.background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_required("card"));
    let border = theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("list.border"))
        .unwrap_or_else(|| theme.color_required("border"));
    let row_hover = theme
        .color_by_key("list.hover.background")
        .or_else(|| theme.color_by_key("list.row.hover"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_required("accent"));
    let row_active = theme
        .color_by_key("list.active.background")
        .or_else(|| theme.color_by_key("list.row.active"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_required("accent"));
    (list_bg, border, row_hover, row_active)
}

fn resolve_row_height(theme: &Theme, size: Size) -> Px {
    let base = theme
        .metric_by_key("component.list.row_height")
        .unwrap_or_else(|| size.list_row_h(theme));
    Px(base.0.max(0.0))
}

fn resolve_row_padding_x(theme: &Theme) -> Px {
    // Prefer component-level Tailwind-like tokens; fall back to baseline metrics to avoid drift.
    MetricRef::space(Space::N2p5).resolve(theme)
}

fn resolve_row_padding_y(theme: &Theme) -> Px {
    MetricRef::space(Space::N1p5).resolve(theme)
}

/// Declarative virtualized list helper (component-friendly, row content is fully composable).
///
/// This intentionally avoids a fixed row schema (`VirtualListRow { text/secondary/trailing... }`)
/// so higher-level shadcn-like components can be built in the component layer via composition.
#[allow(clippy::too_many_arguments)]
pub fn list_virtualized<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selection: Option<Model<Option<usize>>>,
    size: Size,
    row_height: Option<Px>,
    len: usize,
    overscan: usize,
    scroll_handle: &VirtualListScrollHandle,
    items_revision: u64,
    key_at: impl FnMut(usize) -> u64,
    on_select: impl Fn(usize) -> Option<CommandId>,
    row_contents: impl FnMut(&mut ElementContext<'_, H>, usize) -> Vec<AnyElement>,
) -> AnyElement {
    list_virtualized_impl(
        cx,
        selection,
        size,
        row_height,
        len,
        overscan,
        scroll_handle,
        items_revision,
        key_at,
        None,
        on_select,
        row_contents,
    )
}

/// Virtualized list helper that participates in cross-surface clipboard commands (`edit.copy`).
///
/// This is intended for non-text selection surfaces (lists, tables, node graphs) that want to share
/// command IDs and OS/menu gating with text inputs.
#[allow(clippy::too_many_arguments)]
pub fn list_virtualized_copyable<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selection: Model<Option<usize>>,
    size: Size,
    row_height: Option<Px>,
    len: usize,
    overscan: usize,
    scroll_handle: &VirtualListScrollHandle,
    items_revision: u64,
    key_at: impl FnMut(usize) -> u64,
    copy_text_at: Arc<dyn Fn(usize) -> Option<String> + Send + Sync>,
    on_select: impl Fn(usize) -> Option<CommandId>,
    row_contents: impl FnMut(&mut ElementContext<'_, H>, usize) -> Vec<AnyElement>,
) -> AnyElement {
    list_virtualized_impl(
        cx,
        Some(selection),
        size,
        row_height,
        len,
        overscan,
        scroll_handle,
        items_revision,
        key_at,
        Some(copy_text_at),
        on_select,
        row_contents,
    )
}

#[allow(clippy::too_many_arguments)]
fn list_virtualized_impl<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selection: Option<Model<Option<usize>>>,
    size: Size,
    row_height: Option<Px>,
    len: usize,
    overscan: usize,
    scroll_handle: &VirtualListScrollHandle,
    items_revision: u64,
    key_at: impl FnMut(usize) -> u64,
    copy_text_at: Option<Arc<dyn Fn(usize) -> Option<String> + Send + Sync>>,
    on_select: impl Fn(usize) -> Option<CommandId>,
    mut row_contents: impl FnMut(&mut ElementContext<'_, H>, usize) -> Vec<AnyElement>,
) -> AnyElement {
    let selected = match &selection {
        Some(m) => cx.watch_model(m).copied().unwrap_or(None),
        None => None,
    };

    if let Some(selected) = selected {
        scroll_handle.scroll_to_item(selected, ScrollStrategy::Nearest);
    }

    let theme = Theme::global(&*cx.app);
    let (list_bg, border, row_hover, row_active) = resolve_list_colors(theme);
    let radius = theme.metric_required("metric.radius.md");

    let row_h = row_height.unwrap_or_else(|| resolve_row_height(theme, size));
    let row_px = resolve_row_padding_x(theme);
    let row_py = resolve_row_padding_y(theme);

    let mut options = fret_ui::element::VirtualListOptions::new(row_h, overscan);
    options.items_revision = items_revision;
    let set_size = len;

    cx.container(
        ContainerProps {
            background: Some(list_bg),
            border: Edges::all(Px(1.0)),
            border_color: Some(border),
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        |cx| {
            let list_root = cx.root_id();
            if let (Some(selection), Some(copy_text_at)) = (selection.clone(), copy_text_at.clone())
            {
                let selection_for_command = selection.clone();
                let selection_for_availability = selection;
                let copy_text_for_command = copy_text_at.clone();
                cx.command_add_on_command_for(
                    list_root,
                    Arc::new(move |host, _acx, command| {
                        if command.as_str() != "edit.copy" {
                            return false;
                        }
                        let selected = host
                            .models_mut()
                            .get_copied(&selection_for_command)
                            .unwrap_or(None);
                        if let Some(selected) = selected {
                            if let Some(text) = (copy_text_for_command)(selected) {
                                host.push_effect(Effect::ClipboardSetText { text });
                            }
                        }
                        true
                    }),
                );
                cx.command_add_on_command_availability_for(
                    list_root,
                    Arc::new(move |host, acx, command| {
                        if command.as_str() != "edit.copy" {
                            return fret_ui::CommandAvailability::NotHandled;
                        }
                        if !acx.focus_in_subtree {
                            return fret_ui::CommandAvailability::NotHandled;
                        }
                        if !acx.input_ctx.caps.clipboard.text {
                            return fret_ui::CommandAvailability::Blocked;
                        }
                        let selected = host
                            .models_mut()
                            .get_copied(&selection_for_availability)
                            .unwrap_or(None);
                        if selected.is_some() {
                            fret_ui::CommandAvailability::Available
                        } else {
                            fret_ui::CommandAvailability::Blocked
                        }
                    }),
                );
            }
            vec![
                cx.virtual_list_keyed(len, options, scroll_handle, key_at, |cx, i| {
                    let cmd = on_select(i);
                    let enabled = cmd.is_some();
                    let is_selected = selected == Some(i);

                    cx.pressable(
                        PressableProps {
                            enabled,
                            a11y: PressableA11y {
                                role: Some(SemanticsRole::ListItem),
                                selected: is_selected,
                                ..Default::default()
                            }
                            .with_collection_position(i, set_size),
                            ..Default::default()
                        },
                        |cx, st| {
                            cx.pressable_dispatch_command_if_enabled_opt(cmd);
                            let bg = if is_selected || (enabled && st.pressed) {
                                Some(row_active)
                            } else if enabled && st.hovered {
                                Some(row_hover)
                            } else {
                                None
                            };

                            vec![cx.container(
                                ContainerProps {
                                    padding: Edges::symmetric(row_px, row_py),
                                    background: bg,
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![stack::hstack(
                                        cx,
                                        stack::HStackProps::default()
                                            .gap_x(Space::N2)
                                            .justify(Justify::Start)
                                            .items(Items::Center),
                                        |cx| row_contents(cx, i),
                                    )]
                                },
                            )]
                        },
                    )
                }),
            ]
        },
    )
}

/// Compatibility helper for simple string lists (used in demos).
pub fn list_from_strings<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    items: Model<Vec<String>>,
    selection: Option<Model<Option<usize>>>,
    size: Size,
    on_select: impl Fn(usize) -> Option<CommandId>,
) -> AnyElement {
    let values = cx.watch_model(&items).layout().cloned().unwrap_or_default();
    let values = Arc::new(values);

    let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());
    let items_revision = cx.app.models().revision(&items).unwrap_or(0);

    match selection {
        Some(selection) => list_virtualized_copyable(
            cx,
            selection,
            size,
            None,
            values.len(),
            2,
            &scroll_handle,
            items_revision,
            |i| i as u64,
            {
                let values = values.clone();
                Arc::new(move |i| values.get(i).cloned())
            },
            on_select,
            |cx, i| {
                let label = values.get(i).map(String::as_str).unwrap_or("");
                let leading = if i % 3 == 0 { "●" } else { "○" };
                let trailing = if i % 5 == 0 { "⌘O" } else { "" };

                let mut out = Vec::new();
                out.push(cx.text(leading));
                out.push(cx.text(label));
                out.push(cx.spacer(SpacerProps {
                    min: Px(0.0),
                    ..Default::default()
                }));
                if !trailing.is_empty() {
                    out.push(cx.text(trailing));
                }
                out
            },
        ),
        None => list_virtualized(
            cx,
            None,
            size,
            None,
            values.len(),
            2,
            &scroll_handle,
            items_revision,
            |i| i as u64,
            on_select,
            |cx, i| {
                let label = values.get(i).map(String::as_str).unwrap_or("");
                let leading = if i % 3 == 0 { "●" } else { "○" };
                let trailing = if i % 5 == 0 { "⌘O" } else { "" };

                let mut out = Vec::new();
                out.push(cx.text(leading));
                out.push(cx.text(label));
                out.push(cx.spacer(SpacerProps {
                    min: Px(0.0),
                    ..Default::default()
                }));
                if !trailing.is_empty() {
                    out.push(cx.text(trailing));
                }
                out
            },
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, SvgId, SvgService, TextBlobId, TextConstraints, TextInput,
        TextMetrics, TextService,
    };
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Point, Px, Rect};
    use fret_runtime::CommandId;
    use fret_ui::ThemeConfig;
    use fret_ui::{Theme, UiTree};

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    #[test]
    fn list_virtualized_stamps_collection_semantics_on_rows() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let selection = app.models_mut().insert(Some(1usize));
        let scroll_handle = VirtualListScrollHandle::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![list_virtualized(
                    cx,
                    Some(selection.clone()),
                    Size::Medium,
                    None,
                    3,
                    2,
                    &scroll_handle,
                    0,
                    |i| i as u64,
                    |_i| Some(CommandId::new("noop")),
                    |cx, i| vec![cx.text(format!("Item {i}"))],
                )]
            })
        };

        // VirtualList computes the visible window based on viewport metrics populated during layout,
        // so it takes two frames for the first set of rows to mount.
        for _ in 0..2 {
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        }

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let items = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::ListItem)
            .collect::<Vec<_>>();

        assert_eq!(items.len(), 3);
        for (index, node) in items.iter().enumerate() {
            assert_eq!(node.pos_in_set, Some((index + 1) as u32));
            assert_eq!(node.set_size, Some(3));
        }

        assert!(
            items[1].flags.selected,
            "selected row should set semantics selected flag"
        );
    }

    #[test]
    fn list_virtualized_copyable_reports_availability_and_emits_clipboard_text() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut caps = fret_runtime::PlatformCapabilities::default();
        caps.clipboard.text = true;
        app.set_global(caps);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let selection = app.models_mut().insert(Option::<usize>::None);
        let scroll_handle = VirtualListScrollHandle::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let root = {
            let render = |ui: &mut UiTree<App>,
                          app: &mut App,
                          services: &mut FakeServices|
             -> fret_core::NodeId {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                    vec![list_virtualized_copyable(
                        cx,
                        selection.clone(),
                        Size::Medium,
                        None,
                        3,
                        2,
                        &scroll_handle,
                        0,
                        |i| i as u64,
                        Arc::new(|i| Some(format!("Item {i}"))),
                        |_i| Some(CommandId::new("noop")),
                        |cx, i| vec![cx.text(format!("Item {i}"))],
                    )]
                })
            };

            // VirtualList computes the visible window based on viewport metrics populated during layout,
            // so it takes two frames for the first set of rows to mount.
            let mut root = fret_core::NodeId::default();
            for _ in 0..2 {
                root = render(&mut ui, &mut app, &mut services);
                ui.set_root(root);
                ui.layout_all(&mut app, &mut services, bounds, 1.0);
                let mut scene = fret_core::Scene::default();
                ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
            }
            root
        };

        let list_container = ui.children(root)[0];
        ui.set_focus(Some(list_container));

        let copy = CommandId::from("edit.copy");
        assert!(
            !ui.is_command_available(&mut app, &copy),
            "expected edit.copy to be unavailable when selection is empty"
        );
        assert!(
            ui.dispatch_command(&mut app, &mut services, &copy),
            "expected edit.copy to be handled by the list surface"
        );
        let effects = app.flush_effects();
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, fret_runtime::Effect::ClipboardSetText { .. })),
            "expected edit.copy to not emit ClipboardSetText when selection is empty"
        );

        let _ = app
            .models_mut()
            .update(&selection, |v| *v = Some(1))
            .expect("selection update");

        assert!(
            ui.is_command_available(&mut app, &copy),
            "expected edit.copy to be available when selection is non-empty"
        );
        assert!(
            ui.dispatch_command(&mut app, &mut services, &copy),
            "expected edit.copy to be handled by the list surface"
        );
        let effects = app.flush_effects();
        assert!(
            effects.iter().any(|e| {
                matches!(e, fret_runtime::Effect::ClipboardSetText { text } if text == "Item 1")
            }),
            "expected edit.copy to emit ClipboardSetText for the selected row"
        );
    }
}
