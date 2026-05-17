use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::{CommandId, Effect, Model, ModelStore};
use fret_ui::element::{AnyElement, ContainerProps, PressableA11y, PressableProps, SpacerProps};
use fret_ui::scroll::{ScrollStrategy, VirtualListScrollHandle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::action_hooks::ActionHooksExt;
use crate::declarative::collection_semantics::CollectionSemanticsExt as _;
use crate::declarative::model_watch::ModelWatchExt as _;
use crate::ui;
use crate::{IntoUiElement, MetricRef, Size, Space, collect_children};

use std::sync::Arc;

type CopyTextAtFn = dyn Fn(&ModelStore, usize) -> Option<String> + Send + Sync;

fn resolve_list_colors(theme: &Theme) -> (Color, Color, Color, Color) {
    let list_bg = theme
        .color_by_key("list.background")
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_token("card"));
    let border = theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("list.border"))
        .unwrap_or_else(|| theme.color_token("border"));
    let row_hover = theme
        .color_by_key("list.hover.background")
        .or_else(|| theme.color_by_key("list.row.hover"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_token("accent"));
    let row_active = theme
        .color_by_key("list.active.background")
        .or_else(|| theme.color_by_key("list.row.active"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_token("accent"));
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

fn list_from_strings_row_contents<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &str,
    leading: &str,
    trailing: Option<&str>,
) -> Vec<AnyElement> {
    let mut out = Vec::new();
    out.push(crate::declarative::text::text_chrome_glyph(cx, leading));
    out.push(crate::declarative::text::text_list_row_label(cx, label));
    out.push(cx.spacer(SpacerProps {
        min: Px(0.0),
        ..Default::default()
    }));
    if let Some(trailing) = trailing {
        out.push(crate::declarative::text::text_control_readout(cx, trailing));
    }
    out
}

/// Declarative virtualized list helper (component-friendly, row content is fully composable).
///
/// This intentionally avoids a fixed row schema (`VirtualListRow { text/secondary/trailing... }`)
/// so higher-level shadcn-like components can be built in the component layer via composition.
#[allow(clippy::too_many_arguments)]
pub fn list_virtualized<H: UiHost, I, T>(
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
    row_contents: impl FnMut(&mut ElementContext<'_, H>, usize) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
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
pub fn list_virtualized_copyable<H: UiHost, I, T>(
    cx: &mut ElementContext<'_, H>,
    selection: Model<Option<usize>>,
    size: Size,
    row_height: Option<Px>,
    len: usize,
    overscan: usize,
    scroll_handle: &VirtualListScrollHandle,
    items_revision: u64,
    key_at: impl FnMut(usize) -> u64,
    copy_text_at: Arc<CopyTextAtFn>,
    on_select: impl Fn(usize) -> Option<CommandId>,
    row_contents: impl FnMut(&mut ElementContext<'_, H>, usize) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
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

/// Retained-host virtualized list helper (ADR 0177).
///
/// Prefer this over [`list_virtualized`] when the list is hosted inside a view-cache root and
/// scroll stability matters. The retained-host path allows window shifts to attach/detach rows
/// under cache-hit reuse (without rerendering the parent cache root).
#[allow(clippy::too_many_arguments)]
pub fn list_virtualized_retained_v0<H: UiHost + 'static, I, T>(
    cx: &mut ElementContext<'_, H>,
    selection: Option<Model<Option<usize>>>,
    size: Size,
    row_height: Option<Px>,
    len: usize,
    overscan: usize,
    scroll_handle: &VirtualListScrollHandle,
    items_revision: u64,
    key_at: impl Fn(usize) -> u64 + 'static,
    on_select: impl Fn(usize) -> Option<CommandId> + 'static,
    row_contents: impl for<'b> Fn(&mut ElementContext<'b, H>, usize) -> I + 'static,
) -> AnyElement
where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    list_virtualized_retained_impl(
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

/// Retained-host virtualized list helper that participates in cross-surface clipboard commands
/// (`edit.copy`).
#[allow(clippy::too_many_arguments)]
pub fn list_virtualized_copyable_retained_v0<H: UiHost + 'static, I, T>(
    cx: &mut ElementContext<'_, H>,
    selection: Model<Option<usize>>,
    size: Size,
    row_height: Option<Px>,
    len: usize,
    overscan: usize,
    scroll_handle: &VirtualListScrollHandle,
    items_revision: u64,
    key_at: impl Fn(usize) -> u64 + 'static,
    copy_text_at: Arc<CopyTextAtFn>,
    on_select: impl Fn(usize) -> Option<CommandId> + 'static,
    row_contents: impl for<'b> Fn(&mut ElementContext<'b, H>, usize) -> I + 'static,
) -> AnyElement
where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    list_virtualized_retained_impl(
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
fn list_virtualized_impl<H: UiHost, I, T>(
    cx: &mut ElementContext<'_, H>,
    selection: Option<Model<Option<usize>>>,
    size: Size,
    row_height: Option<Px>,
    len: usize,
    overscan: usize,
    scroll_handle: &VirtualListScrollHandle,
    items_revision: u64,
    key_at: impl FnMut(usize) -> u64,
    copy_text_at: Option<Arc<CopyTextAtFn>>,
    on_select: impl Fn(usize) -> Option<CommandId>,
    mut row_contents: impl FnMut(&mut ElementContext<'_, H>, usize) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    let selected = match &selection {
        Some(m) => cx.watch_model(m).copied_or(None),
        None => None,
    };

    if let Some(selected) = selected {
        scroll_handle.scroll_to_item(selected, ScrollStrategy::Nearest);
    }

    let theme = Theme::global(&*cx.app);
    let (list_bg, border, row_hover, row_active) = resolve_list_colors(theme);
    let radius = theme.metric_token("metric.radius.md");

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
                cx.command_on_command_for(
                    list_root,
                    Arc::new(move |host, acx, command| {
                        if command.as_str() != "edit.copy" {
                            return false;
                        }
                        let models = host.models_mut();
                        let selected = models.get_copied(&selection_for_command).unwrap_or(None);
                        if let Some(selected) = selected
                            && let Some(text) = (copy_text_for_command)(&*models, selected)
                        {
                            let token = host.next_clipboard_token();
                            host.push_effect(Effect::ClipboardWriteText {
                                window: acx.window,
                                token,
                                text,
                            });
                        }
                        true
                    }),
                );
                cx.command_on_command_availability_for(
                    list_root,
                    Arc::new(move |host, acx, command| {
                        if command.as_str() != "edit.copy" {
                            return fret_ui::CommandAvailability::NotHandled;
                        }
                        if !acx.focus_in_subtree {
                            return fret_ui::CommandAvailability::NotHandled;
                        }
                        if !acx.input_ctx.caps.clipboard.text.write {
                            return fret_ui::CommandAvailability::Blocked;
                        }
                        let models = host.models_mut();
                        let selected = models
                            .get_copied(&selection_for_availability)
                            .unwrap_or(None);
                        if selected.is_some_and(|selected| selected < len) {
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
                    let enabled = cmd.is_some() || selection.is_some();
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
                            if let Some(selection) = selection.clone() {
                                cx.pressable_set_model(&selection, Some(i));
                            }
                            let bg = if is_selected || (enabled && st.pressed) {
                                Some(row_active)
                            } else if enabled && st.hovered {
                                Some(row_hover)
                            } else {
                                None
                            };

                            vec![cx.container(
                                ContainerProps {
                                    padding: Edges::symmetric(row_px, row_py).into(),
                                    background: bg,
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![
                                        ui::h_row(|cx| row_contents(cx, i))
                                            .gap(Space::N2)
                                            .justify_start()
                                            .items_center()
                                            .into_element(cx),
                                    ]
                                },
                            )]
                        },
                    )
                }),
            ]
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn list_virtualized_retained_impl<H: UiHost + 'static, I, T>(
    cx: &mut ElementContext<'_, H>,
    selection: Option<Model<Option<usize>>>,
    size: Size,
    row_height: Option<Px>,
    len: usize,
    overscan: usize,
    scroll_handle: &VirtualListScrollHandle,
    items_revision: u64,
    key_at: impl Fn(usize) -> u64 + 'static,
    copy_text_at: Option<Arc<CopyTextAtFn>>,
    on_select: impl Fn(usize) -> Option<CommandId> + 'static,
    row_contents: impl for<'b> Fn(&mut ElementContext<'b, H>, usize) -> I + 'static,
) -> AnyElement
where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    let selected = match &selection {
        Some(m) => cx.watch_model(m).copied_or(None),
        None => None,
    };

    if let Some(selected) = selected {
        scroll_handle.scroll_to_item(selected, ScrollStrategy::Nearest);
    }

    let theme = Theme::global(&*cx.app);
    let (list_bg, border, row_hover, row_active) = resolve_list_colors(theme);
    let radius = theme.metric_token("metric.radius.md");

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
                cx.command_on_command_for(
                    list_root,
                    Arc::new(move |host, acx, command| {
                        if command.as_str() != "edit.copy" {
                            return false;
                        }
                        let models = host.models_mut();
                        let selected = models.get_copied(&selection_for_command).unwrap_or(None);
                        if let Some(selected) = selected
                            && let Some(text) = (copy_text_for_command)(&*models, selected)
                        {
                            let token = host.next_clipboard_token();
                            host.push_effect(Effect::ClipboardWriteText {
                                window: acx.window,
                                token,
                                text,
                            });
                        }
                        true
                    }),
                );
                cx.command_on_command_availability_for(
                    list_root,
                    Arc::new(move |host, acx, command| {
                        if command.as_str() != "edit.copy" {
                            return fret_ui::CommandAvailability::NotHandled;
                        }
                        if !acx.focus_in_subtree {
                            return fret_ui::CommandAvailability::NotHandled;
                        }
                        if !acx.input_ctx.caps.clipboard.text.write {
                            return fret_ui::CommandAvailability::Blocked;
                        }
                        let models = host.models_mut();
                        let selected = models
                            .get_copied(&selection_for_availability)
                            .unwrap_or(None);
                        if selected.is_some_and(|selected| selected < len) {
                            fret_ui::CommandAvailability::Available
                        } else {
                            fret_ui::CommandAvailability::Blocked
                        }
                    }),
                );
            }

            vec![cx.virtual_list_keyed_retained_fn(
                len,
                options,
                scroll_handle,
                key_at,
                move |cx, i| {
                    let cmd = on_select(i);
                    let enabled = cmd.is_some() || selection.is_some();
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
                            if let Some(selection) = selection.clone() {
                                cx.pressable_set_model(&selection, Some(i));
                            }
                            let bg = if is_selected || (enabled && st.pressed) {
                                Some(row_active)
                            } else if enabled && st.hovered {
                                Some(row_hover)
                            } else {
                                None
                            };

                            let items = row_contents(cx, i);
                            let row_children = collect_children(cx, items);

                            vec![cx.container(
                                ContainerProps {
                                    padding: Edges::symmetric(row_px, row_py).into(),
                                    background: bg,
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![
                                        ui::h_row(|_cx| row_children)
                                            .gap(Space::N2)
                                            .justify_start()
                                            .items_center()
                                            .into_element(cx),
                                    ]
                                },
                            )]
                        },
                    )
                },
            )]
        },
    )
}

/// Compatibility helper for simple string lists (used in demos).
#[track_caller]
pub fn list_from_strings<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    items: Model<Vec<String>>,
    selection: Option<Model<Option<usize>>>,
    size: Size,
    on_select: impl Fn(usize) -> Option<CommandId> + 'static,
) -> AnyElement {
    let values = cx.watch_model(&items).layout().cloned_or_default();
    let values = Arc::new(values);

    let scroll_handle = cx.slot_state(VirtualListScrollHandle::new, |h| h.clone());
    let items_revision = cx.app.models().revision(&items).unwrap_or(0);

    match selection {
        Some(selection) => list_virtualized_copyable_retained_v0(
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
                let values = Arc::clone(&values);
                Arc::new(move |_models, i| values.get(i).cloned())
            },
            on_select,
            {
                let values = Arc::clone(&values);
                move |cx, i| {
                    let label = values.get(i).map(String::as_str).unwrap_or("");
                    let leading = if i % 3 == 0 { "●" } else { "○" };
                    let trailing = (i % 5 == 0).then_some("⌘O");
                    list_from_strings_row_contents(cx, label, leading, trailing)
                }
            },
        ),
        None => list_virtualized_retained_v0(
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
            {
                let values = Arc::clone(&values);
                move |cx, i| {
                    let label = values.get(i).map(String::as_str).unwrap_or("");
                    let leading = if i % 3 == 0 { "●" } else { "○" };
                    let trailing = (i % 5 == 0).then_some("⌘O");
                    list_from_strings_row_contents(cx, label, leading, trailing)
                }
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
        TextMetrics, TextOverflow, TextService, TextWrap,
    };
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Point, Px, Rect};
    use fret_runtime::CommandId;
    use fret_ui::ThemeConfig;
    use fret_ui::element::{ElementKind, Length};
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

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    fn collect_text_elements<'a>(root: &'a AnyElement, out: &mut Vec<&'a AnyElement>) {
        if matches!(root.kind, ElementKind::Text(_)) {
            out.push(root);
        }
        for child in &root.children {
            collect_text_elements(child, out);
        }
    }

    fn first_text<'a>(root: &'a AnyElement, expected: &str) -> &'a AnyElement {
        let mut texts = Vec::new();
        collect_text_elements(root, &mut texts);
        texts
            .into_iter()
            .find(|element| matches!(&element.kind, ElementKind::Text(props) if props.text.as_ref() == expected))
            .unwrap_or_else(|| panic!("expected text element {expected:?}"))
    }

    #[test]
    fn list_from_strings_uses_shared_single_line_text_roles() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(120.0), Px(32.0)),
        );

        let row = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            ui::h_row(|cx| {
                list_from_strings_row_contents(
                    cx,
                    "A long virtualized list row label that should not wrap",
                    "●",
                    Some("⌘O"),
                )
            })
            .gap(Space::N2)
            .into_element(cx)
        });

        let ElementKind::Text(leading) = &first_text(&row, "●").kind else {
            panic!("leading glyph should be text");
        };
        assert_eq!(leading.wrap, TextWrap::None);
        assert_eq!(leading.overflow, TextOverflow::Clip);
        assert_eq!(leading.layout.size.min_width, Some(Length::Px(Px(0.0))));

        let ElementKind::Text(label) = &first_text(
            &row,
            "A long virtualized list row label that should not wrap",
        )
        .kind
        else {
            panic!("row label should be text");
        };
        assert_eq!(label.layout.size.width, Length::Fill);
        assert_eq!(label.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(label.layout.flex.shrink, 1.0);
        assert_eq!(label.wrap, TextWrap::None);
        assert_eq!(label.overflow, TextOverflow::Ellipsis);

        let ElementKind::Text(trailing) = &first_text(&row, "⌘O").kind else {
            panic!("trailing shortcut should be text");
        };
        assert_eq!(trailing.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(trailing.layout.flex.shrink, 1.0);
        assert_eq!(trailing.wrap, TextWrap::None);
        assert_eq!(trailing.overflow, TextOverflow::Ellipsis);
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
                    |cx, i| [cx.text(format!("Item {i}"))],
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
    fn list_virtualized_retained_stamps_collection_semantics_on_rows() {
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
                vec![list_virtualized_retained_v0(
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
                    |cx, i| [cx.text(format!("Item {i}"))],
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
        caps.clipboard.text.read = true;
        caps.clipboard.text.write = true;
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
                        Arc::new(|_models, i| Some(format!("Item {i}"))),
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
                .any(|e| matches!(e, fret_runtime::Effect::ClipboardWriteText { .. })),
            "expected edit.copy to not emit ClipboardWriteText when selection is empty"
        );

        app.models_mut()
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
                matches!(e, fret_runtime::Effect::ClipboardWriteText { text, .. } if text == "Item 1")
            }),
            "expected edit.copy to emit ClipboardWriteText for the selected row"
        );
    }

    #[test]
    fn list_virtualized_copyable_retained_reports_availability_and_emits_clipboard_text() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut caps = fret_runtime::PlatformCapabilities::default();
        caps.clipboard.text.read = true;
        caps.clipboard.text.write = true;
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
                    vec![list_virtualized_copyable_retained_v0(
                        cx,
                        selection.clone(),
                        Size::Medium,
                        None,
                        3,
                        2,
                        &scroll_handle,
                        0,
                        |i| i as u64,
                        Arc::new(|_models, i| Some(format!("Item {i}"))),
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
                .any(|e| matches!(e, fret_runtime::Effect::ClipboardWriteText { .. })),
            "expected edit.copy to not emit ClipboardWriteText when selection is empty"
        );

        app.models_mut()
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
                matches!(e, fret_runtime::Effect::ClipboardWriteText { text, .. } if text == "Item 1")
            }),
            "expected edit.copy to emit ClipboardWriteText for the selected row"
        );
    }
}
