use std::sync::Arc;

use fret_components_ui::declarative::action_hooks::ActionHooksExt as _;
use fret_components_ui::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::headless::roving_focus;
use fret_components_ui::overlay;
use fret_components_ui::window_overlays;
use fret_components_ui::{MetricRef, Space};
use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::Invalidation;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, InsetStyle, LayoutStyle, Length, MainAlign,
    Overflow, PositionStyle, PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps,
    SemanticsProps, SizeStyle, TextProps,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::overlay_placement::{Align, Side, anchored_panel_bounds_sized};
use fret_ui::{ElementCx, Theme, UiHost};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone)]
pub struct MenubarItem {
    pub label: Arc<str>,
    pub disabled: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
}

impl MenubarItem {
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

#[derive(Debug, Clone)]
pub enum MenubarEntry {
    Item(MenubarItem),
    Separator,
}

#[derive(Clone)]
pub struct Menubar {
    children: Vec<AnyElement>,
    disabled: bool,
}

impl std::fmt::Debug for Menubar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Menubar")
            .field("children_len", &self.children.len())
            .field("disabled", &self.disabled)
            .finish()
    }
}

impl Menubar {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            children,
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let border = theme
                .color_by_key("border")
                .unwrap_or(theme.colors.panel_border);
            let radius = theme.metrics.radius_sm;
            let pad_x = MetricRef::space(Space::N2).resolve(&theme);
            let pad_y = MetricRef::space(Space::N2).resolve(&theme);
            let gap = MetricRef::space(Space::N1).resolve(&theme);

            let disabled = self.disabled;
            let children = self.children;

            cx.semantics(
                SemanticsProps {
                    layout: LayoutStyle::default(),
                    role: SemanticsRole::MenuBar,
                    disabled,
                    ..Default::default()
                },
                |cx| {
                    vec![cx.container(
                        ContainerProps {
                            layout: LayoutStyle::default(),
                            padding: Edges {
                                top: pad_y,
                                right: pad_x,
                                bottom: pad_y,
                                left: pad_x,
                            },
                            background: Some(theme.colors.panel_background),
                            shadow: None,
                            border: Edges::all(Px(1.0)),
                            border_color: Some(border),
                            corner_radii: Corners::all(radius),
                        },
                        move |cx| {
                            vec![cx.flex(
                                FlexProps {
                                    layout: LayoutStyle::default(),
                                    direction: fret_core::Axis::Horizontal,
                                    gap,
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |_cx| children,
                            )]
                        },
                    )]
                },
            )
        })
    }
}

#[derive(Clone)]
struct MenubarActive {
    trigger: GlobalElementId,
    open: Model<bool>,
}

#[derive(Default)]
struct MenubarGroupState {
    active: Option<Model<Option<MenubarActive>>>,
}

#[derive(Default)]
struct MenubarMenuState {
    open: Option<Model<bool>>,
}

#[derive(Clone)]
pub struct MenubarMenu {
    label: Arc<str>,
    disabled: bool,
    window_margin: Px,
    side_offset: Px,
    typeahead_timeout_ticks: u64,
}

impl std::fmt::Debug for MenubarMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenubarMenu")
            .field("label", &self.label.as_ref())
            .field("disabled", &self.disabled)
            .field("window_margin", &self.window_margin)
            .field("side_offset", &self.side_offset)
            .field("typeahead_timeout_ticks", &self.typeahead_timeout_ticks)
            .finish()
    }
}

impl MenubarMenu {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            window_margin: Px(8.0),
            side_offset: Px(4.0),
            typeahead_timeout_ticks: 30,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn window_margin(mut self, margin: Px) -> Self {
        self.window_margin = margin;
        self
    }

    pub fn side_offset(mut self, offset: Px) -> Self {
        self.side_offset = offset;
        self
    }

    pub fn typeahead_timeout_ticks(mut self, ticks: u64) -> Self {
        self.typeahead_timeout_ticks = ticks;
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementCx<'_, H>,
        entries: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<MenubarEntry>,
    ) -> AnyElement {
        let group = cx.root_id();
        let key = self.label.clone();
        cx.keyed(key, |cx| {
            let group_active =
                cx.with_state_for(group, MenubarGroupState::default, |st| st.active.clone());
            let group_active = if let Some(group_active) = group_active {
                group_active
            } else {
                let group_active = cx.app.models_mut().insert(None);
                cx.with_state_for(group, MenubarGroupState::default, |st| {
                    st.active = Some(group_active.clone());
                });
                group_active
            };

            let open = cx.with_state(MenubarMenuState::default, |st| st.open.clone());
            let open = if let Some(open) = open {
                open
            } else {
                let open = cx.app.models_mut().insert(false);
                cx.with_state(MenubarMenuState::default, |st| st.open = Some(open.clone()));
                open
            };

            cx.observe_model(&group_active, Invalidation::Paint);
            cx.observe_model(&open, Invalidation::Paint);

            let theme = Theme::global(&*cx.app).clone();
            let enabled = !self.disabled;

            let radius = theme.metrics.radius_sm;
            let ring = decl_style::focus_ring(&theme, radius);
            let bg_hover = theme.colors.hover_background;
            let bg_open = alpha_mul(theme.colors.selection_background, 0.35);
            let fg = theme
                .color_by_key("foreground")
                .unwrap_or(theme.colors.text_primary);
            let fg_muted = theme
                .color_by_key("muted-foreground")
                .unwrap_or(theme.colors.text_muted);

            let text_style = TextStyle {
                font: FontId::default(),
                size: theme.metrics.font_size,
                weight: FontWeight::MEDIUM,
                line_height: Some(theme.metrics.font_line_height),
                letter_spacing_em: None,
            };

            let label = self.label.clone();

            cx.pressable_with_id_props(|cx, st, trigger_id| {
                let mut trigger_layout = LayoutStyle::default();
                trigger_layout.size.height = Length::Auto;
                trigger_layout.size.width = Length::Auto;

                let active_value = cx.app.models().get_cloned(&group_active).flatten();
                let is_open = cx.app.models().get_copied(&open).unwrap_or(false);

                if active_value
                    .as_ref()
                    .is_some_and(|active_value| active_value.trigger != trigger_id)
                    && is_open
                {
                    let _ = cx.app.models_mut().update(&open, |v| *v = false);
                }

                if active_value
                    .as_ref()
                    .is_some_and(|active_value| active_value.trigger == trigger_id)
                    && !is_open
                {
                    let _ = cx.app.models_mut().update(&group_active, |v| *v = None);
                }

                if active_value.is_none() && is_open {
                    let open_for_state = open.clone();
                    let _ = cx.app.models_mut().update(&group_active, |v| {
                        *v = Some(MenubarActive {
                            trigger: trigger_id,
                            open: open_for_state,
                        });
                    });
                }

                let active_value = cx.app.models().get_cloned(&group_active).flatten();
                if enabled
                    && st.hovered
                    && !st.pressed
                    && active_value
                        .as_ref()
                        .is_some_and(|active_value| active_value.trigger != trigger_id)
                {
                    if let Some(prev) = active_value.as_ref() {
                        let _ = cx.app.models_mut().update(&prev.open, |v| *v = false);
                    }
                    let _ = cx.app.models_mut().update(&open, |v| *v = true);
                    let open_for_state = open.clone();
                    let _ = cx.app.models_mut().update(&group_active, |v| {
                        *v = Some(MenubarActive {
                            trigger: trigger_id,
                            open: open_for_state,
                        });
                    });
                }

                let group_active_for_activate = group_active.clone();
                let open_for_activate = open.clone();
                cx.pressable_add_on_activate(Arc::new(move |host, _cx, _reason| {
                    let cur = host.models_mut().get_cloned(&group_active_for_activate).flatten();
                    match cur {
                        Some(cur) if cur.trigger == trigger_id => {
                            let _ = host.models_mut().update(&open_for_activate, |v| *v = false);
                            let _ =
                                host.models_mut().update(&group_active_for_activate, |v| *v = None);
                        }
                        prev => {
                            if let Some(prev) = prev {
                                let _ = host.models_mut().update(&prev.open, |v| *v = false);
                            }
                            let _ = host.models_mut().update(&open_for_activate, |v| *v = true);
                            let open_for_state = open_for_activate.clone();
                            let _ = host.models_mut().update(&group_active_for_activate, |v| {
                                *v = Some(MenubarActive {
                                    trigger: trigger_id,
                                    open: open_for_state,
                                });
                            });
                        }
                    }
                }));

                let is_open = cx.app.models().get_copied(&open).unwrap_or(false);
                let trigger_bg = if is_open {
                    Some(bg_open)
                } else if st.hovered || st.pressed {
                    Some(alpha_mul(bg_hover, 0.8))
                } else {
                    None
                };

                let props = PressableProps {
                    layout: trigger_layout,
                    enabled,
                    focusable: true,
                    focus_ring: Some(ring),
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::MenuItem),
                        label: Some(label.clone()),
                        expanded: Some(is_open),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                if is_open && enabled {
                    let overlay_root_name = window_overlays::popover_root_name(trigger_id);
                    let side_offset = self.side_offset;
                    let window_margin = self.window_margin;
                    let typeahead_timeout_ticks = self.typeahead_timeout_ticks;
                    let group_active = group_active;
                    let open_for_overlay = open.clone();

                    let overlay_children = cx.with_root_name(&overlay_root_name, move |cx| {
                        let Some(anchor) = overlay::anchor_bounds_for_element(cx, trigger_id) else {
                            return Vec::new();
                        };
                        let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);
                        let estimated = fret_core::Size::new(Px(240.0), Px(220.0));

                        let placed = anchored_panel_bounds_sized(
                            outer,
                            anchor,
                            estimated,
                            side_offset,
                            Side::Bottom,
                            Align::Start,
                        );

                        let entries = entries(cx);
                        let (labels, disabled_flags): (Vec<Arc<str>>, Vec<bool>) = entries
                            .iter()
                            .map(|e| match e {
                                MenubarEntry::Item(item) => (item.label.clone(), item.disabled),
                                MenubarEntry::Separator => (Arc::from(""), true),
                            })
                            .unzip();

                        let labels_arc: Arc<[Arc<str>]> = Arc::from(labels.into_boxed_slice());
                        let disabled_arc: Arc<[bool]> =
                            Arc::from(disabled_flags.clone().into_boxed_slice());
                        let active = roving_focus::first_enabled(&disabled_flags);

                        let roving = RovingFocusProps {
                            enabled: true,
                            wrap: true,
                            disabled: disabled_arc,
                            ..Default::default()
                        };

                        let border = theme
                            .color_by_key("border")
                            .unwrap_or(theme.colors.panel_border);
                        let shadow = decl_style::shadow_sm(&theme, theme.metrics.radius_sm);
                        let item_ring = decl_style::focus_ring(&theme, theme.metrics.radius_sm);
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
                                        padding: Edges::all(Px(6.0)),
                                        background: Some(theme.colors.menu_background),
                                        shadow: Some(shadow),
                                        border: Edges::all(Px(1.0)),
                                        border_color: Some(border),
                                        corner_radii: Corners::all(theme.metrics.radius_sm),
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
                                                roving,
                                            },
                                            move |cx| {
                                                cx.roving_typeahead_prefix_arc_str(
                                                    labels_arc.clone(),
                                                    typeahead_timeout_ticks,
                                                );

                                                let mut out: Vec<AnyElement> =
                                                    Vec::with_capacity(entries.len());

                                                let item_count = entries
                                                    .iter()
                                                    .filter(|e| matches!(e, MenubarEntry::Item(_)))
                                                    .count();
                                                let mut item_ix: usize = 0;

                                                for (idx, entry) in entries.into_iter().enumerate()
                                                {
                                                    match entry {
                                                        MenubarEntry::Separator => {
                                                            out.push(cx.container(
                                                                ContainerProps {
                                                                    layout: {
                                                                        let mut layout =
                                                                            LayoutStyle::default();
                                                                        layout.size.width =
                                                                            Length::Fill;
                                                                        layout.size.height =
                                                                            Length::Px(Px(1.0));
                                                                        layout
                                                                    },
                                                                    background: Some(border),
                                                                    ..Default::default()
                                                                },
                                                                |_| Vec::new(),
                                                            ));
                                                        }
                                                        MenubarEntry::Item(item) => {
                                                            let collection_index = item_ix;
                                                            item_ix = item_ix.saturating_add(1);

                                                            let item_enabled =
                                                                !item.disabled && enabled;
                                                            let focusable =
                                                                active.is_some_and(|a| a == idx);
                                                            let label = item.label.clone();
                                                            let a11y_label =
                                                                item.a11y_label.clone();
                                                            let command = item.command;
                                                            let open = open_for_overlay.clone();
                                                            let group_active = group_active.clone();

                                                            out.push(cx.pressable(
                                                                PressableProps {
                                                                    layout: {
                                                                        let mut layout =
                                                                            LayoutStyle::default();
                                                                        layout.size.width =
                                                                            Length::Fill;
                                                                        layout.size.height =
                                                                            Length::Auto;
                                                                        layout
                                                                    },
                                                                    enabled: item_enabled,
                                                                    focusable,
                                                                    focus_ring: Some(item_ring),
                                                                    a11y: PressableA11y {
                                                                        role: Some(
                                                                            SemanticsRole::MenuItem,
                                                                        ),
                                                                        label: a11y_label
                                                                            .or_else(|| {
                                                                                Some(label.clone())
                                                                            }),
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
                                                                    cx.pressable_set_bool(&open, false);
                                                                    let group_active_for_activate =
                                                                        group_active.clone();
                                                                    cx.pressable_add_on_activate(
                                                                        Arc::new(move |host, _cx, _reason| {
                                                                            let _ = host
                                                                                .models_mut()
                                                                                .update(&group_active_for_activate, |v| *v = None);
                                                                        }),
                                                                    );

                                                                    let mut bg =
                                                                        Color::TRANSPARENT;
                                                                    if st.hovered || st.pressed {
                                                                        bg = alpha_mul(
                                                                            theme
                                                                                .colors
                                                                                .menu_item_hover,
                                                                            0.9,
                                                                        );
                                                                    }
                                                                    let fg = if item_enabled {
                                                                        fg
                                                                    } else {
                                                                        alpha_mul(fg_muted, 0.85)
                                                                    };

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
                                                                            shadow: None,
                                                                            border: Edges::all(
                                                                                Px(0.0),
                                                                            ),
                                                                            border_color: None,
                                                                            corner_radii: Corners::all(
                                                                                theme.metrics.radius_sm,
                                                                            ),
                                                                        },
                                                                        move |cx| {
                                                                            vec![cx.text_props(
                                                                                TextProps {
                                                                                    layout: LayoutStyle::default(),
                                                                                    text: label.clone(),
                                                                                    style: Some(
                                                                                        text_style,
                                                                                    ),
                                                                                    color: Some(fg),
                                                                                    wrap: TextWrap::None,
                                                                                    overflow:
                                                                                        TextOverflow::Clip,
                                                                                },
                                                                            )]
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

                    window_overlays::request_dismissible_popover(
                        cx,
                        window_overlays::DismissiblePopoverRequest {
                            id: trigger_id,
                            root_name: overlay_root_name,
                            trigger: trigger_id,
                            open,
                            present: true,
                            initial_focus: None,
                            children: overlay_children,
                        },
                    );
                }

                let content = cx.container(
                    ContainerProps {
                        layout: LayoutStyle::default(),
                        padding: Edges {
                            top: Px(4.0),
                            right: Px(8.0),
                            bottom: Px(4.0),
                            left: Px(8.0),
                        },
                        background: trigger_bg,
                        shadow: None,
                        border: Edges::all(Px(0.0)),
                        border_color: None,
                        corner_radii: Corners::all(radius),
                    },
                    move |cx| {
                        vec![cx.text_props(TextProps {
                            layout: LayoutStyle::default(),
                            text: label.clone(),
                            style: Some(text_style),
                            color: Some(if enabled { fg } else { fg_muted }),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        })]
                    },
                );

                (props, vec![content])
            })
        })
    }
}

pub fn menubar<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    Menubar::new(f(cx)).into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_components_ui::window_overlays;
    use fret_core::{
        AppWindowId, FrameId, Modifiers, MouseButton, MouseButtons, Point, Rect, TextBlobId,
        TextConstraints, TextMetrics, TextService,
    };
    use fret_core::{PathCommand, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_ui::tree::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: TextStyle,
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

    fn center(r: fret_core::Rect) -> Point {
        Point::new(
            Px(r.origin.x.0 + r.size.width.0 / 2.0),
            Px(r.origin.y.0 + r.size.height.0 / 2.0),
        )
    }

    fn menu_trigger_bounds(snap: &fret_core::SemanticsSnapshot, label: &str) -> Rect {
        snap.nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(label))
            .map(|n| n.bounds)
            .unwrap_or_else(|| panic!("missing menu trigger {label:?}"))
    }

    fn menu_trigger_expanded(snap: &fret_core::SemanticsSnapshot, label: &str) -> bool {
        snap.nodes
            .iter()
            .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(label))
            .map(|n| n.flags.expanded)
            .unwrap_or(false)
    }

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
    ) {
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
        window_overlays::begin_frame(app, window);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "menubar", |cx| {
                vec![menubar(cx, |cx| {
                    vec![
                        MenubarMenu::new("File").into_element(cx, |_cx| {
                            vec![
                                MenubarEntry::Item(MenubarItem::new("New")),
                                MenubarEntry::Separator,
                                MenubarEntry::Item(MenubarItem::new("Open")),
                                MenubarEntry::Item(MenubarItem::new("Exit")),
                            ]
                        }),
                        MenubarMenu::new("Edit").into_element(cx, |_cx| {
                            vec![
                                MenubarEntry::Item(MenubarItem::new("Undo")),
                                MenubarEntry::Separator,
                                MenubarEntry::Item(MenubarItem::new("Redo")),
                            ]
                        }),
                    ]
                })]
            });
        ui.set_root(root);
        window_overlays::render(ui, app, services, window, bounds);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
    }

    #[test]
    fn menubar_hover_switches_open_menu() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        // Frame 0: render and locate triggers.
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file = center(menu_trigger_bounds(&snap0, "File"));
        let edit = center(menu_trigger_bounds(&snap0, "Edit"));

        // Click "File" to open.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        // Frame 1: "File" is expanded.
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap1, "File"));
        assert!(!menu_trigger_expanded(snap1, "Edit"));

        // Hover over "Edit" while a menu is open should switch without click.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: edit,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
            }),
        );

        // Frame 2: switching begins (the hovered menu opens in the same frame).
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap2 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(menu_trigger_expanded(snap2, "Edit"));

        // Frame 3: the previously-open menu is fully closed.
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap3 = ui.semantics_snapshot().expect("semantics snapshot");
        assert!(!menu_trigger_expanded(snap3, "File"));
        assert!(menu_trigger_expanded(snap3, "Edit"));
    }

    #[test]
    fn menubar_items_have_collection_position_metadata_excluding_separators() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        let mut services = FakeServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(480.0), Px(240.0)),
        );

        // Frame 0: render and locate triggers.
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap0 = ui.semantics_snapshot().expect("semantics snapshot").clone();
        let file = center(menu_trigger_bounds(&snap0, "File"));

        // Click "File" to open.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: file,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        // Frame 1: open menu should be present in semantics.
        render_frame(&mut ui, &mut app, &mut services, window, bounds);
        let snap1 = ui.semantics_snapshot().expect("semantics snapshot");

        assert!(menu_trigger_expanded(snap1, "File"));

        let interesting = ["File", "Edit", "New", "Open", "Exit", "Undo", "Redo"];
        let observed: Vec<(SemanticsRole, Option<&str>, Option<u32>, Option<u32>)> = snap1
            .nodes
            .iter()
            .filter(|n| n.label.as_deref().is_some_and(|l| interesting.contains(&l)))
            .map(|n| (n.role, n.label.as_deref(), n.pos_in_set, n.set_size))
            .collect();

        let open = snap1
            .nodes
            .iter()
            .find(|n| n.label.as_deref() == Some("Open"))
            .unwrap_or_else(|| panic!("Open node not found; observed={observed:?}"));
        assert_eq!(open.pos_in_set, Some(2));
        assert_eq!(open.set_size, Some(3));
    }
}
