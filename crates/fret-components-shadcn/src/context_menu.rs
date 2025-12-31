use std::sync::Arc;

use fret_components_ui::declarative::action_hooks::ActionHooksExt as _;
use fret_components_ui::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_components_ui::declarative::model_watch::ModelWatchExt as _;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::overlay;
use fret_components_ui::{MetricRef, OverlayController, OverlayPresence, OverlayRequest, Space};
use fret_core::{
    Edges, KeyCode, MouseButton, Px, SemanticsRole, Size, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::action::PointerDownCx;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, InsetStyle, LayoutStyle, Length, MainAlign,
    Overflow, PointerRegionProps, PointerRegionState, PositionStyle, PressableA11y, PressableProps,
    RovingFlexProps, RovingFocusProps, SemanticsProps, SizeStyle, TextProps,
};
use fret_ui::overlay_placement::{Align, Side, anchored_panel_bounds_sized};
use fret_ui::{ElementCx, Theme, UiHost};

use crate::dropdown_menu::{DropdownMenuAlign, DropdownMenuSide};

#[derive(Debug, Clone)]
pub enum ContextMenuEntry {
    Item(ContextMenuItem),
    Separator,
}

#[derive(Debug, Clone)]
pub struct ContextMenuItem {
    pub label: Arc<str>,
    pub disabled: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
}

impl ContextMenuItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            command: None,
            a11y_label: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }
}

/// shadcn/ui `ContextMenu` root (v4).
///
/// This is a dismissible popover (non-modal) opened by a component-owned pointer policy:
/// - right click
/// - (macOS) ctrl + left click
///
/// Notes:
/// - Position is anchored at the last pointer-down location observed within the trigger region.
/// - Keyboard invocation (Menu key / Shift+F10) is not implemented yet.
#[derive(Clone)]
pub struct ContextMenu {
    open: Model<bool>,
    align: DropdownMenuAlign,
    side: DropdownMenuSide,
    side_offset: Px,
    window_margin: Px,
    typeahead_timeout_ticks: u64,
}

impl std::fmt::Debug for ContextMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextMenu")
            .field("open", &"<model>")
            .field("align", &self.align)
            .field("side", &self.side)
            .field("side_offset", &self.side_offset)
            .field("window_margin", &self.window_margin)
            .field("typeahead_timeout_ticks", &self.typeahead_timeout_ticks)
            .finish()
    }
}

impl ContextMenu {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            align: DropdownMenuAlign::Start,
            side: DropdownMenuSide::Bottom,
            side_offset: Px(4.0),
            window_margin: Px(8.0),
            typeahead_timeout_ticks: 30,
        }
    }

    pub fn align(mut self, align: DropdownMenuAlign) -> Self {
        self.align = align;
        self
    }

    pub fn side(mut self, side: DropdownMenuSide) -> Self {
        self.side = side;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin = margin;
        self
    }

    pub fn typeahead_timeout_ticks(mut self, ticks: u64) -> Self {
        self.typeahead_timeout_ticks = ticks;
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementCx<'_, H>,
        trigger: impl FnOnce(&mut ElementCx<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<ContextMenuEntry>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let is_open = cx.watch_model(&self.open).copied().unwrap_or(false);

            let id = cx.root_id();
            let trigger = trigger(cx);
            let trigger_id = trigger.id;

            let open = self.open;
            let open_for_pointer = open.clone();
            let pointer_policy = Arc::new(move |host: &mut dyn fret_ui::action::UiPointerActionHost,
                                           _cx: fret_ui::action::ActionCx,
                                           down: PointerDownCx| {
                let is_right_click = down.button == MouseButton::Right;
                let is_macos_ctrl_click =
                    cfg!(target_os = "macos") && down.button == MouseButton::Left && down.modifiers.ctrl;

                if !is_right_click && !is_macos_ctrl_click {
                    return false;
                }

                let _ = host.models_mut().update(&open_for_pointer, |v| *v = true);
                true
            });

            let trigger = cx.pointer_region(PointerRegionProps::default(), move |cx| {
                cx.pointer_region_on_pointer_down(pointer_policy);
                vec![trigger]
            });

            let key_open = open.clone();
            cx.key_on_key_down_for(
                trigger_id,
                Arc::new(move |host, _cx, down| {
                    if down.repeat {
                        return false;
                    }
                    let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
                    if !is_shift_f10 {
                        return false;
                    }
                    let _ = host.models_mut().update(&key_open, |v| *v = true);
                    true
                }),
            );

            let pointer_down = cx.with_state(PointerRegionState::default, |st| st.last_down);
            let anchor_point = pointer_down.map(|it| it.position);

            if is_open {
                let overlay_root_name = format!("window-overlays.popover.{:x}", id.0);

                let align = self.align;
                let side = self.side;
                let side_offset = self.side_offset;
                let window_margin = self.window_margin;
                let typeahead_timeout_ticks = self.typeahead_timeout_ticks;
                let open_for_overlay = open.clone();

                let overlay_children = cx.with_root_name(&overlay_root_name, move |cx| {
                    let trigger_bounds =
                        overlay::anchor_bounds_for_element(cx, trigger_id);
                    let anchor = anchor_point.or_else(|| trigger_bounds.map(|r| r.origin));
                    let Some(anchor) = anchor else {
                        return Vec::new();
                    };

                    let entries = entries(cx);
                    let item_count = entries
                        .iter()
                        .filter(|e| matches!(e, ContextMenuEntry::Item(_)))
                        .count();
                    let (labels, disabled_flags): (Vec<Arc<str>>, Vec<bool>) = entries
                        .iter()
                        .map(|e| match e {
                            ContextMenuEntry::Item(item) => (item.label.clone(), item.disabled),
                            ContextMenuEntry::Separator => (Arc::from(""), true),
                        })
                        .unzip();

                    let labels_arc: Arc<[Arc<str>]> = Arc::from(labels.into_boxed_slice());
                    let disabled_arc: Arc<[bool]> = Arc::from(disabled_flags.into_boxed_slice());

                    let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

                    let estimated = Size::new(Px(220.0), Px(200.0));

                    let align = match align {
                        DropdownMenuAlign::Start => Align::Start,
                        DropdownMenuAlign::Center => Align::Center,
                        DropdownMenuAlign::End => Align::End,
                    };
                    let side = match side {
                        DropdownMenuSide::Top => Side::Top,
                        DropdownMenuSide::Right => Side::Right,
                        DropdownMenuSide::Bottom => Side::Bottom,
                        DropdownMenuSide::Left => Side::Left,
                    };

                    let anchor_rect = fret_core::Rect::new(anchor, Size::new(Px(1.0), Px(1.0)));
                    let placed = anchored_panel_bounds_sized(
                        outer,
                        anchor_rect,
                        estimated,
                        side_offset,
                        side,
                        align,
                    );

                    let border = theme
                        .color_by_key("border")
                        .unwrap_or(theme.colors.panel_border);
                    let shadow = decl_style::shadow_sm(&theme, theme.metrics.radius_sm);
                    let ring = decl_style::focus_ring(&theme, theme.metrics.radius_sm);
                    let pad_x = MetricRef::space(Space::N3).resolve(&theme);
                    let pad_y = MetricRef::space(Space::N2).resolve(&theme);

                    let content = cx.semantics(
                        SemanticsProps {
                            layout: LayoutStyle::default(),
                            role: SemanticsRole::Menu,
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.container(
                                ContainerProps {
                                    layout: LayoutStyle {
                                        position: PositionStyle::Absolute,
                                        inset: InsetStyle {
                                            left: Some(placed.origin.x),
                                            top: Some(placed.origin.y),
                                            ..Default::default()
                                        },
                                        size: SizeStyle {
                                            width: Length::Px(placed.size.width),
                                            height: Length::Px(placed.size.height),
                                            ..Default::default()
                                        },
                                        overflow: Overflow::Clip,
                                        ..Default::default()
                                    },
                                    padding: Edges::all(Px(4.0)),
                                    background: Some(theme.colors.panel_background),
                                    shadow: Some(shadow),
                                    border: Edges::all(Px(1.0)),
                                    border_color: Some(border),
                                    corner_radii: fret_core::Corners::all(theme.metrics.radius_sm),
                                },
                                move |cx| {
                                    vec![cx.roving_flex(
                                        RovingFlexProps {
                                            flex: FlexProps {
                                                layout: LayoutStyle::default(),
                                                direction: fret_core::Axis::Vertical,
                                                gap: Px(0.0),
                                                padding: Edges::all(Px(0.0)),
                                                justify: MainAlign::Start,
                                                align: CrossAlign::Stretch,
                                                wrap: false,
                                            },
                                            roving: RovingFocusProps {
                                                enabled: true,
                                                wrap: true,
                                                disabled: disabled_arc.clone(),
                                                ..Default::default()
                                            },
                                        },
                                        move |cx| {
                                            cx.roving_typeahead_prefix_arc_str(
                                                labels_arc.clone(),
                                                typeahead_timeout_ticks,
                                            );

                                            let text_style = TextStyle {
                                                font: fret_core::FontId::default(),
                                                size: theme.metrics.font_size,
                                                weight: fret_core::FontWeight::NORMAL,
                                                line_height: Some(theme.metrics.font_line_height),
                                                letter_spacing_em: None,
                                            };

                                            let mut out: Vec<AnyElement> =
                                                Vec::with_capacity(entries.len());

                                            let mut item_ix: usize = 0;
                                            for entry in entries.clone() {
                                                match entry {
                                                    ContextMenuEntry::Separator => {
                                                        out.push(cx.container(
                                                            ContainerProps {
                                                                layout: {
                                                                    let mut layout =
                                                                        LayoutStyle::default();
                                                                    layout.size.width = Length::Fill;
                                                                    layout.size.height =
                                                                        Length::Px(Px(1.0));
                                                                    layout
                                                                },
                                                                padding: Edges::all(Px(0.0)),
                                                                background: Some(border),
                                                                ..Default::default()
                                                            },
                                                            |_cx| Vec::new(),
                                                        ));
                                                    }
                                                    ContextMenuEntry::Item(item) => {
                                                        let collection_index = item_ix;
                                                        item_ix = item_ix.saturating_add(1);

                                                        let label = item.label.clone();
                                                        let a11y_label = item
                                                            .a11y_label
                                                            .clone()
                                                            .or_else(|| Some(label.clone()));
                                                        let disabled = item.disabled;
                                                        let command = item.command;
                                                        let open = open_for_overlay.clone();
                                                        let text_style = text_style.clone();

                                                        out.push(cx.pressable(
                                                            PressableProps {
                                                                layout: {
                                                                    let mut layout =
                                                                        LayoutStyle::default();
                                                                    layout.size.width = Length::Fill;
                                                                    layout.size.min_height =
                                                                        Some(Px(28.0));
                                                                    layout
                                                                },
                                                                enabled: !disabled,
                                                                focusable: !disabled,
                                                                focus_ring: Some(ring),
                                                                a11y: PressableA11y {
                                                                    role: Some(SemanticsRole::MenuItem),
                                                                    label: a11y_label,
                                                                    ..Default::default()
                                                                }
                                                                .with_collection_position(
                                                                    collection_index,
                                                                    item_count,
                                                                ),
                                                                ..Default::default()
                                                            },
                                                            move |cx, st| {
                                                                cx.pressable_dispatch_command_opt(command);
                                                                if !disabled {
                                                                    cx.pressable_set_bool(&open, false);
                                                                }

                                                                let theme = Theme::global(&*cx.app).clone();
                                                                let mut bg = fret_core::Color::TRANSPARENT;
                                                                if st.hovered || st.pressed {
                                                                    bg = theme
                                                                        .color_by_key("muted")
                                                                        .unwrap_or(theme.colors.hover_background);
                                                                }

                                                                vec![cx.container(
                                                                    ContainerProps {
                                                                        layout: LayoutStyle::default(),
                                                                        padding: Edges {
                                                                            top: pad_y,
                                                                            right: pad_x,
                                                                            bottom: pad_y,
                                                                            left: pad_x,
                                                                        },
                                                                        background: Some(bg),
                                                                        corner_radii:
                                                                            fret_core::Corners::all(
                                                                                theme.metrics.radius_sm,
                                                                            ),
                                                                        ..Default::default()
                                                                    },
                                                                    move |cx| {
                                                                        vec![cx.text_props(TextProps {
                                                                            layout: LayoutStyle::default(),
                                                                            text: label.clone(),
                                                                            style: Some(text_style.clone()),
                                                                            wrap: TextWrap::None,
                                                                            overflow: TextOverflow::Ellipsis,
                                                                            color: Some(if disabled {
                                                                                theme.colors.text_disabled
                                                                            } else {
                                                                                theme.colors.text_primary
                                                                            }),
                                                                        })]
                                                                    },
                                                                )]
                                                            },
                                                        ));
                                                    }
                                                }
                                            }

                                            out
                                        },
                                    )]
                                },
                            )]
                        },
                    );

                    vec![content]
                });

                let mut request = OverlayRequest::dismissible_popover(
                    id,
                    trigger_id,
                    open,
                    OverlayPresence::instant(true),
                    overlay_children,
                );
                request.root_name = Some(overlay_root_name);
                request.initial_focus = None;
                OverlayController::request(cx, request);
            }

            trigger
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics};
    use fret_core::{PathService, PathStyle, Point, Px, Rect, SemanticsRole, Size};
    use fret_core::{SvgId, SvgService, TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_core::{TextStyle, UiServices};
    use fret_runtime::FrameId;
    use fret_ui::tree::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &TextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
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

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: Model<bool>,
    ) -> fret_core::NodeId {
        let next_frame = FrameId(app.frame_id().0.saturating_add(1));
        app.set_frame_id(next_frame);

        OverlayController::begin_frame(app, window);
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "context-menu",
            |cx| {
                vec![ContextMenu::new(open).into_element(
                    cx,
                    |cx| {
                        cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout
                                },
                                ..Default::default()
                            },
                            |_cx| Vec::new(),
                        )
                    },
                    |_cx| {
                        vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Alpha")),
                            ContextMenuEntry::Separator,
                            ContextMenuEntry::Item(ContextMenuItem::new("Beta")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Gamma")),
                        ]
                    },
                )]
            },
        );
        ui.set_root(root);
        OverlayController::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn context_menu_items_have_collection_position_metadata_excluding_separators() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // First frame: establish stable trigger bounds.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let _ = app.models_mut().update(&open, |v| *v = true);

        // Second frame: open the menu and verify item metadata.
        let _ = render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Beta"))
            .expect("Beta menu item");
        assert_eq!(beta.pos_in_set, Some(2));
        assert_eq!(beta.set_size, Some(3));
    }
}
