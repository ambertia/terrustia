use bevy::prelude::*;

// Dot's velocity in pixels per second
const DOT_VELOCITY: f32 = 60.0;
const DOT_ACCEL: f32 = 60.0;
const DRAG_FACTOR: f32 = 0.4;
const VEL_MAX: f32 = 100.0;
const VEL_MIN: f32 = 10.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_dot)
        .run();
}

#[derive(Component)]
struct Dot;

#[derive(Component)]
struct Velocity(Vec2);

// Move the dot around the screen based on keyboard input
fn handle_dot(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut dot_transform: Single<&mut Transform, With<Dot>>,
    mut dot_vel: Single<&mut Velocity, With<Dot>>,
    time: Res<Time>,
) {
    let mut input_direction = Vec2::splat(0.0);

    // Check all the keys and modify the dot's input_direction accordingly
    if keyboard.pressed(KeyCode::KeyW) {
        input_direction.y += 1.0;
    }

    if keyboard.pressed(KeyCode::KeyA) {
        input_direction.x -= 1.0;
    }

    if keyboard.pressed(KeyCode::KeyS) {
        input_direction.y -= 1.0;
    }

    if keyboard.pressed(KeyCode::KeyD) {
        input_direction.x += 1.0;
    }

    input_direction.normalize();

    // Apply a decay to the velocity, but only if there is neutral input along the axis
    if input_direction.x == 0.0 {
        if dot_vel.0.x < VEL_MIN {
            dot_vel.0.x = 0.0;
        } else {
            dot_vel.0.x *= DRAG_FACTOR * time.delta_secs();
        }
    }
    if input_direction.y == 0.0 {
        if dot_vel.0.y < VEL_MIN {
            dot_vel.0.y = 0.0;
        } else {
            dot_vel.0.y *= DRAG_FACTOR * time.delta_secs();
        }
    }

    // Apply acceleration to the dot's velocity
    dot_vel.0.x *= DOT_ACCEL * time.delta_secs();
    dot_vel.0.y *= DOT_ACCEL * time.delta_secs();

    // Apply velocity to the dot's transform
    dot_transform.translation.x += dot_vel.0.x * time.delta_secs();
    dot_transform.translation.y += dot_vel.0.y * time.delta_secs();
}

// Initialize all the stuff in the world
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // Spawn the dot
    commands.spawn((
        Dot,
        Velocity,
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::default().with_scale(Vec2::splat(10.0).extend(1.)),
    ));
}
