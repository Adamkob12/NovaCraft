use crate::chunk::DEFAULT_SL;

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
            GlobalParameter::SLintensity => {
                if let Some(ref mut sl) = render_settings.sl {
                    sl.intensity = value;
                    reply!(set, "smooth lighting value set to {}", value);
                } else {
                    reply!(set, "Cannot set smooth lighting intensity because SL is disabled.\n Run 'set sl 1' to enable it.");
                    set.failed();
                }
            }
            GlobalParameter::SLmax => {
                if let Some(ref mut sl) = render_settings.sl {
                    sl.max = value;
                    reply!(set, "smooth lighting max set to {}", value);
                } else {
                    reply!(set, "Cannot set smooth lighting max because SL is disabled.\n Run 'set sl 1' to enable it.");
                    set.failed();
                }
            }
            GlobalParameter::SLsmoothing => {
                if let Some(ref mut sl) = render_settings.sl {
                    sl.smoothing = value;
                    reply!(set, "smooth lighting smoothing set to {}", value);
                } else {
                    reply!(set, "Cannot set smooth lighting smoothing because SL is disabled.\n Run 'set sl 1' to enable it.");
                    set.failed();
                }
            }
            GlobalParameter::SL => match value as i32 {
                1 => {
                    if render_settings.sl.is_none() {
                        render_settings.sl = DEFAULT_SL;
                    }
                }
                0 => {
                    if render_settings.sl.is_some() {
                        render_settings.sl = None;
                    }
                }
                _ => {
                    set.reply(
                        "Expected either 1.0 to enable Smooth Lighting or 0.0 to disable it.",
                    );
                    set.failed();
                }
            },
        }
        set.ok();
    }
}
