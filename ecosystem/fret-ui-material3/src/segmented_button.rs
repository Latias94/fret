//! Material 3 outlined segmented buttons (MVP).
//!
//! Outcome-oriented implementation:
//! - Token-driven sizing/colors via `md.comp.outlined-segmented-button.*` (Material Web v30).
//! - Roving focus + Home/End + RTL-aware arrow navigation.
//! - State layer + bounded ripple per segment.

use std::collections::BTreeSet;
use std::sync::Arc;

use fret_core::{
    Axis, Color, Corners, KeyCode, LayoutDirection, Modifiers, Px, SemanticsRole, SvgFit,
    TextOverflow, TextWrap,
};
use fret_icons::{IconId, IconRegistry, ResolvedSvgOwned};
use fret_runtime::Model;
use fret_ui::action::{OnActivate, RovingNavigateResult};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, RovingFlexProps, SvgIconProps, TextProps,
};
use fret_ui::elements::ElementContext;
use fret_ui::{SvgSource, Theme, UiHost};

use crate::foundation::context::{resolved_layout_direction, theme_default_layout_direction};
use crate::foundation::focus_ring::material_focus_ring_for_component;
use crate::foundation::indication::{
    material_ink_layer_for_pressable, material_pressable_indication_config, RippleClip,
};
use crate::foundation::interaction::{pressable_interaction, PressableInteraction};
use crate::foundation::interactive_size::{centered_fill, enforce_minimum_interactive_size};
use crate::tokens::segmented_button as segmented_tokens;

#[derive(Debug, Clone)]
pub struct SegmentedButtonItem {
    value: Arc<str>,
    label: Arc<str>,
    icon: Option<IconId>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

impl SegmentedButtonItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct SegmentedButtonSet {
    selection: SegmentedSelection,
    items: Vec<SegmentedButtonItem>,
    disabled: bool,
    loop_navigation: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
}

#[derive(Debug, Clone)]
enum SegmentedSelection {
    Single(Model<Arc<str>>),
    Multi(Model<BTreeSet<Arc<str>>>),
}

impl SegmentedButtonSet {
    pub fn single(model: Model<Arc<str>>) -> Self {
        Self {
            selection: SegmentedSelection::Single(model),
            items: Vec::new(),
            disabled: false,
            loop_navigation: true,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn multi(model: Model<BTreeSet<Arc<str>>>) -> Self {
        Self {
            selection: SegmentedSelection::Multi(model),
            items: Vec::new(),
            disabled: false,
            loop_navigation: true,
            a11y_label: None,
            test_id: None,
        }
    }

    pub fn items(mut self, items: Vec<SegmentedButtonItem>) -> Self {
        self.items = items;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let SegmentedButtonSet {
            selection,
            items,
            disabled,
            loop_navigation,
            a11y_label,
            test_id,
        } = self;

        let theme = Theme::global(&*cx.app).clone();
        let layout_direction =
            resolved_layout_direction(cx, theme_default_layout_direction(&theme));

        let disabled_items: Arc<[bool]> = Arc::from(
            items
                .iter()
                .map(|it| disabled || it.disabled)
                .collect::<Vec<_>>(),
        );

        let tab_stop = match &selection {
            SegmentedSelection::Single(model) => {
                let selected = cx
                    .get_model_cloned(model, fret_ui::Invalidation::Layout)
                    .unwrap_or_else(|| Arc::<str>::from(""));

                items
                    .iter()
                    .position(|it| {
                        !disabled && !it.disabled && it.value.as_ref() == selected.as_ref()
                    })
                    .or_else(|| items.iter().position(|it| !disabled && !it.disabled))
            }
            SegmentedSelection::Multi(model) => {
                let selected = cx
                    .get_model_cloned(model, fret_ui::Invalidation::Layout)
                    .unwrap_or_default();

                items
                    .iter()
                    .position(|it| {
                        !disabled && !it.disabled && selected.contains(it.value.as_ref())
                    })
                    .or_else(|| items.iter().position(|it| !disabled && !it.disabled))
            }
        };

        let sem = fret_ui::element::SemanticsProps {
            role: SemanticsRole::Group,
            label: a11y_label.clone(),
            test_id: test_id.clone(),
            disabled,
            ..Default::default()
        };

        let mut props = RovingFlexProps::default();
        props.flex.direction = Axis::Horizontal;
        props.flex.gap = Px(0.0);
        props.flex.align = CrossAlign::Center;
        props.flex.layout.size.width = Length::Fill;
        props.roving.enabled = !disabled;
        props.roving.wrap = loop_navigation;
        props.roving.disabled = disabled_items.clone();

        cx.semantics(sem, move |cx| {
            let items = items;
            let selection = selection;

            vec![cx.roving_flex(props, move |cx| {
                cx.roving_on_navigate(Arc::new(move |_host, _cx, it| {
                    if it.repeat || it.modifiers != Modifiers::default() {
                        return RovingNavigateResult::NotHandled;
                    }
                    if it.axis != Axis::Horizontal {
                        return RovingNavigateResult::NotHandled;
                    }

                    let is_disabled =
                        |idx: usize| -> bool { it.disabled.get(idx).copied().unwrap_or(false) };

                    if it.key == KeyCode::Home {
                        let target = (0..it.len).find(|&i| !is_disabled(i));
                        return RovingNavigateResult::Handled { target };
                    }
                    if it.key == KeyCode::End {
                        let target = (0..it.len).rev().find(|&i| !is_disabled(i));
                        return RovingNavigateResult::Handled { target };
                    }

                    let forward = match (layout_direction, it.key) {
                        (LayoutDirection::Ltr, KeyCode::ArrowRight) => Some(true),
                        (LayoutDirection::Ltr, KeyCode::ArrowLeft) => Some(false),
                        (LayoutDirection::Rtl, KeyCode::ArrowLeft) => Some(true),
                        (LayoutDirection::Rtl, KeyCode::ArrowRight) => Some(false),
                        _ => None,
                    };
                    let Some(forward) = forward else {
                        return RovingNavigateResult::NotHandled;
                    };

                    let current = it
                        .current
                        .or_else(|| (0..it.len).find(|&i| !is_disabled(i)));
                    let Some(current) = current else {
                        return RovingNavigateResult::Handled { target: None };
                    };

                    let len = it.len;
                    let mut target: Option<usize> = None;
                    if it.wrap {
                        for step in 1..=len {
                            let idx = if forward {
                                (current + step) % len
                            } else {
                                (current + len - (step % len)) % len
                            };
                            if !is_disabled(idx) {
                                target = Some(idx);
                                break;
                            }
                        }
                    } else if forward {
                        target = ((current + 1)..len).find(|&i| !is_disabled(i));
                    } else if current > 0 {
                        target = (0..current).rev().find(|&i| !is_disabled(i));
                    }

                    RovingNavigateResult::Handled { target }
                }));

                let theme = Theme::global(&*cx.app).clone();

                let selected_single = match &selection {
                    SegmentedSelection::Single(model) => cx
                        .get_model_cloned(model, fret_ui::Invalidation::Layout)
                        .unwrap_or_else(|| Arc::<str>::from("")),
                    SegmentedSelection::Multi(_) => Arc::<str>::from(""),
                };
                let selected_multi = match &selection {
                    SegmentedSelection::Multi(model) => cx
                        .get_model_cloned(model, fret_ui::Invalidation::Layout)
                        .unwrap_or_default(),
                    SegmentedSelection::Single(_) => BTreeSet::new(),
                };

                let len = items.len();
                items
                    .into_iter()
                    .enumerate()
                    .map(|(idx, item)| {
                        let enabled = !disabled && !item.disabled;
                        let selected = match &selection {
                            SegmentedSelection::Single(_) => {
                                item.value.as_ref() == selected_single.as_ref()
                            }
                            SegmentedSelection::Multi(_) => {
                                selected_multi.contains(item.value.as_ref())
                            }
                        };

                        let handler =
                            on_activate_for_item(&selection, item.value.clone(), selected, enabled);

                        SegmentedButtonSegment {
                            item,
                            len,
                            idx,
                            selected,
                            enabled,
                            on_activate: handler,
                            roving_tab_stop: tab_stop.is_some_and(|t| t == idx),
                            theme: theme.clone(),
                        }
                        .into_element(cx)
                    })
                    .collect::<Vec<_>>()
            })]
        })
    }
}

fn on_activate_for_item(
    selection: &SegmentedSelection,
    value: Arc<str>,
    selected: bool,
    enabled: bool,
) -> Option<OnActivate> {
    if !enabled {
        return None;
    }

    match selection {
        SegmentedSelection::Single(model) => {
            let model = model.clone();
            Some(Arc::new(move |host, _acx, _reason| {
                let value_for_update = value.clone();
                let _ = host.models_mut().update(&model, |v| {
                    *v = value_for_update;
                });
            }))
        }
        SegmentedSelection::Multi(model) => {
            let model = model.clone();
            Some(Arc::new(move |host, _acx, _reason| {
                let value_for_update = value.clone();
                let _ = host.models_mut().update(&model, |set| {
                    if selected {
                        set.remove(value_for_update.as_ref());
                    } else {
                        set.insert(value_for_update);
                    }
                });
            }))
        }
    }
}

#[derive(Clone)]
struct SegmentedButtonSegment {
    item: SegmentedButtonItem,
    idx: usize,
    len: usize,
    selected: bool,
    enabled: bool,
    on_activate: Option<OnActivate>,
    roving_tab_stop: bool,
    theme: Theme,
}

impl SegmentedButtonSegment {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            cx.pressable_with_id_props(|cx, st, pressable_id| {
                let enabled = self.enabled;
                let focusable = enabled && (self.roving_tab_stop || st.focused);

                if let Some(handler) = self.on_activate.clone() {
                    cx.pressable_on_activate(handler);
                }

                let now_frame = cx.frame_id.0;
                let focus_visible =
                    fret_ui::focus_visible::is_focus_visible(&mut *cx.app, Some(cx.window));

                let is_pressed = enabled && st.pressed;
                let is_hovered = enabled && st.hovered;
                let is_focused = enabled && st.focused && focus_visible;

                let interaction = pressable_interaction(is_pressed, is_hovered, is_focused);
                let tokens_interaction = interaction.map(|s| match s {
                    PressableInteraction::Hovered => {
                        segmented_tokens::SegmentedButtonInteraction::Hovered
                    }
                    PressableInteraction::Focused => {
                        segmented_tokens::SegmentedButtonInteraction::Focused
                    }
                    PressableInteraction::Pressed => {
                        segmented_tokens::SegmentedButtonInteraction::Pressed
                    }
                });

                let corner_radii = segment_corner_radii(&self.theme, self.idx, self.len);

                let pressable_props = PressableProps {
                    enabled,
                    focusable,
                    a11y: PressableA11y {
                        role: Some(SemanticsRole::Button),
                        label: self
                            .item
                            .a11y_label
                            .clone()
                            .or_else(|| Some(self.item.label.clone())),
                        test_id: self.item.test_id.clone(),
                        selected: self.selected,
                        checked: Some(self.selected),
                        pos_in_set: Some((self.idx + 1) as u32),
                        set_size: Some(self.len as u32),
                        ..Default::default()
                    },
                    layout: {
                        let mut l = fret_ui::element::LayoutStyle::default();
                        l.overflow = Overflow::Visible;
                        l.size.width = Length::Fill;
                        l.flex.grow = 1.0;
                        l.flex.basis = Length::Fill;
                        enforce_minimum_interactive_size(&mut l, &self.theme);
                        l
                    },
                    focus_ring: Some(material_focus_ring_for_component(
                        &self.theme,
                        segmented_tokens::COMPONENT_PREFIX,
                        corner_radii,
                    )),
                    focus_ring_bounds: None,
                };

                let pointer_region = cx.named("pointer_region", |cx| {
                    let mut props = PointerRegionProps::default();
                    props.enabled = enabled;
                    cx.pointer_region(props, |cx| {
                        cx.pointer_region_on_pointer_down(Arc::new(|_host, _cx, _down| false));

                        let container_height = segmented_tokens::container_height(&self.theme);
                        let outline_width = segmented_tokens::outline_width(&self.theme);

                        let background =
                            segmented_tokens::container_background(&self.theme, self.selected);
                        let outline_color = segmented_tokens::outline_color(&self.theme, enabled);

                        let label_color = segmented_tokens::label_color(
                            &self.theme,
                            self.selected,
                            enabled,
                            tokens_interaction,
                        );
                        let icon_color = segmented_tokens::icon_color(
                            &self.theme,
                            self.selected,
                            enabled,
                            tokens_interaction,
                        );

                        let (state_layer_color, state_layer_target) = match tokens_interaction {
                            None => (Color::TRANSPARENT, 0.0),
                            Some(interaction) => (
                                segmented_tokens::state_layer_color(
                                    &self.theme,
                                    self.selected,
                                    interaction,
                                ),
                                segmented_tokens::state_layer_opacity(&self.theme, interaction),
                            ),
                        };

                        let ripple_base_opacity =
                            segmented_tokens::pressed_state_layer_opacity(&self.theme);
                        let config = material_pressable_indication_config(&self.theme, None);
                        let overlay = material_ink_layer_for_pressable(
                            cx,
                            pressable_id,
                            now_frame,
                            corner_radii,
                            RippleClip::Bounded,
                            state_layer_color,
                            is_pressed,
                            state_layer_target,
                            ripple_base_opacity,
                            config,
                            false,
                        );

                        let has_icon = self.item.icon.is_some();
                        let leading = if self.selected {
                            Some(material_icon(
                                cx,
                                &fret_icons::ids::ui::CHECK,
                                segmented_tokens::icon_size(&self.theme),
                                icon_color,
                            ))
                        } else if let Some(icon) = self.item.icon.as_ref() {
                            Some(material_icon(
                                cx,
                                icon,
                                segmented_tokens::icon_size(&self.theme),
                                icon_color,
                            ))
                        } else {
                            None
                        };

                        let content = material_segment_content(
                            cx,
                            &self.theme,
                            self.item.label.clone(),
                            leading,
                            label_color,
                            has_icon,
                        );

                        let chrome = material_segment_chrome(
                            cx,
                            self.idx,
                            container_height,
                            outline_width,
                            outline_color,
                            background,
                            corner_radii,
                            overlay,
                            content,
                        );

                        vec![centered_fill(cx, chrome)]
                    })
                });

                (pressable_props, vec![pointer_region])
            })
        })
    }
}

fn segment_corner_radii(theme: &Theme, idx: usize, len: usize) -> Corners {
    let r = segmented_tokens::shape_radius(theme);
    if len <= 1 {
        return Corners::all(r);
    }
    if idx == 0 {
        return Corners {
            top_left: r,
            top_right: Px(0.0),
            bottom_right: Px(0.0),
            bottom_left: r,
        };
    }
    if idx + 1 == len {
        return Corners {
            top_left: Px(0.0),
            top_right: r,
            bottom_right: r,
            bottom_left: Px(0.0),
        };
    }
    Corners::all(Px(0.0))
}

fn material_segment_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    idx: usize,
    height: Px,
    outline_width: Px,
    outline_color: Color,
    background: Option<Color>,
    corner_radii: Corners,
    overlay: AnyElement,
    content: AnyElement,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.overflow = Overflow::Clip;
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Px(height);
    props.background = background;
    props.corner_radii = corner_radii;
    props.border = fret_core::Edges {
        left: if idx == 0 { outline_width } else { Px(0.0) },
        right: outline_width,
        top: outline_width,
        bottom: outline_width,
    };
    props.border_color = Some(outline_color);

    cx.container(props, move |_cx| vec![overlay, content])
}

fn material_segment_content<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    label: Arc<str>,
    leading: Option<AnyElement>,
    label_color: Color,
    has_icon: bool,
) -> AnyElement {
    let mut props = FlexProps::default();
    props.direction = Axis::Horizontal;
    props.gap = if has_icon { Px(8.0) } else { Px(0.0) };
    props.justify = MainAlign::Center;
    props.align = CrossAlign::Center;
    props.wrap = false;

    let style = theme
        .text_style_by_key("md.comp.outlined-segmented-button.label-text")
        .or_else(|| theme.text_style_by_key("md.sys.typescale.label-large"))
        .unwrap_or_else(|| fret_core::TextStyle::default());

    let mut text = TextProps::new(label);
    text.style = Some(style);
    text.color = Some(label_color);
    text.wrap = TextWrap::None;
    text.overflow = TextOverflow::Clip;

    cx.flex(props, move |cx| {
        let mut out: Vec<AnyElement> = Vec::new();
        if let Some(leading) = leading.clone() {
            out.push(leading);
        }
        out.push(cx.text_props(text));
        out
    })
}

fn material_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: &IconId,
    size: Px,
    color: Color,
) -> AnyElement {
    let svg = svg_source_for_icon(cx, icon);
    let mut props = SvgIconProps::new(svg);
    props.fit = SvgFit::Contain;
    props.layout.size.width = Length::Px(size);
    props.layout.size.height = Length::Px(size);
    props.color = color;
    cx.svg_icon_props(props)
}

fn svg_source_for_icon<H: UiHost>(cx: &mut ElementContext<'_, H>, icon: &IconId) -> SvgSource {
    let resolved = cx
        .app
        .with_global_mut(IconRegistry::default, |icons, _app| {
            icons.resolve_or_missing_owned(icon)
        });

    match resolved {
        ResolvedSvgOwned::Static(bytes) => SvgSource::Static(bytes),
        ResolvedSvgOwned::Bytes(bytes) => SvgSource::Bytes(bytes),
    }
}
