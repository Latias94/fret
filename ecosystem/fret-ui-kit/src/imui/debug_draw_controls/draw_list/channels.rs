use super::super::commands::DebugDrawCommand;
use super::super::{DebugDrawChannelSplit, ImUiDebugDrawList};

impl ImUiDebugDrawList {
    pub fn channels_split(&mut self, count: usize) {
        if count <= 1 || self.channel_split.is_some() {
            return;
        }
        self.channel_split = Some(DebugDrawChannelSplit {
            channels: (0..count).map(|_| Vec::new()).collect(),
            current: 0,
        });
    }

    pub fn channels_set_current(&mut self, channel: usize) {
        let Some(split) = self.channel_split.as_mut() else {
            return;
        };
        if channel >= split.channels.len() || channel == split.current {
            return;
        }

        std::mem::swap(&mut split.channels[split.current], &mut self.commands);
        std::mem::swap(&mut split.channels[channel], &mut self.commands);
        split.current = channel;
    }

    pub fn channels_merge(&mut self) {
        let Some(mut split) = self.channel_split.take() else {
            return;
        };
        std::mem::swap(&mut split.channels[split.current], &mut self.commands);

        let total_commands = split.channels.iter().map(Vec::len).sum();
        let mut merged = Vec::with_capacity(total_commands);
        for mut channel in split.channels {
            merged.append(&mut channel);
        }
        self.commands = merged;
    }

    pub(in crate::imui::debug_draw_controls) fn for_each_command_with_channel<F>(
        &self,
        mut visit: F,
    ) where
        F: FnMut(Option<usize>, &DebugDrawCommand),
    {
        let Some(split) = self.channel_split.as_ref() else {
            for command in &self.commands {
                visit(None, command);
            }
            return;
        };

        for (channel, commands) in split.channels.iter().enumerate() {
            let commands = if channel == split.current {
                self.commands.as_slice()
            } else {
                commands.as_slice()
            };
            for command in commands {
                visit(Some(channel), command);
            }
        }
    }
}
