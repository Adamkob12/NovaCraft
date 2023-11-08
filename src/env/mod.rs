use crate::prelude::*;

pub struct EnviornmentPlugin;

impl Plugin for EnviornmentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            brightness: 1.4,
            color: Color::rgb(1.0, 1.0, 0.8),
        })
        .insert_resource(ClearColor(Color::rgb(0.70, 0.95, 1.0)));
    }
}
