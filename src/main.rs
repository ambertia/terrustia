use bevy::{
    color::palettes::css::SADDLE_BROWN,
    math::{
        bounding::*,
        ops::{hypot, powf},
    },
    prelude::*,
    window::PrimaryWindow,
};

mod physics;
mod terrain;

const PLAYER_ACCEL: f32 = 60.0;
const PLAYER_HEIGHT: f32 = BLOCK_SIZE * 3.0;
const PLAYER_WIDTH: f32 = BLOCK_SIZE * 2.0;
const DRAG_FACTOR: f32 = 0.05;
const VEL_MAX: f32 = 300.0;
const GRAVITY: f32 = 15.0;
const BLOCK_SIZE: f32 = 10.0;
const BLOCKS_X: u16 = 80;
const BLOCKS_Y: u16 = 40;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, player_visual_update)
        .add_systems(
            FixedUpdate,
            (
                (player_accel, player_collision, check_bounds, player_move).chain(),
                break_block,
            ),
        )
        .run();
}

#[derive(Component)]
struct Block;

#[derive(Component)]
#[require(Transform, Sprite, MovementState)]
struct Player;

#[derive(Component, Default)]
struct MovementState {
    velocity: Vec2,
    position: Vec2,
}

// Apply acceleration due to input and gravity, but limit the velocity
fn player_accel(
    mut player: Single<&mut MovementState, With<Player>>,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let vel_step_input = get_direction_from_keyboard(keyboard) * PLAYER_ACCEL;
    let vel_step_grav = Vec2::from_array([0.0, -1.0]) * GRAVITY;

    let vel_step = (vel_step_input + vel_step_grav) * time.delta_secs();

    player.velocity = (player.velocity + vel_step)
        .move_towards(Vec2::ZERO, DRAG_FACTOR * time.delta_secs())
        .clamp_length_max(VEL_MAX);
}

enum PlayerCollision {
    Left,
    Right,
    Top,
    Bottom,
}

impl PlayerCollision {
    // Determine what side of a block a player collides on
    fn collision_side(player: Aabb2d, block: Aabb2d) -> PlayerCollision {
        let height_diff = (PLAYER_HEIGHT - PLAYER_WIDTH) / 2.0;
        let offset = player.center() - block.closest_point(player.center());
        if offset.x.abs() > offset.y.abs() - height_diff {
            if offset.x < 0.0 {
                return PlayerCollision::Right;
            } else {
                return PlayerCollision::Left;
            }
        } else if offset.y < 0.0 {
            return PlayerCollision::Top;
        } else {
            return PlayerCollision::Bottom;
        }
    }
}
// Detect if the player is colliding with any blocks and alter their velocity to ensure they don't
// move through blocks
fn player_collision(
    mut player: Single<&mut MovementState, With<Player>>,
    blocks: Query<&Transform, With<Block>>,
) {
    // Find all blocks that could collide with the player. This is limited to within a half
    // diagonal plus block size of the center of the player.
    let mut nearby_blocks: Vec<Aabb2d> = Vec::new();
    let critical_distance_squared =
        powf(hypot(PLAYER_WIDTH, PLAYER_HEIGHT) / 2.0 + BLOCK_SIZE, 2.0);
    let block_half_size = BLOCK_SIZE / 2.0; // I refuse to calculate this every iteration
    for block in blocks {
        if player
            .position
            .distance_squared(block.translation.truncate())
            < critical_distance_squared
        {
            nearby_blocks.push(Aabb2d::new(
                block.translation.truncate(),
                Vec2::new(block_half_size, block_half_size),
            ));
        }
    }

    // Check all nearby blocks to determine which (if any) intersect the player, and on what face
    let player_bound = Aabb2d::new(
        player.position,
        Vec2::new(PLAYER_WIDTH / 2.0, PLAYER_HEIGHT / 2.0),
    );
    for block in nearby_blocks {
        if !block.intersects(&player_bound) {
            continue;
        }

        match PlayerCollision::collision_side(player_bound, block) {
            PlayerCollision::Left => {
                if player.velocity.x < 0.0 {
                    player.velocity.x *= -0.5;
                }
            }
            PlayerCollision::Right => {
                if player.velocity.x > 0.0 {
                    player.velocity.x *= -0.5;
                }
            }
            PlayerCollision::Top => {
                if player.velocity.y > 0.0 {
                    player.velocity.y *= -0.1;
                }
            }
            PlayerCollision::Bottom => {
                if player.velocity.y < 0.0 {
                    player.velocity.y *= -0.4;
                }
            }
        }
    }
}

// Move the player every update based on the current velocity
fn player_move(mut player: Single<&mut MovementState, With<Player>>, time: Res<Time>) {
    let velocity_step = player.velocity * time.delta_secs();
    player.position += velocity_step;
}

// Update the location of the player on screen every frame by extrapolating
fn player_visual_update(
    mut transform: Single<&mut Transform, With<Player>>,
    state: Single<&MovementState, With<Player>>,
    fixed_time: Res<Time<Fixed>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // s = (1/2)at^2 + v_0 * t + s_0
    let acceleration = get_direction_from_keyboard(keyboard) * PLAYER_ACCEL
        + Vec2::new(0.0, -1.0) * GRAVITY
        - state.velocity * DRAG_FACTOR;
    let future_position = 0.5 * acceleration * powf(fixed_time.delta_secs(), 2.0)
        + state.velocity * fixed_time.delta_secs()
        + state.position;
    transform.translation = transform
        .translation
        .lerp(future_position.extend(0.0), fixed_time.overstep_fraction());
}

// Delete blocks when clicked on
fn break_block(
    camera: Single<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    blocks: Query<(&Transform, Entity), With<Block>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    // Only try to break blocks if the left mouse button is pressed
    if !mouse.pressed(MouseButton::Left) {
        return;
    }

    // Get the mouse position and convert to world space coordinates
    let cursor_pos = window.cursor_position().unwrap();
    let world_pos = camera.0.viewport_to_world_2d(camera.1, cursor_pos);
    println!("cursor_pos: {}", cursor_pos);
    println!("world_pos: {:?}", world_pos);

    // Check if there is a block under the cursor
    // Iterate over all the blocks
    // TODO: This is horribly inefficient and iterates over every single extant block in the world
    for block in blocks {
        // Distance between cursor position and the
        let distance: Vec2 = block.0.translation.truncate() - world_pos.unwrap();
        if distance.x.abs() < BLOCK_SIZE / 2.0 && distance.y.abs() < BLOCK_SIZE / 2.0 {
            commands.entity(block.1).despawn();
        }
    }
}

// Check if the player is colliding with certain screen edges and change its properties
fn check_bounds(
    mut player: Single<&mut MovementState, With<Player>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    // Check if the player is colliding with the lower boundary
    // Collision should take place as the bottom of the player touches the edge
    if window.resolution.height() / 2.0 + player.position.y - PLAYER_HEIGHT / 2.0 < 0.0
        && player.velocity.y < 0.0
    {
        // Reflect the dot's vertical velocity
        player.velocity.y *= -1.0;
    }

    // Check if the player is "colliding" with the outer wall
    // Collision should take place when player is just barely completely out of frame
    if player.position.x.abs() - PLAYER_WIDTH > window.resolution.width() / 2.0 {
        // Warp the player to the opposite side of the frame
        player.position.x *= -1.0;
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
            if (i == 39 || i == 40) && (j == 0) {
                continue;
            }
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

    // Spawn the player

    // Spawn the dot
    commands.spawn((
        Player,
        MovementState {
            position: Vec2::new(0.0, 30.0),
            ..default()
        },
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform {
            translation: Vec3::new(0.0, 30.0, 0.0),
            scale: Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT).extend(0.0),
            ..default()
        },
    ));
}
