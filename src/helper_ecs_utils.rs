use crate::prelude::*;

#[derive(Resource)]
pub struct GlobalSecondsCounter(u128);

#[derive(Resource)]
pub struct OneIn2(bool);

#[derive(Resource)]
pub struct OneIn5(bool);

#[derive(Resource)]
pub struct OneIn10(bool);

#[derive(Resource)]
pub struct OneIn30(bool);

#[derive(Resource)]
pub struct OneIn100(bool);

pub struct HelperEcsUtilsPlugin;

impl Plugin for HelperEcsUtilsPlugin {
    #[allow(unused_variables)]
    fn build(&self, app: &mut App) {
        // app.insert_resource(GlobalSecondsCounter(0))
        //     .insert_resource(OneIn2(true))
        //     .insert_resource(OneIn5(true))
        //     .insert_resource(OneIn10(true))
        //     .insert_resource(OneIn30(true))
        //     .insert_resource(OneIn100(true));
        //
        // app.add_systems(
        //     PostUpdate,
        //     (
        //         update_seconds,
        //         update_oi2,
        //         update_oi5,
        //         update_oi10,
        //         update_oi30,
        //         update_oi100,
        //     ),
        // );
    }
}

fn update_seconds(time: Res<Time>, mut sec: ResMut<GlobalSecondsCounter>) {
    if time.elapsed_seconds() as u128 != sec.0 {
        sec.0 = time.elapsed_seconds() as u128;
    }
}

fn update_oi2(time: Res<Time>, mut sec: ResMut<OneIn2>) {
    if time.elapsed().as_millis() % 2 == 0 {
        sec.0 = !sec.0;
    }
}

fn update_oi5(time: Res<Time>, mut sec: ResMut<GlobalSecondsCounter>) {
    if time.elapsed().as_millis() % 5 == 0 {
        sec.0 = !sec.0;
    }
}

fn update_oi10(time: Res<Time>, mut sec: ResMut<GlobalSecondsCounter>) {
    if time.elapsed().as_millis() % 10 == 0 {
        sec.0 = !sec.0;
    }
}

fn update_oi30(time: Res<Time>, mut sec: ResMut<GlobalSecondsCounter>) {
    if time.elapsed().as_millis() % 30 == 0 {
        sec.0 = !sec.0;
    }
}

fn update_oi100(time: Res<Time>, mut sec: ResMut<GlobalSecondsCounter>) {
    if time.elapsed().as_millis() % 100 == 0 {
        sec.0 = !sec.0;
    }
}
