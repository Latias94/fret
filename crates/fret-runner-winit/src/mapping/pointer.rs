use fret_core::{PointerId, PointerType};
use winit::event::{ButtonSource, DeviceId, PointerKind, PointerSource};

pub fn map_pointer_button(button: &ButtonSource) -> Option<winit::event::MouseButton> {
    match button {
        ButtonSource::Mouse(mouse) => Some(*mouse),
        ButtonSource::Touch { .. } => Some(winit::event::MouseButton::Left),
        ButtonSource::TabletTool { .. } => Some(winit::event::MouseButton::Left),
        ButtonSource::Unknown(_) => None,
    }
}

pub fn map_pointer_type(button: &ButtonSource) -> fret_core::PointerType {
    match button {
        ButtonSource::Mouse(_) => fret_core::PointerType::Mouse,
        ButtonSource::Touch { .. } => fret_core::PointerType::Touch,
        ButtonSource::TabletTool { .. } => fret_core::PointerType::Pen,
        ButtonSource::Unknown(_) => fret_core::PointerType::Unknown,
    }
}

fn namespaced_pointer_id(namespace: u64, payload: u64) -> PointerId {
    const POINTER_ID_PAYLOAD_MASK: u64 = (1u64 << 56) - 1;
    PointerId(namespace | (payload & POINTER_ID_PAYLOAD_MASK))
}

fn map_pointer_id_from_device_id(kind_namespace: u64, device_id: Option<DeviceId>) -> PointerId {
    // `DeviceId` has no stable ABI surface, but it can be mapped to a stable token on a given
    // backend. When `device_id` is missing, we still return a stable (namespaced) id so callers
    // can distinguish multiple unknown pointers from the primary mouse pointer.
    let payload = device_id.map(|id| id.into_raw() as u64).unwrap_or(0);
    namespaced_pointer_id(kind_namespace, payload)
}

pub fn map_pointer_id_from_pointer_source(
    device_id: Option<DeviceId>,
    source: &PointerSource,
) -> PointerId {
    const POINTER_ID_NAMESPACE_TOUCH: u64 = 1u64 << 56;
    const POINTER_ID_NAMESPACE_PEN: u64 = 2u64 << 56;
    const POINTER_ID_NAMESPACE_UNKNOWN: u64 = 3u64 << 56;
    match source {
        PointerSource::Mouse => PointerId(0),
        PointerSource::Touch { finger_id, .. } => {
            namespaced_pointer_id(POINTER_ID_NAMESPACE_TOUCH, finger_id.into_raw() as u64)
        }
        PointerSource::TabletTool { .. } => {
            map_pointer_id_from_device_id(POINTER_ID_NAMESPACE_PEN, device_id)
        }
        PointerSource::Unknown => {
            map_pointer_id_from_device_id(POINTER_ID_NAMESPACE_UNKNOWN, device_id)
        }
    }
}

pub fn map_pointer_id_from_pointer_kind(
    device_id: Option<DeviceId>,
    kind: PointerKind,
) -> PointerId {
    const POINTER_ID_NAMESPACE_TOUCH: u64 = 1u64 << 56;
    const POINTER_ID_NAMESPACE_PEN: u64 = 2u64 << 56;
    const POINTER_ID_NAMESPACE_UNKNOWN: u64 = 3u64 << 56;
    match kind {
        PointerKind::Mouse => PointerId(0),
        PointerKind::Touch(finger_id) => {
            namespaced_pointer_id(POINTER_ID_NAMESPACE_TOUCH, finger_id.into_raw() as u64)
        }
        PointerKind::TabletTool(_) => {
            map_pointer_id_from_device_id(POINTER_ID_NAMESPACE_PEN, device_id)
        }
        PointerKind::Unknown => {
            map_pointer_id_from_device_id(POINTER_ID_NAMESPACE_UNKNOWN, device_id)
        }
    }
}

pub fn map_pointer_id_from_button_source(
    device_id: Option<DeviceId>,
    button: &ButtonSource,
) -> PointerId {
    const POINTER_ID_NAMESPACE_TOUCH: u64 = 1u64 << 56;
    const POINTER_ID_NAMESPACE_PEN: u64 = 2u64 << 56;
    const POINTER_ID_NAMESPACE_UNKNOWN: u64 = 3u64 << 56;
    match button {
        ButtonSource::Mouse(_) => PointerId(0),
        ButtonSource::Touch { finger_id, .. } => {
            namespaced_pointer_id(POINTER_ID_NAMESPACE_TOUCH, finger_id.into_raw() as u64)
        }
        ButtonSource::TabletTool { .. } => {
            map_pointer_id_from_device_id(POINTER_ID_NAMESPACE_PEN, device_id)
        }
        ButtonSource::Unknown(_) => {
            map_pointer_id_from_device_id(POINTER_ID_NAMESPACE_UNKNOWN, device_id)
        }
    }
}

pub fn map_pointer_type_from_pointer_source(source: &PointerSource) -> PointerType {
    match source {
        PointerSource::Mouse => PointerType::Mouse,
        PointerSource::Touch { .. } => PointerType::Touch,
        PointerSource::TabletTool { .. } => PointerType::Pen,
        PointerSource::Unknown => PointerType::Unknown,
    }
}

pub fn map_pointer_kind(kind: PointerKind) -> PointerType {
    match kind {
        PointerKind::Mouse => PointerType::Mouse,
        PointerKind::Touch(_) => PointerType::Touch,
        PointerKind::TabletTool(_) => PointerType::Pen,
        PointerKind::Unknown => PointerType::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_id_maps_mouse_to_zero() {
        assert_eq!(
            map_pointer_id_from_pointer_source(None, &winit::event::PointerSource::Mouse),
            PointerId(0)
        );
        assert_eq!(
            map_pointer_id_from_pointer_kind(None, winit::event::PointerKind::Mouse),
            PointerId(0)
        );
        assert_eq!(
            map_pointer_id_from_button_source(
                None,
                &winit::event::ButtonSource::Mouse(winit::event::MouseButton::Left)
            ),
            PointerId(0)
        );
    }

    #[test]
    fn pointer_id_maps_touch_finger_id_consistently() {
        let finger_id = winit::event::FingerId::from_raw(7);
        let source = winit::event::PointerSource::Touch {
            finger_id,
            force: None,
        };
        let button = winit::event::ButtonSource::Touch {
            finger_id,
            force: None,
        };

        let from_source = map_pointer_id_from_pointer_source(None, &source);
        let from_button = map_pointer_id_from_button_source(None, &button);
        let from_kind =
            map_pointer_id_from_pointer_kind(None, winit::event::PointerKind::Touch(finger_id));

        assert_ne!(from_source, PointerId(0));
        assert_eq!(from_source, from_button);
        assert_eq!(from_source, from_kind);
    }

    #[test]
    fn pointer_id_maps_tablet_tool_using_device_id() {
        let device_id = winit::event::DeviceId::from_raw(123);
        let source = winit::event::PointerSource::TabletTool {
            kind: winit::event::TabletToolKind::Pen,
            data: winit::event::TabletToolData::default(),
        };
        let button = winit::event::ButtonSource::TabletTool {
            kind: winit::event::TabletToolKind::Pen,
            button: winit::event::TabletToolButton::Contact,
            data: winit::event::TabletToolData::default(),
        };

        let from_source = map_pointer_id_from_pointer_source(Some(device_id), &source);
        let from_button = map_pointer_id_from_button_source(Some(device_id), &button);
        let from_kind = map_pointer_id_from_pointer_kind(
            Some(device_id),
            winit::event::PointerKind::TabletTool(winit::event::TabletToolKind::Pen),
        );

        assert_ne!(from_source, PointerId(0));
        assert_eq!(from_source, from_button);
        assert_eq!(from_source, from_kind);
    }

    #[test]
    fn pointer_id_maps_unknown_to_namespaced_device_id() {
        let device_id = winit::event::DeviceId::from_raw(123);

        let from_source = map_pointer_id_from_pointer_source(
            Some(device_id),
            &winit::event::PointerSource::Unknown,
        );
        let from_kind =
            map_pointer_id_from_pointer_kind(Some(device_id), winit::event::PointerKind::Unknown);
        let from_button =
            map_pointer_id_from_button_source(Some(device_id), &ButtonSource::Unknown(0));

        assert_ne!(from_source, PointerId(0));
        assert_eq!(from_source, from_kind);
        assert_eq!(from_source, from_button);
    }
}
