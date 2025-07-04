use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*};

use crate::player::Player;

pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, (track_camera_to_player, zoom_camera));
    }
}

#[derive(Component)]
#[require(Camera2d)]
struct MainCamera;

fn setup_camera(mut commands: Commands) {
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
