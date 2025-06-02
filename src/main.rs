use bevy::{
    color::palettes::css::SADDLE_BROWN,
    math::{
        bounding::*,
        ops::{hypot, powf},
    },
    prelude::*,
    window::PrimaryWindow,
};

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
        .add_systems(
            FixedUpdate,
            (player_accel, player_collision, check_bounds, player_move).chain(),
        )
        .run();
}

#[derive(Component)]
struct Block;

#[derive(Component)]
#[require(Velocity, Transform, Sprite)]
struct Player;

#[derive(Component, Default)]
struct Velocity(Vec2);

// Apply acceleration due to input and gravity, but limit the velocity
fn player_accel(
    mut player: Single<&mut Velocity, With<Player>>,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let vel_step_input = get_direction_from_keyboard(keyboard) * PLAYER_ACCEL;
    let vel_step_grav = Vec2::from_array([0.0, -1.0]) * GRAVITY;

    let vel_step = (vel_step_input + vel_step_grav) * time.delta_secs();

    player.0 = (player.0 + vel_step)
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
    mut player: Single<(&Transform, &mut Velocity), With<Player>>,
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
            .0
            .translation
            .truncate()
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
        player.0.translation.truncate(),
        Vec2::new(PLAYER_WIDTH / 2.0, PLAYER_HEIGHT / 2.0),
    );
    for block in nearby_blocks {
        if !block.intersects(&player_bound) {
            continue;
        }

        match PlayerCollision::collision_side(player_bound, block) {
            PlayerCollision::Left => {
                if player.1.0.x < 0.0 {
                    player.1.0.x *= -1.0;
                }
            }
            PlayerCollision::Right => {
                if player.1.0.x > 0.0 {
                    player.1.0.x *= -1.0;
                }
            }
            PlayerCollision::Top => {
                if player.1.0.y > 0.0 {
                    player.1.0.y *= -1.0;
                }
            }
            PlayerCollision::Bottom => {
                if player.1.0.y < 0.0 {
                    player.1.0.y *= -1.0;
                }
            }
        }
    }
}

// Move the player every update based on the current velocity
fn player_move(player: Single<(&Velocity, &mut Transform), With<Player>>, time: Res<Time>) {
    let (velocity, mut transform) = player.into_inner();
    transform.translation += (velocity.0 * time.delta_secs()).extend(0.0);
}

// Check if the player is colliding with certain screen edges and change its properties
fn check_bounds(
    mut player: Single<(&mut Velocity, &mut Transform), With<Player>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    // Check if the player is colliding with the lower boundary
    // Collision should take place as the bottom of the player touches the edge
    if window.resolution.height() / 2.0 + player.1.translation.y - PLAYER_HEIGHT / 2.0 < 0.0
        && player.0.0.y < 0.0
    {
        // Reflect the dot's vertical velocity
        player.0.0.y *= -1.0;
    }

    // Check if the player is "colliding" with the outer wall
    // Collision should take place when player is just barely completely out of frame
    if player.1.translation.x.abs() - PLAYER_WIDTH > window.resolution.width() / 2.0 {
        // Warp the player to the opposite side of the frame
        player.1.translation.x *= -1.0;
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
        Velocity(Vec2::new(0.0, 0.0)),
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(Color::WHITE)),
        Transform {
            translation: Vec3::new(0.0, 30.0, 0.0),
            scale: Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT).extend(0.0),
            ..default()
        },
    ));
}
