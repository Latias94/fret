fn invalidation_label(inv: Invalidation) -> &'static str {
    match inv {
        Invalidation::Paint => "paint",
        Invalidation::Layout => "layout",
        Invalidation::HitTest => "hit_test",
        Invalidation::HitTestOnly => "hit_test_only",
    }
}

fn pointer_occlusion_label(occlusion: fret_ui::tree::PointerOcclusion) -> String {
    match occlusion {
        fret_ui::tree::PointerOcclusion::None => "none",
        fret_ui::tree::PointerOcclusion::BlockMouse => "block_mouse",
        fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll => "block_mouse_except_scroll",
    }
    .to_string()
}

fn viewport_pointer_type_label(pointer_type: fret_core::PointerType) -> &'static str {
    match pointer_type {
        fret_core::PointerType::Mouse => "mouse",
        fret_core::PointerType::Touch => "touch",
        fret_core::PointerType::Pen => "pen",
        fret_core::PointerType::Unknown => "unknown",
    }
}

fn color_scheme_label(scheme: fret_core::ColorScheme) -> &'static str {
    match scheme {
        fret_core::ColorScheme::Light => "light",
        fret_core::ColorScheme::Dark => "dark",
    }
}

fn contrast_preference_label(preference: fret_core::ContrastPreference) -> &'static str {
    match preference {
        fret_core::ContrastPreference::NoPreference => "no_preference",
        fret_core::ContrastPreference::More => "more",
        fret_core::ContrastPreference::Less => "less",
        fret_core::ContrastPreference::Custom => "custom",
    }
}

fn forced_colors_mode_label(mode: fret_core::ForcedColorsMode) -> &'static str {
    match mode {
        fret_core::ForcedColorsMode::None => "none",
        fret_core::ForcedColorsMode::Active => "active",
    }
}

fn viewport_cancel_reason_label(reason: fret_core::PointerCancelReason) -> &'static str {
    match reason {
        fret_core::PointerCancelReason::LeftWindow => "left_window",
    }
}

fn event_kind(event: &Event) -> String {
    match event {
        Event::Pointer(p) => format!("pointer.{}", p.kind()),
        Event::KeyDown { .. } => "key.down".to_string(),
        Event::KeyUp { .. } => "key.up".to_string(),
        Event::TextInput(_) => "text.input".to_string(),
        Event::Ime(_) => "ime".to_string(),
        Event::Timer { .. } => "timer".to_string(),
        Event::WindowCloseRequested => "window.close_requested".to_string(),
        other => format!("{other:?}")
            .split_whitespace()
            .next()
            .unwrap_or("event")
            .to_string(),
    }
}

fn event_debug_string(event: &Event, redact_text: bool) -> String {
    if !redact_text {
        return format!("{event:?}");
    }

    match event {
        Event::TextInput(text) => format!("TextInput(len={})", text.len()),
        Event::Ime(_) => "Ime(<redacted>)".to_string(),
        _ => format!("{event:?}"),
    }
}

fn unix_ms_now() -> u64 {
    fret_core::time::SystemTime::now()
        .duration_since(fret_core::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default()
}

fn reason_code_for_script_failure(reason: &str) -> Option<&'static str> {
    let reason = reason.trim();
    if reason.is_empty() {
        return None;
    }

    match reason {
        "no_semantics_snapshot" => Some("semantics.missing"),
        "assert_failed" => Some("assert.failed"),
        "window_target_unresolved" => Some("window.target_unresolved"),
        _ if reason.contains("focus") => Some("focus.mismatch"),
        _ if reason.ends_with("_timeout") => Some("timeout"),
        _ if reason.contains("no_semantics_match") || reason.contains("no_match") => {
            Some("selector.not_found")
        }
        _ => None,
    }
}
