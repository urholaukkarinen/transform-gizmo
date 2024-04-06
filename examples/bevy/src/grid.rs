use bevy::{
    app::{Plugin, Startup},
    ecs::system::Commands,
    prelude::default,
};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};

pub struct GridPlugin;
impl Plugin for GridPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(InfiniteGridPlugin)
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(InfiniteGridBundle {
        settings: InfiniteGridSettings {
            fadeout_distance: 40000.,
            scale: 1.0,
            ..default()
        },
        ..default()
    });
}
