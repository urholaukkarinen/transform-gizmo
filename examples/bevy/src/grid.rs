use bevy::{
    app::{Plugin, Startup},
    dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings},
    ecs::system::Commands,
    prelude::default,
};

pub struct GridPlugin;
impl Plugin for GridPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(InfiniteGridPlugin)
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            fadeout_distance: 40000.,
            scale: 1.0,
            ..default()
        },
    ));
}
