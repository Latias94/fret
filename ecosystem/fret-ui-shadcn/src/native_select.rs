use std::sync::{Arc, Mutex};

use fret_core::{Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::OnCloseAutoFocus;
use fret_ui::element::{
    AnyElement, ContainerProps, InsetStyle, LayoutStyle, Length, PositionStyle, PressableA11y,
    PressableProps, SizeStyle,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::controllable_state;
use fret_ui_kit::primitives::popover as radix_popover;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, Radius, ShadowPreset,
    Size as ComponentSize, Space, ui,
};

use crate::{
    CommandEntry, CommandGroup, CommandItem, CommandList, CommandSeparator, Popover, PopoverContent,
};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NativeSelectSize {
    Sm,
    #[default]
    Default,
}

#[derive(Debug, Clone)]
pub struct NativeSelectOption {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub disabled: bool,
}

impl NativeSelectOption {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// Convenience for the common shadcn NativeSelect pattern where the "placeholder" is the first
    /// option with an empty value. Selecting it clears the model to `None`.
    pub fn placeholder(label: impl Into<Arc<str>>) -> Self {
        Self::new("", label)
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Debug, Clone)]
pub struct NativeSelectOptGroup {
    pub label: Arc<str>,
    pub disabled: bool,
    pub options: Vec<NativeSelectOption>,
}

impl NativeSelectOptGroup {
    pub fn new(
        label: impl Into<Arc<str>>,
        options: impl IntoIterator<Item = NativeSelectOption>,
    ) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            options: options.into_iter().collect(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Default)]
struct NativeSelectState {
    focus_restore_target: Option<Arc<Mutex<Option<GlobalElementId>>>>,
}

#[derive(Clone)]
pub struct NativeSelect {
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    placeholder: Arc<str>,
    options: Vec<NativeSelectOption>,
    optgroups: Vec<NativeSelectOptGroup>,
    test_id_prefix: Option<Arc<str>>,
    trigger_test_id: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    aria_invalid: bool,
    disabled: bool,
    size: NativeSelectSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for NativeSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeSelect")
            .field("placeholder", &self.placeholder.as_ref())
            .field("options_len", &self.options.len())
            .field("optgroups_len", &self.optgroups.len())
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("aria_invalid", &self.aria_invalid)
            .field("disabled", &self.disabled)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl NativeSelect {
    pub fn new(model: Model<Option<Arc<str>>>, open: Model<bool>) -> Self {
        Self {
            model,
            open,
            placeholder: Arc::from("Select..."),
            options: Vec::new(),
            optgroups: Vec::new(),
            test_id_prefix: None,
            trigger_test_id: None,
            a11y_label: None,
            aria_invalid: false,
            disabled: false,
            size: NativeSelectSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Creates a NativeSelect with controlled/uncontrolled `value` + `open` models.
    ///
    /// This mirrors the Radix-style controlled/uncontrolled pattern used across this crate.
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        value: Option<Model<Option<Arc<str>>>>,
        default_value: Option<Arc<str>>,
        open: Option<Model<bool>>,
        default_open: bool,
    ) -> Self {
        let open = radix_popover::PopoverRoot::new()
            .open(open)
            .default_open(default_open)
            .open_model(cx);
        let value = controllable_state::use_controllable_model(cx, value, || default_value).model();
        Self::new(value, open)
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn option(mut self, option: NativeSelectOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = NativeSelectOption>) -> Self {
        self.options.extend(options);
        self
    }

    pub fn optgroup(mut self, optgroup: NativeSelectOptGroup) -> Self {
        self.optgroups.push(optgroup);
        self
    }

    pub fn optgroups(mut self, optgroups: impl IntoIterator<Item = NativeSelectOptGroup>) -> Self {
        self.optgroups.extend(optgroups);
        self
    }

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    pub fn trigger_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.trigger_test_id = Some(id.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn aria_invalid(mut self, aria_invalid: bool) -> Self {
        self.aria_invalid = aria_invalid;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(mut self, size: NativeSelectSize) -> Self {
        self.size = size;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        native_select(
            cx,
            self.model,
            self.open,
            self.placeholder,
            &self.options,
            &self.optgroups,
            self.test_id_prefix,
            self.trigger_test_id,
            self.a11y_label,
            self.aria_invalid,
            self.disabled,
            self.size,
            self.chrome,
            self.layout,
        )
    }
}

fn test_id_slug(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

#[allow(clippy::too_many_arguments)]
pub fn native_select<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    open: Model<bool>,
    placeholder: Arc<str>,
    options: &[NativeSelectOption],
    optgroups: &[NativeSelectOptGroup],
    test_id_prefix: Option<Arc<str>>,
    trigger_test_id: Option<Arc<str>>,
    a11y_label: Option<Arc<str>>,
    aria_invalid: bool,
    disabled: bool,
    size: NativeSelectSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
) -> AnyElement {
    cx.scope(|cx| {
        let theme = Theme::global(&*cx.app).snapshot();

        let focus_restore_target = {
            let existing = cx.with_state(NativeSelectState::default, |st| {
                st.focus_restore_target.clone()
            });
            if let Some(target) = existing {
                target
            } else {
                let target = Arc::new(Mutex::new(None));
                cx.with_state(NativeSelectState::default, |st| {
                    st.focus_restore_target = Some(target.clone());
                });
                target
            }
        };

        let is_open: bool = cx.watch_model(&open).layout().copied().unwrap_or(false);
        let selected: Option<Arc<str>> = cx.watch_model(&model).cloned().unwrap_or_default();

        let resolved = resolve_input_chrome(
            Theme::global(&*cx.app),
            ComponentSize::default(),
            &chrome,
            InputTokenKeys::none(),
        );

        let (h, py) = match size {
            NativeSelectSize::Sm => (Px(32.0), Px(4.0)),
            NativeSelectSize::Default => (Px(36.0), Px(8.0)),
        };

        let mut text_style = typography::control_text_style_scaled(
            Theme::global(&*cx.app),
            FontId::ui(),
            resolved.text_px,
        );
        text_style.weight = FontWeight::NORMAL;

        let muted_fg = theme.color_token("muted-foreground");

        let (resolved_label, has_selection): (Arc<str>, bool) = selected
            .as_ref()
            .and_then(|v: &Arc<str>| {
                options
                    .iter()
                    .chain(optgroups.iter().flat_map(|g| g.options.iter()))
                    .find(|opt| opt.value.as_ref() == v.as_ref())
            })
            .map(|opt| (opt.label.clone(), true))
            .unwrap_or((placeholder.clone(), false));

        let mut border_color = resolved.border_color;
        let mut focus_ring = decl_style::focus_ring(&theme, resolved.radius);
        if aria_invalid {
            border_color = theme.color_token("destructive");
            focus_ring.color =
                crate::theme_variants::invalid_control_ring_color(&theme, border_color);
        }

        let layout = decl_style::layout_style(&theme, layout.relative().min_w_0());
        let mut pressable_layout = layout;
        pressable_layout.size = SizeStyle {
            height: Length::Px(h),
            ..pressable_layout.size
        };

        let icon_size = Px(16.0);
        let icon_right = Px(14.0);
        let icon_top = Px((h.0 - icon_size.0) * 0.5);
        let icon_color = alpha_mul(theme.color_token("muted-foreground"), 0.5);

        let has_entries = !options.is_empty() || !optgroups.is_empty();

        let open_for_trigger = open.clone();
        let a11y_label_for_trigger = a11y_label
            .clone()
            .or_else(|| Some(Arc::from("Native select")));
        let trigger_test_id_for_trigger = trigger_test_id.clone();
        let theme_for_trigger = theme.clone();
        let focus_restore_target_for_trigger = focus_restore_target.clone();
        let test_id_prefix_for_trigger = test_id_prefix.clone();
        let test_id_prefix_for_content = test_id_prefix.clone();

        let trigger = control_chrome_pressable_with_id_props(cx, move |cx, _st, trigger_id| {
            *focus_restore_target_for_trigger
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = Some(trigger_id);

            if has_entries && !disabled {
                cx.pressable_toggle_bool(&open_for_trigger);
            }

            let pressable_props = PressableProps {
                layout: pressable_layout,
                enabled: !disabled,
                focusable: !disabled,
                focus_ring: Some(focus_ring),
                a11y: PressableA11y {
                    role: Some(SemanticsRole::ComboBox),
                    label: a11y_label_for_trigger.clone(),
                    test_id: trigger_test_id_for_trigger.clone(),
                    expanded: Some(is_open),
                    ..Default::default()
                },
                ..Default::default()
            };

            // IMPORTANT: absolute positioning in `Container` is resolved relative to the container's
            // inner content box (padding+border are excluded). shadcn's NativeSelect positions the
            // chevron relative to a zero-padding wrapper, so we mirror that structure here:
            //
            // Pressable -> (clip) chrome wrapper (no padding) -> surface (padding+border) + chevron (absolute)
            let chrome_props = ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                padding: Edges::all(Px(0.0)).into(),
                background: None,
                shadow: Some(decl_style::shadow_xs(&theme_for_trigger, resolved.radius)),
                border: Edges::all(Px(0.0)),
                border_color: None,
                corner_radii: Corners::all(resolved.radius),
                ..Default::default()
            };

            let label = resolved_label.clone();
            let label_is_placeholder = !has_selection;
            let text_style = text_style.clone();
            (
                pressable_props,
                chrome_props,
                move |cx: &mut ElementContext<'_, H>| {
                    let surface = {
                        let surface_props = ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            padding: Edges {
                                left: resolved.padding.left,
                                right: Px(36.0),
                                top: py,
                                bottom: py,
                            }
                            .into(),
                            background: Some(resolved.background),
                            shadow: None,
                            border: Edges::all(resolved.border_width),
                            border_color: Some(border_color),
                            corner_radii: Corners::all(resolved.radius),
                            ..Default::default()
                        };
                        cx.container(surface_props, move |cx| {
                            let fg = if label_is_placeholder {
                                ColorRef::Color(muted_fg)
                            } else {
                                ColorRef::Color(resolved.text_color)
                            };
                            let mut content = ui::text(cx, label)
                                .text_size_px(text_style.size)
                                .fixed_line_box_px(
                                    text_style.line_height.unwrap_or(text_style.size),
                                )
                                .line_box_in_bounds()
                                .font_normal()
                                .nowrap()
                                .text_color(fg)
                                .truncate();

                            content = content.overflow(fret_core::TextOverflow::Clip);
                            vec![content.into_element(cx)]
                        })
                    };

                    let icon = decl_icon::icon_with(
                        cx,
                        fret_icons::ids::ui::CHEVRON_DOWN,
                        Some(icon_size),
                        Some(ColorRef::Color(icon_color)),
                    );
                    let mut icon = cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: InsetStyle {
                                    left: None.into(),
                                    top: Some(icon_top).into(),
                                    right: Some(icon_right).into(),
                                    bottom: None.into(),
                                },
                                size: SizeStyle {
                                    width: Length::Px(icon_size),
                                    height: Length::Px(icon_size),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            padding: Edges::all(Px(0.0)).into(),
                            background: None,
                            shadow: None,
                            border: Edges::all(Px(0.0)),
                            border_color: None,
                            corner_radii: Corners::all(Px(0.0)),
                            ..Default::default()
                        },
                        move |_cx| vec![icon],
                    );

                    if let Some(prefix) = test_id_prefix_for_trigger.as_deref() {
                        icon = icon.test_id(format!("{prefix}-icon"));
                    }

                    let disabled = disabled;
                    let out: Vec<AnyElement> = if disabled {
                        vec![cx.opacity(0.5, move |_cx| vec![surface, icon])]
                    } else {
                        vec![surface, icon]
                    };
                    out
                },
            )
        });

        let close_auto_focus: OnCloseAutoFocus = Arc::new({
            let focus_restore_target = focus_restore_target.clone();
            move |host, _action_cx, req| {
                req.prevent_default();
                let target = *focus_restore_target
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                if let Some(target) = target {
                    host.request_focus(target);
                }
            }
        });

        let popover = Popover::new(open.clone())
            .auto_focus(true)
            .on_close_auto_focus(Some(close_auto_focus));

        popover.into_element_with_anchor(
            cx,
            move |_cx| trigger,
            move |cx, anchor| {
                let desired_w = Px(anchor.size.width.0.max(180.0));
                let max_list_h = theme
                    .metric_by_key("component.native_select.max_list_height")
                    .or_else(|| theme.metric_by_key("component.select.max_list_height"))
                    .unwrap_or(Px(280.0));

                let selected: Option<Arc<str>> =
                    cx.watch_model(&model).cloned().unwrap_or_default();

                let fg = theme
                    .color_by_key("foreground")
                    .unwrap_or_else(|| theme.color_token("foreground"));
                let fg_disabled = alpha_mul(fg, 0.5);
                let item_text_style = crate::command::item_text_style(&theme);

                let open_for_select = open.clone();
                let model_for_select = model.clone();

                let mut make_item =
                    |option: NativeSelectOption, group_disabled: bool| -> CommandItem {
                        let item_disabled = disabled || group_disabled || option.disabled;
                        let is_selected = selected
                            .as_ref()
                            .is_some_and(|v: &Arc<str>| v.as_ref() == option.value.as_ref());

                        let value_for_select = option.value.clone();
                        let on_select: fret_ui::action::OnActivate = Arc::new({
                            let model_for_select = model_for_select.clone();
                            let open_for_select = open_for_select.clone();
                            move |host, action_cx, _reason| {
                                let next_value = if value_for_select.as_ref().is_empty() {
                                    None
                                } else {
                                    Some(value_for_select.clone())
                                };
                                let _ = host.models_mut().update(&model_for_select, |v| {
                                    *v = next_value;
                                });
                                let _ = host.models_mut().update(&open_for_select, |v| *v = false);
                                host.request_redraw(action_cx.window);
                            }
                        });

                        let label_text = option.label.clone();
                        let label_style = item_text_style.clone();
                        let icon = decl_icon::icon_with(
                            cx,
                            fret_icons::ids::ui::CHECK,
                            Some(Px(16.0)),
                            Some(ColorRef::Color(if item_disabled {
                                fg_disabled
                            } else {
                                fg
                            })),
                        );
                        let icon =
                            cx.opacity(if is_selected { 1.0 } else { 0.0 }, move |_cx| vec![icon]);

                        let text = {
                            let mut label = ui::label(cx, label_text.clone())
                                .text_size_px(label_style.size)
                                .font_weight(label_style.weight)
                                .text_color(ColorRef::Color(if item_disabled {
                                    fg_disabled
                                } else {
                                    fg
                                }))
                                .truncate();
                            if let Some(line_height) = label_style.line_height {
                                label = label.line_height_px(line_height).line_height_policy(
                                    fret_core::TextLineHeightPolicy::FixedFromStyle,
                                );
                            }
                            if let Some(letter_spacing_em) = label_style.letter_spacing_em {
                                label = label.letter_spacing_em(letter_spacing_em);
                            }
                            label.into_element(cx)
                        };

                        let mut item = CommandItem::new(label_text)
                            .value(option.value.clone())
                            .disabled(item_disabled)
                            .on_select_action(on_select)
                            .children(vec![text, icon]);

                        if let Some(prefix) = test_id_prefix_for_content.as_deref() {
                            item = item.test_id(format!(
                                "{prefix}-item-{}",
                                test_id_slug(option.value.as_ref())
                            ));
                        }

                        item
                    };

                let mut entries: Vec<CommandEntry> =
                    Vec::with_capacity(options.len() + optgroups.len());

                for option in options.iter().cloned() {
                    entries.push(CommandEntry::Item(make_item(option, false)));
                }

                let non_empty_groups: Vec<NativeSelectOptGroup> = optgroups
                    .iter()
                    .cloned()
                    .filter(|g| !g.options.is_empty())
                    .collect();
                if !options.is_empty() && !non_empty_groups.is_empty() {
                    entries.push(CommandEntry::Separator(CommandSeparator::new()));
                }

                for group in non_empty_groups {
                    let group_disabled = group.disabled;
                    let group_items: Vec<CommandItem> = group
                        .options
                        .into_iter()
                        .map(|opt| make_item(opt, group_disabled))
                        .collect();
                    entries.push(CommandEntry::Group(
                        CommandGroup::new(group_items).heading(group.label),
                    ));
                }

                let popover_surface = ChromeRefinement::default()
                    .rounded(Radius::Md)
                    .border_width(Px(1.0))
                    .border_color(ColorRef::Token {
                        key: "border",
                        fallback: ColorFallback::ThemePanelBorder,
                    })
                    .bg(ColorRef::Token {
                        key: "popover.background",
                        fallback: ColorFallback::ThemePanelBackground,
                    })
                    .shadow(ShadowPreset::Md)
                    .p(Space::N1);

                let mut list = CommandList::new_entries(entries)
                    .disabled(disabled)
                    .empty_text("No options.")
                    .refine_scroll_layout(LayoutRefinement::default().max_h(max_list_h))
                    .into_element(cx);

                if let Some(prefix) = test_id_prefix_for_content.as_deref() {
                    list = list.test_id(format!("{prefix}-listbox"));
                }

                PopoverContent::new(vec![list])
                    .refine_style(popover_surface)
                    .refine_layout(LayoutRefinement::default().w_px(desired_w).min_w_0())
                    .into_element(cx)
            },
        )
    })
}
