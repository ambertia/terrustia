use avian2d::prelude::*;
use bevy::prelude::*;

mod camera;
mod player;
mod terrain;
mod ui;

pub struct TerrustiaGamePlugin;

impl Plugin for TerrustiaGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            camera::CameraPlugin,
            player::CharacterControllerPlugin,
            terrain::TerrainPlugin,
            ui::UiPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Gravity(Vec2::NEG_Y * 50.));
    }
}
