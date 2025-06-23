use bevy::prelude::*;
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
#[require(Transform, Sprite, MovementState)]
struct Player;

// Initialize all the stuff in the world
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // Spawn the player
    commands.spawn((
        Player,
        MovementState::from_pos(0.0, 50.0),
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform {
            translation: Vec3::new(0.0, 50.0, 0.0),
            scale: Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT).extend(1.0),
            ..default()
        },
    ));
}
