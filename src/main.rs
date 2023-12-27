mod mage;

use bevy::{
    prelude::*,
    window::{self, WindowResolution},
};
use mage::MagePlugin;
use prelude::{WINDOW_HEIGHT, WINDOW_WIDTH};

pub mod prelude {
    pub const TILE_SIZE: f32 = 32.0;
    pub const WINDOW_HEIGHT: f32 = TILE_SIZE * 15.0;
    pub const WINDOW_WIDTH: f32 = TILE_SIZE * 15.0;
}

fn camera_setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Magic Mania".to_string(),
                        resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Update, window::close_on_esc)
        .add_systems(Startup, camera_setup)
        .add_plugins(MagePlugin)
        .run();
}
