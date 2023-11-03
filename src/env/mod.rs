use crate::prelude::*;

pub struct EnviornmentPlugin;

impl Plugin for EnviornmentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            brightness: 1.0,
            color: Color::ANTIQUE_WHITE,
        });
    }
}
