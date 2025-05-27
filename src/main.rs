use bevy::prelude::*;

const DOT_ACCEL: f32 = 30.0;
const DRAG_FACTOR: f32 = 0.6;
const VEL_MAX: f32 = 100.0;

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

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component)]
struct Ground;

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

    input_direction = input_direction.normalize_or_zero();

    dot_vel.0 = (dot_vel.0.lerp(Vec2::ZERO, DRAG_FACTOR * time.delta_secs())
        + (input_direction * DOT_ACCEL * time.delta_secs()))
    .clamp_length_max(VEL_MAX);

    // Apply velocity to the dot's transform
    dot_transform.translation += dot_vel.0.extend(0.0);
}

// Initialize all the stuff in the world
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // Spawn the ground
    commands.spawn((
        Ground,
        Sprite::from_color(Color::WHITE, Vec2::ONE),
        Transform {
            translation: Vec3::from_array([0.0, -50.0, 1.0]),
            scale: Vec3::from_array([1000.0, 5.0, 1.0]),
            ..default()
        },
    ));

    // Spawn the dot
    commands.spawn((
        Dot,
        Velocity(Vec2::ZERO),
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::default().with_scale(Vec2::splat(10.0).extend(1.)),
    ));
}
