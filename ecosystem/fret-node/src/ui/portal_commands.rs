use fret_runtime::CommandId;
use uuid::Uuid;

use crate::core::NodeId;
use crate::ops::GraphTransaction;

pub const CMD_SUBMIT_TEXT_PREFIX: &str = "fret_node.portal.submit_text:";
pub const CMD_CANCEL_TEXT_PREFIX: &str = "fret_node.portal.cancel_text:";
pub const CMD_STEP_TEXT_PREFIX: &str = "fret_node.portal.step_text:";

pub fn portal_submit_text_command(node: NodeId) -> CommandId {
    CommandId::new(format!("{CMD_SUBMIT_TEXT_PREFIX}{}", node.0))
}

pub fn portal_cancel_text_command(node: NodeId) -> CommandId {
    CommandId::new(format!("{CMD_CANCEL_TEXT_PREFIX}{}", node.0))
}

pub fn portal_step_text_command(node: NodeId, delta: i32) -> CommandId {
    CommandId::new(format!(
        "{CMD_STEP_TEXT_PREFIX}{}:{delta}:{}",
        node.0,
        PortalTextStepMode::Normal.as_str()
    ))
}

pub fn portal_step_text_command_with_mode(
    node: NodeId,
    delta: i32,
    mode: PortalTextStepMode,
) -> CommandId {
    CommandId::new(format!(
        "{CMD_STEP_TEXT_PREFIX}{}:{delta}:{}",
        node.0,
        mode.as_str()
    ))
}

pub fn parse_portal_text_command(command: &CommandId) -> Option<PortalTextCommand> {
    let s = command.as_str();
    if let Some(rest) = s.strip_prefix(CMD_SUBMIT_TEXT_PREFIX) {
        let uuid = Uuid::parse_str(rest).ok()?;
        return Some(PortalTextCommand::Submit { node: NodeId(uuid) });
    }
    if let Some(rest) = s.strip_prefix(CMD_CANCEL_TEXT_PREFIX) {
        let uuid = Uuid::parse_str(rest).ok()?;
        return Some(PortalTextCommand::Cancel { node: NodeId(uuid) });
    }
    if let Some(rest) = s.strip_prefix(CMD_STEP_TEXT_PREFIX) {
        let mut parts = rest.split(':');
        let uuid_str = parts.next()?;
        let delta_str = parts.next()?;
        let mode_str = parts.next().unwrap_or("normal");
        let uuid = Uuid::parse_str(uuid_str).ok()?;
        let delta = delta_str.parse::<i32>().ok()?;
        let mode = PortalTextStepMode::parse(mode_str)?;
        return Some(PortalTextCommand::Step {
            node: NodeId(uuid),
            delta,
            mode,
        });
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalTextStepMode {
    Fine,
    Normal,
    Coarse,
}

impl PortalTextStepMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fine => "fine",
            Self::Normal => "normal",
            Self::Coarse => "coarse",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "fine" => Some(Self::Fine),
            "normal" => Some(Self::Normal),
            "coarse" => Some(Self::Coarse),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalTextCommand {
    Submit {
        node: NodeId,
    },
    Cancel {
        node: NodeId,
    },
    Step {
        node: NodeId,
        delta: i32,
        mode: PortalTextStepMode,
    },
}

#[derive(Debug, Clone)]
pub enum PortalCommandOutcome {
    NotHandled,
    Handled,
    Commit(GraphTransaction),
}

#[cfg(test)]
mod tests {
    use fret_runtime::CommandId;
    use uuid::Uuid;

    use super::{
        PortalTextCommand, PortalTextStepMode, parse_portal_text_command,
        portal_cancel_text_command, portal_step_text_command, portal_step_text_command_with_mode,
        portal_submit_text_command,
    };
    use crate::core::NodeId;

    fn test_node() -> NodeId {
        NodeId(Uuid::from_u128(0x12345678123456781234567812345678))
    }

    #[test]
    fn portal_text_command_protocol_roundtrips_submit_cancel_and_steps() {
        let node = test_node();

        assert_eq!(
            parse_portal_text_command(&portal_submit_text_command(node)),
            Some(PortalTextCommand::Submit { node })
        );
        assert_eq!(
            parse_portal_text_command(&portal_cancel_text_command(node)),
            Some(PortalTextCommand::Cancel { node })
        );
        assert_eq!(
            parse_portal_text_command(&portal_step_text_command(node, -2)),
            Some(PortalTextCommand::Step {
                node,
                delta: -2,
                mode: PortalTextStepMode::Normal
            })
        );
        assert_eq!(
            parse_portal_text_command(&portal_step_text_command_with_mode(
                node,
                3,
                PortalTextStepMode::Fine
            )),
            Some(PortalTextCommand::Step {
                node,
                delta: 3,
                mode: PortalTextStepMode::Fine
            })
        );
        assert_eq!(
            parse_portal_text_command(&portal_step_text_command_with_mode(
                node,
                4,
                PortalTextStepMode::Coarse
            )),
            Some(PortalTextCommand::Step {
                node,
                delta: 4,
                mode: PortalTextStepMode::Coarse
            })
        );
    }

    #[test]
    fn portal_text_command_protocol_rejects_malformed_commands() {
        for command in [
            CommandId::from("fret_node.portal.submit_text:not-a-uuid"),
            CommandId::from("fret_node.portal.cancel_text:not-a-uuid"),
            CommandId::from("fret_node.portal.step_text:not-a-uuid:1:normal"),
            CommandId::from(
                "fret_node.portal.step_text:12345678-1234-5678-1234-567812345678:nope:normal",
            ),
            CommandId::from(
                "fret_node.portal.step_text:12345678-1234-5678-1234-567812345678:1:huge",
            ),
            CommandId::from("fret_node.portal.unknown:12345678-1234-5678-1234-567812345678"),
        ] {
            assert_eq!(parse_portal_text_command(&command), None);
        }
    }
}
