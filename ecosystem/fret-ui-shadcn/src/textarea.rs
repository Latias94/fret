use std::sync::Arc;

use fret_core::{Color, Corners, CursorIcon, Edges, FontId, MouseButton, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, Length, PressableProps, SizeStyle, TextAreaProps,
};
use fret_ui::{ElementContext, TextAreaStyle, Theme, UiHost, action};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::recipes::input::{InputTokenKeys, resolve_input_chrome};
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Size as ComponentSize, Space};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TextareaResizeDrag {
    start: fret_core::Point,
    start_height: Px,
}

#[derive(Default)]
struct TextareaResizeState {
    height_override: Option<Model<Option<Px>>>,
    drag: Option<Model<Option<TextareaResizeDrag>>>,
}

fn textarea_resize_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<Option<Px>>, Model<Option<TextareaResizeDrag>>) {
    let needs_init = cx.with_state(TextareaResizeState::default, |st| {
        st.height_override.is_none() || st.drag.is_none()
    });

    if needs_init {
        let height_override = cx.app.models_mut().insert(None::<Px>);
        let drag = cx.app.models_mut().insert(None::<TextareaResizeDrag>);
        cx.with_state(TextareaResizeState::default, |st| {
            st.height_override = Some(height_override.clone());
            st.drag = Some(drag.clone());
        });
        return (height_override, drag);
    }

    cx.with_state(TextareaResizeState::default, |st| {
        (
            st.height_override.clone().expect("height_override"),
            st.drag.clone().expect("drag"),
        )
    })
}

#[derive(Clone)]
pub struct Textarea {
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    aria_invalid: bool,
    disabled: bool,
    min_height: Px,
    resizable: bool,
    stable_line_boxes: bool,
    size: ComponentSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Textarea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Textarea")
            .field("model", &"<model>")
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field(
                "placeholder",
                &self.placeholder.as_ref().map(|s| s.as_ref()),
            )
            .field("min_height", &self.min_height)
            .field("resizable", &self.resizable)
            .field("size", &self.size)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Textarea {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            a11y_label: None,
            placeholder: None,
            aria_invalid: false,
            disabled: false,
            min_height: Px(64.0),
            resizable: true,
            stable_line_boxes: true,
            size: ComponentSize::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = Some(placeholder.into());
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

    pub fn min_height(mut self, min_height: Px) -> Self {
        self.min_height = min_height;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// If true, uses a fixed line box + forced strut for stable multiline metrics (UI/form
    /// surfaces). If false, uses an expand-to-fit line box to avoid clipping (content surfaces).
    pub fn stable_line_boxes(mut self, stable: bool) -> Self {
        self.stable_line_boxes = stable;
        self
    }

    pub fn size(mut self, size: ComponentSize) -> Self {
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
        textarea(
            cx,
            self.model,
            self.a11y_label,
            self.placeholder,
            self.aria_invalid,
            self.disabled,
            self.min_height,
            self.resizable,
            self.stable_line_boxes,
            self.size,
            self.chrome,
            self.layout,
        )
    }
}

pub fn textarea<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    a11y_label: Option<Arc<str>>,
    placeholder: Option<Arc<str>>,
    aria_invalid: bool,
    disabled: bool,
    min_height: Px,
    resizable: bool,
    stable_line_boxes: bool,
    size: ComponentSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
) -> AnyElement {
    let show_resize_handle = resizable && !disabled;

    let theme = Theme::global(&*cx.app).clone();

    let resolved = resolve_input_chrome(&theme, size, &chrome, InputTokenKeys::none());

    let text_style = if stable_line_boxes {
        typography::text_area_control_text_style_scaled(&theme, FontId::ui(), resolved.text_px)
    } else {
        typography::text_area_content_text_style_scaled(&theme, FontId::ui(), resolved.text_px)
    };

    let mut chrome = TextAreaStyle::default();
    chrome.padding_x = resolved.padding.left;
    chrome.padding_y = resolved.padding.top;
    chrome.background = resolved.background;
    chrome.border = Edges::all(resolved.border_width);
    chrome.border_color = resolved.border_color;
    chrome.corner_radii = Corners::all(resolved.radius);
    chrome.text_color = resolved.text_color;
    chrome.placeholder_color = theme
        .color_by_key("muted-foreground")
        .unwrap_or(chrome.placeholder_color);
    chrome.selection_color = alpha_mul(resolved.selection_color, 0.65);
    chrome.caret_color = resolved.text_color;
    chrome.preedit_bg_color = alpha_mul(resolved.selection_color, 0.22);
    chrome.preedit_underline_color = resolved.selection_color;
    chrome.focus_ring = Some(decl_style::focus_ring(&theme, resolved.radius));

    if aria_invalid {
        let border_color = theme.color_token("destructive");
        chrome.border_color = border_color;
        if let Some(mut ring) = chrome.focus_ring.take() {
            let ring_key = if theme.name.contains("/dark") {
                "destructive/40"
            } else {
                "destructive/20"
            };
            ring.color = theme
                .color_by_key(ring_key)
                .or_else(|| theme.color_by_key("destructive/20"))
                .unwrap_or(border_color);
            chrome.focus_ring = Some(ring);
        }
    }

    let root_layout = decl_style::layout_style(&theme, layout.relative().w_full());

    let mut props = TextAreaProps::new(model);
    props.enabled = !disabled;
    props.focusable = !disabled;
    props.a11y_label = a11y_label;
    props.placeholder = placeholder;
    props.chrome = chrome;
    props.text_style = text_style;
    props.min_height = min_height;
    props.layout = root_layout;
    props.layout.size = SizeStyle {
        width: Length::Fill,
        height: Length::Auto,
        min_height: Some(min_height),
        ..Default::default()
    };

    if disabled {
        return cx.opacity(0.5, move |cx| vec![cx.text_area(props)]);
    }

    if !show_resize_handle {
        return cx.text_area(props);
    }

    let outer_layout = props.layout;
    let size_style = props.layout.size;
    let mut inner_layout = decl_style::layout_style(&theme, LayoutRefinement::default().w_full());
    inner_layout.size = size_style;
    props.layout = inner_layout;

    cx.container(
        ContainerProps {
            layout: outer_layout,
            padding: Edges::all(Px(0.0)),
            background: None,
            shadow: None,
            border: Edges::all(Px(0.0)),
            border_color: None,
            corner_radii: Corners::all(Px(0.0)),
            ..Default::default()
        },
        move |cx| {
            let (height_override, drag) = textarea_resize_models(cx);
            let override_px = cx
                .app
                .models_mut()
                .read(&height_override, |v| *v)
                .ok()
                .flatten();

            let theme = Theme::global(&*cx.app).clone();
            let resize_handle_layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default()
                    .absolute()
                    .right(Space::N1)
                    .bottom(Space::N1)
                    .w_px(Px(14.0))
                    .h_px(Px(14.0)),
            );
            let grip_color = theme
                .color_by_key("muted-foreground")
                .unwrap_or_else(|| theme.color_token("foreground"));
            let grip_border_color = alpha_mul(grip_color, 0.55);
            let grip_layout =
                decl_style::layout_style(&theme, LayoutRefinement::default().size_full());

            let mut props = props;
            if let Some(px) = override_px {
                props.layout.size.height = Length::Px(px);
            }

            let textarea = cx.text_area(props);

            let resize_handle = cx.pressable_with_id_props(move |cx, _st, id| {
                let height_override_down = height_override.clone();
                let drag_down = drag.clone();
                cx.pressable_on_pointer_down_for(
                    id,
                    Arc::new(move |host, _action_cx, down| {
                        if down.button != MouseButton::Left {
                            return action::PressablePointerDownResult::SkipDefault;
                        }

                        host.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
                        host.capture_pointer();
                        host.set_cursor_icon(CursorIcon::RowResize);

                        let start = down.position_window.unwrap_or(down.position);
                        let start_height = host
                            .models_mut()
                            .read(&height_override_down, |v| *v)
                            .ok()
                            .flatten()
                            .unwrap_or(min_height);

                        let _ = host.models_mut().update(&drag_down, |v| {
                            *v = Some(TextareaResizeDrag {
                                start,
                                start_height,
                            });
                        });

                        action::PressablePointerDownResult::SkipDefaultAndStopPropagation
                    }),
                );

                let height_override_move = height_override.clone();
                let drag_move = drag.clone();
                cx.pressable_on_pointer_move_for(
                    id,
                    Arc::new(move |host, action_cx, mv| {
                        host.set_cursor_icon(CursorIcon::RowResize);

                        let Some(drag) = host.models_mut().read(&drag_move, |v| *v).ok().flatten()
                        else {
                            return false;
                        };

                        let current = mv.position_window.unwrap_or(mv.position);
                        let delta = current.y.0 - drag.start.y.0;
                        let next_height = Px((drag.start_height.0 + delta).max(min_height.0));

                        let _ = host
                            .models_mut()
                            .update(&height_override_move, |v| *v = Some(next_height));

                        host.request_redraw(action_cx.window);
                        true
                    }),
                );

                let drag_up = drag.clone();
                cx.pressable_on_pointer_up_for(
                    id,
                    Arc::new(move |host, _action_cx, _up| {
                        host.release_pointer_capture();
                        let _ = host.models_mut().update(&drag_up, |v| *v = None);
                        action::PressablePointerUpResult::SkipActivate
                    }),
                );

                let mut pressable = PressableProps::default();
                pressable.layout = resize_handle_layout;
                let grip = cx.container(
                    ContainerProps {
                        layout: grip_layout,
                        padding: Edges::all(Px(0.0)),
                        background: None,
                        shadow: None,
                        border: Edges::all(Px(1.0)),
                        border_color: Some(grip_border_color),
                        corner_radii: Corners::all(Px(3.0)),
                        ..Default::default()
                    },
                    move |_cx| [],
                );
                (pressable, vec![grip])
            });

            vec![textarea, resize_handle]
        },
    )
}
