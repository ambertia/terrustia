use bevy::{math::bounding::*, prelude::*};

const DOT_ACCEL: f32 = 30.0;
const DRAG_FACTOR: f32 = 0.6;
const VEL_MAX: f32 = 100.0;
const GRAVITY: f32 = 15.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (accelerate_dot, check_collision, move_dot).chain(),
        )
        .run();
}

#[derive(Component)]
struct Dot;

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component)]
struct Ground;

// Move the dot around the screen based on keyboard input
fn accelerate_dot(
    keyboard: Res<ButtonInput<KeyCode>>,
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

    // Apply decay, gravity, and acceleration to the dot before clamping it to the max
    dot_vel.0 = (dot_vel.0.lerp(Vec2::ZERO, DRAG_FACTOR * time.delta_secs())
        + (input_direction * DOT_ACCEL * time.delta_secs()))
        - (Vec2::ZERO.with_y(GRAVITY * time.delta_secs())).clamp_length_max(VEL_MAX);
}

fn move_dot(dot_query: Single<(&Velocity, &mut Transform), With<Dot>>, time: Res<Time>) {
    let (velocity, mut transform) = dot_query.into_inner();
    transform.translation += (velocity.0 * time.delta_secs()).extend(0.0);
}

// Check if the dot is colliding with the ground and ensure its vertical velocity becomes positive
fn check_collision(
    mut transforms: ParamSet<(
        Single<&Transform, With<Dot>>,
        Single<&Transform, With<Ground>>,
    )>,
    mut dot_vel: Single<&mut Velocity, With<Dot>>,
) {
    // TODO: Turn dot size into a constant
    let dot_collider = BoundingCircle::new(transforms.p0().translation.truncate(), 5.0);
    let ground_collider = Aabb2d::new(
        transforms.p1().translation.clone().truncate(),
        transforms.p1().scale.clone().truncate() / 2.0,
    );

    if dot_collider.intersects(&ground_collider) && dot_vel.0.y < 0.0 {
        dot_vel.0.y *= -0.9;
    }
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
        Transform::default().with_scale(Vec2::splat(10.0).extend(0.0)),
    ));
}
