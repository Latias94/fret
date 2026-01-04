use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{
    Edges, MouseButton, Point, Px, Rect, SemanticsRole, Size, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::action::PointerDownCx;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PointerRegionProps, PointerRegionState, PressableProps, RovingFlexProps, RovingFocusProps,
    SemanticsProps, TextProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::{Align, LayoutDirection, Side};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::overlay;
use fret_ui_kit::primitives::menu;
use fret_ui_kit::primitives::popper;
use fret_ui_kit::primitives::popper_content;
use fret_ui_kit::{MetricRef, OverlayController, OverlayPresence, OverlayRequest, Space};

use crate::dropdown_menu::{DropdownMenuAlign, DropdownMenuSide};
use crate::popper_arrow::{self, DiamondArrowStyle};

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
/// - Keyboard invocation via Shift+F10 is supported (there is no dedicated `ContextMenu` key in
///   `fret_core::KeyCode` yet).
#[derive(Clone)]
pub struct ContextMenu {
    open: Model<bool>,
    align: DropdownMenuAlign,
    side: DropdownMenuSide,
    side_offset: Px,
    window_margin: Px,
    typeahead_timeout_ticks: u64,
    arrow: bool,
    arrow_size_override: Option<Px>,
    arrow_padding_override: Option<Px>,
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
            arrow: false,
            arrow_size_override: None,
            arrow_padding_override: None,
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

    /// Enables a ContextMenu arrow (Radix `ContextMenuArrow`-style).
    pub fn arrow(mut self, arrow: bool) -> Self {
        self.arrow = arrow;
        self
    }

    pub fn arrow_size(mut self, size: Px) -> Self {
        self.arrow_size_override = Some(size);
        self
    }

    pub fn arrow_padding(mut self, padding: Px) -> Self {
        self.arrow_padding_override = Some(padding);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<ContextMenuEntry>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let is_open = cx.watch_model(&self.open).copied().unwrap_or(false);
            let arrow = self.arrow;
            let arrow_size = self.arrow_size_override.unwrap_or_else(|| {
                theme
                    .metric_by_key("component.context_menu.arrow_size")
                    .or_else(|| theme.metric_by_key("component.popover.arrow_size"))
                    .unwrap_or(Px(12.0))
            });
            let arrow_padding = self.arrow_padding_override.unwrap_or_else(|| {
                theme
                    .metric_by_key("component.context_menu.arrow_padding")
                    .or_else(|| theme.metric_by_key("component.popover.arrow_padding"))
                    .unwrap_or(theme.metrics.radius_md)
            });

            let id = cx.root_id();
            let trigger = trigger(cx);
            let trigger_id = trigger.id;

            menu::trigger::wire_open_on_shift_f10(cx, trigger_id, self.open.clone());

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

            let pointer_down = cx.with_state(PointerRegionState::default, |st| st.last_down);
            let anchor_point = pointer_down.map(|it| it.position);

            if is_open {
                let overlay_root_name = OverlayController::popover_root_name(id);

                let align = self.align;
                let side = self.side;
                let side_offset = self.side_offset;
                let window_margin = self.window_margin;
                let typeahead_timeout_ticks = self.typeahead_timeout_ticks;
                let open_for_overlay = open.clone();
                let content_focus_id: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
                let content_focus_id_for_children = content_focus_id.clone();

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

                    let (arrow_options, arrow_protrusion) =
                        popper::diamond_arrow_options(arrow, arrow_size, arrow_padding);

                    let anchor_rect = overlay::anchor_rect_from_point(anchor);
                    let layout = popper::popper_content_layout_sized(
                        outer,
                        anchor_rect,
                        estimated,
                        popper::PopperContentPlacement::new(
                            LayoutDirection::Ltr,
                            side,
                            align,
                            side_offset,
                        )
                        .with_arrow(arrow_options, arrow_protrusion),
                    );

                    let placed = layout.rect;
                    let wrapper_insets = popper_arrow::wrapper_insets(&layout, arrow_protrusion);
                    let extra_left = wrapper_insets.left;
                    let extra_top = wrapper_insets.top;

                    let border = theme
                        .color_by_key("border")
                        .unwrap_or(theme.colors.panel_border);
                    let shadow = decl_style::shadow_sm(&theme, theme.metrics.radius_sm);
                    let ring = decl_style::focus_ring(&theme, theme.metrics.radius_sm);
                    let pad_x = MetricRef::space(Space::N3).resolve(&theme);
                    let pad_y = MetricRef::space(Space::N2).resolve(&theme);

                    let arrow_el = popper_arrow::diamond_arrow_element(
                        cx,
                        &layout,
                        wrapper_insets,
                        arrow_size,
                        DiamondArrowStyle {
                            bg: theme.colors.panel_background,
                            border: Some(border),
                            border_width: Px(1.0),
                        },
                    );

                    let content = cx.semantics(
                        SemanticsProps {
                            layout: LayoutStyle::default(),
                            role: SemanticsRole::Menu,
                            ..Default::default()
                        },
                        move |cx| {
                            vec![popper_content::popper_wrapper_at(
                                cx,
                                placed,
                                wrapper_insets,
                                move |cx| {
                                    let panel = menu::content_panel::menu_panel_container_at(
                                        cx,
                                        Rect::new(Point::new(extra_left, extra_top), placed.size),
                                        move |layout| ContainerProps {
                                            layout,
                                            padding: Edges::all(Px(4.0)),
                                            background: Some(theme.colors.panel_background),
                                            shadow: Some(shadow),
                                            border: Edges::all(Px(1.0)),
                                            border_color: Some(border),
                                            corner_radii: fret_core::Corners::all(
                                                theme.metrics.radius_sm,
                                            ),
                                        },
                                        move |cx| {
                                            let content_focus_id_for_panel =
                                                content_focus_id_for_children.clone();
                                            let roving = menu::content::menu_roving_group_apg_prefix_typeahead(
                                                cx,
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
                                                labels_arc.clone(),
                                                typeahead_timeout_ticks,
                                                move |cx| {

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
                                                                a11y: menu::item::menu_item_a11y(
                                                                    a11y_label,
                                                                    None,
                                                                )
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
                                            );
                                            if content_focus_id_for_panel.get().is_none() {
                                                content_focus_id_for_panel.set(Some(roving.id));
                                            }
                                            vec![roving]
                                        },
                                    );

                                    if let Some(arrow_el) = arrow_el {
                                        vec![arrow_el, panel]
                                    } else {
                                        vec![panel]
                                    }
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
                if !fret_ui::input_modality::is_keyboard(cx.app, Some(cx.window)) {
                    request.initial_focus = content_focus_id.get();
                }
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
    use fret_core::{
        AppWindowId, Event, KeyCode, Modifiers, PathCommand, PathConstraints, PathId, PathMetrics,
    };
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

    fn render_frame_focusable_trigger(
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
            "context-menu-shift-f10",
            |cx| {
                vec![ContextMenu::new(open).into_element(
                    cx,
                    |cx| {
                        cx.pressable(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(120.0));
                                    layout.size.height = Length::Px(Px(40.0));
                                    layout
                                },
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |cx, _st| {
                                vec![cx.container(ContainerProps::default(), |_cx| Vec::new())]
                            },
                        )
                    },
                    |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Alpha"))],
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
    fn context_menu_opens_on_shift_f10() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert(false);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        // First frame: build the tree and establish stable trigger bounds.
        let root = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let trigger = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable trigger");
        ui.set_focus(Some(trigger));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::F10,
                modifiers: Modifiers {
                    shift: true,
                    ..Default::default()
                },
                repeat: false,
            },
        );

        // Second frame: ContextMenu emits its OverlayRequest while rendering.
        // Re-rendering the root is required for the menu items to appear.
        let _ = render_frame_focusable_trigger(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
        );

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(
            snap.nodes.iter().any(|n| {
                n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Alpha")
            }),
            "menu items should render after Shift+F10 opens the context menu"
        );
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
