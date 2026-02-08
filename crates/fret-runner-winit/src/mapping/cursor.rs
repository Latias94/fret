pub fn map_cursor_icon(icon: fret_core::CursorIcon) -> winit::cursor::CursorIcon {
    match icon {
        fret_core::CursorIcon::Default => winit::cursor::CursorIcon::Default,
        fret_core::CursorIcon::Pointer => winit::cursor::CursorIcon::Pointer,
        fret_core::CursorIcon::Text => winit::cursor::CursorIcon::Text,
        fret_core::CursorIcon::ColResize => winit::cursor::CursorIcon::ColResize,
        fret_core::CursorIcon::RowResize => winit::cursor::CursorIcon::RowResize,
        fret_core::CursorIcon::NwseResize => winit::cursor::CursorIcon::NwseResize,
        fret_core::CursorIcon::NeswResize => winit::cursor::CursorIcon::NeswResize,
    }
}
