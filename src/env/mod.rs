use crate::prelude::*;

mod sky;

use sky::*;

pub struct EnviornmentPlugin;

impl Plugin for EnviornmentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            brightness: 1.0,
            color: Color::rgb(0.8, 0.9, 0.9),
        })
        // .insert_resource(Msaa::Sample4)
        .insert_resource(AtmosphereModel::default()) // Default Atmosphere material, we can edit it to simulate another planet
        .insert_resource(CycleTimer(Timer::new(
            bevy::utils::Duration::from_millis(50), // Update our atmosphere every 50ms (in a real game, this would be much slower, but for the sake of an example we use a faster update)
            TimerMode::Repeating,
        )))
        .add_plugins((
            AtmospherePlugin, // Default AtmospherePlugin
        ))
        .add_systems(Startup, setup_environment)
        .add_systems(Update, daylight_cycle)
        .insert_resource(ClearColor(Color::rgb(0.70, 0.95, 1.0)));
    }
}
