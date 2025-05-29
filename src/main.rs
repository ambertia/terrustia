use bevy::{
    color::palettes::css::{RED, SADDLE_BROWN, YELLOW},
    math::{
        bounding::*,
        ops::{powf, sqrt},
    },
    prelude::*,
    window::PrimaryWindow,
};

const DOT_ACCEL: f32 = 60.0;
const DRAG_FACTOR: f32 = 0.05;
const VEL_MAX: f32 = 300.0;
const GRAVITY: f32 = 15.0;
const ACCEL_ARROW_LENGTH: f32 = 100.0;
const VEL_ARROW_LENGTH: f32 = 100.0;
const DOT_RADIUS: f32 = 5.0;
const BLOCK_SIZE: f32 = 10.0;
const BLOCKS_X: u16 = 80;
const BLOCKS_Y: u16 = 40;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (accelerate_dot, check_collision, check_bounds, move_dot).chain(),
        )
        .add_systems(Update, render_arrows)
        .run();
}

#[derive(Component)]
struct Dot;

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct Block;

// Move the dot around the screen based on keyboard input
fn accelerate_dot(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut dot_vel: Single<&mut Velocity, With<Dot>>,
    time: Res<Time>,
) {
    // Apply decay, gravity, and acceleration to the dot before clamping it to the max
    dot_vel.0 = (dot_vel.0.lerp(Vec2::ZERO, DRAG_FACTOR * time.delta_secs())
        + (get_direction_from_keyboard(keyboard) * DOT_ACCEL * time.delta_secs()))
        - (Vec2::ZERO.with_y(GRAVITY * time.delta_secs())).clamp_length_max(VEL_MAX);
}

fn move_dot(dot_query: Single<(&Velocity, &mut Transform), With<Dot>>, time: Res<Time>) {
    let (velocity, mut transform) = dot_query.into_inner();
    transform.translation += (velocity.0 * time.delta_secs()).extend(0.0);
}

// Check if the dot is colliding with certain screen edges and change its properties
fn check_bounds(
    mut dot_query: Single<(&mut Velocity, &mut Transform), With<Dot>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    // Check if the dot is colliding with the lower boundary
    // Collision should take place as the bottom of the dot touches the edge
    if window.resolution.height() / 2.0 + dot_query.1.translation.y - DOT_RADIUS < 0.0
        && dot_query.0.0.y < 0.0
    {
        // Reflect the dot's vertical velocity
        dot_query.0.0.y *= -1.0;
    }

    // Check if the dot is "colliding" with the outer wall
    // Collision should take place when ball is just barely completely out of frame
    if dot_query.1.translation.x.abs() - DOT_RADIUS > window.resolution.width() / 2.0 {
        // Warp the ball to the opposite side of the frame
        dot_query.1.translation.x *= -1.0;
    }
}

// Check if the dot is colliding with the ground and ensure its vertical velocity becomes positive
fn check_collision(
    dot_transform: Single<&Transform, With<Dot>>,
    blocks_query: Query<&Transform, With<Block>>,
    mut dot_vel: Single<&mut Velocity, With<Dot>>,
) {
    // So far the only collision is with the ground, and the dot shouldn't collide when moving up
    if dot_vel.0.y > 0.0 {
        return;
    }

    let dot_collider = BoundingCircle::new(dot_transform.translation.truncate(), 5.0);

    // Add colliders to a vec to be used for collision checking, but only add blocks the dot can
    // actually collide with.
    let critical_distance_squared: f32 = powf(sqrt(2.0) * BLOCK_SIZE + DOT_RADIUS, 2.0);
    let mut nearby_blocks: Vec<Aabb2d> = Vec::new();
    for block in blocks_query {
        // The furthest case where the dot could collide with a block is diagonal on the corner,
        // where the distance between block and dot centers is DOT_RADIUS + sqrt(2)*BLOCK_SIZE
        if block
            .translation
            .truncate()
            .distance_squared(dot_transform.translation.truncate())
            <= critical_distance_squared
        {
            nearby_blocks.push(Aabb2d::new(
                block.translation.truncate(),
                block.scale.truncate() / 2.0,
            ));
        }
    }

    // Iterate over all the potentially colliding tiles and check for collision
    for bound in nearby_blocks {
        if bound.intersects(&dot_collider) {
            dot_vel.0.y *= -1.0;
            return;
        }
    }
}

// Return a normalized Vec2 for acceleration vector based on keyboard input
fn get_direction_from_keyboard(keyboard: Res<ButtonInput<KeyCode>>) -> Vec2 {
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

    input_direction.normalize_or_zero()
}

// Create indicators for dot acceleration and velocity using a gizmo
fn render_arrows(
    dot_query: Single<(&Velocity, &Transform), With<Dot>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut gizmos: Gizmos,
) {
    let dot_position = dot_query.1.translation.truncate();

    // Render the acceleration arrow in red based on the keyboard input
    let mut input_direction = get_direction_from_keyboard(keyboard);
    // Because gravity should be included in the display, some more complicated math has to be done
    // Gravity strength relative to DOT_ACCEL defines how far down the arrow gets shifted
    input_direction -= Vec2::from([0.0, GRAVITY / DOT_ACCEL]);
    gizmos.arrow_2d(
        dot_position,
        dot_position + input_direction * ACCEL_ARROW_LENGTH,
        RED,
    );

    // Render the velocity arrow in yellow based on the entity's component value
    // Arrow should equal the constant's length at VEL_MAX
    gizmos.arrow_2d(
        dot_position,
        dot_position + dot_query.0.0 * VEL_ARROW_LENGTH / VEL_MAX,
        YELLOW,
    );
}

// Initialize all the stuff in the world
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // Spawn all of the blocks
    // The ground should start at zero level, and be evenly distributed left and right
    let y_offset = 0.0 - BLOCK_SIZE / 2.0;
    let stage_width = f32::from(BLOCKS_X) * BLOCK_SIZE;
    let x_offset = -(stage_width / 2.0) + BLOCK_SIZE / 2.0;
    for j in 0..BLOCKS_Y {
        for i in 0..BLOCKS_X {
            commands.spawn((
                Block,
                Transform {
                    translation: Vec3::from_array([
                        x_offset + f32::from(i) * BLOCK_SIZE,
                        y_offset - f32::from(j) * BLOCK_SIZE,
                        0.0,
                    ]),
                    scale: Vec3::from_array([BLOCK_SIZE, BLOCK_SIZE, 1.0]),
                    ..default()
                },
                Sprite {
                    color: Color::srgb(SADDLE_BROWN.red, SADDLE_BROWN.green, SADDLE_BROWN.blue),
                    ..default()
                },
            ));
        }
    }

    // Spawn the dot
    commands.spawn((
        Dot,
        Velocity(Vec2::ZERO),
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform::default().with_scale(Vec2::splat(DOT_RADIUS * 2.0).extend(0.0)),
    ));
}
