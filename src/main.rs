use avian2d::{
    PhysicsPlugins,
    math::Vector,
    prelude::{Collider, Gravity, LockedAxes, RigidBody, ShapeCaster},
};
use bevy::{color::palettes::css::WHITE, input::mouse::AccumulatedMouseScroll, prelude::*};
use player::{CharacterControllerPlugin, Player};
use terrain::TerrainPlugin;

mod physics;
mod player;
mod terrain;

const PLAYER_HEIGHT: f32 = BLOCK_SIZE * 3.0;
const PLAYER_WIDTH: f32 = BLOCK_SIZE * 2.0;
const BLOCK_SIZE: f32 = 1.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (track_camera_to_player, zoom_camera, update_coordinates_ui),
        )
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec2::NEG_Y * 50.))
        .add_plugins(TerrainPlugin)
        .add_plugins(CharacterControllerPlugin)
        .run();
}

#[derive(Component)]
#[require(Camera2d)]
struct MainCamera;

#[derive(Component)]
#[require(Text)]
struct UiCoordinateText;

// Initialize all the stuff in the world
fn setup(mut commands: Commands) {
    commands.spawn(MainCamera);

    // Spawn the player
    commands.spawn((
        Player,
        RigidBody::Dynamic,
        Collider::rectangle(PLAYER_WIDTH, PLAYER_HEIGHT),
        Sprite {
            color: Color::from(WHITE),
            ..default()
        },
        Transform {
            translation: Vec3::new(0., 10., 1.),
            scale: Vec3::new(PLAYER_WIDTH, PLAYER_HEIGHT, 1.),
            ..default()
        },
        // A ShapeCaster to help detect if the player is touching the ground.
        ShapeCaster::new(
            Collider::rectangle(PLAYER_WIDTH * 0.99, PLAYER_HEIGHT * 0.99),
            Vector::ZERO,
            0.,
            Dir2::NEG_Y,
        )
        .with_max_distance(0.1),
        LockedAxes::ROTATION_LOCKED,
    ));

    commands.spawn(UiCoordinateText);
}

const CATCH_UP_TIME: f32 = 0.33;
fn track_camera_to_player(
    mut camera: Single<&mut Transform, (With<Camera>, Without<Player>)>,
    player: Single<&Transform, With<Player>>,
    time: Res<Time>,
) {
    let target = Vec3::new(
        player.translation.x,
        player.translation.y,
        camera.translation.z,
    );
    camera.translation = camera
        .translation
        .lerp(target, time.delta_secs() / CATCH_UP_TIME);
}

const ZOOM_SPEED: f32 = 1.0;
const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 1.;
fn zoom_camera(
    projection: Single<&mut Projection, With<Camera>>,
    scroll_input: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
) {
    match projection.into_inner().into_inner() {
        Projection::Orthographic(ortho_projection) => {
            // Zoom in when scrolling up
            let zoom_delta = -scroll_input.delta.y * ZOOM_SPEED * time.delta_secs();

            // Logarithmic (multiplicative) zoom scaling
            let zoom_scale = 1. + zoom_delta;

            ortho_projection.scale =
                (ortho_projection.scale * zoom_scale).clamp(ZOOM_MIN, ZOOM_MAX);
        }
        _ => {}
    }
}

fn update_coordinates_ui(
    mut text: Single<&mut Text, With<UiCoordinateText>>,
    player: Single<&Transform, With<Player>>,
) {
    text.0 = format!(
        "({0:.1}, {1:.1})",
        player.translation.x / BLOCK_SIZE,
        (player.translation.y - PLAYER_HEIGHT / 2.) / BLOCK_SIZE
    );
}
