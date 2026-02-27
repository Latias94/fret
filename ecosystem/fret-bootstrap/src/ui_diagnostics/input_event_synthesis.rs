fn pointer_type_from_kind(kind: Option<UiPointerKindV1>) -> PointerType {
    match kind.unwrap_or_default() {
        UiPointerKindV1::Mouse => PointerType::Mouse,
        UiPointerKindV1::Touch => PointerType::Touch,
    }
}

fn move_pointer_event(position: Point, pointer_type: PointerType) -> Event {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();

    Event::Pointer(PointerEvent::Move {
        pointer_id,
        position,
        buttons: MouseButtons::default(),
        modifiers,
        pointer_type,
    })
}

fn wheel_event(position: Point, delta_x: f32, delta_y: f32, pointer_type: PointerType) -> Event {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();

    Event::Pointer(PointerEvent::Wheel {
        pointer_id,
        position,
        delta: Point::new(fret_core::Px(delta_x), fret_core::Px(delta_y)),
        modifiers,
        pointer_type,
    })
}

fn click_events(
    position: Point,
    button: UiMouseButtonV1,
    click_count: u8,
    pointer_type: PointerType,
) -> [Event; 3] {
    click_events_with_modifiers(
        position,
        button,
        click_count,
        Modifiers::default(),
        pointer_type,
    )
}

fn click_events_with_modifiers(
    position: Point,
    button: UiMouseButtonV1,
    click_count: u8,
    modifiers: Modifiers,
    pointer_type: PointerType,
) -> [Event; 3] {
    let pointer_id = PointerId(0);
    let click_count = click_count.max(1);

    let move_event = Event::Pointer(PointerEvent::Move {
        pointer_id,
        position,
        buttons: MouseButtons::default(),
        modifiers,
        pointer_type,
    });
    let button = match button {
        UiMouseButtonV1::Left => MouseButton::Left,
        UiMouseButtonV1::Right => MouseButton::Right,
        UiMouseButtonV1::Middle => MouseButton::Middle,
    };
    let down = Event::Pointer(PointerEvent::Down {
        pointer_id,
        position,
        button,
        modifiers,
        click_count,
        pointer_type,
    });
    let up = Event::Pointer(PointerEvent::Up {
        pointer_id,
        position,
        button,
        modifiers,
        is_click: true,
        click_count,
        pointer_type,
    });

    [move_event, down, up]
}

fn drag_events(
    start: Point,
    end: Point,
    button: UiMouseButtonV1,
    steps: u32,
    pointer_type: PointerType,
) -> Vec<Event> {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();

    let button = match button {
        UiMouseButtonV1::Left => MouseButton::Left,
        UiMouseButtonV1::Right => MouseButton::Right,
        UiMouseButtonV1::Middle => MouseButton::Middle,
    };

    let mut pressed_buttons = MouseButtons::default();
    match button {
        MouseButton::Left => pressed_buttons.left = true,
        MouseButton::Right => pressed_buttons.right = true,
        MouseButton::Middle => pressed_buttons.middle = true,
        _ => {}
    }

    let mut out = Vec::with_capacity(3 + steps as usize);
    out.push(Event::Pointer(PointerEvent::Move {
        pointer_id,
        position: start,
        buttons: MouseButtons::default(),
        modifiers,
        pointer_type,
    }));
    out.push(Event::Pointer(PointerEvent::Down {
        pointer_id,
        position: start,
        button,
        modifiers,
        click_count: 1,
        pointer_type,
    }));

    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let x = start.x.0 + (end.x.0 - start.x.0) * t;
        let y = start.y.0 + (end.y.0 - start.y.0) * t;
        let position = Point::new(fret_core::Px(x), fret_core::Px(y));
        out.push(Event::Pointer(PointerEvent::Move {
            pointer_id,
            position,
            buttons: pressed_buttons,
            modifiers,
            pointer_type,
        }));

        // For scripted diagnostics, also emit `InternalDrag` events during pointer drags. The
        // runtime routes these to the active internal-drag anchor when a cross-window drag session
        // is active (e.g. docking tear-off / drop indicators).
        //
        // This is intentionally safe for generic scripts: `UiTree` ignores `InternalDrag` events
        // unless `app.drag(pointer_id)` exists and is marked `cross_window_hover`.
        out.push(Event::InternalDrag(fret_core::InternalDragEvent {
            pointer_id,
            position,
            kind: fret_core::InternalDragKind::Over,
            modifiers,
        }));
    }

    out.push(Event::Pointer(PointerEvent::Up {
        pointer_id,
        position: end,
        button,
        modifiers,
        is_click: false,
        click_count: 1,
        pointer_type,
    }));

    // Mirror the runner's "mouse-up routes a drop then clears hover" behavior for internal drags.
    out.push(Event::InternalDrag(fret_core::InternalDragEvent {
        pointer_id,
        position: end,
        kind: fret_core::InternalDragKind::Drop,
        modifiers,
    }));
    out
}

fn pointer_move_with_internal_over_events(
    button: UiMouseButtonV1,
    position: Point,
    pointer_type: PointerType,
) -> [Event; 2] {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();

    let pressed_buttons = match button {
        UiMouseButtonV1::Left => MouseButtons {
            left: true,
            ..Default::default()
        },
        UiMouseButtonV1::Right => MouseButtons {
            right: true,
            ..Default::default()
        },
        UiMouseButtonV1::Middle => MouseButtons {
            middle: true,
            ..Default::default()
        },
    };

    let move_event = Event::Pointer(PointerEvent::Move {
        pointer_id,
        position,
        buttons: pressed_buttons,
        modifiers,
        pointer_type,
    });
    let over = Event::InternalDrag(fret_core::InternalDragEvent {
        pointer_id,
        position,
        kind: fret_core::InternalDragKind::Over,
        modifiers,
    });
    [move_event, over]
}

fn pointer_up_with_internal_drop_events(
    button: UiMouseButtonV1,
    position: Point,
    pointer_type: PointerType,
) -> [Event; 2] {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();

    let button = match button {
        UiMouseButtonV1::Left => MouseButton::Left,
        UiMouseButtonV1::Right => MouseButton::Right,
        UiMouseButtonV1::Middle => MouseButton::Middle,
    };

    let up = Event::Pointer(PointerEvent::Up {
        pointer_id,
        position,
        button,
        modifiers,
        is_click: false,
        click_count: 1,
        pointer_type,
    });
    let drop = Event::InternalDrag(fret_core::InternalDragEvent {
        pointer_id,
        position,
        kind: fret_core::InternalDragKind::Drop,
        modifiers,
    });
    [up, drop]
}

fn push_drag_playback_frame(
    state: &mut V2DragPointerState,
    events: &mut Vec<Event>,
    pointer_type: PointerType,
) -> bool {
    let pointer_id = PointerId(0);
    let modifiers = Modifiers::default();

    let (button, pressed_buttons) = match state.button {
        UiMouseButtonV1::Left => (
            MouseButton::Left,
            MouseButtons {
                left: true,
                ..Default::default()
            },
        ),
        UiMouseButtonV1::Right => (
            MouseButton::Right,
            MouseButtons {
                right: true,
                ..Default::default()
            },
        ),
        UiMouseButtonV1::Middle => (
            MouseButton::Middle,
            MouseButtons {
                middle: true,
                ..Default::default()
            },
        ),
    };

    let steps = state.steps.max(1);
    let final_frame = steps.saturating_add(1);

    match state.frame {
        0 => {
            events.push(Event::Pointer(PointerEvent::Move {
                pointer_id,
                position: state.start,
                buttons: MouseButtons::default(),
                modifiers,
                pointer_type,
            }));
            events.push(Event::Pointer(PointerEvent::Down {
                pointer_id,
                position: state.start,
                button,
                modifiers,
                click_count: 1,
                pointer_type,
            }));
            state.frame = 1;
            false
        }
        f if (1..=steps).contains(&f) => {
            let t = f as f32 / steps as f32;
            let x = state.start.x.0 + (state.end.x.0 - state.start.x.0) * t;
            let y = state.start.y.0 + (state.end.y.0 - state.start.y.0) * t;
            let position = Point::new(fret_core::Px(x), fret_core::Px(y));
            events.push(Event::Pointer(PointerEvent::Move {
                pointer_id,
                position,
                buttons: pressed_buttons,
                modifiers,
                pointer_type,
            }));
            events.push(Event::InternalDrag(fret_core::InternalDragEvent {
                pointer_id,
                position,
                kind: fret_core::InternalDragKind::Over,
                modifiers,
            }));
            state.frame = state.frame.saturating_add(1);
            false
        }
        f if f >= final_frame => {
            events.push(Event::Pointer(PointerEvent::Up {
                pointer_id,
                position: state.end,
                button,
                modifiers,
                is_click: false,
                click_count: 1,
                pointer_type,
            }));
            events.push(Event::InternalDrag(fret_core::InternalDragEvent {
                pointer_id,
                position: state.end,
                kind: fret_core::InternalDragKind::Drop,
                modifiers,
            }));
            true
        }
        _ => true,
    }
}

fn drag_playback_last_position(state: &V2DragPointerState) -> Point {
    let steps = state.steps.max(1);
    let final_frame = steps.saturating_add(1);

    match state.frame {
        0 | 1 => state.start,
        f if (2..=final_frame).contains(&f) => {
            let move_frame = (f - 1).min(steps);
            let t = move_frame as f32 / steps as f32;
            let x = state.start.x.0 + (state.end.x.0 - state.start.x.0) * t;
            let y = state.start.y.0 + (state.end.y.0 - state.start.y.0) * t;
            Point::new(fret_core::Px(x), fret_core::Px(y))
        }
        _ => state.end,
    }
}

fn write_cursor_override_window_client_logical(
    out_dir: &Path,
    window: AppWindowId,
    x_px: f32,
    y_px: f32,
) -> Result<(), std::io::Error> {
    let payload = format!(
        "schema_version=1\nkind=window_client_logical\nwindow={}\nx_px={}\ny_px={}\n",
        window.data().as_ffi(),
        x_px,
        y_px
    );
    let text_path = out_dir.join("cursor_screen_pos.override.txt");
    let trigger_path = out_dir.join("cursor_screen_pos.touch");
    std::fs::create_dir_all(out_dir)?;
    std::fs::write(text_path, payload)?;
    touch_file(&trigger_path)?;
    Ok(())
}

fn write_mouse_buttons_override_window_v1(
    out_dir: &Path,
    window: AppWindowId,
    left: Option<bool>,
    right: Option<bool>,
    middle: Option<bool>,
) -> Result<(), std::io::Error> {
    let mut payload = format!("schema_version=1\nwindow={}\n", window.data().as_ffi());
    if let Some(v) = left {
        payload.push_str(&format!("left={}\n", if v { 1 } else { 0 }));
    }
    if let Some(v) = right {
        payload.push_str(&format!("right={}\n", if v { 1 } else { 0 }));
    }
    if let Some(v) = middle {
        payload.push_str(&format!("middle={}\n", if v { 1 } else { 0 }));
    }
    let text_path = out_dir.join("mouse_buttons.override.txt");
    let trigger_path = out_dir.join("mouse_buttons.touch");
    std::fs::create_dir_all(out_dir)?;
    std::fs::write(text_path, payload)?;
    touch_file(&trigger_path)?;
    Ok(())
}

fn write_mouse_buttons_override_all_windows_v1(
    out_dir: &Path,
    left: Option<bool>,
    right: Option<bool>,
    middle: Option<bool>,
) -> Result<(), std::io::Error> {
    let mut payload = "schema_version=1\n".to_string();
    if let Some(v) = left {
        payload.push_str(&format!("left={}\n", if v { 1 } else { 0 }));
    }
    if let Some(v) = right {
        payload.push_str(&format!("right={}\n", if v { 1 } else { 0 }));
    }
    if let Some(v) = middle {
        payload.push_str(&format!("middle={}\n", if v { 1 } else { 0 }));
    }
    let text_path = out_dir.join("mouse_buttons.override.txt");
    let trigger_path = out_dir.join("mouse_buttons.touch");
    std::fs::create_dir_all(out_dir)?;
    std::fs::write(text_path, payload)?;
    touch_file(&trigger_path)?;
    Ok(())
}

fn press_key_events(key: KeyCode, modifiers: UiKeyModifiersV1, repeat: bool) -> [Event; 2] {
    let modifiers = core_modifiers_from_ui(Some(modifiers));
    let down = Event::KeyDown {
        key,
        modifiers,
        repeat,
    };
    let up = Event::KeyUp { key, modifiers };
    [down, up]
}

fn core_modifiers_from_ui(modifiers: Option<UiKeyModifiersV1>) -> Modifiers {
    let modifiers = modifiers.unwrap_or_default();
    Modifiers {
        shift: modifiers.shift,
        ctrl: modifiers.ctrl,
        alt: modifiers.alt,
        meta: modifiers.meta,
        ..Modifiers::default()
    }
}

fn ime_event_kind_name(event: &UiImeEventV1) -> &'static str {
    match event {
        UiImeEventV1::Enabled => "enabled",
        UiImeEventV1::Disabled => "disabled",
        UiImeEventV1::Commit { .. } => "commit",
        UiImeEventV1::Preedit { .. } => "preedit",
        UiImeEventV1::DeleteSurrounding { .. } => "delete_surrounding",
    }
}

fn ime_event_from_v1(event: &UiImeEventV1) -> ImeEvent {
    match event {
        UiImeEventV1::Enabled => ImeEvent::Enabled,
        UiImeEventV1::Disabled => ImeEvent::Disabled,
        UiImeEventV1::Commit { text } => ImeEvent::Commit(text.clone()),
        UiImeEventV1::Preedit { text, cursor_bytes } => ImeEvent::Preedit {
            text: text.clone(),
            cursor: cursor_bytes.map(|(a, b)| (a as usize, b as usize)),
        },
        UiImeEventV1::DeleteSurrounding {
            before_bytes,
            after_bytes,
        } => ImeEvent::DeleteSurrounding {
            before_bytes: (*before_bytes).min(usize::MAX as u32) as usize,
            after_bytes: (*after_bytes).min(usize::MAX as u32) as usize,
        },
    }
}

fn parse_shortcut(shortcut: &str) -> Option<(KeyCode, UiKeyModifiersV1)> {
    let mut parts = shortcut
        .split('+')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return None;
    }

    let key = parts.pop()?;

    let mut modifiers = UiKeyModifiersV1::default();
    for part in parts {
        match part.to_ascii_lowercase().as_str() {
            "shift" => modifiers.shift = true,
            "ctrl" | "control" => modifiers.ctrl = true,
            "alt" => modifiers.alt = true,
            "meta" | "cmd" | "command" | "super" => modifiers.meta = true,
            "primary" => {
                if cfg!(target_os = "macos") {
                    modifiers.meta = true;
                } else {
                    modifiers.ctrl = true;
                }
            }
            _ => return None,
        }
    }

    Some((parse_key_code(key)?, modifiers))
}

fn rect_inset(rect: Rect, insets: UiPaddingInsetsV1) -> Rect {
    let left = Px(insets.left_px.max(0.0));
    let top = Px(insets.top_px.max(0.0));
    let right = Px(insets.right_px.max(0.0));
    let bottom = Px(insets.bottom_px.max(0.0));

    let origin = Point {
        x: rect.origin.x + left,
        y: rect.origin.y + top,
    };
    let w = (rect.size.width.0 - left.0 - right.0).max(0.0);
    let h = (rect.size.height.0 - top.0 - bottom.0).max(0.0);
    Rect {
        origin,
        size: fret_core::Size {
            width: Px(w),
            height: Px(h),
        },
    }
}

fn rect_fully_contains(outer: Rect, inner: Rect) -> bool {
    let ox0 = outer.origin.x.0;
    let oy0 = outer.origin.y.0;
    let ox1 = ox0 + outer.size.width.0;
    let oy1 = oy0 + outer.size.height.0;

    let ix0 = inner.origin.x.0;
    let iy0 = inner.origin.y.0;
    let ix1 = ix0 + inner.size.width.0;
    let iy1 = iy0 + inner.size.height.0;

    ix0 >= ox0 && iy0 >= oy0 && ix1 <= ox1 && iy1 <= oy1
}

fn parse_key_code(key: &str) -> Option<KeyCode> {
    let key = key.trim().to_ascii_lowercase();
    match key.as_str() {
        "shift" => Some(KeyCode::ShiftLeft),
        "ctrl" | "control" => Some(KeyCode::ControlLeft),
        "alt" | "option" => Some(KeyCode::AltLeft),
        "meta" | "super" | "cmd" | "command" => Some(KeyCode::MetaLeft),
        "escape" | "esc" => Some(KeyCode::Escape),
        "enter" | "return" => Some(KeyCode::Enter),
        "tab" => Some(KeyCode::Tab),
        "space" => Some(KeyCode::Space),
        "backspace" => Some(KeyCode::Backspace),
        "delete" | "del" => Some(KeyCode::Delete),
        "f1" => Some(KeyCode::F1),
        "f2" => Some(KeyCode::F2),
        "f3" => Some(KeyCode::F3),
        "f4" => Some(KeyCode::F4),
        "f5" => Some(KeyCode::F5),
        "f6" => Some(KeyCode::F6),
        "f7" => Some(KeyCode::F7),
        "f8" => Some(KeyCode::F8),
        "f9" => Some(KeyCode::F9),
        "f10" => Some(KeyCode::F10),
        "f11" => Some(KeyCode::F11),
        "f12" => Some(KeyCode::F12),
        "arrow_up" | "up" => Some(KeyCode::ArrowUp),
        "arrow_down" | "down" => Some(KeyCode::ArrowDown),
        "arrow_left" | "left" => Some(KeyCode::ArrowLeft),
        "arrow_right" | "right" => Some(KeyCode::ArrowRight),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "page_up" => Some(KeyCode::PageUp),
        "page_down" => Some(KeyCode::PageDown),
        _ => {
            if key.len() == 1 {
                return Some(match key.as_bytes()[0] {
                    b'a' => KeyCode::KeyA,
                    b'b' => KeyCode::KeyB,
                    b'c' => KeyCode::KeyC,
                    b'd' => KeyCode::KeyD,
                    b'e' => KeyCode::KeyE,
                    b'f' => KeyCode::KeyF,
                    b'g' => KeyCode::KeyG,
                    b'h' => KeyCode::KeyH,
                    b'i' => KeyCode::KeyI,
                    b'j' => KeyCode::KeyJ,
                    b'k' => KeyCode::KeyK,
                    b'l' => KeyCode::KeyL,
                    b'm' => KeyCode::KeyM,
                    b'n' => KeyCode::KeyN,
                    b'o' => KeyCode::KeyO,
                    b'p' => KeyCode::KeyP,
                    b'q' => KeyCode::KeyQ,
                    b'r' => KeyCode::KeyR,
                    b's' => KeyCode::KeyS,
                    b't' => KeyCode::KeyT,
                    b'u' => KeyCode::KeyU,
                    b'v' => KeyCode::KeyV,
                    b'w' => KeyCode::KeyW,
                    b'x' => KeyCode::KeyX,
                    b'y' => KeyCode::KeyY,
                    b'z' => KeyCode::KeyZ,
                    b'0' => KeyCode::Digit0,
                    b'1' => KeyCode::Digit1,
                    b'2' => KeyCode::Digit2,
                    b'3' => KeyCode::Digit3,
                    b'4' => KeyCode::Digit4,
                    b'5' => KeyCode::Digit5,
                    b'6' => KeyCode::Digit6,
                    b'7' => KeyCode::Digit7,
                    b'8' => KeyCode::Digit8,
                    b'9' => KeyCode::Digit9,
                    _ => return None,
                });
            }
            None
        }
    }
}

trait PointerEventExt {
    fn kind(&self) -> &'static str;
    fn position(&self) -> Point;
}

impl PointerEventExt for fret_core::PointerEvent {
    fn kind(&self) -> &'static str {
        match self {
            fret_core::PointerEvent::Down { .. } => "down",
            fret_core::PointerEvent::Up { .. } => "up",
            fret_core::PointerEvent::Move { .. } => "move",
            fret_core::PointerEvent::Wheel { .. } => "wheel",
            fret_core::PointerEvent::PinchGesture { .. } => "pinch_gesture",
        }
    }

    fn position(&self) -> Point {
        match self {
            fret_core::PointerEvent::Down { position, .. } => *position,
            fret_core::PointerEvent::Up { position, .. } => *position,
            fret_core::PointerEvent::Move { position, .. } => *position,
            fret_core::PointerEvent::Wheel { position, .. } => *position,
            fret_core::PointerEvent::PinchGesture { position, .. } => *position,
        }
    }
}

trait EventPointerExt {
    fn pointer_event(&self) -> Option<&fret_core::PointerEvent>;
}

impl EventPointerExt for Event {
    fn pointer_event(&self) -> Option<&fret_core::PointerEvent> {
        match self {
            Event::Pointer(p) => Some(p),
            _ => None,
        }
    }
}
