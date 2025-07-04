use avian2d::{PhysicsPlugins, math::Vector, prelude::*};
use bevy::{color::palettes::css::WHITE, input::mouse::AccumulatedMouseScroll, prelude::*};
use camera::GameCameraPlugin;
use player::{CharacterControllerPlugin, Player};
use terrain::TerrainPlugin;
use ui::GameUiPlugin;

mod camera;
mod player;
mod terrain;
mod ui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            TerrainPlugin,
            CharacterControllerPlugin,
            GameUiPlugin,
            GameCameraPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Gravity(Vec2::NEG_Y * 50.))
        .run();
}
