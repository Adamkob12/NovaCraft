use crate::prelude::*;

pub struct EnviornmentPlugin;

impl Plugin for EnviornmentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            brightness: 1.7,
            color: Color::rgb(0.8, 0.9, 0.9),
        })
        .insert_resource(ClearColor(Color::rgb(0.70, 0.95, 1.0)));
    }
}
