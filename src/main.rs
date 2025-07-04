use avian2d::{PhysicsPlugins, math::Vector, prelude::*};
use bevy::{color::palettes::css::WHITE, input::mouse::AccumulatedMouseScroll, prelude::*};
use player::{CharacterControllerPlugin, Player};
use terrain::TerrainPlugin;
use ui::GameUiPlugin;

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
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Gravity(Vec2::NEG_Y * 50.))
        .add_systems(Startup, setup)
        .add_systems(Update, (track_camera_to_player, zoom_camera))
        .run();
}

#[derive(Component)]
#[require(Camera2d)]
struct MainCamera;

// Initialize all the stuff in the world
fn setup(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        // Make the camera start zoomed in
        // TODO: This is better than leaving it at 1, but 0.5 feels pretty arbitrary and having
        // this code block just to build an OrthoProj with a different scale feels stinky
        Projection::Orthographic({
            let mut projection = OrthographicProjection::default_2d();
            projection.scale = 0.1;
            projection
        }),
    ));

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
const ZOOM_MIN: f32 = 0.05;
const ZOOM_MAX: f32 = 0.2;
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
