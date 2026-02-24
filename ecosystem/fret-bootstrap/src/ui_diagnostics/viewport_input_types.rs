#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiViewportInputEventV1 {
    pub target: u64,
    pub pointer_id: u64,
    pub pointer_type: String,
    pub cursor_px: PointV1,
    pub uv: (f32, f32),
    pub target_px: (u32, u32),
    pub kind: UiViewportInputKindV1,
}

impl UiViewportInputEventV1 {
    fn from_event(event: fret_core::ViewportInputEvent) -> Self {
        Self {
            target: event.target.data().as_ffi(),
            pointer_id: event.pointer_id.0 as u64,
            pointer_type: viewport_pointer_type_label(event.pointer_type).to_string(),
            cursor_px: PointV1::from(event.cursor_px),
            uv: event.uv,
            target_px: event.target_px,
            kind: UiViewportInputKindV1::from_kind(event.kind),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiViewportInputKindV1 {
    PointerMove {
        buttons: UiMouseButtonsV1,
        modifiers: UiKeyModifiersV1,
    },
    PointerDown {
        button: UiMouseButtonV1,
        modifiers: UiKeyModifiersV1,
        click_count: u8,
    },
    PointerUp {
        button: UiMouseButtonV1,
        modifiers: UiKeyModifiersV1,
        is_click: bool,
        click_count: u8,
    },
    PointerCancel {
        buttons: UiMouseButtonsV1,
        modifiers: UiKeyModifiersV1,
        reason: String,
    },
    Wheel {
        delta: PointV1,
        modifiers: UiKeyModifiersV1,
    },
}

impl UiViewportInputKindV1 {
    fn from_kind(kind: fret_core::ViewportInputKind) -> Self {
        match kind {
            fret_core::ViewportInputKind::PointerMove { buttons, modifiers } => Self::PointerMove {
                buttons: UiMouseButtonsV1::from_buttons(buttons),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
            },
            fret_core::ViewportInputKind::PointerDown {
                button,
                modifiers,
                click_count,
            } => Self::PointerDown {
                button: UiMouseButtonV1::from_button(button),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
                click_count,
            },
            fret_core::ViewportInputKind::PointerUp {
                button,
                modifiers,
                is_click,
                click_count,
            } => Self::PointerUp {
                button: UiMouseButtonV1::from_button(button),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
                is_click,
                click_count,
            },
            fret_core::ViewportInputKind::PointerCancel {
                buttons,
                modifiers,
                reason,
            } => Self::PointerCancel {
                buttons: UiMouseButtonsV1::from_buttons(buttons),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
                reason: viewport_cancel_reason_label(reason).to_string(),
            },
            fret_core::ViewportInputKind::Wheel { delta, modifiers } => Self::Wheel {
                delta: PointV1::from(delta),
                modifiers: UiKeyModifiersV1::from_modifiers(modifiers),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiMouseButtonsV1 {
    #[serde(default)]
    pub left: bool,
    #[serde(default)]
    pub right: bool,
    #[serde(default)]
    pub middle: bool,
}

impl UiMouseButtonsV1 {
    fn from_buttons(buttons: fret_core::MouseButtons) -> Self {
        Self {
            left: buttons.left,
            right: buttons.right,
            middle: buttons.middle,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiAxisV1 {
    Horizontal,
    Vertical,
}

impl UiAxisV1 {
    fn from_axis(axis: fret_core::Axis) -> Self {
        match axis {
            fret_core::Axis::Horizontal => Self::Horizontal,
            fret_core::Axis::Vertical => Self::Vertical,
        }
    }
}
