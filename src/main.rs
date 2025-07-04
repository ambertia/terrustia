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

const PLAYER_HEIGHT: f32 = 3.0;
const PLAYER_WIDTH: f32 = 2.0;

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
        .add_systems(Startup, setup)
        .run();
}

// Initialize all the stuff in the world
fn setup(mut commands: Commands) {
    // Spawn the player
    commands.spawn((
        Player,
        RigidBody::Dynamic,
        Collider::rectangle(PLAYER_WIDTH - 0.1, PLAYER_HEIGHT - 0.1),
        Sprite {
            color: Color::from(WHITE),
            custom_size: Some(Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0., 30., 1.),
        // A ShapeCaster to help detect if the player is touching the ground.
        ShapeCaster::new(
            Collider::rectangle(PLAYER_WIDTH * 0.99, PLAYER_HEIGHT * 0.99),
            Vector::ZERO,
            0.,
            Dir2::NEG_Y,
        )
        .with_max_distance(0.1),
        LockedAxes::ROTATION_LOCKED,
        Friction::new(0.1).with_combine_rule(CoefficientCombine::Min),
        CollisionMargin(0.05),
        LinearDamping(0.1),
    ));
}
