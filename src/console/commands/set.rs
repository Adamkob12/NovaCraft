use super::*;

/// Set a global parameter to a value
#[derive(Parser, ConsoleCommand)]
#[command(name = "set")]
pub struct SetCommand {
    /// Parameter to change:
    ///     -render-distance
    pub parameter_to_change: GlobalParameter,
    /// The value to set
    pub value: f32,
}

pub fn set_command(
    mut set: ConsoleCommand<SetCommand>,
    mut render_settings: ResMut<RenderSettings>,
) {
    if let Some(Ok(SetCommand {
        parameter_to_change,
        value,
    })) = set.take()
    {
        match parameter_to_change {
            GlobalParameter::RenderDistance => {
                render_settings.render_distance = value as i32;
                reply!(set, "render-distance set to {}", value);
            }
            GlobalParameter::PbsValue => {
                if let Some(ref mut pbs) = render_settings.pbs {
                    pbs.pbs_value = value;
                    reply!(set, "pbs-value set to {}", value);
                } else {
                    reply!(set, "Cannot set pbs-value because PBS is disabled.\n Run 'set pbs 1' to enable it.");
                    set.failed();
                }
            }
            GlobalParameter::PbsMin => {
                if let Some(ref mut pbs) = render_settings.pbs {
                    pbs.min = value;
                    reply!(set, "pbs-min set to {}", value);
                } else {
                    reply!(set, "Cannot set pbs-min because PBS is disabled.\n Run 'set pbs 1' to enable it.");
                    set.failed();
                }
            }
            GlobalParameter::PbsSmoothing => {
                if let Some(ref mut pbs) = render_settings.pbs {
                    pbs.smoothing = PbsSmoothing::Custom(value);
                    reply!(set, "pbs-smoothing set to {}", value);
                } else {
                    reply!(set, "Cannot set pbs-smoothing because PBS is disabled.\n Run 'set pbs 1' to enable it.");
                    set.failed();
                }
            }
            GlobalParameter::Pbs => match value as i32 {
                1 => {
                    if render_settings.pbs.is_none() {
                        render_settings.pbs = DEFAULT_PBS;
                    }
                }
                0 => {
                    if render_settings.pbs.is_some() {
                        render_settings.pbs = None;
                    }
                }
                _ => {
                    set.reply("Expected either 1.0 to enable PBS or 0.0 to disable PBS");
                    set.failed();
                }
            },
        }
        set.ok();
    }
}
