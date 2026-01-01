use std::sync::Arc;

use fret_components_ui::declarative::action_hooks::ActionHooksExt as _;
use fret_components_ui::declarative::collection_semantics::CollectionSemanticsExt as _;
use fret_components_ui::declarative::model_watch::ModelWatchExt as _;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::overlay;
use fret_components_ui::{MetricRef, OverlayController, OverlayPresence, OverlayRequest, Space};
use fret_core::{
    Edges, FontId, FontWeight, Px, SemanticsRole, Size, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, InsetStyle, LayoutStyle, Length, MainAlign,
    Overflow, PositionStyle, PressableA11y, PressableProps, RovingFlexProps, RovingFocusProps,
    ScrollAxis, ScrollProps, SemanticsProps, SizeStyle, TextProps,
};
use fret_ui::overlay_placement::{Align, Side, anchored_panel_bounds_sized};
use fret_ui::{ElementCx, Theme, UiHost};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DropdownMenuAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DropdownMenuSide {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
}

#[derive(Debug, Clone)]
pub enum DropdownMenuEntry {
    Item(DropdownMenuItem),
    Label(DropdownMenuLabel),
    Group(DropdownMenuGroup),
    Separator,
}

#[derive(Debug, Clone)]
pub struct DropdownMenuItem {
    pub label: Arc<str>,
    pub disabled: bool,
    pub command: Option<CommandId>,
    pub a11y_label: Option<Arc<str>>,
    pub trailing: Option<AnyElement>,
    pub variant: DropdownMenuItemVariant,
}

impl DropdownMenuItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            command: None,
            a11y_label: None,
            trailing: None,
            variant: DropdownMenuItemVariant::Default,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn variant(mut self, variant: DropdownMenuItemVariant) -> Self {
        self.variant = variant;
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

    pub fn trailing(mut self, element: AnyElement) -> Self {
        self.trailing = Some(element);
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DropdownMenuItemVariant {
    #[default]
    Default,
    Destructive,
}

/// shadcn/ui `DropdownMenuLabel` (v4).
#[derive(Debug, Clone)]
pub struct DropdownMenuLabel {
    pub text: Arc<str>,
}

impl DropdownMenuLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }
}

/// shadcn/ui `DropdownMenuGroup` (v4).
///
/// In the upstream DOM implementation, this is a structural wrapper. In Fret, we currently treat
/// it as a transparent grouping node and simply flatten its entries for rendering/navigation.
#[derive(Debug, Clone)]
pub struct DropdownMenuGroup {
    pub entries: Vec<DropdownMenuEntry>,
}

impl DropdownMenuGroup {
    pub fn new(entries: Vec<DropdownMenuEntry>) -> Self {
        Self { entries }
    }
}

/// shadcn/ui `DropdownMenuShortcut` (v4).
///
/// This is typically rendered as trailing, muted text inside a menu item.
#[derive(Debug, Clone)]
pub struct DropdownMenuShortcut {
    pub text: Arc<str>,
}

impl DropdownMenuShortcut {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme
            .color_by_key("muted.foreground")
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or(theme.colors.text_muted);

        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: theme.metrics.font_size,
                weight: FontWeight::NORMAL,
                line_height: Some(theme.metrics.font_line_height),
                letter_spacing_em: Some(0.12),
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })
    }
}

fn flatten_entries(into: &mut Vec<DropdownMenuEntry>, entries: Vec<DropdownMenuEntry>) {
    for entry in entries {
        match entry {
            DropdownMenuEntry::Group(group) => flatten_entries(into, group.entries),
            other => into.push(other),
        }
    }
}

/// shadcn/ui `Dropdown Menu` (v4).
///
/// This is a dismissible popover overlay (non-modal) backed by the component-layer overlay
/// manager (`fret-components-ui/overlay_controller.rs`).
#[derive(Clone)]
pub struct DropdownMenu {
    open: Model<bool>,
    align: DropdownMenuAlign,
    side: DropdownMenuSide,
    side_offset: Px,
    window_margin: Px,
    typeahead_timeout_ticks: u64,
    min_width: Px,
}

impl std::fmt::Debug for DropdownMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropdownMenu")
            .field("open", &"<model>")
            .field("align", &self.align)
            .field("side", &self.side)
            .field("side_offset", &self.side_offset)
            .field("window_margin", &self.window_margin)
            .field("typeahead_timeout_ticks", &self.typeahead_timeout_ticks)
            .finish()
    }
}

impl DropdownMenu {
    pub fn new(open: Model<bool>) -> Self {
        Self {
            open,
            align: DropdownMenuAlign::default(),
            side: DropdownMenuSide::default(),
            side_offset: Px(4.0),
            window_margin: Px(8.0),
            typeahead_timeout_ticks: 30,
            min_width: Px(128.0),
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

    pub fn min_width(mut self, min_width: Px) -> Self {
        self.min_width = min_width;
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementCx<'_, H>,
        trigger: impl FnOnce(&mut ElementCx<'_, H>) -> AnyElement,
        entries: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<DropdownMenuEntry>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let is_open = cx.watch_model(&self.open).copied().unwrap_or(false);

            let trigger = trigger(cx);
            let trigger_id = trigger.id;

            if is_open {
                let overlay_root_name = OverlayController::popover_root_name(trigger_id);

                let align = self.align;
                let side = self.side;
                let side_offset = self.side_offset;
                let window_margin = self.window_margin;
                let open = self.open;
                let open_for_overlay = open.clone();
                let typeahead_timeout_ticks = self.typeahead_timeout_ticks;
                let min_width = self.min_width;

                let overlay_children = cx.with_root_name(&overlay_root_name, move |cx| {
                    let theme = &theme;
                    let anchor = overlay::anchor_bounds_for_element(cx, trigger_id);
                    let Some(anchor) = anchor else {
                        return Vec::new();
                    };

                    let mut flat: Vec<DropdownMenuEntry> = Vec::new();
                    flatten_entries(&mut flat, entries(cx));
                    let entries = flat;
                    let item_count = entries
                        .iter()
                        .filter(|e| matches!(e, DropdownMenuEntry::Item(_)))
                        .count();
                    let (labels, disabled_flags): (Vec<Arc<str>>, Vec<bool>) = entries
                        .iter()
                        .map(|e| match e {
                            DropdownMenuEntry::Item(item) => (item.label.clone(), item.disabled),
                            DropdownMenuEntry::Label(_) | DropdownMenuEntry::Separator => {
                                (Arc::from(""), true)
                            }
                            DropdownMenuEntry::Group(_) => unreachable!("groups are flattened"),
                        })
                        .unzip();

                    let labels_arc: Arc<[Arc<str>]> = Arc::from(labels.into_boxed_slice());
                    let disabled_arc: Arc<[bool]> = Arc::from(disabled_flags.into_boxed_slice());

                    let outer = overlay::outer_bounds_with_window_margin(cx.bounds, window_margin);

                    // shadcn: content width tracks trigger width (with a minimum), and height clamps
                    // to available space (scrolls internally).
                    let desired = Size::new(Px(anchor.size.width.0.max(min_width.0)), Px(1.0e9));

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

                    let placed =
                        anchored_panel_bounds_sized(outer, anchor, desired, side_offset, side, align);

                    let border = theme
                        .color_by_key("border")
                        .unwrap_or(theme.colors.panel_border);
                    let shadow = decl_style::shadow_sm(&theme, theme.metrics.radius_sm);
                    let ring = decl_style::focus_ring(&theme, theme.metrics.radius_sm);
                    let pad_x = MetricRef::space(Space::N3).resolve(&theme);
                    let pad_y = MetricRef::space(Space::N2).resolve(&theme);
                    let bg = theme
                        .color_by_key("popover")
                        .or_else(|| theme.color_by_key("popover.background"))
                        .unwrap_or(theme.colors.panel_background);
                    let fg = theme
                        .color_by_key("popover.foreground")
                        .or_else(|| theme.color_by_key("popover-foreground"))
                        .unwrap_or(theme.colors.text_primary);
                    let accent = theme
                        .color_by_key("accent")
                        .unwrap_or(theme.colors.hover_background);
                    let accent_fg = theme
                        .color_by_key("accent.foreground")
                        .or_else(|| theme.color_by_key("accent-foreground"))
                        .unwrap_or(theme.colors.text_primary);

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
                                    background: Some(bg),
                                    shadow: Some(shadow),
                                    border: Edges::all(Px(1.0)),
                                    border_color: Some(border),
                                    corner_radii: fret_core::Corners::all(theme.metrics.radius_sm),
                                },
                                move |cx| {
                                    let scroll_layout = LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Fill,
                                            ..Default::default()
                                        },
                                        overflow: Overflow::Clip,
                                        ..Default::default()
                                    };

                                    vec![cx.scroll(
                                        ScrollProps {
                                            layout: scroll_layout,
                                            axis: ScrollAxis::Y,
                                            ..Default::default()
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
                                                    cx.roving_nav_apg();
                                                    cx.roving_typeahead_prefix_arc_str(
                                                        labels_arc.clone(),
                                                        typeahead_timeout_ticks,
                                                    );

                                                    let font_size = theme.metrics.font_size;
                                                    let font_line_height =
                                                        theme.metrics.font_line_height;
                                                    let radius_sm = theme.metrics.radius_sm;
                                                    let text_disabled = theme.colors.text_disabled;
                                                    let destructive_fg = theme
                                                        .color_by_key("destructive")
                                                        .or_else(|| {
                                                            theme.color_by_key(
                                                                "destructive.background",
                                                            )
                                                        })
                                                        .unwrap_or(theme.colors.text_primary);

                                                    let text_style = TextStyle {
                                                        font: fret_core::FontId::default(),
                                                        size: font_size,
                                                        weight: fret_core::FontWeight::NORMAL,
                                                        line_height: Some(font_line_height),
                                                        letter_spacing_em: None,
                                                    };

                                                    let mut out: Vec<AnyElement> =
                                                        Vec::with_capacity(entries.len());

                                                    let mut item_ix: usize = 0;
                                            for entry in entries.clone() {
                                                match entry {
                                                    DropdownMenuEntry::Label(label) => {
                                                        let fg = theme
                                                            .color_by_key("muted.foreground")
                                                            .or_else(|| theme.color_by_key("muted-foreground"))
                                                            .unwrap_or(theme.colors.text_muted);
                                                        let text = label.text.clone();
                                                        out.push(cx.container(
                                                            ContainerProps {
                                                                layout: LayoutStyle::default(),
                                                                padding: Edges {
                                                                    top: pad_y,
                                                                    right: pad_x,
                                                                    bottom: pad_y,
                                                                    left: pad_x,
                                                                },
                                                                ..Default::default()
                                                            },
                                                            move |cx| {
                                                                vec![cx.text_props(TextProps {
                                                                    layout: LayoutStyle::default(),
                                                                    text,
                                                                    style: Some(TextStyle {
                                                                        font: FontId::default(),
                                                                        size: font_size,
                                                                        weight: FontWeight::MEDIUM,
                                                                        line_height: Some(
                                                                            font_line_height,
                                                                        ),
                                                                        letter_spacing_em: None,
                                                                    }),
                                                                    wrap: TextWrap::None,
                                                                    overflow: TextOverflow::Ellipsis,
                                                                    color: Some(fg),
                                                                })]
                                                            },
                                                        ));
                                                    }
                                                    DropdownMenuEntry::Group(_) => {
                                                        unreachable!("groups are flattened")
                                                    }
                                                    DropdownMenuEntry::Separator => {
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
                                                                        padding: Edges::all(Px(0.0)),
                                                                        background: Some(border),
                                                                        ..Default::default()
                                                                    },
                                                                    |_cx| Vec::new(),
                                                                ));
                                                            }
                                                    DropdownMenuEntry::Item(item) => {
                                                        let collection_index = item_ix;
                                                        item_ix = item_ix.saturating_add(1);

                                                        let label = item.label.clone();
                                                        let a11y_label = item
                                                            .a11y_label
                                                            .clone()
                                                            .or_else(|| Some(label.clone()));
                                                        let disabled = item.disabled;
                                                        let command = item.command;
                                                        let trailing = item.trailing.clone();
                                                        let variant = item.variant;
                                                        let open = open_for_overlay.clone();
                                                        let text_style = text_style.clone();

                                                        out.push(cx.pressable(
                                                            PressableProps {
                                                                        layout: {
                                                                            let mut layout =
                                                                                LayoutStyle::default();
                                                                            layout.size.width =
                                                                                Length::Fill;
                                                                            layout.size.min_height =
                                                                                Some(Px(28.0));
                                                                            layout
                                                                        },
                                                                        enabled: !disabled,
                                                                        focusable: !disabled,
                                                                        focus_ring: Some(ring),
                                                                        a11y: PressableA11y {
                                                                            role: Some(
                                                                                SemanticsRole::MenuItem,
                                                                            ),
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
                                                                        cx.pressable_dispatch_command_opt(
                                                                            command,
                                                                        );
                                                                        if !disabled {
                                                                            cx.pressable_set_bool(
                                                                                &open,
                                                                                false,
                                                                            );
                                                                        }

                                                                        let mut row_bg =
                                                                            fret_core::Color::TRANSPARENT;
                                                                        let mut row_fg = if variant
                                                                            == DropdownMenuItemVariant::Destructive
                                                                        {
                                                                            destructive_fg
                                                                        } else {
                                                                            fg
                                                                        };
                                                                        if st.hovered
                                                                            || st.pressed
                                                                            || st.focused
                                                                        {
                                                                            row_bg = accent;
                                                                            row_fg = accent_fg;
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
                                                                                background: Some(row_bg),
                                                                                corner_radii:
                                                                                    fret_core::Corners::all(
                                                                                        radius_sm,
                                                                                    ),
                                                                                ..Default::default()
                                                                            },
                                                                            move |cx| {
                                                                                let mut row: Vec<AnyElement> =
                                                                                    Vec::with_capacity(1 + usize::from(trailing.is_some()));
                                                                                row.push(cx.text_props(TextProps {
                                                                                    layout: {
                                                                                        let mut layout = LayoutStyle::default();
                                                                                        layout.size.width = Length::Fill;
                                                                                        layout
                                                                                    },
                                                                                    text: label.clone(),
                                                                                    style: Some(text_style.clone()),
                                                                                    wrap: TextWrap::None,
                                                                                    overflow: TextOverflow::Ellipsis,
                                                                                    color: Some(if disabled {
                                                                                        text_disabled
                                                                                    } else {
                                                                                        row_fg
                                                                                    }),
                                                                                }));

                                                                                if let Some(t) = trailing.clone() {
                                                                                    row.push(t);
                                                                                }

                                                                                vec![cx.flex(
                                                                                    FlexProps {
                                                                                        layout: {
                                                                                            let mut layout = LayoutStyle::default();
                                                                                            layout.size.width = Length::Fill;
                                                                                            layout
                                                                                        },
                                                                                        direction: fret_core::Axis::Horizontal,
                                                                                        gap: Px(8.0),
                                                                                        padding: Edges::all(Px(0.0)),
                                                                                        justify: MainAlign::Start,
                                                                                        align: CrossAlign::Center,
                                                                                        wrap: false,
                                                                                    },
                                                                                    move |_cx| row.clone(),
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
                            )]
                        },
                    );

                    vec![content]
                });

                let mut request = OverlayRequest::dismissible_popover(
                    trigger_id,
                    trigger_id,
                    open,
                    OverlayPresence::instant(true),
                    overlay_children,
                );
                request.root_name = Some(overlay_root_name);
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
    use fret_core::{AppWindowId, PathCommand, Point, Rect, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Px, SemanticsRole, Size as CoreSize};
    use fret_core::{
        TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle as CoreTextStyle,
    };
    use fret_runtime::FrameId;
    use fret_ui::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &CoreTextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: CoreSize::new(Px(0.0), Px(0.0)),
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

    fn render_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
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
            "dropdown-menu",
            |cx| {
                vec![DropdownMenu::new(open).into_element(
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
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Alpha")),
                            DropdownMenuEntry::Separator,
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Beta")),
                            DropdownMenuEntry::Item(DropdownMenuItem::new("Gamma")),
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
    fn dropdown_menu_items_have_collection_position_metadata_excluding_separators() {
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
