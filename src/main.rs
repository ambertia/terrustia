use bevy::{color::palettes::css::WHITE, prelude::*};
use physics::{MovementState, PhysicsPlugin};
use terrain::TerrainPlugin;

mod physics;
mod terrain;

const PLAYER_HEIGHT: f32 = BLOCK_SIZE * 3.0;
const PLAYER_WIDTH: f32 = BLOCK_SIZE * 2.0;
const BLOCK_SIZE: f32 = 10.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_plugins(PhysicsPlugin)
        .add_plugins(TerrainPlugin)
        .run();
}

#[derive(Component)]
#[require(Transform, MovementState)]
struct Player;

// Initialize all the stuff in the world
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawn the player
    commands.spawn((
        Player,
        MovementState::from_pos(0.0, 50.0),
        Sprite {
            color: Color::from(WHITE),
            ..default()
        },
        Transform {
            translation: Vec3::new(0., 50., 1.),
            scale: Vec3::new(PLAYER_WIDTH, PLAYER_HEIGHT, 1.),
            ..default()
        },
    ));
}
