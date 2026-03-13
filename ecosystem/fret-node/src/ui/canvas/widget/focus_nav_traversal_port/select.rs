use crate::ui::canvas::widget::*;

pub(super) fn next_port(
    ports: &[PortId],
    current: Option<PortId>,
    forward: bool,
) -> Option<PortId> {
    if ports.is_empty() {
        return None;
    }

    let current = current.filter(|id| ports.iter().any(|p| *p == *id));
    match current.and_then(|id| ports.iter().position(|p| *p == id)) {
        Some(ix) => {
            let len = ports.len();
            let next_ix = if forward {
                (ix + 1) % len
            } else {
                (ix + len - 1) % len
            };
            Some(ports[next_ix])
        }
        None => Some(if forward {
            ports[0]
        } else {
            ports[ports.len() - 1]
        }),
    }
}
