use super::*;

pub(super) fn pending_nonrepeating_timer_tokens_after(
    app: &TestHost,
    after: std::time::Duration,
) -> Vec<TimerToken> {
    app.effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::SetTimer {
                token,
                after: effect_after,
                repeat,
                ..
            } if repeat.is_none() && *effect_after == after => Some(*token),
            _ => None,
        })
        .collect()
}
